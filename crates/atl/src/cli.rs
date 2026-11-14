//! Command-line parsing and dispatch.
//!
//! Product commands (`atl jira issue list`, `atl confluence page get`, ...)
//! and the tool commands (`auth`, `services`, `schema`, `mcp`) use clap
//! derive. Generated commands (`atl api jira issues get-issue ...`) are
//! built at runtime from the catalog. Only the service named on the command
//! line gets its resource tree, and only the named method gets its
//! parameter flags.

use crate::Exit;
use crate::login::{self, AuthMethod, LoginRequest, Product};
use crate::output::{self, Format, Out, col, flex};
use crate::tasks::{self, Ctx, RepoConfig};
use anyhow::{Context, Result};
use atl_sdk::catalog::{self, Catalog, Operation, ServiceDef};
use atl_sdk::{
    CallArgs, Client, Config, Deployment, Error, Family, HttpMethod, Pager, PreparedRequest,
    Response, schema,
};
use clap::{
    Arg, ArgAction, ArgMatches, Args, Command, CommandFactory, FromArgMatches, Parser, Subcommand,
};
use serde_json::{Map, Value, json};
use std::io::{IsTerminal, Read, Write};
use std::path::PathBuf;
use std::time::Duration;

#[derive(Parser, Debug)]
#[command(
    name = "atl",
    version,
    about = "One CLI for Jira, Confluence, Bitbucket, Trello and Atlassian Admin, on Cloud and Data Center",
    long_about = "One CLI for Jira, Confluence, Bitbucket, Trello and Atlassian Admin, on Cloud and Data Center.\n\n\
        Everyday tasks, as atl <product> <resource> <verb>:\n  \
        atl jira issue list|view|create|move|comment\n  \
        atl confluence page get|create|update, atl confluence search\n  \
        atl bitbucket pr list|view|create|merge, atl jsm request list|create|comment\n  \
        atl trello card list|create|move, atl admin user list|suspend\n  \
        atl browse (full-screen browser)\n\
        Every API operation, generated from Atlassian's OpenAPI documents:\n  \
        atl api <service> <resource> <method> [--param-flags] [--params JSON] [--json BODY]\n  \
        atl api <service> get|post|put|patch|delete <path> [--params JSON] [--json BODY]\n\n\
        Start with `atl auth login`. Agents: add --agent (or set ATL_AGENT=1) and read `atl skill`.\n\
        Output: text and tables in a terminal (`--browse` on issue lists opens the browser), JSON when piped, one JSON envelope with --agent.\n\
        Exit codes: 0 ok, 1 API error, 2 auth error, 3 invalid input, 4 unknown command, 5 other failure.",
    subcommand_required = true,
    arg_required_else_help = true
)]
struct Cli {
    #[command(flatten)]
    global: GlobalArgs,
    #[command(subcommand)]
    command: Static,
}

#[derive(Args, Debug, Clone)]
struct GlobalArgs {
    /// Credential profile (default: ATL_PROFILE, then `default_profile` in the config file, then "default").
    #[arg(long, global = true, value_name = "NAME")]
    profile: Option<String>,
    /// Use the Cloud or Data Center command set regardless of the stored credential.
    #[arg(long, global = true, value_name = "cloud|data-center")]
    deployment: Option<String>,
    /// Print the HTTP request as JSON instead of sending it. Task commands still
    /// send the reads they need and stop at the first write.
    #[arg(long, global = true)]
    dry_run: bool,
    /// Follow pagination and print every page as one JSON line (NDJSON).
    #[arg(long, global = true)]
    page_all: bool,
    /// Maximum number of pages with --page-all.
    #[arg(long, global = true, default_value_t = 10, value_name = "N")]
    page_limit: usize,
    /// Milliseconds to wait between pages with --page-all.
    #[arg(long, global = true, default_value_t = 100, value_name = "MS")]
    page_delay: u64,
    /// Write the response body to a file (for attachments and other downloads).
    #[arg(long, short = 'o', global = true, value_name = "PATH")]
    output: Option<PathBuf>,
    /// Agent mode: one compact JSON envelope on stdout (errors too), no prompts, no TUI, lean responses.
    #[arg(long, global = true, env = "ATL_AGENT", value_parser = clap::builder::BoolishValueParser::new(), num_args = 0, default_missing_value = "true", default_value = "false")]
    agent: bool,
    /// Output format.
    #[arg(long, global = true, value_enum, default_value_t = Format::Auto)]
    format: Format,
    /// Keep only these fields, and show them as the table/CSV columns (comma-separated, dot paths allowed: key,fields.summary).
    #[arg(
        long,
        global = true,
        visible_alias = "columns",
        value_delimiter = ',',
        value_name = "FIELDS"
    )]
    select: Vec<String>,
    /// Filter the output data with a jq expression (results one per line; in agent mode, the envelope data).
    #[arg(long, global = true, value_name = "EXPR")]
    jq: Option<String>,
    /// Print long text directly instead of through $ATL_PAGER or $PAGER (default less -FRX).
    #[arg(long, global = true)]
    no_pager: bool,
    /// Omit the header row of tables, CSV and TSV.
    #[arg(long, global = true)]
    no_headers: bool,
    /// Do not shorten table cells to fit the terminal.
    #[arg(long, global = true)]
    no_truncate: bool,
    /// With --agent, return API responses unmodified (no lean cleaning).
    #[arg(long, global = true)]
    raw: bool,
    /// Skip confirmation of destructive actions (required for DELETE outside a terminal).
    #[arg(long, global = true)]
    yes: bool,
    /// Print each HTTP request and response to stderr; --debug=full adds headers and bodies (secrets masked). Also ATL_DEBUG=1|full.
    #[arg(long, global = true, value_name = "full", num_args = 0..=1, require_equals = true, default_missing_value = "requests", value_parser = ["requests", "full"])]
    debug: Option<String>,
    /// Never ask questions; fail with the missing value and its choices instead (agent mode implies it).
    #[arg(long, global = true, env = "ATL_NO_INPUT", value_parser = clap::builder::BoolishValueParser::new(), num_args = 0, default_missing_value = "true", default_value = "false")]
    no_input: bool,
}

/// Global flags that take a value; used to find positionals before clap runs.
const VALUE_FLAGS: &[&str] = &[
    "--profile",
    "--deployment",
    "--page-limit",
    "--page-delay",
    "--output",
    "-o",
    "--format",
    "--select",
    "--columns",
    "--jq",
];

// Parsed once per process.
#[allow(clippy::large_enum_variant)]
#[derive(Subcommand, Debug)]
enum Static {
    /// Jira: attachment, board, comment, component, epic, field, filter, issue, jql, me, project, release, sprint.
    #[command(subcommand)]
    Jira(crate::jira::JiraCommand),
    /// Confluence: attachment, blog, comment, page, search, space.
    #[command(subcommand)]
    Confluence(crate::confluence::ConfluenceCommand),
    /// Bitbucket: branch, commit, pipeline, pr, repo.
    Bitbucket(crate::bitbucket::BitbucketArgs),
    /// Jira Service Management: desk, queue, request.
    #[command(subcommand)]
    Jsm(crate::jsm::JsmCommand),
    /// Trello: board, card, checklist, list.
    #[command(subcommand)]
    Trello(crate::trello::TrelloCommand),
    /// Atlassian Admin: directory, domain, event, group, org, user.
    Admin(crate::admin::AdminArgs),
    /// Sign in, inspect, and remove stored credentials.
    #[command(subcommand)]
    Auth(AuthCommand),
    /// List services with their Cloud and Data Center operation counts.
    Services,
    /// Describe an operation's parameters, body, and response, or a named schema.
    Schema {
        /// `<service>.<operationId>`, `<service>.<resource>.<method>`, or `<service>.<SchemaName>`.
        target: String,
        /// How many levels of `$ref` to inline.
        #[arg(long, default_value_t = 2)]
        depth: usize,
    },
    /// Run a Model Context Protocol server on stdin/stdout.
    Mcp(McpArgs),
    /// Open an issue, page, or URL in the browser (default: the issue in the git branch name).
    Open {
        /// Issue key, page ID/URL/SPACE:Title, or URL.
        target: Option<String>,
    },
    /// Print the shell completion script: `source <(atl completions zsh)` (bash, zsh, fish, elvish, powershell).
    Completions {
        /// Shell to print the script for.
        shell: clap_complete::Shell,
    },
    /// Browse Jira, Confluence and the API catalog in a full-screen terminal interface.
    Browse,
    /// Update atl to the latest release (Homebrew installs: brew upgrade racetozero/tap/atl).
    Update,
    /// Convert rich text offline: Markdown, ADF (Jira Cloud), storage (Confluence), wiki (Jira Data Center).
    Convert {
        /// Input file, or - (the default) for stdin.
        input: Option<PathBuf>,
        /// Format of the input (default: from the file extension, else markdown).
        #[arg(long, value_enum, default_value_t = Markup::Markdown)]
        from: Markup,
        /// Format of the output.
        #[arg(long, value_enum)]
        to: Markup,
    },
    /// Run any task with JSON arguments: the same tasks the MCP server and the SDKs expose.
    #[command(subcommand)]
    Task(TaskCommand),
    /// Shortcuts for commands: `atl alias set mine "jira issue list -a me"`, then `atl mine`.
    #[command(subcommand)]
    Alias(AliasCommand),
    /// Write man pages (atl.1, atl-jira-issue-list.1, ...) into a directory.
    Man {
        /// For example ~/.local/share/man/man1.
        #[arg(long)]
        dir: PathBuf,
    },
    /// Print an agent skill (Claude Code, Codex and others), list them, or install them all.
    Skill {
        /// Skill to print: atl (the default), atl-jira, atl-confluence, atl-bitbucket, atl-jsm, atl-trello or atl-admin.
        name: Option<String>,
        /// List the skills with their descriptions.
        #[arg(long, conflicts_with_all = ["name", "install"])]
        list: bool,
        /// Write every skill to DIR/<name>/SKILL.md (default .claude/skills).
        #[arg(long, num_args = 0..=1, default_missing_value = ".claude/skills", value_name = "DIR", conflicts_with = "name")]
        install: Option<PathBuf>,
    },
}

#[derive(Subcommand, Debug)]
enum AliasCommand {
    /// Aliases and what they expand to.
    #[command(visible_alias = "ls")]
    List,
    /// Add or replace an alias; COMMAND is the words after `atl`, quoted as one argument.
    Set {
        /// Alias name.
        name: String,
        /// Command to run, without `atl`, for example "jira issue list -a me".
        command: String,
    },
    /// Remove an alias.
    Delete {
        /// Alias name.
        name: String,
    },
}

/// Replaces the first command word with its alias's words, unless it names
/// a built-in command.
fn expand_alias(
    mut argv: Vec<String>,
    aliases: &std::collections::BTreeMap<String, String>,
) -> Result<Vec<String>> {
    if aliases.is_empty() {
        return Ok(argv);
    }
    let mut skip_next = false;
    let mut at = None;
    for (i, arg) in argv.iter().enumerate().skip(1) {
        if skip_next {
            skip_next = false;
            continue;
        }
        if arg == "--" {
            break;
        }
        if arg.starts_with('-') {
            skip_next = VALUE_FLAGS.contains(&arg.as_str());
            continue;
        }
        at = Some(i);
        break;
    }
    let Some(at) = at else {
        return Ok(argv);
    };
    let name = argv[at].clone();
    let Some(text) = aliases.get(&name) else {
        return Ok(argv);
    };
    if is_builtin(&name) {
        return Ok(argv);
    }
    let words = shlex::split(text).ok_or_else(|| {
        Error::Invalid(format!(
            "alias `{name}` has an unclosed quote: {text}; fix it with `atl alias set {name} \"...\"`"
        ))
    })?;
    argv.splice(at..=at, words);
    Ok(argv)
}

fn is_builtin(name: &str) -> bool {
    name == "api" || Cli::command().find_subcommand(name).is_some()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
enum Markup {
    Markdown,
    /// Atlassian Document Format (JSON), Jira Cloud rich text.
    Adf,
    /// Confluence storage format (XHTML).
    Storage,
    /// Jira wiki markup (Jira Data Center).
    Wiki,
}

#[derive(Subcommand, Debug)]
enum TaskCommand {
    /// Task names, whether they write, and what they do.
    #[command(visible_alias = "ls")]
    List,
    /// One task's description and the JSON Schema of its arguments.
    Describe {
        /// Task name, from `atl task list`.
        name: String,
    },
    /// Run a task: ARGS is a JSON object, @file, or - for stdin (default {}).
    Run {
        /// Task name, from `atl task list`.
        name: String,
        /// Arguments as one JSON object: inline, @file, or - for stdin.
        args: Option<String>,
    },
}

fn find_task(name: &str) -> Result<&'static atl_sdk::tasks::TaskDef, Error> {
    atl_sdk::tasks::find(name).ok_or_else(|| {
        Error::Unknown(format!(
            "unknown task `{name}`; list them with `atl task list`"
        ))
    })
}

#[derive(Args, Debug)]
struct McpArgs {
    /// Comma-separated services to expose (default: all).
    #[arg(long, short = 's', value_delimiter = ',')]
    services: Vec<String>,
    /// `compact`: three tools (search, describe, call). `full`: one tool per operation.
    #[arg(long, default_value = "compact", value_parser = ["compact", "full"])]
    mode: String,
    /// Comma-separated resources to expose in full mode (for example issues,issue-search).
    #[arg(long, value_delimiter = ',')]
    resources: Vec<String>,
    /// Expose and allow only GET, HEAD and OPTIONS operations.
    #[arg(long)]
    read_only: bool,
    /// Do not expose the task tools (atl_jira_issue_list, atl_confluence_page_get, ...).
    #[arg(long)]
    no_tasks: bool,
}

#[derive(Subcommand, Debug)]
enum AuthCommand {
    /// Sign in with an interactive form, or non-interactively with --product and --url.
    Login(LoginArgs),
    /// Show profiles and stored credentials (secrets masked).
    Status {
        /// Call each server to check the credential.
        #[arg(long)]
        verify: bool,
    },
    /// Remove one product's credential, or the whole profile.
    Logout {
        /// Product family to remove (jira, confluence, bitbucket, trello, admin, admin-scim). Omit to remove the profile.
        #[arg(long)]
        product: Option<String>,
    },
    /// Print one product's stored token (or password) for scripts: `curl -u "me:$(atl auth token jira)"`.
    Token {
        /// jira, confluence, bitbucket, trello, admin or admin-scim (optional when the profile has one product).
        product: Option<String>,
    },
    /// Make a profile the default.
    Default {
        /// Profile name.
        profile: String,
    },
}

#[derive(Args, Debug)]
struct LoginArgs {
    /// Skip the form: product to sign in to.
    #[arg(long, value_enum)]
    product: Option<Product>,
    /// Site or server URL, for example https://acme.atlassian.net or https://jira.corp.example.
    #[arg(long)]
    url: Option<String>,
    /// Authentication method (default depends on product and deployment).
    #[arg(long, value_enum)]
    method: Option<AuthMethod>,
    /// Email (Cloud), username (Data Center), or API key (Trello).
    #[arg(long)]
    user: Option<String>,
    /// Read the token, password, or API key from stdin.
    #[arg(long)]
    token_stdin: bool,
    /// Save without calling the server.
    #[arg(long)]
    no_verify: bool,
    /// Store the credential as read-only: atl refuses every write with it.
    #[arg(long)]
    read_only: bool,
    /// --method oauth: local port of the callback http://localhost:PORT/callback (must match the app).
    #[arg(long, default_value_t = login::OAUTH_PORT)]
    callback_port: u16,
    /// --method oauth: space-separated scopes instead of the defaults (see docs/auth.md).
    #[arg(long)]
    scopes: Option<String>,
    /// PEM client certificate for servers that require mutual TLS (it may also hold the key).
    #[arg(long, value_name = "PATH")]
    client_cert: Option<PathBuf>,
    /// PEM private key of --client-cert.
    #[arg(long, value_name = "PATH")]
    client_key: Option<PathBuf>,
    /// PEM CA certificates to trust in addition to the system ones (private company CAs).
    #[arg(long, value_name = "PATH")]
    ca_cert: Option<PathBuf>,
}

pub async fn run(argv: Vec<String>) -> Result<Exit> {
    output::set_agent(
        argv.iter().any(|a| a == "--agent") || atl_sdk::config::env_flag("ATL_AGENT"),
    );
    // The profile and deployment decide which catalog builds the command
    // tree, so read them before clap parses the full command line.
    let config = Config::load()?;
    let argv = expand_alias(argv, &config.aliases)?;
    let repo = RepoConfig::discover()?;
    // Profile precedence: --profile, ATL_PROFILE, .atl.toml, config default.
    let profile = flag_value(&argv, "--profile")
        .map(str::to_owned)
        .or_else(|| std::env::var("ATL_PROFILE").ok().filter(|p| !p.is_empty()))
        .or_else(|| repo.profile.clone());
    let mut client = Client::new(&config, profile.as_deref());
    client.set_deployment_override(
        flag_value(&argv, "--deployment")
            .map(str::parse)
            .transpose()?,
    );

    let positionals = positionals(&argv[1.min(argv.len())..]);
    let mut api = api_command();
    for def in catalog::services() {
        let selected = positionals.first() == Some(&"api") && positionals.get(1) == Some(&def.name);
        api = api.subcommand(service_command(
            def,
            client.deployment(def.family),
            selected.then(|| &positionals[2..]),
        ));
    }
    let mut command = grouped_help(alphabetical(Cli::command().subcommand(api)));
    let matches = match command.try_get_matches_from_mut(&argv) {
        Ok(matches) => matches,
        Err(err) if !err.use_stderr() => {
            let _ = err.print(); // --help and --version
            return Ok(Exit::Ok);
        }
        // `atl` or `atl jira` alone: people asked for the help, so print it
        // like --help (stdout, exit 0). Agents still get an error.
        Err(err)
            if err.kind() == clap::error::ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand
                && !output::agent_mode() =>
        {
            let help = argv.iter().cloned().chain(["--help".to_owned()]);
            if let Err(help) = command.try_get_matches_from(help) {
                let _ = help.print();
            }
            return Ok(Exit::Ok);
        }
        Err(err) => {
            let unknown = err.kind() == clap::error::ErrorKind::InvalidSubcommand;
            if output::agent_mode() {
                let message = err.render().to_string();
                return Err(match unknown {
                    true => Error::Unknown(message),
                    false => Error::Invalid(message),
                }
                .into());
            }
            let _ = err.print();
            return Ok(if unknown {
                Exit::Unknown
            } else {
                Exit::Invalid
            });
        }
    };
    let global = GlobalArgs::from_arg_matches(&matches)?;
    crate::prompt::set_disabled(global.no_input);
    match global.debug.as_deref() {
        Some("full") => atl_sdk::set_debug(atl_sdk::DebugLevel::Full),
        Some(_) => atl_sdk::set_debug(atl_sdk::DebugLevel::Requests),
        None => {}
    }
    let mut path = Vec::new();
    let mut cursor = &matches;
    while let Some((name, sub)) = cursor.subcommand() {
        path.push(name);
        cursor = sub;
    }
    output::set_command(path.join(" "));
    let out = Out {
        no_headers: global.no_headers,
        no_truncate: global.no_truncate,
        jq: global.jq.clone(),
        pager: !global.no_pager && !global.agent,
        ..Out::new(
            global.agent,
            global.format,
            global.select.clone(),
            global.raw,
        )
    };

    client.set_dry_run(global.dry_run);
    if let Some(("api", api)) = matches.subcommand() {
        let (name, sub) = api.subcommand().expect("clap requires a service");
        let def = catalog::service(name).expect("clap only accepts known services");
        return run_service(&client, &global, &out, def, sub).await;
    }
    let ctx = Ctx {
        client: &client,
        out: &out,
        repo: &repo,
        yes: global.yes,
        dry_run: global.dry_run,
        output: global.output.as_deref(),
    };
    let cli = Cli::from_arg_matches(&matches)?;
    let result = match cli.command {
        Static::Auth(command) => {
            let profile = client.profile().to_owned();
            run_auth(command, profile, global.deployment.as_deref(), &out, config).await
        }
        Static::Jira(command) => crate::jira::run(&ctx, command).await,
        Static::Confluence(command) => crate::confluence::run(&ctx, command).await,
        Static::Bitbucket(args) => crate::bitbucket::run(&ctx, args).await,
        Static::Jsm(command) => crate::jsm::run(&ctx, command).await,
        Static::Trello(command) => crate::trello::run(&ctx, command).await,
        Static::Admin(args) => crate::admin::run(&ctx, args).await,
        Static::Completions { shell } => {
            // The registration script is tiny: the shell asks `atl` itself
            // for candidates (COMPLETE=<shell>), so all generated commands
            // complete without a multi-megabyte script.
            let exe = std::env::current_exe().context("cannot find the atl executable")?;
            let script = std::process::Command::new(exe)
                .env("COMPLETE", shell.to_string())
                .output()
                .context("cannot generate the completion script")?;
            std::io::stdout().write_all(&script.stdout)?;
            Ok(Exit::Ok)
        }
        Static::Open { target } => tasks::run_open(&ctx, target).await,
        Static::Browse => {
            if !out.interactive() {
                return Err(Error::Invalid("`atl browse` needs a terminal; agents and scripts use `atl jira issue list`, `atl confluence page get` and `atl api`".into()).into());
            }
            crate::browse::home(&client).await
        }
        Static::Skill { name, list, install } => crate::skill::run(&out, name, list, install),
        Static::Alias(command) => run_alias(command, &out, config),
        Static::Man { dir } => {
            std::fs::create_dir_all(&dir)
                .with_context(|| format!("cannot create {}", dir.display()))?;
            let mut written = Vec::new();
            let mut stack = vec![(alphabetical(Cli::command()).name("atl"), "atl".to_owned())];
            while let Some((command, name)) = stack.pop() {
                for sub in command.get_subcommands() {
                    if sub.get_name() != "help" {
                        stack.push((sub.clone(), format!("{name}-{}", sub.get_name())));
                    }
                }
                let path = dir.join(format!("{name}.1"));
                let mut page = Vec::new();
                clap_mangen::Man::new(command.name(name.clone()))
                    .title(name.to_ascii_uppercase())
                    .render(&mut page)
                    .context("cannot render a man page")?;
                std::fs::write(&path, page)
                    .with_context(|| format!("cannot write {}", path.display()))?;
                written.push(path.display().to_string());
            }
            written.sort();
            out.item(
                json!({ "dir": dir.display().to_string(), "pages": written }),
                Value::Null,
                |v| {
                    format!(
                        "Wrote {} man pages to {}",
                        v["pages"].as_array().map_or(0, Vec::len),
                        dir.display()
                    )
                },
            )
        }
        Static::Update => crate::update::run(&out).await,
        Static::Convert { input, from, to } => {
            use atl_sdk::markup;
            let path = input.filter(|p| p.as_os_str() != "-");
            let text = tasks::read_text(None, Some(path.as_deref().unwrap_or(std::path::Path::new("-"))))?
                .unwrap_or_default();
            let markdown = match from {
                Markup::Markdown => text,
                Markup::Storage => markup::storage_to_markdown(&text),
                Markup::Wiki => markup::wiki_to_markdown(&text),
                Markup::Adf => {
                    let adf: Value = serde_json::from_str(&text).map_err(|err| {
                        Error::Invalid(format!("the ADF input is not JSON ({err}); pass an ADF document or choose another --from"))
                    })?;
                    markup::adf_to_markdown(&adf)
                }
            };
            let converted = match to {
                Markup::Markdown => markdown,
                Markup::Storage => markup::markdown_to_storage(&markdown),
                Markup::Wiki => markup::markdown_to_wiki(&markdown),
                Markup::Adf => serde_json::to_string_pretty(&markup::markdown_to_adf(&markdown))?,
            };
            if out.agent {
                return out.item(json!({ "text": converted }), Value::Null, |_| String::new());
            }
            output::print_line(converted.trim_end());
            Ok(Exit::Ok)
        }
        Static::Task(TaskCommand::List) => out.list(
            atl_sdk::tasks::list()
                .iter()
                .map(|t| json!({ "name": t.name, "writes": if t.read_only { "no" } else { "yes" }, "read_only": t.read_only, "description": t.description }))
                .collect::<Vec<_>>(),
            &[col("NAME", "name"), col("WRITES", "writes"), flex("DESCRIPTION", "description")],
            Value::Null,
        ),
        Static::Task(TaskCommand::Describe { name }) => {
            let task = find_task(&name)?;
            print_json(
                &out,
                task.to_json(),
            )
        }
        Static::Task(TaskCommand::Run { name, args }) => {
            let task = find_task(&name)?;
            let args = match args {
                Some(text) => read_json_arg(&text)?,
                None => json!({}),
            };
            if task.destructive {
                tasks::confirm(global.yes, &format!("run task {}", task.name))?;
            }
            print_json(&out, tasks::run_task(&client, task.name, args).await?)
        }
        Static::Services => {
            let list: Vec<Value> = catalog::services()
                .iter()
                .map(|def| {
                    let count = |key: Option<&str>| key.and_then(catalog::operation_count);
                    json!({
                        "name": def.name,
                        "title": def.title,
                        "family": def.family.as_str(),
                        "active_deployment": client.deployment(def.family).as_str(),
                        "cloud_operations": count(def.cloud),
                        "data_center_operations": count(def.data_center),
                        "data_center_note": def.data_center_note,
                    })
                })
                .collect();
            out.list(
                list,
                &[
                    col("SERVICE", "name"),
                    col("CLOUD OPS", "cloud_operations"),
                    col("DC OPS", "data_center_operations"),
                    flex("COVERS", "title"),
                ],
                Value::Null,
            )
        }
        Static::Schema { target, depth } => {
            print_json(&out, describe_target(&client, &target, depth)?)
        }
        Static::Mcp(args) => {
            let options = atl_mcp::ServerOptions {
                mode: if args.mode == "full" {
                    atl_mcp::ToolMode::Full
                } else {
                    atl_mcp::ToolMode::Compact
                },
                services: args.services,
                resources: args.resources,
                read_only: args.read_only,
                tasks: !args.no_tasks,
            };
            atl_mcp::serve_stdio(client, options).await?;
            Ok(Exit::Ok)
        }
    };
    match result {
        Err(err) => match err.downcast::<atl_sdk::Error>() {
            Ok(atl_sdk::Error::DryRun(request)) => print_json(&out, *request),
            Ok(other) => Err(other.into()),
            Err(err) => Err(err),
        },
        ok => ok,
    }
}

/// Positional words before clap parses, skipping flags and their values.
fn positionals(args: &[String]) -> Vec<&str> {
    let mut out = Vec::new();
    let mut skip_next = false;
    for arg in args {
        if skip_next {
            skip_next = false;
            continue;
        }
        if arg == "--" {
            break;
        }
        if arg.starts_with('-') {
            skip_next = VALUE_FLAGS.contains(&arg.as_str());
            continue;
        }
        out.push(arg.as_str());
    }
    out
}

/// The value of a global flag given as `--flag value` or `--flag=value`.
fn flag_value<'a>(argv: &'a [String], flag: &str) -> Option<&'a str> {
    let mut iter = argv.iter();
    while let Some(arg) = iter.next() {
        if let Some(value) = arg
            .strip_prefix(flag)
            .and_then(|rest| rest.strip_prefix('='))
        {
            return Some(value);
        }
        if arg == flag {
            return iter.next().map(String::as_str);
        }
    }
    None
}

/// The whole command tree for dynamic shell completion (`COMPLETE=zsh atl`).
pub fn completion_command() -> Command {
    let config = Config::load().unwrap_or_default();
    let client = Client::new(&config, None);
    // The shell runs `COMPLETE=zsh atl -- atl <words>`: only the service
    // being typed needs its resource and method tree.
    let words: Vec<String> = std::env::args().skip_while(|a| a != "--").skip(2).collect();
    let words = positionals(&words);
    let service = (words.first() == Some(&"api"))
        .then(|| words.get(1))
        .flatten();
    let mut api = api_command();
    for def in catalog::services() {
        api = api.subcommand(match service == Some(&def.name) {
            true => service_tree(def, client.deployment(def.family)),
            false => Command::new(def.name).about(def.title),
        });
    }
    alphabetical(Cli::command().subcommand(api))
}

/// Lists every level's subcommands in alphabetical order in help, man pages
/// and completion: clap sorts subcommands with the same display order by name.
fn alphabetical(command: Command) -> Command {
    command.mut_subcommands(|sub| alphabetical(sub).display_order(0))
}

/// Puts the product commands in their own section of `atl --help`: clap
/// lists every subcommand under one heading.
fn grouped_help(command: Command) -> Command {
    const PRODUCTS: [&str; 6] = ["admin", "bitbucket", "confluence", "jira", "jsm", "trello"];
    let styles = command.get_styles();
    let (header, literal) = (styles.get_header(), styles.get_literal());
    let mut subcommands: Vec<(&str, String)> = command
        .get_subcommands()
        .map(|sub| {
            (
                sub.get_name(),
                sub.get_about().map(ToString::to_string).unwrap_or_default(),
            )
        })
        .collect();
    subcommands.sort();
    // clap adds `help` when it builds the command, after this runs.
    subcommands.push((
        "help",
        "Print this message or the help of the given subcommand(s)".to_owned(),
    ));
    let width = subcommands
        .iter()
        .map(|(name, _)| name.len())
        .max()
        .unwrap_or(0);
    let (mut products, mut others) = (String::new(), String::new());
    for (name, about) in subcommands {
        let section = match PRODUCTS.contains(&name) {
            true => &mut products,
            false => &mut others,
        };
        let pad = width - name.len();
        section.push_str(&format!(
            "  {literal}{name}{literal:#}{:pad$}  {about}\n",
            ""
        ));
    }
    let template = format!(
        "{{before-help}}{{about-with-newline}}\n{{usage-heading}} {{usage}}\n\n\
         {header}Products:{header:#}\n{products}\n\
         {header}Commands:{header:#}\n{others}\n\
         {header}Options:{header:#}\n{{options}}{{after-help}}"
    );
    command
        .help_template(template)
        .subcommand_value_name("PRODUCT|COMMAND")
}

fn api_command() -> Command {
    Command::new("api")
        .about("Every Atlassian REST operation (generated from the OpenAPI documents), and raw requests")
        .long_about(
            "Every Atlassian REST operation, generated from Atlassian's OpenAPI documents:\n  \
             atl api <service> <resource> <method> [--param-flags] [--params JSON] [--json BODY]\n\
             Any path on a service's server:\n  \
             atl api <service> get|post|put|patch|delete|head|options <path> [--params JSON] [--json BODY]\n\n\
             Run `atl services` to list services and `atl schema <service>.<operationId>` for an operation's shape.",
        )
        .subcommand_required(true)
        .arg_required_else_help(true)
}

/// `atl api <service> get <path>` and the other HTTP methods.
fn raw_commands(def: &ServiceDef) -> impl Iterator<Item = Command> + '_ {
    ["get", "post", "put", "patch", "delete", "head", "options"]
        .into_iter()
        .map(move |method| {
            Command::new(method)
                .about(format!(
                    "Send {} <path> to the {} server",
                    method.to_ascii_uppercase(),
                    def.family
                ))
                .arg(
                    Arg::new("path")
                        .required(true)
                        .help("Path relative to the base URL, for example /rest/api/3/myself"),
                )
                .arg(
                    Arg::new("params")
                        .long("params")
                        .value_name("JSON")
                        .help("Query parameters as a JSON object"),
                )
                .arg(
                    Arg::new("json")
                        .long("json")
                        .value_name("JSON")
                        .help("Request body: inline JSON, @file, or - for stdin"),
                )
                .arg(
                    Arg::new("raw_field")
                        .short('f')
                        .long("raw-field")
                        .value_name("KEY=VALUE")
                        .action(ArgAction::Append)
                        .help("String field: query parameter for GET/HEAD/DELETE/OPTIONS, else JSON body key (nested[key]=v, list[]=v); repeatable"),
                )
                .arg(
                    Arg::new("field")
                        .short('F')
                        .long("field")
                        .value_name("KEY=VALUE")
                        .action(ArgAction::Append)
                        .help("Typed field: true, false, null and numbers become JSON values, @path reads a file; repeatable"),
                )
                .arg(
                    Arg::new("input")
                        .long("input")
                        .value_name("PATH")
                        .help("Request body (JSON) from a file, or - for stdin"),
                )
                .args(transport_args())
        })
}

/// `-H`, `-i` and `--silent`, shared by raw and generated commands.
fn transport_args() -> [Arg; 3] {
    [
        Arg::new("header")
            .short('H')
            .long("header")
            .value_name("NAME: VALUE")
            .action(ArgAction::Append)
            .help("Extra request header; repeatable"),
        Arg::new("include")
            .short('i')
            .long("include")
            .action(ArgAction::SetTrue)
            .help("Print the HTTP status line and response headers before the body"),
        Arg::new("silent")
            .long("silent")
            .action(ArgAction::SetTrue)
            .help("Print nothing on success"),
    ]
}

/// Output options of `atl api` requests.
#[derive(Clone, Copy, Default)]
struct Include {
    headers: bool,
    silent: bool,
}

impl Include {
    fn from(matches: &ArgMatches) -> Self {
        Self {
            headers: matches.get_flag("include"),
            silent: matches.get_flag("silent"),
        }
    }
}

/// `-H "Name: value"` flags.
fn headers(matches: &ArgMatches) -> Result<Vec<(String, String)>, Error> {
    matches
        .get_many::<String>("header")
        .into_iter()
        .flatten()
        .map(|header| match header.split_once(':') {
            Some((name, value)) if !name.trim().is_empty() => {
                Ok((name.trim().to_owned(), value.trim().to_owned()))
            }
            _ => Err(Error::Invalid(format!(
                "-H `{header}` is not NAME: VALUE; pass for example -H \"X-Request-Id: 1\""
            ))),
        })
        .collect()
}

/// `-f` and `-F` flags in command-line order, as one JSON object.
fn field_flags(matches: &ArgMatches) -> Result<Map<String, Value>, Error> {
    let mut flags: Vec<(usize, bool, &String)> = Vec::new();
    for (id, typed) in [("raw_field", false), ("field", true)] {
        if let (Some(indices), Some(values)) =
            (matches.indices_of(id), matches.get_many::<String>(id))
        {
            flags.extend(indices.zip(values).map(|(i, v)| (i, typed, v)));
        }
    }
    flags.sort_by_key(|(index, ..)| *index);
    let mut out = Map::new();
    for (_, typed, flag) in flags {
        let letter = if typed { 'F' } else { 'f' };
        let (key, text) = flag.split_once('=').ok_or_else(|| {
            Error::Invalid(format!(
                "-{letter} `{flag}` is not KEY=VALUE; use key=value, nested[key]=value or list[]=value"
            ))
        })?;
        let value = match (typed, text) {
            (false, text) => Value::String(text.to_owned()),
            (true, "true") => Value::Bool(true),
            (true, "false") => Value::Bool(false),
            (true, "null") => Value::Null,
            (true, text) if text.starts_with('@') => {
                Value::String(std::fs::read_to_string(&text[1..]).map_err(|err| {
                    Error::Invalid(format!(
                        "-F {key}: cannot read {} ({err}); pass @<existing file>",
                        &text[1..]
                    ))
                })?)
            }
            (true, text) => match text.parse::<i64>() {
                Ok(n) => n.into(),
                Err(_) => match text
                    .parse::<f64>()
                    .ok()
                    .and_then(serde_json::Number::from_f64)
                {
                    Some(n) => Value::Number(n),
                    None => Value::String(text.to_owned()),
                },
            },
        };
        insert_field(&mut out, key, value).map_err(|reason| {
            Error::Invalid(format!(
                "-{letter} `{flag}`: {reason}; use key=value, nested[key]=value or list[]=value"
            ))
        })?;
    }
    Ok(out)
}

/// Inserts `value` at a gh-style key: `a`, `a[b]`, `a[]` (append), `a[b][]`.
fn insert_field(map: &mut Map<String, Value>, key: &str, value: Value) -> Result<(), String> {
    let (head, mut rest) = key.split_at(key.find('[').unwrap_or(key.len()));
    if head.is_empty() {
        return Err("the key is empty".into());
    }
    let mut segments = Vec::new();
    while !rest.is_empty() {
        let inner = rest.strip_prefix('[').and_then(|r| r.split_once(']'));
        let Some((segment, after)) = inner else {
            return Err(format!("`{key}` has an unclosed or stray bracket"));
        };
        segments.push(segment);
        rest = after;
    }
    let mut slot = map.entry(head.to_owned()).or_insert(Value::Null);
    for segment in segments {
        if slot.is_null() {
            *slot = if segment.is_empty() {
                json!([])
            } else {
                json!({})
            };
        }
        slot = match (segment.is_empty(), slot) {
            (true, Value::Array(items)) => {
                items.push(Value::Null);
                items.last_mut().expect("just pushed")
            }
            (false, Value::Object(object)) => {
                object.entry(segment.to_owned()).or_insert(Value::Null)
            }
            _ => return Err(format!("`{key}` mixes a list and an object at `{segment}`")),
        };
    }
    *slot = value;
    Ok(())
}

/// The full resource and method tree of a service, without parameter flags
/// (for shell completion).
fn service_tree(def: &'static ServiceDef, deployment: Deployment) -> Command {
    let mut command = Command::new(def.name)
        .about(def.title)
        .subcommands(raw_commands(def));
    if let Some(key) = def.catalog_key(deployment) {
        let catalog = catalog::catalog(key);
        command = command.subcommands(catalog.tags.iter().enumerate().map(|(index, tag)| {
            Command::new(tag.name.as_ref())
                .about(tag.title.as_ref())
                .subcommands(catalog.ops_in(catalog::TagId(index as u32)).map(|op| {
                    Command::new(op.name.as_ref())
                        .about(op.summary.as_deref().unwrap_or(op.id.as_ref()).to_owned())
                }))
        }));
    }
    command
}

/// Builds `atl api <service>`. `path` is `Some([resource, method, ...])` when the
/// command line names this service; only then is the resource tree built.
fn service_command(
    def: &'static ServiceDef,
    deployment: Deployment,
    path: Option<&[&str]>,
) -> Command {
    let command = Command::new(def.name)
        .about(def.title)
        .subcommand_required(true)
        .arg_required_else_help(true);
    let Some(path) = path else {
        return command;
    };
    let command = command.subcommands(raw_commands(def));
    let Some(key) = def.catalog_key(deployment) else {
        return command.after_help(format!(
            "`{}` is not available on {deployment}: {}",
            def.name,
            def.data_center_note
                .unwrap_or("no OpenAPI document exists for it")
        ));
    };
    let catalog = catalog::catalog(key);
    let command = command.after_help(format!(
        "{} resources from the {deployment} API. Run `atl api {} <resource> --help` to list methods.",
        catalog.tags.len(),
        def.name
    ));
    let (wanted_tag, wanted_op) = (path.first().copied(), path.get(1).copied());
    let subcommands = catalog.tags.iter().enumerate().map(|(index, tag)| {
        let tag_id = catalog::TagId(index as u32);
        let mut resource = Command::new(tag.name.as_ref())
            .about(tag.title.as_ref())
            .subcommand_required(true)
            .arg_required_else_help(true);
        if let Some(description) = &tag.description {
            resource = resource.long_about(format!("{}\n\n{}", tag.title, description));
        }
        if wanted_tag != Some(tag.name.as_ref()) {
            // Clap needs the resource to exist to print `atl jira --help`;
            // its methods are only needed when it is selected.
            return resource;
        }
        resource.subcommands(catalog.ops_in(tag_id).map(|op| {
            let selected = wanted_op == Some(op.name.as_ref());
            operation_command(catalog, op, selected)
        }))
    });
    command.subcommands(subcommands.collect::<Vec<_>>())
}

fn operation_command(
    catalog: &'static Catalog,
    op: &'static Operation,
    with_args: bool,
) -> Command {
    let mut about = op.summary.as_deref().unwrap_or(op.id.as_ref()).to_owned();
    if op.deprecated {
        about.push_str(" (deprecated)");
    }
    let mut command = Command::new(op.name.as_ref()).about(about.clone());
    let mut long = format!(
        "{about}\n\n{} {}\noperationId: {}",
        op.method, op.path, op.id
    );
    if let Some(description) = &op.description {
        long.push_str("\n\n");
        long.push_str(description);
    }
    command = command.long_about(long);
    if !with_args {
        return command;
    }
    for param in catalog.params_of(op) {
        let kind = param.schema_type().unwrap_or_else(|| "string".into());
        let mut help = format!(
            "[{}{}{}] ",
            param.location.as_str(),
            if param.required { ", required" } else { "" },
            if kind == "array" { ", repeatable" } else { "" }
        );
        if let Some(description) = &param.description {
            help.push_str(first_sentence(description));
        }
        command = command.arg(
            Arg::new(param.flag.as_ref())
                .long(param.flag.as_ref())
                .value_name(kind.to_ascii_uppercase())
                .action(ArgAction::Append)
                .help(help)
                .help_heading("Parameters"),
        );
    }
    command = command.args(transport_args()).arg(
        Arg::new("params")
            .long("params")
            .value_name("JSON")
            .help("Parameters as a JSON object keyed by API name, for example '{\"issueIdOrKey\":\"PROJ-1\"}'"),
    );
    if let Some(body) = &op.body {
        let required = if body.required {
            "required"
        } else {
            "optional"
        };
        if body.is_multipart() {
            command = command
                .arg(
                    Arg::new("upload")
                        .long("upload")
                        .value_name("PATH")
                        .action(ArgAction::Append)
                        .value_parser(clap::value_parser!(PathBuf))
                        .help(format!(
                            "File to upload as multipart field `file` ({required}); repeatable"
                        )),
                )
                .arg(
                    Arg::new("json")
                        .long("json")
                        .value_name("JSON")
                        .help("Extra form fields as a JSON object"),
                );
        } else {
            command = command.arg(Arg::new("json").long("json").value_name("JSON").help(format!(
                "Request body ({}, {required}): inline JSON, @file, or - for stdin. See `atl schema` for its shape",
                body.content_type
            )));
        }
    }
    command
}

fn first_sentence(text: &str) -> &str {
    let text = text.trim();
    let end = text
        .char_indices()
        .find(|(i, c)| *c == '\n' || (*c == '.' && text[i + 1..].starts_with(' ')))
        .map(|(i, c)| if c == '.' { i + 1 } else { i })
        .unwrap_or(text.len());
    &text[..end]
}

async fn run_service(
    client: &Client,
    global: &GlobalArgs,
    out: &Out,
    def: &'static ServiceDef,
    matches: &ArgMatches,
) -> Result<Exit> {
    let (resource, resource_matches) = matches.subcommand().expect("clap requires a resource");
    if let Some(path) = resource_matches
        .try_get_one::<String>("path")
        .ok()
        .flatten()
    {
        let method: HttpMethod = resource.parse()?;
        let mut query = match resource_matches.get_one::<String>("params") {
            Some(text) => parse_object(text, "--params")?,
            None => Map::new(),
        };
        let input = resource_matches.get_one::<String>("input");
        let mut body = match (resource_matches.get_one::<String>("json"), input) {
            (Some(_), Some(_)) => {
                return Err(Error::Invalid(
                    "pass the body with --json or --input, not both".into(),
                )
                .into());
            }
            (Some(text), None) => Some(read_json_arg(text)?),
            (None, Some(path)) if path == "-" => Some(read_json_arg("-")?),
            (None, Some(path)) => Some(read_json_arg(&format!("@{path}"))?),
            (None, None) => None,
        };
        let fields = field_flags(resource_matches)?;
        if !fields.is_empty() {
            if matches!(
                method,
                HttpMethod::Get | HttpMethod::Head | HttpMethod::Delete | HttpMethod::Options
            ) {
                for (key, value) in fields {
                    let nested = value.is_object()
                        || value
                            .as_array()
                            .is_some_and(|a| a.iter().any(|v| v.is_object() || v.is_array()));
                    if nested {
                        return Err(Error::Invalid(format!(
                            "-f/-F `{key}` is nested, but {method} sends fields as query parameters; use flat keys (list[]=v is fine)"
                        ))
                        .into());
                    }
                    query.insert(key, value);
                }
            } else {
                match &mut body {
                    None => body = Some(Value::Object(fields)),
                    Some(Value::Object(object)) => object.extend(fields),
                    Some(_) => {
                        return Err(Error::Invalid(
                            "-f/-F add keys to a JSON object body, but --json/--input is not an object".into(),
                        )
                        .into());
                    }
                }
            }
        }
        let mut request = client.prepare_raw(def.family, method, path, &query, body)?;
        request.headers.extend(headers(resource_matches)?);
        return send(
            client,
            global,
            out,
            request,
            Include::from(resource_matches),
        )
        .await;
    }
    let catalog = client.catalog_for(def)?;
    let (method, method_matches) = resource_matches
        .subcommand()
        .expect("clap requires a method");
    let tag = catalog
        .find_tag(resource)
        .expect("clap only accepts known resources");
    let op = catalog
        .ops_in(tag)
        .find(|op| op.name == method)
        .expect("clap only accepts known methods");

    let mut args = CallArgs::default();
    if let Some(text) = method_matches.get_one::<String>("params") {
        args.params = parse_object(text, "--params")?;
    }
    for param in catalog.params_of(op) {
        let Some(values) = method_matches.get_many::<String>(param.flag.as_ref()) else {
            continue;
        };
        let values: Vec<Value> = values.cloned().map(Value::String).collect();
        let value = if values.len() == 1 {
            values.into_iter().next().expect("one value")
        } else {
            Value::Array(values)
        };
        args.params.remove(param.flag.as_ref());
        args.params.insert(param.name.to_string(), value);
    }
    if let Some(text) = method_matches.try_get_one::<String>("json").ok().flatten() {
        args.body = Some(read_json_arg(text)?);
    }
    if let Some(files) = method_matches
        .try_get_many::<PathBuf>("upload")
        .ok()
        .flatten()
    {
        args.files = files.cloned().collect();
    }
    let mut request = client.prepare(def, catalog, op, &args)?;
    request.headers.extend(headers(method_matches)?);
    send(client, global, out, request, Include::from(method_matches)).await
}

async fn send(
    client: &Client,
    global: &GlobalArgs,
    out: &Out,
    request: PreparedRequest,
    include: Include,
) -> Result<Exit> {
    if global.dry_run {
        return print_json(out, request.describe());
    }
    if request.method == HttpMethod::Delete {
        tasks::confirm(global.yes, &format!("DELETE {}", request.url))?;
    }
    if global.page_all {
        let mut pager = Pager::new(
            client,
            request,
            global.page_limit,
            Duration::from_millis(global.page_delay),
        );
        if out.agent {
            let mut pages = Vec::new();
            while let Some(page) = pager.next_page().await? {
                pages.push(page);
            }
            let meta = json!({ "pages": pages.len(), "truncated": pager.truncated() });
            return out.api(Value::Array(pages), meta);
        }
        let mut stdout = std::io::stdout().lock();
        while let Some(page) = pager.next_page().await? {
            if writeln!(stdout, "{page}").is_err() {
                return Ok(Exit::Ok); // Reader closed the pipe (for example `| head`).
            }
        }
        if pager.truncated() {
            eprintln!(
                "stopped after {} pages and more exist; raise --page-limit to get them",
                global.page_limit.max(1)
            );
        }
        return Ok(Exit::Ok);
    }
    let response = client.execute(&request).await?;
    if include.silent {
        return match out.agent {
            true => print_json(out, json!({ "status": response.status })),
            false => Ok(Exit::Ok),
        };
    }
    if include.headers && !out.agent {
        output::print_line(&format!("HTTP/1.1 {}", response.status));
        for (name, value) in &response.headers {
            output::print_line(&format!("{name}: {}", value.to_str().unwrap_or("")));
        }
        output::print_line("");
    }
    write_response(out, &response, global.output.as_ref(), include)
}

fn write_response(
    out: &Out,
    response: &Response,
    output: Option<&PathBuf>,
    include: Include,
) -> Result<Exit> {
    let mut meta = json!({ "status": response.status });
    if include.headers {
        meta["headers"] = response
            .headers
            .iter()
            .map(|(name, value)| (name.to_string(), Value::from(value.to_str().unwrap_or(""))))
            .collect::<Map<_, _>>()
            .into();
    }
    if let Some(path) = output {
        std::fs::write(path, &response.body)
            .with_context(|| format!("cannot write {}", path.display()))?;
        return print_json(
            out,
            json!({ "status": response.status, "saved": path.display().to_string(), "bytes": response.body.len() }),
        );
    }
    if response.body.is_empty() {
        return print_json(out, json!({ "status": response.status }));
    }
    if let Some(value) = response.json() {
        return out.api(value, meta);
    }
    if out.agent {
        let text = String::from_utf8_lossy(&response.body).into_owned();
        return out.api(
            Value::String(text),
            json!({ "status": response.status, "content_type": response.content_type() }),
        );
    }
    let _ = std::io::stdout().lock().write_all(&response.body);
    Ok(Exit::Ok)
}

/// JSON results of static commands: pretty JSON for people, an envelope for agents.
fn print_json(out: &Out, value: Value) -> Result<Exit> {
    out.item(value, Value::Null, |v| {
        serde_json::to_string_pretty(v).unwrap_or_default()
    })
}

fn parse_object(text: &str, flag: &str) -> Result<Map<String, Value>, Error> {
    match read_json_arg(text)? {
        Value::Object(map) => Ok(map),
        other => Err(Error::Invalid(format!(
            "{flag} must be a JSON object such as '{{\"key\":\"value\"}}', got {}",
            kind_name(&other)
        ))),
    }
}

/// Inline JSON, `@path` for a file, or `-` for stdin.
fn read_json_arg(text: &str) -> Result<Value, Error> {
    let (source, content) = if text == "-" {
        let mut buffer = String::new();
        std::io::stdin()
            .read_to_string(&mut buffer)
            .map_err(|err| Error::Invalid(format!("cannot read JSON from stdin: {err}")))?;
        ("stdin".to_owned(), buffer)
    } else if let Some(path) = text.strip_prefix('@') {
        let content = std::fs::read_to_string(path).map_err(|err| {
            Error::Invalid(format!(
                "cannot read JSON file `{path}`: {err}; pass @<existing file>"
            ))
        })?;
        (format!("file `{path}`"), content)
    } else {
        ("argument".to_owned(), text.to_owned())
    };
    serde_json::from_str(&content).map_err(|err| {
        Error::Invalid(format!(
            "the JSON {source} is invalid ({err}); quote it in single quotes, for example --json '{{\"fields\":{{}}}}'"
        ))
    })
}

fn kind_name(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "a boolean",
        Value::Number(_) => "a number",
        Value::String(_) => "a string",
        Value::Array(_) => "an array",
        Value::Object(_) => "an object",
    }
}

fn describe_target(client: &Client, target: &str, depth: usize) -> Result<Value> {
    let (service, rest) = target.split_once('.').ok_or_else(|| {
        Error::Invalid(format!(
            "`{target}` is not `<service>.<operation>`; for example `atl schema jira.getIssue`"
        ))
    })?;
    let def = catalog::service(service).ok_or_else(|| {
        Error::Unknown(format!("unknown service `{service}`; run `atl services`"))
    })?;
    let catalog = client.catalog_for(def)?;
    if let Some(op) = catalog.find(rest) {
        return Ok(schema::describe_operation(def, catalog, op, depth));
    }
    if let Some(named) = schema::named(catalog, rest, depth) {
        return Ok(json!({ "service": service, "schema": rest, "definition": named }));
    }
    Err(Error::Unknown(format!(
        "`{service}` has no operation or schema `{rest}`; use an operationId (getIssue), <resource>.<method> (issues.get-issue), or a schema name"
    ))
    .into())
}

fn run_alias(command: AliasCommand, out: &Out, mut config: Config) -> Result<Exit> {
    match command {
        AliasCommand::List => {
            let rows: Vec<Value> = config
                .aliases
                .iter()
                .map(|(name, command)| json!({ "name": name, "command": command }))
                .collect();
            out.list(
                Value::Array(rows),
                &[
                    output::col("NAME", "name"),
                    output::flex("COMMAND", "command"),
                ],
                Value::Null,
            )
        }
        AliasCommand::Set { name, command } => {
            if is_builtin(&name) || name.starts_with('-') || name.contains(char::is_whitespace) {
                return Err(Error::Invalid(format!(
                    "`{name}` cannot be an alias: it is a built-in command or not one word; pick another name"
                ))
                .into());
            }
            if shlex::split(&command).is_none_or(|words| words.is_empty()) {
                return Err(Error::Invalid(format!(
                    "`{command}` is not a command (empty or an unclosed quote); pass the words after `atl`, for example \"jira issue list -a me\""
                ))
                .into());
            }
            config.aliases.insert(name.clone(), command.clone());
            let path = config.save()?;
            out.item(
                json!({ "name": name, "command": command, "saved": path.display().to_string() }),
                Value::Null,
                |_| format!("atl {name} now runs: atl {command}"),
            )
        }
        AliasCommand::Delete { name } => {
            if config.aliases.remove(&name).is_none() {
                return Err(Error::Invalid(format!(
                    "there is no alias `{name}`; list them with `atl alias list`"
                ))
                .into());
            }
            config.save()?;
            out.item(
                json!({ "name": name, "deleted": true }),
                Value::Null,
                |_| format!("Deleted alias {name}"),
            )
        }
    }
}

async fn run_auth(
    command: AuthCommand,
    profile: String,
    deployment: Option<&str>,
    out: &Out,
    mut config: Config,
) -> Result<Exit> {
    match command {
        AuthCommand::Login(args) => {
            let oauth = args.method == Some(AuthMethod::OAuth);
            if oauth && args.product.is_none() {
                return Err(Error::Invalid(
                    "--method oauth needs --product (cloud-site, jira, confluence or bitbucket) and --user CLIENT_ID, with the client secret on --token-stdin or in ATL_OAUTH_CLIENT_SECRET".into(),
                )
                .into());
            }
            let interactive = !out.agent
                && args.product.is_none()
                && std::io::stdin().is_terminal()
                && std::io::stdout().is_terminal();
            let outcome = if interactive {
                match login::run_form(&profile, &config).await? {
                    Some(outcome) => outcome,
                    None => {
                        eprintln!("login cancelled; nothing was saved");
                        return Ok(Exit::Ok);
                    }
                }
            } else {
                let product = args.product.ok_or_else(|| {
                    Error::Invalid("`atl auth login` needs a terminal for the form; in scripts pass --product, --url, --user and --token-stdin".into())
                })?;
                let mut secret = String::new();
                if args.token_stdin {
                    std::io::stdin()
                        .read_to_string(&mut secret)
                        .context("cannot read the token from stdin")?;
                }
                if oauth && secret.trim().is_empty() {
                    secret = std::env::var("ATL_OAUTH_CLIENT_SECRET").unwrap_or_default();
                }
                let mut request = LoginRequest::from_flags(
                    product,
                    args.url,
                    args.method,
                    args.user,
                    secret.trim().to_owned(),
                    deployment,
                    args.read_only,
                )?;
                request.oauth_port = args.callback_port;
                request.oauth_scopes = args.scopes;
                let absolute =
                    |path: Option<PathBuf>| path.map(|p| std::path::absolute(&p).unwrap_or(p));
                request.client_cert = absolute(args.client_cert);
                request.client_key = absolute(args.client_key);
                request.ca_cert = absolute(args.ca_cert);
                login::perform(&request, !args.no_verify).await?
            };
            let path = login::save(&mut config, &outcome.profile_or(&profile), &outcome)?;
            print_json(
                out,
                json!({
                    "profile": outcome.profile_or(&profile),
                    "saved": path.display().to_string(),
                    "connections": outcome.summary(),
                }),
            )
        }
        AuthCommand::Status { verify } => {
            let path = Config::path()?;
            let mut profiles = Map::new();
            for (name, stored) in &config.profiles {
                let mut families = Map::new();
                for (&family, connection) in stored {
                    let mut entry = json!({
                        "deployment": connection.deployment.as_str(),
                        "base_url": connection.base_url,
                        "auth": connection.auth.describe(),
                        "account": connection.account,
                    });
                    for (key, path) in [
                        ("client_cert", &connection.client_cert),
                        ("client_key", &connection.client_key),
                        ("ca_cert", &connection.ca_cert),
                    ] {
                        if let Some(path) = path {
                            entry[key] = path.display().to_string().into();
                        }
                    }
                    if verify {
                        let verified = match connection.resolve_secret() {
                            Ok(resolved) => atl_sdk::verify(family, &resolved)
                                .await
                                .map_err(|err| err.to_string()),
                            Err(reason) => Err(reason),
                        };
                        entry["verified"] = match verified {
                            Ok(account) => json!({ "ok": true, "account": account }),
                            Err(error) => json!({ "ok": false, "error": error }),
                        };
                    }
                    families.insert(family.as_str().into(), entry);
                }
                profiles.insert(name.clone(), Value::Object(families));
            }
            let env: Vec<&str> = Family::ALL
                .iter()
                .filter(|family| {
                    std::env::var_os(format!("{}_TOKEN", family.env_prefix())).is_some()
                })
                .map(|family| family.as_str())
                .collect();
            if !out.wants_text() {
                return print_json(
                    out,
                    json!({
                        "config": path.display().to_string(),
                        "active_profile": profile,
                        "default_profile": config.default_profile,
                        "environment_overrides": env,
                        "profiles": profiles,
                    }),
                );
            }
            // People get a summary and one row per login.
            let mut rows = Vec::new();
            for (name, families) in &profiles {
                for (family, entry) in families.as_object().into_iter().flatten() {
                    let text = |key: &str| entry[key].as_str().unwrap_or_default().to_owned();
                    let verified = match &entry["verified"] {
                        Value::Null => String::new(),
                        v if v["ok"] == true => {
                            format!("ok ({})", v["account"].as_str().unwrap_or("signed in"))
                        }
                        v => format!("failed: {}", v["error"].as_str().unwrap_or_default()),
                    };
                    rows.push(json!({
                        "profile": name,
                        "product": family,
                        "deployment": text("deployment"),
                        "url": text("base_url"),
                        "login": text("auth"),
                        "verified": verified,
                    }));
                }
            }
            println!("Config file: {}", path.display());
            if !env.is_empty() {
                println!("Environment variables set for: {}", env.join(", "));
            }
            if rows.is_empty() {
                if env.is_empty() {
                    println!(
                        "Not signed in. Run `atl auth login`, or set ATL_<PRODUCT>_URL and ATL_<PRODUCT>_TOKEN (for example ATL_JIRA_URL)."
                    );
                }
                return Ok(Exit::Ok);
            }
            match config.default_profile.as_deref().unwrap_or("default") {
                default if default == profile => println!("Profile: {profile}\n"),
                default => println!("Profile: {profile} (the default is {default})\n"),
            }
            let mut columns = vec![
                output::col("PROFILE", "profile"),
                output::col("PRODUCT", "product"),
                output::col("DEPLOYMENT", "deployment"),
                output::flex("URL", "url"),
                output::col("LOGIN", "login"),
            ];
            if verify {
                columns.push(output::col("VERIFIED", "verified"));
            }
            out.list(rows, &columns, Value::Null)
        }
        AuthCommand::Token { product } => {
            use atl_sdk::Auth;
            let client = atl_sdk::Client::new(&config, Some(&profile));
            let stored: Vec<Family> = Family::ALL
                .into_iter()
                .filter(|family| client.connection(*family).is_some())
                .collect();
            let names = || {
                stored
                    .iter()
                    .map(|f| f.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            };
            let family: Family = match (product, stored.as_slice()) {
                (Some(product), _) => product.parse()?,
                (None, [only]) => *only,
                (None, []) => {
                    return Err(Error::Invalid(format!(
                        "profile `{profile}` has no logins; run `atl auth login` first"
                    ))
                    .into());
                }
                (None, _) => {
                    let choices: Vec<atl_sdk::error::Choice> = stored
                        .iter()
                        .map(|f| atl_sdk::error::Choice {
                            value: f.as_str().to_owned(),
                            label: f.as_str().to_owned(),
                        })
                        .collect();
                    if !crate::prompt::allowed() {
                        return Err(Error::needs_input(
                            "product",
                            format!(
                                "profile `{profile}` has several logins; name the product: `atl auth token PRODUCT` with one of: {}",
                                names()
                            ),
                            choices,
                        )
                        .into());
                    }
                    crate::prompt::select("Product", &choices)?.parse()?
                }
            };
            let connection = client.connection(family).ok_or_else(|| {
                Error::Invalid(format!(
                    "profile `{profile}` has no {family} login (it has: {}); run `atl auth login --product {family}` or pick one of those",
                    names()
                ))
            })?;
            let resolved = connection.resolve_secret().map_err(|reason| {
                Error::Invalid(format!(
                    "cannot use the {family} login ({reason}); run `atl auth login --product {family}` again"
                ))
            })?;
            let (kind, user, token) = match &resolved.auth {
                Auth::Basic { username, secret } => {
                    ("basic", Some(username.clone()), secret.clone())
                }
                Auth::Bearer { token } => ("bearer", None, token.clone()),
                Auth::OAuth { access_token, .. } => ("oauth", None, access_token.clone()),
                Auth::Trello { key, token } => ("trello", Some(key.clone()), token.clone()),
                Auth::Cookie { cookie } => ("cookie", None, cookie.clone()),
                Auth::Netrc {} | Auth::None {} => {
                    return Err(Error::Invalid(format!(
                        "the {family} login has no token (it signs in with `{}`); log in with a token: `atl auth login --product {family}`",
                        connection.auth.describe()
                    ))
                    .into());
                }
            };
            out.item(
                json!({ "product": family.as_str(), "kind": kind, "user": user, "token": token }),
                Value::Null,
                |v| crate::tasks::s(v, "token").to_owned(),
            )
        }
        AuthCommand::Logout { product } => {
            let removed = match product {
                Some(product) => {
                    let family: Family = product.parse()?;
                    let stored = config.profiles.get_mut(&profile).ok_or_else(|| {
                        Error::Invalid(format!("profile `{profile}` does not exist; run `atl auth status` to list profiles"))
                    })?;
                    stored.remove(&family);
                    if stored.is_empty() {
                        config.profiles.remove(&profile);
                    }
                    format!("{profile}.{family}")
                }
                None => {
                    config.profiles.remove(&profile).ok_or_else(|| {
                        Error::Invalid(format!("profile `{profile}` does not exist; run `atl auth status` to list profiles"))
                    })?;
                    profile.clone()
                }
            };
            if config
                .default_profile
                .as_deref()
                .is_some_and(|name| !config.profiles.contains_key(name))
            {
                config.default_profile = None;
            }
            let path = config.save()?;
            print_json(
                out,
                json!({ "removed": removed, "config": path.display().to_string() }),
            )
        }
        AuthCommand::Default { profile } => {
            if !config.profiles.contains_key(&profile) {
                return Err(Error::Invalid(format!(
                    "profile `{profile}` does not exist; create it with `atl auth login --profile {profile}`"
                ))
                .into());
            }
            config.default_profile = Some(profile.clone());
            config.save()?;
            print_json(out, json!({ "default_profile": profile }))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fields_of(args: &[&str]) -> Result<Map<String, Value>, Error> {
        let def = catalog::service("jira").expect("jira");
        let post = raw_commands(def)
            .find(|c| c.get_name() == "post")
            .expect("post");
        let matches = post
            .try_get_matches_from(
                std::iter::once("post")
                    .chain(["/x"])
                    .chain(args.iter().copied()),
            )
            .expect("parse");
        field_flags(&matches)
    }

    /// Every flag and argument of the product and tool commands says what it
    /// is in `--help`.
    #[test]
    fn every_argument_has_help() {
        let mut missing = Vec::new();
        let mut stack = vec![(Cli::command(), "atl".to_owned())];
        while let Some((command, path)) = stack.pop() {
            for arg in command.get_arguments() {
                let id = arg.get_id().as_str();
                if !arg.is_hide_set()
                    && arg.get_help().is_none()
                    && !matches!(id, "help" | "version")
                {
                    missing.push(format!("{path} {id}"));
                }
            }
            for sub in command.get_subcommands() {
                stack.push((sub.clone(), format!("{path} {}", sub.get_name())));
            }
        }
        missing.sort();
        assert!(
            missing.is_empty(),
            "arguments without help:\n{}",
            missing.join("\n")
        );
    }

    /// `atl --help` lists each product's resources in alphabetical order;
    /// adding a resource must update the summary too.
    #[test]
    fn product_summaries_name_every_resource() {
        for product in ["jira", "confluence", "bitbucket", "jsm", "trello", "admin"] {
            let command = Cli::command();
            let command = command.find_subcommand(product).expect("product");
            let about = command.get_about().expect("about").to_string();
            let listed: Vec<&str> = about
                .split_once(": ")
                .expect("`Product: a, b`")
                .1
                .trim_end_matches('.')
                .split(", ")
                .collect();
            let mut actual: Vec<&str> = command.get_subcommands().map(Command::get_name).collect();
            actual.sort_unstable();
            assert_eq!(listed, actual, "summary of `atl {product}`");
        }
    }

    #[test]
    fn field_flags_nest_type_and_keep_order() {
        assert_eq!(
            Value::Object(
                fields_of(&[
                    "-F",
                    "fields[project][key]=ABC",
                    "-f",
                    "fields[labels][]=a",
                    "-f",
                    "fields[labels][]=b"
                ])
                .expect("nested")
            ),
            json!({ "fields": { "project": { "key": "ABC" }, "labels": ["a", "b"] } })
        );
        assert_eq!(
            Value::Object(
                fields_of(&[
                    "-F",
                    "n=3",
                    "-F",
                    "ok=true",
                    "-F",
                    "none=null",
                    "-f",
                    "s=3",
                    "-F",
                    "x=1.5"
                ])
                .expect("typed")
            ),
            json!({ "n": 3, "ok": true, "none": null, "s": "3", "x": 1.5 })
        );
        for bad in [["-f", "novalue"], ["-f", "a[b=1"], ["-f", "=1"]] {
            assert!(fields_of(&bad).is_err(), "{bad:?}");
        }
        assert!(fields_of(&["-f", "a[]=1", "-f", "a[b]=2"]).is_err());
    }
}
