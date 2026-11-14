//! `atl jsm`: Jira Service Management service desks, queues and customer
//! requests (Cloud and Data Center; uses the Jira login).

use crate::Exit;
use crate::jira::ISSUE_COLUMNS;
use crate::output::{bold, col, dim, flex};
use crate::tasks::{Ctx, key_and_text, open_url, read_text, s};
use anyhow::Result;
use atl_sdk::Error;
use clap::Subcommand;
use serde_json::{Value, json};
use std::path::PathBuf;

#[derive(Subcommand, Debug)]
pub enum JsmCommand {
    /// Service desks and their request types.
    #[command(subcommand)]
    Desk(DeskCommand),
    /// Queues of a service desk.
    #[command(subcommand)]
    Queue(QueueCommand),
    /// Customer requests: list, view, create, comment, transitions, move.
    #[command(subcommand)]
    Request(RequestCommand),
}

#[derive(Subcommand, Debug)]
pub enum DeskCommand {
    /// List service desks.
    #[command(visible_alias = "ls")]
    List,
    /// Request types of a desk (ID or project key; default: [jsm] desk in .atl.toml).
    Types {
        /// Service desk ID or project key (default: `[jsm] desk` in .atl.toml).
        desk: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
pub enum QueueCommand {
    /// Queues of a desk with issue counts.
    #[command(visible_alias = "ls")]
    List {
        /// Service desk ID or project key (default: `[jsm] desk` in .atl.toml).
        desk: Option<String>,
    },
    /// Issues in a queue: `atl jsm queue issues "Open requests" [--desk HELP]`.
    Issues {
        /// Queue ID or name.
        queue: String,
        /// Service desk ID or project key (default: `[jsm] desk` in .atl.toml).
        #[arg(long)]
        desk: Option<String>,
        /// Maximum number of results.
        #[arg(long, default_value_t = 50)]
        limit: usize,
    },
}

#[derive(Subcommand, Debug)]
pub enum RequestCommand {
    /// Requests by status (all desks unless --desk or [jsm] desk is set).
    #[command(visible_alias = "ls")]
    List {
        /// Service desk ID or project key (default: `[jsm] desk` in .atl.toml).
        #[arg(long)]
        desk: Option<String>,
        /// open, closed, or all.
        #[arg(long, short = 's', default_value = "open")]
        status: String,
        /// Words to find.
        #[arg(long)]
        search: Option<String>,
        /// Only requests you raised.
        #[arg(long)]
        mine: bool,
        /// Maximum number of results.
        #[arg(long, default_value_t = 50)]
        limit: usize,
    },
    /// Show a request with its description and latest comments.
    View {
        /// Request key, for example HELP-7.
        key: String,
        /// Number of comments to show.
        #[arg(long, default_value_t = 5)]
        comments: usize,
        /// Open the request in the browser instead.
        #[arg(long, short = 'w')]
        web: bool,
    },
    /// Raise a request: `atl jsm request create --type "Get IT help" -s "Laptop broken"`.
    Create {
        /// Service desk ID or project key (default: `[jsm] desk` in .atl.toml).
        #[arg(long)]
        desk: Option<String>,
        /// Request type ID or name (see `atl jsm desk types`).
        #[arg(long = "type", short = 't')]
        request_type: String,
        /// Summary of the request.
        #[arg(long, short = 's')]
        summary: String,
        /// Description of the request.
        #[arg(long, short = 'b')]
        body: Option<String>,
        /// Read the description from a file, or - for stdin.
        #[arg(long)]
        body_file: Option<PathBuf>,
        /// Customer email or account ID to raise the request for.
        #[arg(long)]
        on_behalf_of: Option<String>,
    },
    /// Reply to a request (`--internal` for a note customers cannot see).
    Comment {
        /// `KEY [TEXT]`.
        #[arg(required = true, num_args = 1..=2, value_names = ["KEY", "TEXT"])]
        args: Vec<String>,
        /// Read the text from a file (`-` for stdin).
        #[arg(long)]
        body_file: Option<PathBuf>,
        /// Add an internal note that customers cannot see.
        #[arg(long)]
        internal: bool,
    },
    /// List the transitions a customer can run on a request.
    Transitions {
        /// Request key, for example HELP-7.
        key: String,
    },
    /// Run a transition: `atl jsm request move HELP-1 "Resolve this issue"`.
    Move {
        /// Request key, for example HELP-7.
        key: String,
        /// Transition name or ID.
        transition: String,
        /// Comment to add with the transition.
        #[arg(long)]
        comment: Option<String>,
    },
    /// List a request's approvals.
    Approvals {
        /// Request key, for example HELP-7.
        key: String,
    },
    /// Approve the pending approval you can answer (or the one with APPROVAL_ID).
    Approve {
        /// Request key, for example HELP-7.
        key: String,
        /// Approval ID (default: the one pending approval you can answer).
        approval: Option<String>,
    },
    /// Decline the pending approval you can answer (or the one with APPROVAL_ID).
    Decline {
        /// Request key, for example HELP-7.
        key: String,
        /// Approval ID (default: the one pending approval you can answer).
        approval: Option<String>,
    },
    /// List participants; with --add or --remove (me, an email or a name), change them.
    Participants {
        /// Request key, for example HELP-7.
        key: String,
        /// Add a participant: `me`, an email or a name (repeatable).
        #[arg(long)]
        add: Vec<String>,
        /// Remove a participant: `me`, an email or a name (repeatable).
        #[arg(long)]
        remove: Vec<String>,
    },
    /// SLAs: state, time remaining, goal.
    Sla {
        /// Request key, for example HELP-7.
        key: String,
    },
    /// Attach files (customers see them unless --internal).
    Attach {
        /// Request key, for example HELP-7.
        key: String,
        /// Files to attach.
        #[arg(required = true)]
        files: Vec<PathBuf>,
        /// Attach for agents only; customers cannot see it.
        #[arg(long)]
        internal: bool,
    },
    /// List attachments with download URLs.
    Attachments {
        /// Request key, for example HELP-7.
        key: String,
    },
}

/// `request approve` and `request decline`.
async fn answer(
    ctx: &Ctx<'_>,
    key: String,
    approval: Option<String>,
    decision: &str,
) -> Result<Exit> {
    ctx.item(
        "jsm_request_approve",
        json!({ "key": key, "approval": approval, "decision": decision }),
        |a| format!("{key}: {} ({})", s(a, "decision"), s(a, "name")),
    )
    .await
}

fn render_request(r: &Value, tty: bool) -> String {
    let mut out = format!(
        "{}  {}  {}\n{}\n{}\n\n{}\n",
        bold(s(r, "key"), tty),
        s(r, "summary"),
        s(r, "status"),
        dim(
            &format!(
                "{} · by {} · {}",
                s(r, "type"),
                s(r, "reporter"),
                s(r, "created").chars().take(10).collect::<String>()
            ),
            tty
        ),
        dim(s(r, "url"), tty),
        match s(r, "description") {
            "" => "(no description)".to_owned(),
            text => crate::output::markdown(text, tty),
        }
    );
    for comment in r["comments"].as_array().into_iter().flatten() {
        let note = if comment["public"] == false {
            " (internal)"
        } else {
            ""
        };
        out.push_str(&format!(
            "\n{}{note} · {}\n{}\n",
            bold(s(comment, "author"), tty),
            s(comment, "created").chars().take(10).collect::<String>(),
            crate::output::markdown(s(comment, "body"), tty)
        ));
    }
    out
}

pub async fn run(ctx: &Ctx<'_>, command: JsmCommand) -> Result<Exit> {
    match command {
        JsmCommand::Desk(DeskCommand::List) => {
            ctx.list(
                "jsm_desk_list",
                json!({}),
                &[
                    col("ID", "id"),
                    col("PROJECT", "project"),
                    flex("NAME", "name"),
                ],
            )
            .await
        }
        JsmCommand::Desk(DeskCommand::Types { desk: d }) => {
            ctx.list(
                "jsm_request_types",
                json!({ "desk": ctx.repo.require_desk(d)? }),
                &[
                    col("ID", "id"),
                    col("NAME", "name"),
                    flex("DESCRIPTION", "description"),
                ],
            )
            .await
        }
        JsmCommand::Queue(QueueCommand::List { desk: d }) => {
            ctx.list(
                "jsm_queue_list",
                json!({ "desk": ctx.repo.require_desk(d)? }),
                &[
                    col("ID", "id"),
                    col("ISSUES", "issues"),
                    flex("NAME", "name"),
                ],
            )
            .await
        }
        JsmCommand::Queue(QueueCommand::Issues {
            queue,
            desk: d,
            limit,
        }) => {
            ctx.list(
                "jsm_queue_issues",
                json!({ "desk": ctx.repo.require_desk(d)?, "queue": queue, "limit": limit }),
                &ISSUE_COLUMNS,
            )
            .await
        }
        JsmCommand::Request(command) => run_request(ctx, command).await,
    }
}

async fn run_request(ctx: &Ctx<'_>, command: RequestCommand) -> Result<Exit> {
    match command {
        RequestCommand::List {
            desk: d,
            status,
            search,
            mine,
            limit,
        } => {
            ctx.list(
                "jsm_request_list",
                json!({
                    "desk": ctx.repo.desk(d),
                    "status": status,
                    "search": search,
                    "mine": mine,
                    "limit": limit,
                }),
                &[
                    col("KEY", "key"),
                    col("STATUS", "status"),
                    col("TYPE", "type"),
                    col("REPORTER", "reporter"),
                    flex("SUMMARY", "summary"),
                ],
            )
            .await
        }
        RequestCommand::View { key, comments, web } => {
            let request = ctx
                .task(
                    "jsm_request_view",
                    json!({ "key": key, "comments": comments }),
                )
                .await?;
            if web {
                return open_url(ctx.out, s(&request, "url"));
            }
            let tty = ctx.out.tty;
            ctx.out
                .item(request, Value::Null, |r| render_request(r, tty))
        }
        RequestCommand::Create {
            desk: d,
            request_type,
            summary,
            body,
            body_file,
            on_behalf_of,
        } => {
            let description = read_text(body, body_file.as_deref())?;
            ctx.item(
                "jsm_request_create",
                json!({
                    "desk": ctx.repo.require_desk(d)?,
                    "type": request_type,
                    "summary": summary,
                    "description": description,
                    "on_behalf_of": on_behalf_of,
                }),
                |r| format!("Raised {} {}", s(r, "key"), s(r, "url")),
            )
            .await
        }
        RequestCommand::Comment {
            args,
            body_file,
            internal,
        } => {
            let (key, text) = key_and_text(
                args,
                body_file.as_deref(),
                |_| true,
                |key| {
                    key.map(|k| k.to_ascii_uppercase()).ok_or_else(|| {
                        Error::Invalid("pass `KEY TEXT` or `KEY --body-file <path>`".into())
                    })
                },
            )?;
            ctx.item(
                "jsm_request_comment",
                json!({ "key": key, "body": text, "internal": internal }),
                |_| format!("Commented on {key}"),
            )
            .await
        }
        RequestCommand::Transitions { key } => {
            ctx.list(
                "jsm_request_transitions",
                json!({ "key": key }),
                &[col("ID", "id"), flex("TRANSITION", "name")],
            )
            .await
        }
        RequestCommand::Move {
            key,
            transition,
            comment,
        } => {
            ctx.item(
                "jsm_request_move",
                json!({ "key": key, "transition": transition, "comment": comment }),
                |m| format!("{key}: {}", s(m, "transition")),
            )
            .await
        }
        RequestCommand::Approvals { key } => {
            ctx.list(
                "jsm_request_approvals",
                json!({ "key": key }),
                &[
                    col("ID", "id"),
                    col("DECISION", "decision"),
                    col("YOU CAN ANSWER", "can_answer"),
                    col("NAME", "name"),
                    flex("APPROVERS", "approvers"),
                ],
            )
            .await
        }
        RequestCommand::Approve { key, approval } => answer(ctx, key, approval, "approve").await,
        RequestCommand::Decline { key, approval } => answer(ctx, key, approval, "decline").await,
        RequestCommand::Participants { key, add, remove } => {
            ctx.item(
                "jsm_request_participants",
                json!({ "key": key, "add": add, "remove": remove }),
                |p| {
                    let names: Vec<&str> = p["participants"]
                        .as_array()
                        .into_iter()
                        .flatten()
                        .filter_map(Value::as_str)
                        .collect();
                    match names.is_empty() {
                        true => format!("{key} has no participants"),
                        false => format!("Participants of {key}: {}", names.join(", ")),
                    }
                },
            )
            .await
        }
        RequestCommand::Sla { key } => {
            ctx.list(
                "jsm_request_sla",
                json!({ "key": key }),
                &[
                    col("STATE", "state"),
                    col("REMAINING", "remaining"),
                    col("GOAL", "goal"),
                    flex("NAME", "name"),
                ],
            )
            .await
        }
        RequestCommand::Attach {
            key,
            files,
            internal,
        } => {
            let files: Vec<PathBuf> = files
                .into_iter()
                .map(|f| std::path::absolute(&f).unwrap_or(f))
                .collect();
            ctx.item(
                "jsm_request_attach",
                json!({ "key": key, "files": files, "internal": internal }),
                |a| {
                    format!(
                        "Attached {} file(s) to {key}",
                        a.as_array().map_or(0, Vec::len)
                    )
                },
            )
            .await
        }
        RequestCommand::Attachments { key } => {
            ctx.list(
                "jsm_request_attachments",
                json!({ "key": key }),
                &[
                    col("FILE", "filename"),
                    col("SIZE", "size"),
                    col("AUTHOR", "author"),
                    flex("URL", "url"),
                ],
            )
            .await
        }
    }
}
