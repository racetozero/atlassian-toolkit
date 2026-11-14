//! End-to-end tests of the `atl` binary against a local mock server.

use serde_json::{Value, json};
use std::io::Write;
use std::process::{Command, Output, Stdio};
use wiremock::matchers::{header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

struct Env {
    config: tempfile::TempDir,
    vars: Vec<(&'static str, String)>,
    cwd: Option<std::path::PathBuf>,
}

impl Env {
    fn new() -> Self {
        Self {
            config: tempfile::tempdir().expect("tempdir"),
            vars: Vec::new(),
            cwd: None,
        }
    }

    fn var(mut self, key: &'static str, value: impl Into<String>) -> Self {
        self.vars.push((key, value.into()));
        self
    }

    fn command(&self, args: &[&str]) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_atl"));
        command
            .args(args)
            .env("ATL_CONFIG_DIR", self.config.path())
            .env_remove("ATL_PROFILE")
            .env_remove("ATL_AGENT")
            .env_remove("ATL_READ_ONLY")
            .env_remove("ATL_DEBUG")
            // Run outside this repository so no .atl.toml or git branch leaks in.
            .current_dir(
                self.cwd
                    .clone()
                    .unwrap_or_else(|| self.config.path().to_path_buf()),
            );
        for family in [
            "JIRA",
            "CONFLUENCE",
            "BITBUCKET",
            "TRELLO",
            "ADMIN",
            "ADMIN_SCIM",
        ] {
            for suffix in ["URL", "USER", "TOKEN", "DEPLOYMENT"] {
                command.env_remove(format!("ATL_{family}_{suffix}"));
            }
        }
        command.envs(self.vars.iter().map(|(k, v)| (*k, v.as_str())));
        command
    }

    /// Runs the binary off the async runtime's worker so the mock server keeps serving.
    async fn run(&self, args: &[&str], stdin: Option<&str>) -> Output {
        let mut command = self.command(args);
        let stdin = stdin.map(str::to_owned);
        tokio::task::spawn_blocking(move || {
            command
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped());
            let mut child = command.spawn().expect("spawn atl");
            let mut pipe = child.stdin.take().expect("stdin");
            if let Some(text) = stdin {
                pipe.write_all(text.as_bytes()).expect("write stdin");
            }
            drop(pipe);
            child.wait_with_output().expect("atl exits")
        })
        .await
        .expect("join")
    }
}

fn stdout_json(output: &Output) -> Value {
    serde_json::from_slice(&output.stdout).unwrap_or_else(|err| {
        panic!(
            "stdout is not JSON ({err}): {}\nstderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        )
    })
}

#[tokio::test(flavor = "multi_thread")]
async fn dry_runs_pick_the_catalog_for_the_deployment() {
    let env = Env::new()
        .var("ATL_JIRA_URL", "https://jira.corp.example")
        .var("ATL_JIRA_TOKEN", "pat");

    // Global flags before the service must not hide the service from the
    // command-tree scan.
    let output = env
        .run(
            &[
                "--profile",
                "default",
                "--dry-run",
                "api",
                "jira",
                "issue",
                "get-issue",
                "--issue-id-or-key",
                "ABC-1",
                "--fields",
                "summary",
            ],
            None,
        )
        .await;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        stdout_json(&output),
        json!({
            "method": "GET",
            "url": "https://jira.corp.example/rest/api/2/issue/ABC-1?fields=summary",
            "headers": { "authorization": "Bearer ***" },
            "body": null
        })
    );

    let output = env
        .run(
            &[
                "--deployment",
                "cloud",
                "api",
                "jira",
                "issues",
                "get-issue",
                "--params",
                r#"{"issueIdOrKey":"ABC-1"}"#,
                "--dry-run",
            ],
            None,
        )
        .await;
    assert_eq!(
        stdout_json(&output)["url"],
        "https://jira.corp.example/rest/api/3/issue/ABC-1"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn dry_run_task_commands_send_reads_but_print_the_write() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/rest/api/3/issue/A-1/transitions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(
            json!({ "transitions": [{ "id": "31", "name": "Finish", "to": { "name": "Done" } }] }),
        ))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(204))
        .expect(0)
        .mount(&server)
        .await;
    let env = Env::new()
        .var("ATL_JIRA_URL", server.uri())
        .var("ATL_JIRA_DEPLOYMENT", "cloud")
        .var("ATL_JIRA_USER", "a@example.com")
        .var("ATL_JIRA_TOKEN", "t");

    let output = env
        .run(
            &[
                "--agent",
                "--dry-run",
                "jira",
                "issue",
                "move",
                "A-1",
                "Done",
            ],
            None,
        )
        .await;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let data = &stdout_json(&output)["data"];
    assert_eq!(data["method"], "POST");
    assert_eq!(data["body"], json!({ "transition": { "id": "31" } }));
}

#[tokio::test(flavor = "multi_thread")]
async fn calls_print_json_and_page_all_prints_one_line_per_page() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/rest/api/3/search/jql"))
        .and(header("authorization", "Basic YWRhOnRva2Vu"))
        .respond_with(|request: &wiremock::Request| {
            let body: Value = serde_json::from_slice(&request.body).unwrap_or_default();
            let page = match body.get("nextPageToken") {
                None => json!({ "issues": [{ "key": "A-1" }], "nextPageToken": "p2" }),
                Some(_) => json!({ "issues": [{ "key": "A-2" }] }),
            };
            ResponseTemplate::new(200).set_body_json(page)
        })
        .mount(&server)
        .await;
    let env = Env::new()
        .var("ATL_JIRA_URL", server.uri())
        .var("ATL_JIRA_USER", "ada")
        .var("ATL_JIRA_TOKEN", "token")
        .var("ATL_JIRA_DEPLOYMENT", "cloud");
    let search = [
        "api",
        "jira",
        "issue-search",
        "search-and-reconsile-issues-using-jql-post",
        "--json",
        r#"{"jql":"project = A"}"#,
    ];

    let output = env.run(&search, None).await;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(stdout_json(&output)["issues"][0]["key"], "A-1");

    let mut paged = search.to_vec();
    paged.push("--page-all");
    let output = env.run(&paged, None).await;
    let lines: Vec<Value> = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|line| serde_json::from_str(line).expect("NDJSON line"))
        .collect();
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[1]["issues"][0]["key"], "A-2");
}

#[tokio::test(flavor = "multi_thread")]
async fn failures_exit_with_documented_codes_and_json_errors() {
    let server = MockServer::start().await;
    Mock::given(path("/rest/api/3/issue/NOPE-1"))
        .respond_with(
            ResponseTemplate::new(404)
                .set_body_json(json!({ "errorMessages": ["Issue does not exist"] })),
        )
        .mount(&server)
        .await;
    let with_creds = Env::new()
        .var("ATL_JIRA_URL", server.uri())
        .var("ATL_JIRA_TOKEN", "t")
        .var("ATL_JIRA_DEPLOYMENT", "cloud");
    let without_creds = Env::new();

    let cases: [(&Env, &[&str], i32, Option<&str>); 5] = [
        (
            &with_creds,
            &[
                "api",
                "jira",
                "issues",
                "get-issue",
                "--issue-id-or-key",
                "NOPE-1",
            ],
            1,
            Some("api"),
        ),
        (
            &without_creds,
            &[
                "api",
                "jira",
                "issues",
                "get-issue",
                "--issue-id-or-key",
                "A-1",
            ],
            2,
            Some("auth"),
        ),
        (
            &with_creds,
            &["api", "jira", "issues", "get-issue"],
            3,
            Some("invalid"),
        ),
        (
            &with_creds,
            &["api", "jira", "issues", "no-such-method"],
            4,
            None,
        ),
        (
            &with_creds,
            &["schema", "jira.noSuchOperation"],
            4,
            Some("unknown"),
        ),
    ];
    for (env, args, code, kind) in cases {
        let output = env.run(args, None).await;
        assert_eq!(
            output.status.code(),
            Some(code),
            "{args:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        if let Some(kind) = kind {
            let error: Value = serde_json::from_slice(&output.stderr).expect("stderr is JSON");
            assert_eq!(error["error"]["code"], kind, "{args:?}");
        }
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn scripted_login_verifies_saves_masks_and_logs_out() {
    let server = MockServer::start().await;
    Mock::given(path("/rest/api/2/myself"))
        .and(header("authorization", "Bearer good-pat"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({ "displayName": "Ada Lovelace" })),
        )
        .mount(&server)
        .await;
    Mock::given(path("/rest/api/2/myself"))
        .respond_with(ResponseTemplate::new(401).set_body_string(""))
        .mount(&server)
        .await;
    let env = Env::new();
    let login = [
        "auth",
        "login",
        "--product",
        "jira",
        "--url",
        &server.uri(),
        "--token-stdin",
    ];

    let output = env.run(&login, Some("bad-pat\n")).await;
    assert_eq!(
        output.status.code(),
        Some(2),
        "a rejected token is an auth error"
    );
    assert!(
        !env.config.path().join("config.toml").exists(),
        "nothing is saved on failure"
    );

    let output = env.run(&login, Some("good-pat\n")).await;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        stdout_json(&output)["connections"]["jira"]["deployment"],
        "data-center"
    );
    let saved = std::fs::read_to_string(env.config.path().join("config.toml")).expect("config");
    assert!(
        saved.contains("good-pat") && saved.contains("Ada Lovelace"),
        "{saved}"
    );

    let status = stdout_json(&env.run(&["auth", "status"], None).await);
    assert_eq!(status["default_profile"], "default");
    assert_eq!(
        status["profiles"]["default"]["jira"]["auth"],
        "bearer …-pat"
    );
    assert!(!status.to_string().contains("good-pat"));

    // The stored credential now drives ordinary commands.
    let dry = stdout_json(
        &env.run(&["api", "jira", "myself", "get-user", "--dry-run"], None)
            .await,
    );
    assert_eq!(dry["url"], format!("{}/rest/api/2/myself", server.uri()));

    assert!(
        env.run(&["auth", "logout", "--product", "jira"], None)
            .await
            .status
            .success()
    );
    let status = stdout_json(&env.run(&["auth", "status"], None).await);
    assert_eq!(status["profiles"], json!({}));
}

#[tokio::test(flavor = "multi_thread")]
async fn mcp_cancelled_calls_stop_without_a_reply() {
    let server = MockServer::start().await;
    Mock::given(path("/rest/api/3/myself"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!({ "accountId": "1" }))
                .set_delay(std::time::Duration::from_secs(20)),
        )
        .mount(&server)
        .await;
    let env = Env::new()
        .var("ATL_JIRA_URL", server.uri())
        .var("ATL_JIRA_TOKEN", "t")
        .var("ATL_JIRA_DEPLOYMENT", "cloud");
    let input = [
        json!({ "jsonrpc": "2.0", "id": 0, "method": "initialize", "params": { "protocolVersion": "2025-06-18", "capabilities": {}, "clientInfo": { "name": "test", "version": "0" } } }),
        json!({ "jsonrpc": "2.0", "method": "notifications/initialized" }),
        json!({ "jsonrpc": "2.0", "id": 1, "method": "tools/call", "params": { "name": "atl_jira_me", "arguments": {} } }),
        json!({ "jsonrpc": "2.0", "method": "notifications/cancelled", "params": { "requestId": 1, "reason": "user" } }),
        json!({ "jsonrpc": "2.0", "id": 2, "method": "ping" }),
    ]
    .map(|m| m.to_string())
    .join("\n")
        + "\n";
    let started = std::time::Instant::now();
    let output = env.run(&["mcp"], Some(&input)).await;
    let ids: Vec<Value> = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|l| serde_json::from_str::<Value>(l).expect("JSON-RPC line")["id"].clone())
        .collect();
    assert_eq!(
        ids,
        [json!(0), json!(2)],
        "the cancelled call sends no reply"
    );
    assert!(
        started.elapsed() < std::time::Duration::from_secs(10),
        "the cancelled call does not keep the server running"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn mcp_server_searches_describes_and_calls_within_read_only_limits() {
    let server = MockServer::start().await;
    Mock::given(path("/wiki/api/v2/spaces"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "results": [{ "id": "1", "key": "DOCS", "name": "Docs", "_links": { "webui": "/spaces/DOCS" } }] })))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/wiki/api/v2/pages"))
        .and(query_param("limit", "1"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!({ "results": [{ "id": "42", "title": "Home" }] })),
        )
        .expect(1)
        .mount(&server)
        .await;
    let env = Env::new()
        .var("ATL_CONFLUENCE_URL", server.uri())
        .var("ATL_CONFLUENCE_TOKEN", "t")
        .var("ATL_CONFLUENCE_DEPLOYMENT", "cloud");
    let call = |id: u32, name: &str, arguments: Value| {
        json!({ "jsonrpc": "2.0", "id": id, "method": "tools/call", "params": { "name": name, "arguments": arguments } }).to_string()
    };
    let input = [
        json!({ "jsonrpc": "2.0", "id": 1, "method": "initialize", "params": { "protocolVersion": "2025-06-18", "capabilities": {}, "clientInfo": { "name": "test", "version": "0" } } }).to_string(),
        json!({ "jsonrpc": "2.0", "method": "notifications/initialized" }).to_string(),
        json!({ "jsonrpc": "2.0", "id": 2, "method": "tools/list" }).to_string(),
        call(3, "atlassian_search_operations", json!({ "query": "get pages", "service": "confluence" })),
        call(4, "atlassian_describe_operation", json!({ "service": "confluence", "operation": "getPages" })),
        call(5, "atlassian_call_operation", json!({ "service": "confluence", "operation": "getPages", "params": { "limit": 1 } })),
        call(6, "atlassian_call_operation", json!({ "service": "confluence", "operation": "createPage", "body": {} })),
        call(7, "atl_confluence_space_list", json!({ "limit": 5 })),
    ]
    .join("\n")
        + "\n";

    let output = env
        .run(
            &["mcp", "--services", "confluence", "--read-only"],
            Some(&input),
        )
        .await;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let mut replies: Vec<Value> = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|l| serde_json::from_str(l).expect("JSON-RPC line"))
        .collect();
    replies.sort_by_key(|reply| reply["id"].as_u64());
    assert_eq!(
        replies.len(),
        7,
        "one reply per request, none for the notification"
    );
    let result = |id: usize| &replies[id - 1]["result"];
    let text = |id: usize| -> Value {
        serde_json::from_str(result(id)["content"][0]["text"].as_str().expect("text"))
            .expect("tool JSON")
    };

    assert_eq!(result(1)["protocolVersion"], "2025-06-18");
    let tools: Vec<&str> = result(2)["tools"]
        .as_array()
        .expect("tools")
        .iter()
        .filter_map(|t| t["name"].as_str())
        .collect();
    assert!(
        tools.contains(&"atlassian_call_operation") && tools.contains(&"atl_confluence_page_get"),
        "{tools:?}"
    );
    assert!(
        !tools.contains(&"atl_confluence_page_create"),
        "read-only hides write tasks"
    );
    assert!(
        !tools.contains(&"atl_jira_issue_list"),
        "--services confluence hides Jira tasks"
    );
    assert_eq!(text(7)[0]["key"], "DOCS");
    let hits = text(3);
    assert!(
        hits["results"]
            .as_array()
            .expect("results")
            .iter()
            .any(|hit| hit["operation"] == "getPages"),
        "{hits}"
    );
    assert!(
        hits["results"]
            .as_array()
            .expect("results")
            .iter()
            .all(|hit| hit["method"] == "GET"),
        "read-only search shows only reads"
    );
    assert_eq!(text(4)["path"], "/wiki/api/v2/pages");
    assert_eq!(result(5)["isError"], false);
    assert_eq!(text(5)["results"][0]["title"], "Home");
    assert_eq!(result(6)["isError"], true);
    assert!(
        result(6)["content"][0]["text"]
            .as_str()
            .expect("text")
            .contains("read-only")
    );
}

fn envelope(output: &Output) -> Value {
    let text = String::from_utf8_lossy(&output.stdout);
    assert_eq!(
        text.lines().count(),
        1,
        "agent mode prints one line: {text}"
    );
    serde_json::from_str(&text).expect("envelope JSON")
}

#[tokio::test(flavor = "multi_thread")]
async fn agent_mode_prints_one_envelope_with_small_records() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/rest/api/3/search/jql"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "nextPageToken": "p2", "issues": [{
            "key": "A-1", "self": "https://x/rest/api/3/issue/1",
            "fields": { "summary": "Fix login", "status": { "name": "To Do", "statusCategory": { "key": "new" } }, "issuetype": { "name": "Bug" }, "assignee": null }
        }]})))
        .mount(&server)
        .await;
    let env = Env::new()
        .var("ATL_JIRA_URL", server.uri())
        .var("ATL_JIRA_TOKEN", "t")
        .var("ATL_JIRA_DEPLOYMENT", "cloud")
        .var("ATL_AGENT", "1");

    let output = env.run(&["jira", "issue", "list", "-p", "A"], None).await;
    assert!(output.status.success());
    let reply = envelope(&output);
    assert_eq!(
        (reply["ok"].clone(), reply["command"].clone()),
        (json!(true), json!("jira issue list"))
    );
    assert_eq!(reply["data"][0]["key"], "A-1");
    assert_eq!(reply["data"][0]["status"], "To Do");
    assert_eq!(reply["meta"]["next_page_token"], "p2");
    assert_eq!(reply["meta"]["count"], 1);

    let output = env
        .run(
            &[
                "jira",
                "issue",
                "list",
                "-p",
                "A",
                "--select",
                "key,summary",
            ],
            None,
        )
        .await;
    assert_eq!(
        envelope(&output)["data"],
        json!([{ "key": "A-1", "summary": "Fix login" }])
    );

    // The full-screen browser is opt-in and needs a terminal.
    let output = env
        .run(&["jira", "issue", "list", "-p", "A", "--browse"], None)
        .await;
    assert_eq!(output.status.code(), Some(3));
    assert_eq!(envelope(&output)["error"]["code"], "invalid");

    let output = env.run(&["jira", "issue", "view", "NOPE-1"], None).await;
    assert_eq!(output.status.code(), Some(1));
    let reply = envelope(&output);
    assert_eq!(
        (reply["ok"].clone(), reply["error"]["code"].clone()),
        (json!(false), json!("api"))
    );
    assert!(output.stderr.is_empty(), "agent errors go to stdout");
}

#[tokio::test(flavor = "multi_thread")]
async fn agent_mode_leans_generated_responses_unless_raw() {
    let server = MockServer::start().await;
    Mock::given(path("/rest/api/3/issue/A-1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "key": "A-1", "self": "https://x", "expand": "names",
            "fields": { "customfield_1": null, "description": { "type": "doc", "version": 1, "content": [{ "type": "paragraph", "content": [{ "type": "text", "text": "hi", "marks": [{ "type": "strong" }] }] }] } }
        })))
        .mount(&server)
        .await;
    let env = Env::new()
        .var("ATL_JIRA_URL", server.uri())
        .var("ATL_JIRA_TOKEN", "t")
        .var("ATL_JIRA_DEPLOYMENT", "cloud");
    let get = [
        "api",
        "jira",
        "issues",
        "get-issue",
        "--issue-id-or-key",
        "A-1",
        "--agent",
    ];

    assert_eq!(
        envelope(&env.run(&get, None).await)["data"],
        json!({ "key": "A-1", "fields": { "description": "**hi**" } })
    );
    let mut raw = get.to_vec();
    raw.push("--raw");
    assert_eq!(
        envelope(&env.run(&raw, None).await)["data"]["self"],
        "https://x"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn deletes_need_yes_and_read_only_refuses_writes() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/rest/api/3/issue/A-1"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;
    let env = Env::new()
        .var("ATL_JIRA_URL", server.uri())
        .var("ATL_JIRA_TOKEN", "t")
        .var("ATL_JIRA_DEPLOYMENT", "cloud");
    let delete = [
        "api",
        "jira",
        "issues",
        "delete-issue",
        "--issue-id-or-key",
        "A-1",
        "--agent",
    ];

    let output = env.run(&delete, None).await;
    assert_eq!(output.status.code(), Some(3));
    assert_eq!(envelope(&output)["error"]["code"], "confirmation_required");
    let mut confirmed = delete.to_vec();
    confirmed.push("--yes");
    assert_eq!(
        envelope(&env.run(&confirmed, None).await)["data"],
        json!({ "status": 204 })
    );

    let read_only = Env {
        vars: env.vars.clone(),
        ..Env::new()
    }
    .var("ATL_READ_ONLY", "1");
    let output = read_only
        .run(
            &["jira", "issue", "create", "-p", "A", "-s", "x", "--agent"],
            None,
        )
        .await;
    assert_eq!(output.status.code(), Some(3));
    assert_eq!(envelope(&output)["error"]["code"], "read_only");
}

#[tokio::test(flavor = "multi_thread")]
async fn repository_config_and_git_branch_supply_defaults() {
    let repo = tempfile::tempdir().expect("repo");
    std::fs::write(repo.path().join(".atl.toml"), "[jira]\nproject = \"WEB\"\n")
        .expect("write .atl.toml");
    let git = |args: &[&str]| {
        Command::new("git")
            .args(args)
            .current_dir(repo.path())
            .output()
            .expect("git")
    };
    git(&["init", "-q"]);
    git(&["checkout", "-q", "-b", "feature/web-42-fix-header"]);
    let mut env = Env::new()
        .var("ATL_JIRA_URL", "https://acme.atlassian.net")
        .var("ATL_JIRA_TOKEN", "t");
    env.cwd = Some(repo.path().to_path_buf());

    let jql = envelope(
        &env.run(
            &[
                "jira",
                "issue",
                "list",
                "-s",
                "~Done",
                "--dry-run",
                "--agent",
            ],
            None,
        )
        .await,
    );
    assert_eq!(
        jql["data"]["jql"],
        "project = \"WEB\" AND status NOT IN (\"Done\") ORDER BY updated DESC"
    );
    let open = envelope(&env.run(&["open", "--agent"], None).await);
    assert_eq!(
        open["data"]["url"],
        "https://acme.atlassian.net/browse/WEB-42"
    );

    // The pull request goes to the origin repository from the current branch.
    git(&["remote", "add", "origin", "git@bitbucket.org:acme/web.git"]);
    let pr = envelope(
        &env.run(
            &[
                "bitbucket",
                "pr",
                "create",
                "-t",
                "Fix",
                "--dry-run",
                "--agent",
            ],
            None,
        )
        .await,
    );
    assert_eq!(
        (&pr["data"]["url"], &pr["data"]["body"]["source"]),
        (
            &json!("https://api.bitbucket.org/2.0/repositories/acme/web/pullrequests"),
            &json!({ "branch": { "name": "feature/web-42-fix-header" } })
        )
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn jql_check_exits_3_with_jiras_parse_errors() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/rest/api/3/jql/parse"))
        .respond_with(|request: &wiremock::Request| {
            let body: Value = serde_json::from_slice(&request.body).unwrap_or_default();
            let errors = if body["queries"][0] == "project = A" {
                json!([])
            } else {
                json!(["Expecting a value after '='."])
            };
            ResponseTemplate::new(200).set_body_json(json!({ "queries": [{ "errors": errors }] }))
        })
        .mount(&server)
        .await;
    let env = Env::new()
        .var("ATL_JIRA_URL", server.uri())
        .var("ATL_JIRA_TOKEN", "t")
        .var("ATL_JIRA_DEPLOYMENT", "cloud")
        .var("ATL_AGENT", "1");

    let output = env.run(&["jira", "jql", "check", "project ="], None).await;
    assert_eq!(output.status.code(), Some(3));
    assert!(
        envelope(&output)["error"]["message"]
            .as_str()
            .expect("message")
            .contains("Expecting a value after '='.")
    );
    assert_eq!(
        envelope(
            &env.run(&["jira", "jql", "check", "project = A"], None)
                .await
        )["data"]["valid"],
        true
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn shell_completion_is_dynamic_and_covers_generated_commands() {
    let env = Env::new();
    let script = env.run(&["completions", "zsh"], None).await;
    assert!(script.status.success());
    assert!(
        script.stdout.len() < 10_000,
        "the script stays small ({} bytes)",
        script.stdout.len()
    );

    let mut command = env.command(&["--", "atl", "api", "jira", "issues", "get-is"]);
    command
        .env("COMPLETE", "zsh")
        .env("_CLAP_COMPLETE_INDEX", "5")
        .env("_CLAP_COMPLETE_IFS", "\n");
    let output = tokio::task::spawn_blocking(move || command.output().expect("run"))
        .await
        .expect("join");
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .lines()
            .any(|l| l.starts_with("get-issue:")),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn jira_field_flags_need_name_equals_value_and_delete_asks_first() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/rest/api/3/issue/A-1"))
        .and(query_param("deleteSubtasks", "true"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;
    let env = Env::new()
        .var("ATL_JIRA_URL", server.uri())
        .var("ATL_JIRA_TOKEN", "t")
        .var("ATL_JIRA_DEPLOYMENT", "cloud");

    let output = env
        .run(
            &[
                "jira", "issue", "edit", "A-1", "--field", "Points", "--agent",
            ],
            None,
        )
        .await;
    assert_eq!(output.status.code(), Some(3));
    let message = envelope(&output)["error"]["message"].to_string();
    assert!(message.contains("pass NAME=VALUE"), "{message}");

    let delete = ["jira", "issue", "delete", "a-1", "--subtasks", "--agent"];
    let output = env.run(&delete, None).await;
    assert_eq!(envelope(&output)["error"]["code"], "confirmation_required");
    let mut confirmed = delete.to_vec();
    confirmed.push("--yes");
    assert_eq!(
        envelope(&env.run(&confirmed, None).await)["data"],
        json!({ "key": "A-1", "deleted": true })
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn login_saves_the_secret_in_the_config_file() {
    let env = Env::new();
    let output = env
        .run(
            &[
                "auth",
                "login",
                "--product",
                "jira",
                "--url",
                "https://jira.example.com",
                "--token-stdin",
                "--no-verify",
            ],
            Some("tok-123"),
        )
        .await;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let text = std::fs::read_to_string(env.config.path().join("config.toml")).expect("config");
    assert!(text.contains("tok-123"), "{text}");

    let status = stdout_json(&env.run(&["auth", "status"], None).await);
    assert_eq!(
        status["profiles"]["default"]["jira"]["auth"],
        "bearer …-123"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn oauth_login_exchanges_the_code_and_saves_tokens() {
    use std::io::{BufRead, Read};
    use wiremock::matchers::body_partial_json;
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/oauth/token"))
        .and(body_partial_json(json!({ "grant_type": "authorization_code", "code": "c1", "client_id": "cid", "client_secret": "sec" })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "access_token": "a1", "refresh_token": "r1", "expires_in": 3600 })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/oauth/token/accessible-resources"))
        .and(header("authorization", "Bearer a1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(
            json!([{ "id": "cloud-1", "url": "https://acme.atlassian.net", "name": "acme" }]),
        ))
        .mount(&server)
        .await;
    let port = std::net::TcpListener::bind("127.0.0.1:0")
        .expect("free port")
        .local_addr()
        .expect("addr")
        .port();
    let env = Env::new()
        .var("ATL_NO_BROWSER", "1")
        .var("ATL_OAUTH_AUTH_SERVER", server.uri())
        .var("ATL_OAUTH_API_SERVER", server.uri());
    let port_text = port.to_string();
    let mut command = env.command(&[
        "auth",
        "login",
        "--method",
        "oauth",
        "--product",
        "cloud-site",
        "--user",
        "cid",
        "--token-stdin",
        "--no-verify",
        "--callback-port",
        &port_text,
    ]);
    let output = tokio::task::spawn_blocking(move || {
        let mut child = command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("spawn atl");
        child
            .stdin
            .take()
            .expect("stdin")
            .write_all(b"sec")
            .expect("secret");
        let mut stderr = std::io::BufReader::new(child.stderr.take().expect("stderr"));
        let url = loop {
            let mut line = String::new();
            assert!(
                stderr.read_line(&mut line).expect("stderr line") > 0,
                "atl exited before printing the URL"
            );
            if line.starts_with("http") {
                break line.trim().to_owned();
            }
        };
        assert!(
            url.contains("client_id=cid") && url.contains("offline_access"),
            "{url}"
        );
        let state = url
            .split("state=")
            .nth(1)
            .and_then(|rest| rest.split('&').next())
            .expect("state")
            .to_owned();
        let mut socket = std::net::TcpStream::connect(("127.0.0.1", port)).expect("callback");
        socket
            .write_all(
                format!("GET /callback?code=c1&state={state} HTTP/1.1\r\nHost: localhost\r\n\r\n")
                    .as_bytes(),
            )
            .expect("request");
        let mut page = String::new();
        socket.read_to_string(&mut page).expect("page");
        assert!(
            page.starts_with("HTTP/1.1 200") && page.contains("Signed in"),
            "{page}"
        );
        child.wait_with_output().expect("atl exits")
    })
    .await
    .expect("join");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let saved = stdout_json(&output);
    assert_eq!(
        saved["connections"]["jira"]["base_url"],
        format!("{}/ex/jira/cloud-1", server.uri())
    );
    assert_eq!(
        saved["connections"]["confluence"]["base_url"],
        format!("{}/ex/confluence/cloud-1", server.uri())
    );
    let text = std::fs::read_to_string(env.config.path().join("config.toml")).expect("config");
    assert!(
        text.contains("kind = \"oauth\"") && text.contains("refresh_token = \"r1\""),
        "{text}"
    );
    assert!(
        text.contains("site_url = \"https://acme.atlassian.net\""),
        "{text}"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn input_required_errors_reach_agents_and_task_run_takes_json() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/rest/api/3/issue/AB-1/transitions"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({ "transitions": [
                { "id": "11", "name": "Start", "to": { "name": "In Progress" } },
                { "id": "31", "name": "Finish", "to": { "name": "Done" } }
            ]})),
        )
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/rest/api/3/issue/AB-1/transitions"))
        .and(wiremock::matchers::body_partial_json(
            json!({ "transition": { "id": "31" } }),
        ))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/rest/api/3/issue/AB-1/comment"))
        .and(wiremock::matchers::body_string_contains("from a pipe"))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({ "id": "5" })))
        .expect(1)
        .mount(&server)
        .await;
    let env = Env::new()
        .var("ATL_JIRA_URL", server.uri())
        .var("ATL_JIRA_TOKEN", "t")
        .var("ATL_JIRA_DEPLOYMENT", "cloud");

    // An agent learns the valid statuses from the error.
    let output = env
        .run(&["--agent", "jira", "issue", "move", "AB-1"], None)
        .await;
    assert_eq!(output.status.code(), Some(3));
    let error = envelope(&output)["error"].clone();
    assert_eq!(error["code"], "input_required");
    assert_eq!(error["input"]["arg"], "status");
    assert_eq!(
        error["input"]["choices"][1],
        json!({ "value": "Done", "label": "Done (Finish)" })
    );

    // Without a terminal (and with --no-input), people get the same error JSON.
    let output = env
        .run(&["--no-input", "jira", "issue", "move", "AB-1"], None)
        .await;
    assert_eq!(output.status.code(), Some(3));
    let error: Value = serde_json::from_slice(&output.stderr).expect("error JSON");
    assert_eq!(error["error"]["input"]["arg"], "status");

    // `atl task run` takes the arguments as one JSON object.
    let output = env
        .run(
            &[
                "--agent",
                "task",
                "run",
                "jira_issue_move",
                r#"{"key":"AB-1","status":"Done"}"#,
            ],
            None,
        )
        .await;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert_eq!(envelope(&output)["data"]["status"], "Done");
    let output = env
        .run(
            &[
                "--agent",
                "task",
                "run",
                "jira_issue_delete",
                r#"{"key":"AB-1"}"#,
            ],
            None,
        )
        .await;
    assert_eq!(output.status.code(), Some(3));
    assert_eq!(envelope(&output)["error"]["code"], "confirmation_required");
    let output = env
        .run(&["--agent", "task", "describe", "jira_issue_move"], None)
        .await;
    assert_eq!(
        envelope(&output)["data"]["schema"]["required"],
        json!(["key", "status"])
    );

    // A piped body is read outside agent mode; agents must say --body-file -.
    let output = env
        .run(&["jira", "issue", "comment", "AB-1"], Some("from a pipe\n"))
        .await;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let output = env
        .run(
            &["--agent", "jira", "issue", "comment", "AB-1"],
            Some("never read"),
        )
        .await;
    assert_eq!(output.status.code(), Some(3));
    assert!(
        envelope(&output)["error"]["message"]
            .as_str()
            .is_some_and(|m| m.contains("--body-file")),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn jira_sprint_and_epic_issues_resolve_then_list() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/rest/agile/1.0/board/7/sprint"))
        .and(query_param("state", "closed"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "isLast": true, "values": [
            { "id": 11, "name": "S11", "state": "closed", "endDate": "2026-10-01T00:00:00.000Z" },
            { "id": 12, "name": "S12", "state": "closed", "endDate": "2026-10-15T00:00:00.000Z" }
        ]})))
        .mount(&server)
        .await;
    let found = json!({ "issues": [{ "key": "AB-1", "fields": { "summary": "Fix login", "status": { "name": "Done", "statusCategory": { "key": "done" } }, "issuetype": { "name": "Bug" } } }] });
    Mock::given(method("POST"))
        .and(path("/rest/api/3/search/jql"))
        .and(wiremock::matchers::body_string_contains("sprint = 12"))
        .respond_with(ResponseTemplate::new(200).set_body_json(found.clone()))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/rest/api/3/search/jql"))
        .and(wiremock::matchers::body_string_contains(
            "parent = \\\"AB-5\\\"",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(found.clone()))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/rest/api/3/search/jql"))
        .and(wiremock::matchers::body_string_contains(
            "watcher = currentUser()",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(found))
        .expect(1)
        .mount(&server)
        .await;
    let env = Env::new()
        .var("ATL_JIRA_URL", server.uri())
        .var("ATL_JIRA_TOKEN", "t")
        .var("ATL_JIRA_DEPLOYMENT", "cloud");

    let output = env
        .run(
            &[
                "--agent", "jira", "sprint", "issues", "--prev", "--board", "7",
            ],
            None,
        )
        .await;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert_eq!(
        envelope(&output)["meta"]["jql"],
        "sprint = 12 ORDER BY updated DESC"
    );

    let output = env
        .run(&["--agent", "jira", "epic", "issues", "ab-5"], None)
        .await;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );

    let output = env
        .run(
            &[
                "jira",
                "issue",
                "list",
                "-w",
                "--format",
                "tsv",
                "--no-headers",
                "--columns",
                "key,summary",
            ],
            None,
        )
        .await;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&output.stdout), "AB-1\tFix login\n");

    let output = env
        .run(
            &[
                "--agent",
                "--dry-run",
                "jira",
                "epic",
                "issues",
                "AB-5",
                "-q",
                "a = b OR c = d ORDER BY rank",
            ],
            None,
        )
        .await;
    assert_eq!(
        envelope(&output)["data"]["jql"],
        "parent = \"AB-5\" AND (a = b OR c = d) ORDER BY rank"
    );
    let dc = Env::new()
        .var("ATL_JIRA_URL", "https://jira.corp.example")
        .var("ATL_JIRA_TOKEN", "t");
    let output = dc
        .run(
            &[
                "--agent",
                "--dry-run",
                "jira",
                "epic",
                "issues",
                "AB-5",
                "-q",
                "a = b OR c = d ORDER BY rank",
            ],
            None,
        )
        .await;
    assert_eq!(
        envelope(&output)["data"]["jql"],
        "(\"Epic Link\" = AB-5 AND (a = b OR c = d)) ORDER BY rank"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn confluence_find_attachments_comments_and_convert() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/wiki/rest/api/search"))
        .and(query_param("cql", "type = page AND title ~ \"onboard\" AND space = \"DOCS\""))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "results": [
            { "content": { "id": "5", "type": "page", "title": "Onboarding" }, "resultGlobalContainer": { "title": "Docs" }, "url": "/spaces/DOCS/pages/5" }
        ]})))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/wiki/api/v2/pages/5/attachments"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({ "results": [
                { "id": "a1", "title": "a.png", "downloadLink": "/download/a.png" },
                { "id": "a2", "title": "b.pdf", "downloadLink": "/download/b.pdf" }
            ]})),
        )
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/wiki/download/a.png"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(b"PNG".to_vec()))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/wiki/api/v2/footer-comments"))
        .and(wiremock::matchers::body_partial_json(
            json!({ "parentCommentId": "9" }),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "id": "10" })))
        .expect(1)
        .mount(&server)
        .await;
    let env = Env::new()
        .var("ATL_CONFLUENCE_URL", server.uri())
        .var("ATL_CONFLUENCE_TOKEN", "t")
        .var("ATL_CONFLUENCE_DEPLOYMENT", "cloud");

    let output = env
        .run(
            &[
                "--agent",
                "confluence",
                "page",
                "find",
                "onboard",
                "--space",
                "DOCS",
            ],
            None,
        )
        .await;
    assert_eq!(envelope(&output)["data"][0]["title"], "Onboarding");

    let dest = env.config.path().join("img");
    let output = env
        .run(
            &[
                "--agent",
                "confluence",
                "page",
                "attachments",
                "5",
                "--pattern",
                "*.PNG",
                "--download",
                "--dest",
                dest.to_str().expect("utf-8"),
            ],
            None,
        )
        .await;
    assert_eq!(envelope(&output)["meta"]["count"], 1);
    assert_eq!(std::fs::read(dest.join("a.png")).expect("saved"), b"PNG");

    let output = env
        .run(
            &[
                "--agent",
                "confluence",
                "page",
                "comment",
                "5",
                "Agreed",
                "--parent",
                "9",
            ],
            None,
        )
        .await;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );

    let bare = Env::new();
    let output = bare
        .run(&["convert", "--to", "storage"], Some("# T\n"))
        .await;
    assert_eq!(String::from_utf8_lossy(&output.stdout), "<h1>T</h1>\n");
    let output = bare
        .run(
            &["convert", "-", "--from", "storage", "--to", "markdown"],
            Some("<h1>T</h1>"),
        )
        .await;
    assert_eq!(String::from_utf8_lossy(&output.stdout), "# T\n");
    let output = bare
        .run(
            &["convert", "--from", "adf", "--to", "markdown"],
            Some("not json"),
        )
        .await;
    assert_eq!(output.status.code(), Some(3));
}

#[tokio::test(flavor = "multi_thread")]
async fn api_fields_headers_include_and_jq() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/rest/api/3/search/jql"))
        .and(query_param("jql", "project = AB"))
        .and(query_param("maxResults", "5"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("x-trace", "t1")
                .set_body_json(json!({ "issues": [{ "key": "AB-1" }, { "key": "AB-2" }] })),
        )
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/rest/api/3/issue"))
        .and(header("x-test", "1"))
        .and(wiremock::matchers::body_json(json!({ "fields": { "summary": "S", "project": { "key": "AB" }, "labels": ["a", "b"] }, "notify": false })))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({ "key": "AB-3" })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/rest/api/3/search/jql"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "issues": [
            { "key": "AB-1", "fields": { "summary": "Fix", "status": { "name": "Done", "statusCategory": { "key": "done" } }, "issuetype": { "name": "Bug" } } }
        ]})))
        .mount(&server)
        .await;
    let env = Env::new()
        .var("ATL_JIRA_URL", server.uri())
        .var("ATL_JIRA_TOKEN", "t")
        .var("ATL_JIRA_DEPLOYMENT", "cloud");

    let search = [
        "api",
        "jira",
        "get",
        "/rest/api/3/search/jql",
        "-f",
        "jql=project = AB",
        "-F",
        "maxResults=5",
    ];
    let output = env
        .run(&[&search[..], &["--jq", ".issues[].key"]].concat(), None)
        .await;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&output.stdout), "AB-1\nAB-2\n");

    let output = env.run(&[&search[..], &["-i"]].concat(), None).await;
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(text.starts_with("HTTP/1.1 200\n"), "{text}");
    assert!(text.contains("x-trace: t1\n"), "{text}");

    let output = env.run(&[&search[..], &["--silent"]].concat(), None).await;
    assert!(output.status.success() && output.stdout.is_empty());

    let output = env
        .run(
            &[
                "api",
                "jira",
                "post",
                "/rest/api/3/issue",
                "-f",
                "fields[summary]=S",
                "-F",
                "fields[project][key]=AB",
                "-f",
                "fields[labels][]=a",
                "-f",
                "fields[labels][]=b",
                "-F",
                "notify=false",
                "-H",
                "X-Test: 1",
            ],
            None,
        )
        .await;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let output = env
        .run(
            &[
                "--agent", "jira", "issue", "list", "-p", "AB", "--jq", ".[0].key",
            ],
            None,
        )
        .await;
    assert_eq!(envelope(&output)["data"], "AB-1");

    let output = env
        .run(&["api", "jira", "get", "/x", "-f", "a[b]=1"], None)
        .await;
    assert_eq!(output.status.code(), Some(3));
}

#[tokio::test(flavor = "multi_thread")]
async fn login_with_netrc_and_none_saves_no_secret() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/rest/api/2/myself"))
        .and(header("authorization", "Basic YWRhOnMzY3JldA=="))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!({ "name": "ada", "displayName": "Ada" })),
        )
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/rest/api/2/serverInfo"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "version": "10.3" })))
        .expect(1)
        .mount(&server)
        .await;
    let env = Env::new();
    let netrc = env.config.path().join("netrc");
    std::fs::write(&netrc, "machine 127.0.0.1 login ada password s3cret\n").expect("netrc");
    let env = env.var("NETRC", netrc.to_str().expect("utf-8"));

    let output = env
        .run(
            &[
                "--agent",
                "auth",
                "login",
                "--product",
                "jira",
                "--url",
                &server.uri(),
                "--method",
                "netrc",
            ],
            None,
        )
        .await;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert_eq!(
        envelope(&output)["data"]["connections"]["jira"]["account"],
        "Ada"
    );
    let saved = std::fs::read_to_string(env.config.path().join("config.toml")).expect("config");
    assert!(
        saved.contains("kind = \"netrc\"") && !saved.contains("s3cret"),
        "{saved}"
    );
    let output = env.run(&["--agent", "jira", "me"], None).await;
    assert_eq!(envelope(&output)["data"]["name"], "Ada");

    // A URL alone in the environment uses the netrc entry too.
    let bare = Env::new()
        .var("NETRC", netrc.to_str().expect("utf-8"))
        .var("ATL_JIRA_URL", server.uri());
    let output = bare.run(&["--agent", "jira", "me"], None).await;
    assert_eq!(envelope(&output)["data"]["name"], "Ada");

    let output = env
        .run(
            &[
                "--agent",
                "--profile",
                "public",
                "auth",
                "login",
                "--product",
                "jira",
                "--url",
                &server.uri(),
                "--method",
                "none",
            ],
            None,
        )
        .await;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert_eq!(
        envelope(&output)["data"]["connections"]["jira"]["account"],
        "anonymous"
    );
    let saved = std::fs::read_to_string(env.config.path().join("config.toml")).expect("config");
    assert!(saved.contains("kind = \"none\""), "{saved}");
}

#[tokio::test(flavor = "multi_thread")]
async fn debug_prints_masked_traffic_to_stderr_only() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/rest/api/3/myself"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!({ "accountId": "a1", "displayName": "Ada Lovelace" })),
        )
        .mount(&server)
        .await;
    let env = Env::new()
        .var("ATL_JIRA_URL", server.uri())
        .var("ATL_JIRA_TOKEN", "very-secret-token")
        .var("ATL_JIRA_DEPLOYMENT", "cloud");

    let output = env
        .run(&["--debug=full", "jira", "me", "--agent"], None)
        .await;
    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("atl: > GET "), "{stderr}");
    assert!(stderr.contains("/rest/api/3/myself"), "{stderr}");
    assert!(stderr.contains("atl: < 200 "), "{stderr}");
    assert!(stderr.contains("authorization: Bearer ***"), "{stderr}");
    assert!(stderr.contains("Ada Lovelace"), "{stderr}");
    assert!(!stderr.contains("very-secret-token"), "{stderr}");
    assert_eq!(envelope(&output)["ok"], true);

    let env = env.var("ATL_DEBUG", "1");
    let output = env.run(&["jira", "me", "--agent"], None).await;
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("atl: < 200 "), "{stderr}");
    assert!(
        !stderr.contains("Ada Lovelace"),
        "the short level prints no bodies: {stderr}"
    );

    let quiet = Env::new()
        .var("ATL_JIRA_URL", server.uri())
        .var("ATL_JIRA_TOKEN", "t")
        .var("ATL_JIRA_DEPLOYMENT", "cloud");
    let output = quiet.run(&["jira", "me", "--agent"], None).await;
    assert!(output.stderr.is_empty());
}

#[tokio::test(flavor = "multi_thread")]
async fn auth_token_prints_one_products_secret() {
    let env = Env::new();
    for (product, url, token) in [
        ("jira", "https://jira.example.com", "jira-pat"),
        ("bitbucket", "https://git.example.com", "bb-pat"),
    ] {
        let output = env
            .run(
                &[
                    "auth",
                    "login",
                    "--product",
                    product,
                    "--url",
                    url,
                    "--token-stdin",
                    "--no-verify",
                ],
                Some(token),
            )
            .await;
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let output = env
        .run(&["auth", "token", "jira", "--format", "table"], None)
        .await;
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "jira-pat");
    let reply = envelope(
        &env.run(&["auth", "token", "bitbucket", "--agent"], None)
            .await,
    );
    assert_eq!(
        (
            reply["data"]["kind"].clone(),
            reply["data"]["token"].clone()
        ),
        (json!("bearer"), json!("bb-pat"))
    );

    let output = env.run(&["auth", "token", "--agent"], None).await;
    assert_eq!(output.status.code(), Some(3));
    let reply = envelope(&output);
    assert_eq!(reply["error"]["code"], "input_required");
    assert_eq!(
        reply["error"]["input"]["choices"],
        json!([{ "value": "jira", "label": "jira" }, { "value": "bitbucket", "label": "bitbucket" }])
    );

    let output = env.run(&["auth", "token", "trello", "--agent"], None).await;
    assert_eq!(output.status.code(), Some(3));
    let message = envelope(&output)["error"]["message"]
        .as_str()
        .unwrap_or_default()
        .to_owned();
    assert!(message.contains("jira, bitbucket"), "{message}");
}

#[tokio::test(flavor = "multi_thread")]
async fn pr_checks_exit_1_when_a_build_failed() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/2.0/repositories/acme/web/pullrequests/7/statuses"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "values": [
            { "key": "build", "name": "Build", "state": "SUCCESSFUL" },
            { "key": "lint", "name": "Lint", "state": "FAILED", "url": "https://ci/2" }
        ] })))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/2.0/repositories/acme/web/pullrequests/8/statuses"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "values": [
            { "key": "build", "name": "Build", "state": "INPROGRESS" }
        ] })))
        .mount(&server)
        .await;
    let env = Env::new()
        .var("ATL_BITBUCKET_URL", server.uri())
        .var("ATL_BITBUCKET_TOKEN", "t")
        .var("ATL_BITBUCKET_DEPLOYMENT", "cloud");

    let output = env
        .run(
            &[
                "bitbucket",
                "pr",
                "checks",
                "7",
                "-R",
                "acme/web",
                "--agent",
            ],
            None,
        )
        .await;
    assert_eq!(output.status.code(), Some(1));
    let reply = envelope(&output);
    assert_eq!(reply["data"]["passed"], false);
    assert_eq!(reply["data"]["checks"][1]["url"], "https://ci/2");

    let output = env
        .run(
            &[
                "bitbucket",
                "pr",
                "checks",
                "8",
                "-R",
                "acme/web",
                "--format",
                "table",
            ],
            None,
        )
        .await;
    assert_eq!(output.status.code(), Some(0));
    assert!(String::from_utf8_lossy(&output.stdout).contains("INPROGRESS"));
}

#[tokio::test(flavor = "multi_thread")]
async fn bulk_move_reports_each_key_and_exits_1_on_a_failure() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/rest/api/3/issue/AB-1/transitions"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({ "transitions": [
            { "id": "31", "name": "Finish", "to": { "name": "Done" } }
        ] })),
        )
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/rest/api/3/issue/AB-2/transitions"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({ "transitions": [
            { "id": "11", "name": "Start", "to": { "name": "In Progress" } }
        ] })),
        )
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/rest/api/3/issue/AB-1/transitions"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;
    let env = Env::new()
        .var("ATL_JIRA_URL", server.uri())
        .var("ATL_JIRA_TOKEN", "t")
        .var("ATL_JIRA_DEPLOYMENT", "cloud");

    let output = env
        .run(
            &["jira", "issue", "move", "AB-1", "ab-2", "Done", "--agent"],
            None,
        )
        .await;
    assert_eq!(output.status.code(), Some(1));
    let rows = envelope(&output)["data"].clone();
    assert_eq!(
        (
            rows[0]["key"].clone(),
            rows[0]["ok"].clone(),
            rows[0]["status"].clone()
        ),
        (json!("AB-1"), json!(true), json!("Done"))
    );
    assert_eq!(
        (rows[1]["key"].clone(), rows[1]["ok"].clone()),
        (json!("AB-2"), json!(false))
    );
    assert!(
        rows[1]["error"]
            .as_str()
            .unwrap_or_default()
            .contains("In Progress"),
        "{rows}"
    );

    let output = env
        .run(
            &["jira", "issue", "move", "AB-1", "In", "Progress", "--agent"],
            None,
        )
        .await;
    assert_eq!(output.status.code(), Some(3));
    assert!(
        envelope(&output)["error"]["message"]
            .as_str()
            .unwrap_or_default()
            .contains("quote values with spaces")
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn aliases_expand_before_parsing_and_cannot_shadow_commands() {
    let env = Env::new()
        .var("ATL_JIRA_URL", "https://acme.atlassian.net")
        .var("ATL_JIRA_TOKEN", "t")
        .var("ATL_JIRA_DEPLOYMENT", "cloud");
    let output = env
        .run(
            &["alias", "set", "mine", "jira issue list -a me -s '~Done'"],
            None,
        )
        .await;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let reply = envelope(
        &env.run(&["--agent", "--dry-run", "mine", "-p", "AB"], None)
            .await,
    );
    assert_eq!(reply["command"], "jira issue list");
    assert_eq!(
        reply["data"]["jql"],
        "project = \"AB\" AND status NOT IN (\"Done\") AND assignee = currentUser() ORDER BY updated DESC"
    );
    let output = env.run(&["alias", "set", "jira", "issue list"], None).await;
    assert_eq!(output.status.code(), Some(3));
    let rows = envelope(&env.run(&["alias", "list", "--agent"], None).await)["data"].clone();
    assert_eq!(
        rows,
        json!([{ "name": "mine", "command": "jira issue list -a me -s '~Done'" }])
    );
    assert!(
        env.run(&["alias", "delete", "mine"], None)
            .await
            .status
            .success()
    );
    let output = env.run(&["mine", "--agent"], None).await;
    assert_eq!(
        output.status.code(),
        Some(4),
        "an unknown command once the alias is gone"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn man_pages_cover_commands_but_not_the_api_tree() {
    let env = Env::new();
    let dir = tempfile::tempdir().expect("tempdir");
    let output = env
        .run(
            &["man", "--dir", dir.path().to_str().expect("utf-8 path")],
            None,
        )
        .await;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let names: Vec<String> = std::fs::read_dir(dir.path())
        .expect("read dir")
        .filter_map(|e| e.ok()?.file_name().into_string().ok())
        .collect();
    for wanted in [
        "atl.1",
        "atl-jira-issue-list.1",
        "atl-bitbucket-pr-checks.1",
    ] {
        assert!(
            names.iter().any(|n| n == wanted),
            "{wanted} missing from {names:?}"
        );
    }
    assert!(!names.iter().any(|n| n.starts_with("atl-api")), "{names:?}");
    let page = std::fs::read_to_string(dir.path().join("atl-jira-issue-list.1")).expect("page");
    assert!(page.contains("atl\\-jira\\-issue\\-list"), "{page}");
}

#[tokio::test(flavor = "multi_thread")]
async fn issue_list_web_returns_the_search_url_to_agents() {
    let env = Env::new()
        .var("ATL_JIRA_URL", "https://acme.atlassian.net")
        .var("ATL_JIRA_TOKEN", "t")
        .var("ATL_JIRA_DEPLOYMENT", "cloud");
    let reply = envelope(
        &env.run(
            &["jira", "issue", "list", "-p", "AB", "--web", "--agent"],
            None,
        )
        .await,
    );
    assert_eq!(
        reply["data"]["url"],
        "https://acme.atlassian.net/issues/?jql=project+%3D+%22AB%22+ORDER+BY+updated+DESC"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn skill_prints_lists_and_installs_the_skills() {
    let env = Env::new();
    let core = env.run(&["skill"], None).await;
    assert!(String::from_utf8_lossy(&core.stdout).starts_with("---\nname: atl\n"));
    let jira = env.run(&["skill", "atl-jira"], None).await;
    assert!(String::from_utf8_lossy(&jira.stdout).contains("atl jira issue list"));
    let list = stdout_json(&env.run(&["skill", "--list"], None).await);
    let names: Vec<&str> = list
        .as_array()
        .expect("rows")
        .iter()
        .filter_map(|r| r["name"].as_str())
        .collect();
    assert_eq!(
        names,
        [
            "atl",
            "atl-admin",
            "atl-bitbucket",
            "atl-confluence",
            "atl-jira",
            "atl-jsm",
            "atl-trello"
        ]
    );
    let unknown = env.run(&["skill", "atl-nope"], None).await;
    assert_eq!(unknown.status.code(), Some(3));
    assert!(String::from_utf8_lossy(&unknown.stderr).contains("use one of: atl, atl-admin"));
    let dir = tempfile::tempdir().expect("tempdir");
    let installed = env
        .run(
            &["skill", "--install", dir.path().to_str().expect("utf-8")],
            None,
        )
        .await;
    assert!(installed.status.success());
    assert!(dir.path().join("atl-jsm/SKILL.md").is_file());
}

fn command_names(help: &str) -> Vec<String> {
    help.lines()
        .skip_while(|line| *line != "Commands:")
        .skip(1)
        .take_while(|line| !line.is_empty())
        .filter(|line| line.starts_with("  ") && !line.starts_with("   "))
        .filter_map(|line| line.split_whitespace().next().map(str::to_owned))
        .filter(|name| name != "help")
        .collect()
}

#[tokio::test(flavor = "multi_thread")]
async fn help_lists_commands_alphabetically_at_every_level() {
    let env = Env::new();
    for args in [
        &["--help"][..],
        &["jira", "--help"],
        &["jira", "issue", "--help"],
        &["confluence", "page", "--help"],
        &["bitbucket", "pr", "--help"],
        &["auth", "--help"],
        &["api", "jira", "--help"],
    ] {
        let output = env.run(args, None).await;
        let names = command_names(&String::from_utf8_lossy(&output.stdout));
        let mut sorted = names.clone();
        sorted.sort();
        assert!(names.len() > 2, "atl {args:?} lists commands");
        assert_eq!(names, sorted, "atl {}", args.join(" "));
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn bare_commands_print_help_for_people_and_fail_for_agents() {
    let env = Env::new();
    for args in [&[][..], &["jira"], &["jira", "issue"]] {
        let output = env.run(args, None).await;
        assert_eq!(output.status.code(), Some(0), "atl {args:?}");
        assert!(
            String::from_utf8_lossy(&output.stdout).contains("Usage:"),
            "atl {args:?} prints help on stdout"
        );
    }
    let agent = env.run(&["--agent", "jira"], None).await;
    assert_eq!(agent.status.code(), Some(3));
    assert_eq!(stdout_json(&agent)["error"]["code"], "invalid");
}

#[tokio::test(flavor = "multi_thread")]
async fn dry_runs_without_a_login_show_the_first_request() {
    let env = Env::new();
    let cloud = stdout_json(
        &env.run(&["jira", "issue", "view", "AB-1", "--dry-run"], None)
            .await,
    );
    assert_eq!(cloud["method"], "GET");
    assert!(
        cloud["url"]
            .as_str()
            .expect("url")
            .starts_with("https://your-domain.atlassian.net/rest/api/3/issue/AB-1"),
        "{cloud}"
    );
    assert!(
        cloud["warning"]
            .as_str()
            .expect("warning")
            .contains("atl auth login"),
        "{cloud}"
    );
    let dc = stdout_json(
        &env.run(
            &[
                "jira",
                "issue",
                "view",
                "AB-1",
                "--dry-run",
                "--deployment",
                "data-center",
            ],
            None,
        )
        .await,
    );
    assert!(
        dc["url"]
            .as_str()
            .expect("url")
            .starts_with("https://jira.example.com/rest/api/2/issue/AB-1"),
        "{dc}"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn csv_quotes_only_when_needed_and_tsv_never_quotes() {
    let env = Env::new();
    let csv = env.run(&["services", "--format", "csv"], None).await;
    let csv = String::from_utf8_lossy(&csv.stdout);
    let mut lines = csv.lines();
    assert_eq!(lines.next(), Some("SERVICE,CLOUD OPS,DC OPS,COVERS"));
    assert!(
        lines.any(|l| l.starts_with("jira,")
            && l.ends_with(
                "\"Jira platform: issues, projects, workflows, users, fields, search\""
            )),
        "{csv}"
    );
    let tsv = env
        .run(&["services", "--format", "tsv", "--no-headers"], None)
        .await;
    let tsv = String::from_utf8_lossy(&tsv.stdout);
    assert!(!tsv.contains('"'), "{tsv}");
    assert!(tsv.lines().all(|l| l.split('\t').count() == 4), "{tsv}");
}

#[tokio::test(flavor = "multi_thread")]
async fn mcp_full_mode_lists_and_calls_one_tool_per_operation() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/rest/api/3/issue/AB-1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "key": "AB-1" })))
        .expect(1)
        .mount(&server)
        .await;
    let env = Env::new()
        .var("ATL_JIRA_URL", server.uri())
        .var("ATL_JIRA_TOKEN", "t")
        .var("ATL_JIRA_DEPLOYMENT", "cloud");
    let input = [
        json!({ "jsonrpc": "2.0", "id": 1, "method": "initialize", "params": { "protocolVersion": "2025-06-18", "capabilities": {}, "clientInfo": { "name": "test", "version": "0" } } }),
        json!({ "jsonrpc": "2.0", "method": "notifications/initialized" }),
        json!({ "jsonrpc": "2.0", "id": 2, "method": "tools/list" }),
        json!({ "jsonrpc": "2.0", "id": 3, "method": "tools/call", "params": { "name": "jira_getIssue", "arguments": { "issueIdOrKey": "AB-1" } } }),
        json!({ "jsonrpc": "2.0", "id": 4, "method": "tools/call", "params": { "name": "jira_nope", "arguments": {} } }),
    ]
    .map(|m| m.to_string())
    .join("\n")
        + "\n";
    let output = env
        .run(
            &[
                "mcp",
                "--mode",
                "full",
                "--services",
                "jira",
                "--resources",
                "issues",
                "--no-tasks",
            ],
            Some(&input),
        )
        .await;
    let mut replies: Vec<Value> = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|l| serde_json::from_str(l).expect("JSON-RPC line"))
        .collect();
    replies.sort_by_key(|reply| reply["id"].as_u64());
    assert_eq!(replies.len(), 4, "{replies:#?}");
    assert!(
        replies[0]["result"]["instructions"]
            .as_str()
            .expect("instructions")
            .contains("jira")
    );
    let tools = replies[1]["result"]["tools"].as_array().expect("tools");
    assert!(tools.iter().any(|t| t["name"] == "jira_getIssue"));
    assert!(
        tools
            .iter()
            .all(|t| !t["name"].as_str().unwrap_or_default().starts_with("atl_")),
        "--no-tasks"
    );
    let get = tools
        .iter()
        .find(|t| t["name"] == "jira_getIssue")
        .expect("getIssue");
    assert_eq!(get["annotations"]["readOnlyHint"], true);
    assert_eq!(get["inputSchema"]["type"], "object");
    let called = &replies[2]["result"];
    assert_eq!(called["isError"], false, "{called}");
    assert!(
        called["content"][0]["text"]
            .as_str()
            .expect("text")
            .contains("AB-1")
    );
    let unknown = &replies[3]["result"];
    assert_eq!(unknown["isError"], true);
    assert!(
        unknown["content"][0]["text"]
            .as_str()
            .expect("text")
            .contains("tools/list")
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn a_broken_netrc_file_is_an_error_that_names_it() {
    let dir = tempfile::tempdir().expect("tempdir");
    let netrc = dir.path().join("netrc");
    std::fs::write(&netrc, "machine jira.corp.example login\n").expect("netrc");
    let env = Env::new()
        .var("NETRC", netrc.display().to_string())
        .var("ATL_JIRA_URL", "https://jira.corp.example")
        .var("ATL_JIRA_DEPLOYMENT", "data-center");
    let output = env.run(&["--agent", "jira", "me"], None).await;
    let reply = stdout_json(&output);
    assert_eq!(reply["ok"], false, "{reply}");
    assert!(
        reply["error"]["message"]
            .as_str()
            .expect("message")
            .contains("netrc"),
        "{reply}"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn mcp_hides_writes_for_read_only_logins_and_marks_destructive_tools() {
    let list = |env: Env| async move {
        let input = [
            json!({ "jsonrpc": "2.0", "id": 1, "method": "initialize", "params": { "protocolVersion": "2025-06-18", "capabilities": {}, "clientInfo": { "name": "test", "version": "0" } } }),
            json!({ "jsonrpc": "2.0", "method": "notifications/initialized" }),
            json!({ "jsonrpc": "2.0", "id": 2, "method": "tools/list" }),
        ]
        .map(|m| m.to_string())
        .join("\n")
            + "\n";
        let output = env
            .run(&["mcp", "--services", "jira,bitbucket"], Some(&input))
            .await;
        let reply: Value = String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(|l| serde_json::from_str::<Value>(l).expect("JSON-RPC line"))
            .find(|m| m["id"] == 2)
            .expect("tools/list reply");
        reply["result"]["tools"].as_array().expect("tools").clone()
    };
    let tools = list(Env::new()).await;
    let tool = |name: &str| {
        tools
            .iter()
            .find(|t| t["name"] == name)
            .cloned()
            .unwrap_or(Value::Null)
    };
    assert_eq!(
        tool("atl_bitbucket_pr_merge")["annotations"]["destructiveHint"],
        true
    );
    assert_eq!(
        tool("atl_jira_issue_delete")["annotations"]["destructiveHint"],
        true
    );
    assert_eq!(
        tool("atl_jira_issue_create")["annotations"]["destructiveHint"],
        false
    );
    let read_only = list(Env::new().var("ATL_READ_ONLY", "1")).await;
    let names: Vec<&str> = read_only
        .iter()
        .filter_map(|t| t["name"].as_str())
        .collect();
    assert!(names.contains(&"atl_jira_issue_list"), "{names:?}");
    assert!(
        !names.contains(&"atl_jira_issue_create"),
        "a read-only login hides writes: {names:?}"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn a_token_without_a_url_is_not_sent_anywhere() {
    let env = Env::new().var("ATL_JIRA_TOKEN", "secret-token");
    let output = env
        .run(
            &["--agent", "jira", "issue", "view", "AB-1", "--dry-run"],
            None,
        )
        .await;
    let reply = stdout_json(&output);
    assert!(
        reply["data"]["warning"]
            .as_str()
            .unwrap_or_default()
            .contains("ATL_JIRA_URL"),
        "the dry run shows the missing login: {reply}"
    );
    assert!(
        !output.stdout.windows(12).any(|w| w == b"secret-token"),
        "the token is not used"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn a_repository_default_in_the_wrong_table_is_an_error() {
    let repo = tempfile::tempdir().expect("repo");
    std::fs::write(repo.path().join(".atl.toml"), "[jira]\ndesk = \"HELP\"\n").expect("write");
    let mut env = Env::new();
    env.cwd = Some(repo.path().to_path_buf());
    let output = env
        .run(&["--agent", "jira", "issue", "list", "--dry-run"], None)
        .await;
    assert_eq!(
        output.status.code(),
        Some(3),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
    let reply = stdout_json(&output);
    assert!(
        reply["error"]["message"]
            .as_str()
            .expect("message")
            .contains("[jira] project"),
        "{reply}"
    );
}
