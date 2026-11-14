//! A Model Context Protocol server for the Atlassian suite, built on the
//! official Rust SDK (`rmcp`) over stdio.
//!
//! Compact mode (default) exposes three tools that reach every operation:
//! search, describe, and call. Full mode exposes one tool per operation of
//! the selected services and resources. Task tools (`atl_jira_issue_list`,
//! ...) run the same code as the CLI commands.

use atl_sdk::catalog::{self, Catalog, Operation, ServiceDef};
use atl_sdk::{CallArgs, Client, Error, Pager, schema, tasks};
use rmcp::model::{
    CallToolRequestParams, CallToolResponse, CallToolResult, ContentBlock, Implementation,
    ListToolsResult, PaginatedRequestParams, ServerCapabilities, ServerConfig, Tool,
};
use rmcp::service::RequestContext;
use rmcp::{ErrorData, RoleServer, ServerHandler, ServiceExt};
use serde_json::{Map, Value, json};

/// Tool results longer than this are cut, so one call cannot flood the
/// model's context.
const MAX_RESULT_CHARS: usize = 100_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolMode {
    Compact,
    Full,
}

#[derive(Debug, Clone)]
pub struct ServerOptions {
    pub mode: ToolMode,
    /// Services to expose; empty means all.
    pub services: Vec<String>,
    /// Resources to expose in full mode; empty means all.
    pub resources: Vec<String>,
    /// Expose and allow only GET, HEAD and OPTIONS operations.
    pub read_only: bool,
    /// Also expose the task tools (`atl_jira_issue_list`, `atl_confluence_page_get`, ...).
    pub tasks: bool,
}

struct FullTool {
    name: String,
    service: &'static ServiceDef,
    catalog: &'static Catalog,
    op: &'static Operation,
}

struct Server {
    client: Client,
    options: ServerOptions,
    /// Services available on the configured deployment, after filtering,
    /// with their catalog keys. A catalog is parsed when a tool first needs
    /// it, so compact mode starts without parsing all 13.
    services: Vec<(&'static ServiceDef, &'static str)>,
    full_tools: Vec<FullTool>,
    /// The `tools/list` result, built once.
    tools: Vec<Tool>,
}

/// Serves until stdin closes.
pub async fn serve_stdio(client: Client, options: ServerOptions) -> anyhow::Result<()> {
    let server = Server::new(client, options)?;
    server
        .serve(rmcp::transport::stdio())
        .await?
        .waiting()
        .await?;
    Ok(())
}

impl ServerHandler for Server {
    fn get_info(&self) -> ServerConfig {
        let mut info = Implementation::new("atl", env!("CARGO_PKG_VERSION"));
        info.title = Some("Atlassian (atl)".into());
        ServerConfig::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(info)
            .with_instructions(self.instructions())
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, ErrorData> {
        Ok(ListToolsResult::with_all_items(self.tools.clone()))
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResponse, ErrorData> {
        let arguments = Value::Object(request.arguments.unwrap_or_default());
        // A `notifications/cancelled` from the client stops the call; rmcp
        // then sends no reply.
        tokio::select! {
            result = self.run_tool(&request.name, arguments) => Ok(result.into()),
            () = context.ct.cancelled() => Err(ErrorData::internal_error("the call was cancelled", None)),
        }
    }
}

/// MCP tool names allow `[A-Za-z0-9_-]{1,64}`.
fn tool_name(service: &str, id: &str) -> String {
    let raw: String = format!("{service}_{id}")
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect();
    if raw.len() <= 64 {
        return raw;
    }
    // FNV-1a keeps truncated names unique and stable across runs.
    let hash = raw.bytes().fold(0xcbf2_9ce4_8422_2325_u64, |h, b| {
        (h ^ u64::from(b)).wrapping_mul(0x100_0000_01b3)
    });
    format!("{}_{:08x}", &raw[..55], hash as u32)
}

impl Server {
    fn new(mut client: Client, options: ServerOptions) -> anyhow::Result<Self> {
        if options.read_only {
            client.set_read_only(true);
        }
        for name in &options.services {
            if catalog::service(name).is_none() {
                return Err(Error::Unknown(format!(
                    "unknown service `{name}` in --services; use one of: {}",
                    catalog::services()
                        .iter()
                        .map(|s| s.name)
                        .collect::<Vec<_>>()
                        .join(", ")
                ))
                .into());
            }
        }
        let services: Vec<_> = catalog::services()
            .iter()
            .filter(|def| {
                options.services.is_empty() || options.services.iter().any(|s| s == def.name)
            })
            .filter_map(|def| {
                def.catalog_key(client.deployment(def.family))
                    .map(|key| (def, key))
            })
            .collect();
        let mut full_tools = Vec::new();
        if options.mode == ToolMode::Full {
            for (service, key) in &services {
                let catalog = catalog::catalog(key);
                for op in &catalog.ops {
                    let resource = catalog.tag(op.tag).name.as_ref();
                    if (options.read_only && !op.is_safe())
                        || (!options.resources.is_empty()
                            && !options.resources.iter().any(|r| r == resource))
                    {
                        continue;
                    }
                    full_tools.push(FullTool {
                        name: tool_name(service.name, &op.id),
                        service,
                        catalog,
                        op,
                    });
                }
            }
            if full_tools.is_empty() {
                return Err(Error::Invalid(
                    "full mode selected no operations; check --services and --resources (resource names come from `atl api <service> --help`)".into(),
                )
                .into());
            }
        }
        let mut server = Self {
            client,
            options,
            services,
            full_tools,
            tools: Vec::new(),
        };
        server.tools = server
            .build_tools()
            .into_iter()
            .map(serde_json::from_value)
            .collect::<Result<_, _>>()?;
        Ok(server)
    }

    fn instructions(&self) -> String {
        let services: Vec<String> = self
            .services
            .iter()
            .map(|(def, _)| format!("{} ({})", def.name, def.title))
            .collect();
        let mut text = format!("Atlassian REST APIs. Services: {}.", services.join("; "));
        if self.options.mode == ToolMode::Compact {
            text.push_str(
                " Find operations with atlassian_search_operations, read their parameters with atlassian_describe_operation, then run them with atlassian_call_operation.",
            );
        }
        if self.options.read_only {
            text.push_str(" This server is read-only: only reads and searches run.");
        }
        text
    }

    fn build_tools(&self) -> Vec<Value> {
        if self.options.mode == ToolMode::Full {
            return self
                .full_tools
                .iter()
                .map(|tool| {
                    let mut description =
                        tool.op.summary.as_deref().unwrap_or(&tool.op.id).to_owned();
                    description.push_str(&format!("\n\n{} {}", tool.op.method, tool.op.path));
                    if let Some(details) = &tool.op.description {
                        description.push_str("\n\n");
                        description.extend(details.chars().take(1000));
                    }
                    json!({
                        "name": tool.name,
                        "description": description,
                        "inputSchema": schema::operation_input(tool.catalog, tool.op, 2),
                        "annotations": { "readOnlyHint": tool.op.is_safe() },
                    })
                })
                .collect();
        }
        let names: Vec<&str> = self.services.iter().map(|(def, _)| def.name).collect();
        let service_schema =
            json!({ "type": "string", "enum": names, "description": "Service name" });
        let mut tools = self.task_tools();
        tools.extend([
            json!({
                "name": "atlassian_search_operations",
                "description": format!(
                    "Search Atlassian REST operations by keywords (matched against operation id, resource, summary and path). Services: {}.",
                    self.services.iter().map(|(def, _)| format!("{} = {}", def.name, def.title)).collect::<Vec<_>>().join("; ")
                ),
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "query": { "type": "string", "description": "Keywords, for example \"create issue\" or \"page children\"" },
                        "service": service_schema,
                        "limit": { "type": "integer", "minimum": 1, "maximum": 100, "default": 20 }
                    },
                    "required": ["query"]
                },
                "annotations": { "readOnlyHint": true },
            }),
            json!({
                "name": "atlassian_describe_operation",
                "description": "Describe one operation: HTTP method and path, parameters, and request and response JSON schemas.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "service": service_schema,
                        "operation": { "type": "string", "description": "operationId from atlassian_search_operations" },
                        "depth": { "type": "integer", "minimum": 0, "maximum": 6, "default": 2, "description": "Levels of $ref to inline" }
                    },
                    "required": ["service", "operation"]
                },
                "annotations": { "readOnlyHint": true },
            }),
            json!({
                "name": "atlassian_call_operation",
                "description": "Run one operation and return the JSON response.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "service": service_schema,
                        "operation": { "type": "string", "description": "operationId from atlassian_search_operations" },
                        "params": { "type": "object", "description": "Path, query and header parameters keyed by name; arrays repeat query parameters" },
                        "body": { "description": "JSON request body, when the operation takes one" },
                        "files": { "type": "array", "items": { "type": "string" }, "description": "Local file paths for multipart upload operations" },
                        "page_all": { "type": "boolean", "default": false, "description": "Follow pagination and return {pages: [...], truncated} (truncated: page_limit stopped it and more pages exist)" },
                        "page_limit": { "type": "integer", "minimum": 1, "maximum": 50, "default": 5 }
                    },
                    "required": ["service", "operation"]
                },
                "annotations": { "readOnlyHint": self.options.read_only, "destructiveHint": !self.options.read_only },
            }),
        ]);
        tools
    }

    /// Task tools for the services this server exposes. A task name starts
    /// with its service (`jira_`, `confluence_`, `bitbucket_`, `jsm_`).
    fn visible_tasks(&self) -> impl Iterator<Item = &'static tasks::TaskDef> + '_ {
        // A task name starts with its service: `jira_`, `jsm_`, `confluence_`, ...
        tasks::list().iter().filter(move |task| {
            self.options.tasks
                && self
                    .services
                    .iter()
                    .find(|(def, _)| task.name.split('_').next() == Some(def.name))
                    .is_some_and(|(def, _)| task.read_only || !self.client.is_read_only(def.family))
        })
    }

    fn task_tools(&self) -> Vec<Value> {
        self.visible_tasks()
            .map(|task| {
                json!({
                    "name": format!("atl_{}", task.name),
                    "description": task.description,
                    "inputSchema": (task.schema)(),
                    "annotations": { "readOnlyHint": task.read_only, "destructiveHint": task.destructive },
                })
            })
            .collect()
    }

    async fn run_tool(&self, name: &str, arguments: Value) -> CallToolResult {
        let task = name
            .strip_prefix("atl_")
            .and_then(|n| self.visible_tasks().find(|t| t.name == n));
        let compact = self.options.mode == ToolMode::Compact;
        let outcome = match (task, name) {
            (Some(task), _) => tasks::run(&self.client, task.name, arguments)
                .await
                .map(|value| clip(value.to_string()))
                .map_err(|err| error_text(&err)),
            (None, "atlassian_search_operations") if compact => self.search(&arguments),
            (None, "atlassian_describe_operation") if compact => self.describe(&arguments),
            (None, "atlassian_call_operation") if compact => self.call_compact(&arguments).await,
            _ => match self.full_tools.iter().find(|tool| tool.name == name) {
                Some(tool) => {
                    let mut params = match arguments {
                        Value::Object(map) => map,
                        _ => Map::new(),
                    };
                    let body = params.remove("body");
                    let args = CallArgs {
                        params,
                        body,
                        files: Vec::new(),
                    };
                    self.run(tool.service, tool.catalog, tool.op, &args, false, 1)
                        .await
                }
                None => Err(format!(
                    "unknown tool `{name}`; call tools/list for the available tools"
                )),
            },
        };
        match outcome {
            Ok(text) => CallToolResult::success(vec![ContentBlock::text(text)]),
            Err(text) => CallToolResult::error(vec![ContentBlock::text(text)]),
        }
    }

    fn lookup(
        &self,
        arguments: &Value,
    ) -> Result<(&'static ServiceDef, &'static Catalog, &'static Operation), String> {
        let service = arguments
            .get("service")
            .and_then(Value::as_str)
            .ok_or("pass `service`, for example \"jira\"")?;
        let operation = arguments
            .get("operation")
            .and_then(Value::as_str)
            .ok_or("pass `operation`, an operationId from atlassian_search_operations")?;
        let (def, key) = self
            .services
            .iter()
            .find(|(def, _)| def.name == service)
            .copied()
            .ok_or_else(|| {
                format!(
                    "service `{service}` is not served here; available: {}",
                    self.service_names()
                )
            })?;
        let catalog = catalog::catalog(key);
        let op = catalog
            .find(operation)
            .ok_or_else(|| format!("`{service}` has no operation `{operation}`; find one with atlassian_search_operations"))?;
        Ok((def, catalog, op))
    }

    fn service_names(&self) -> String {
        self.services
            .iter()
            .map(|(def, _)| def.name)
            .collect::<Vec<_>>()
            .join(", ")
    }

    fn search(&self, arguments: &Value) -> Result<String, String> {
        let query = arguments
            .get("query")
            .and_then(Value::as_str)
            .ok_or("pass `query`, for example \"create issue\"")?;
        let only = arguments.get("service").and_then(Value::as_str);
        let limit = arguments
            .get("limit")
            .and_then(Value::as_u64)
            .unwrap_or(20)
            .clamp(1, 100) as usize;
        let terms: Vec<String> = query.split_whitespace().map(str::to_lowercase).collect();
        if terms.is_empty() {
            return Err("`query` is empty; pass keywords such as \"search issues jql\"".into());
        }
        let mut hits: Vec<(u32, Value)> = Vec::new();
        for (def, key) in &self.services {
            let catalog = catalog::catalog(key);
            if only.is_some_and(|only| only != def.name) {
                continue;
            }
            for op in &catalog.ops {
                if self.options.read_only && !op.is_safe() {
                    continue;
                }
                let resource = catalog.tag(op.tag).name.as_ref();
                let strong = format!("{} {} {}", op.id, op.name, resource).to_lowercase();
                let weak =
                    format!("{} {}", op.summary.as_deref().unwrap_or(""), op.path).to_lowercase();
                let mut score = 0;
                for term in &terms {
                    if strong.contains(term.as_str()) {
                        score += 3;
                    } else if weak.contains(term.as_str()) {
                        score += 1;
                    } else {
                        score = 0;
                        break;
                    }
                }
                if score == 0 {
                    continue;
                }
                if op.deprecated {
                    score -= 1;
                }
                hits.push((
                    score,
                    json!({
                        "service": def.name,
                        "operation": op.id,
                        "resource": resource,
                        "method": op.method.as_str(),
                        "path": op.path,
                        "summary": op.summary,
                        "deprecated": op.deprecated,
                    }),
                ));
            }
        }
        hits.sort_by_key(|hit| std::cmp::Reverse(hit.0));
        let total = hits.len();
        let results: Vec<Value> = hits.into_iter().take(limit).map(|(_, hit)| hit).collect();
        Ok(json!({ "total_matches": total, "results": results }).to_string())
    }

    fn describe(&self, arguments: &Value) -> Result<String, String> {
        let (def, catalog, op) = self.lookup(arguments)?;
        let depth = arguments
            .get("depth")
            .and_then(Value::as_u64)
            .unwrap_or(2)
            .min(6) as usize;
        Ok(clip(
            schema::describe_operation(def, catalog, op, depth).to_string(),
        ))
    }

    async fn call_compact(&self, arguments: &Value) -> Result<String, String> {
        let (def, catalog, op) = self.lookup(arguments)?;
        let args: CallArgs = serde_json::from_value(arguments.clone()).map_err(|err| {
            format!(
                "`params` must be an object keyed by parameter name, `files` a list of paths: {err}"
            )
        })?;
        let page_all = arguments
            .get("page_all")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        let page_limit = arguments
            .get("page_limit")
            .and_then(Value::as_u64)
            .unwrap_or(5)
            .clamp(1, 50) as usize;
        self.run(def, catalog, op, &args, page_all, page_limit)
            .await
    }

    async fn run(
        &self,
        def: &ServiceDef,
        catalog: &Catalog,
        op: &Operation,
        args: &CallArgs,
        page_all: bool,
        page_limit: usize,
    ) -> Result<String, String> {
        let request = self
            .client
            .prepare(def, catalog, op, args)
            .map_err(|err| error_text(&err))?;
        if page_all {
            let mut pager = Pager::new(&self.client, request, page_limit, Pager::DEFAULT_DELAY);
            let mut pages = Vec::new();
            while let Some(page) = pager.next_page().await.map_err(|err| error_text(&err))? {
                pages.push(atl_sdk::markup::lean(page));
            }
            let truncated = pager.truncated();
            return Ok(clip(
                json!({ "pages": pages, "truncated": truncated }).to_string(),
            ));
        }
        let response = self
            .client
            .execute(&request)
            .await
            .map_err(|err| error_text(&err))?;
        if response.body.is_empty() {
            return Ok(json!({ "status": response.status }).to_string());
        }
        if let Some(value) = response.json() {
            // The same slimmed JSON as `atl --agent`.
            return Ok(clip(atl_sdk::markup::lean(value).to_string()));
        }
        match std::str::from_utf8(&response.body) {
            Ok(text) => Ok(clip(text.to_owned())),
            Err(_) => Ok(format!(
                "binary response of {} bytes ({}); download it with the CLI: atl {} ... --output <file>",
                response.body.len(),
                response.content_type(),
                def.name
            )),
        }
    }
}

fn error_text(err: &Error) -> String {
    err.to_json().to_string()
}

fn clip(mut text: String) -> String {
    if text.len() <= MAX_RESULT_CHARS {
        return text;
    }
    let mut cut = MAX_RESULT_CHARS;
    while !text.is_char_boundary(cut) {
        cut -= 1;
    }
    let dropped = text.len() - cut;
    text.truncate(cut);
    text.push_str(&format!(
        "\n…[truncated {dropped} bytes; narrow the request with fields, limit or maxResults parameters]"
    ));
    text
}
