//! `atl bitbucket`: repositories, branches, pull requests, pipelines.

use crate::Exit;
use crate::output::{bold, col, dim, flex};
use crate::tasks::{Ctx, confirm, git, git_branch, key_and_text, open_url, read_text, s};
use anyhow::Result;
use atl_sdk::bitbucket::RepoRef;
use atl_sdk::{Error, Family};
use clap::{Args, Subcommand};
use serde_json::{Value, json};
use std::path::PathBuf;

#[derive(Args, Debug)]
pub struct BitbucketArgs {
    /// WORKSPACE/REPO (Cloud) or PROJECT/REPO (Data Center). Default:
    /// [bitbucket] repo in .atl.toml, then the git `origin` remote.
    #[arg(long, short = 'R', global = true)]
    repo: Option<String>,
    #[command(subcommand)]
    command: BitbucketCommand,
}

#[derive(Subcommand, Debug)]
enum BitbucketCommand {
    /// Repositories: list, view.
    #[command(subcommand)]
    Repo(RepoCommand),
    /// Branches.
    #[command(subcommand)]
    Branch(BranchCommand),
    /// Pull requests: list, view, create, edit, checkout, merge, approve, unapprove, request-changes, decline, ready, reviewers, comment, comments, checks, diff.
    #[command(subcommand)]
    Pr(PrCommand),
    /// Commits.
    #[command(subcommand)]
    Commit(CommitCommand),
    /// Bitbucket Pipelines (Cloud only): list, view, logs, run, stop.
    #[command(subcommand)]
    Pipeline(PipelineCommand),
}

#[derive(Subcommand, Debug)]
enum RepoCommand {
    /// Repositories of a workspace or project; without --owner, the ones you belong to.
    #[command(visible_alias = "ls")]
    List {
        /// Workspace (Cloud) or project key (Data Center).
        #[arg(long)]
        owner: Option<String>,
        /// Maximum number of results.
        #[arg(long, default_value_t = 50)]
        limit: usize,
    },
    /// Show the repository (default: the current one).
    View {
        /// Open it in the browser instead.
        #[arg(long, short = 'w')]
        web: bool,
    },
}

#[derive(Subcommand, Debug)]
enum BranchCommand {
    /// Branches, most recently changed first.
    #[command(visible_alias = "ls")]
    List {
        /// Part of the branch name.
        filter: Option<String>,
        /// Maximum number of results.
        #[arg(long, default_value_t = 50)]
        limit: usize,
    },
    /// Create a branch from another branch, a tag or a commit (default: the main branch).
    Create {
        /// Branch name.
        name: String,
        /// Branch or commit to start from (default: the main branch).
        #[arg(long)]
        from: Option<String>,
    },
    /// Delete a branch (asks first; --yes skips the question).
    Delete {
        /// Branch name.
        name: String,
    },
}

#[derive(Subcommand, Debug)]
enum CommitCommand {
    /// Recent commits, newest first.
    #[command(visible_alias = "ls")]
    List {
        /// Branch (default: the main branch).
        #[arg(long, short = 'b')]
        branch: Option<String>,
        /// Maximum number of results.
        #[arg(long, default_value_t = 20)]
        limit: usize,
    },
}

#[derive(Subcommand, Debug)]
enum PrCommand {
    /// Pull requests by state.
    #[command(visible_alias = "ls")]
    List {
        /// open, merged, declined, or all.
        #[arg(long, short = 's', default_value = "open")]
        state: String,
        /// Maximum number of results.
        #[arg(long, default_value_t = 50)]
        limit: usize,
    },
    /// Show a pull request with its description, reviewers and approvals.
    View {
        /// Pull request ID.
        id: u64,
        /// Open it in the browser instead.
        #[arg(long, short = 'w')]
        web: bool,
    },
    /// Open a pull request from the current branch (or --from).
    Create {
        /// Pull request title.
        #[arg(long, short = 't')]
        title: String,
        /// Source branch (default: the current git branch).
        #[arg(long)]
        from: Option<String>,
        /// Destination branch (default: the repository's main branch).
        #[arg(long)]
        to: Option<String>,
        /// Description in Markdown.
        #[arg(long, short = 'b')]
        body: Option<String>,
        /// Read the description from a file, or - for stdin.
        #[arg(long)]
        body_file: Option<PathBuf>,
        /// Open it as a draft (not ready for review).
        #[arg(long)]
        draft: bool,
    },
    /// Merge a pull request (asks first; --yes skips the question).
    Merge {
        /// Pull request ID.
        id: u64,
        /// Cloud only: merge_commit, squash, or fast_forward.
        #[arg(long)]
        strategy: Option<String>,
    },
    /// Approve a pull request.
    Approve {
        /// Pull request ID.
        id: u64,
    },
    /// Remove your approval.
    Unapprove {
        /// Pull request ID.
        id: u64,
    },
    /// Mark a draft as ready for review (--undo turns it back into a draft).
    Ready {
        /// Pull request ID.
        id: u64,
        /// Turn the pull request back into a draft.
        #[arg(long)]
        undo: bool,
    },
    /// List comments (general and inline, with file and line), oldest first, as Markdown.
    Comments {
        /// Pull request ID.
        id: u64,
        /// Maximum number of results.
        #[arg(long, default_value_t = 200)]
        limit: usize,
    },
    /// Build statuses of the latest commit; exits 1 when one failed.
    Checks {
        /// Pull request ID.
        id: u64,
    },
    /// Comment: `atl bitbucket pr comment ID "text"` or --body-file.
    Comment {
        /// `ID [TEXT]`.
        #[arg(required = true, num_args = 1..=2, value_names = ["ID", "TEXT"])]
        args: Vec<String>,
        /// Read the comment (Markdown) from a file, or - for stdin.
        #[arg(long)]
        body_file: Option<PathBuf>,
    },
    /// Print the unified diff.
    Diff {
        /// Pull request ID.
        id: u64,
    },
    /// Decline a pull request (asks first; --yes skips; Cloud cannot reopen it).
    Decline {
        /// Pull request ID.
        id: u64,
        /// Post this comment (Markdown) before declining.
        #[arg(long)]
        comment: Option<String>,
    },
    /// Ask for changes (Cloud: request changes; Data Center: needs work).
    RequestChanges {
        /// Pull request ID.
        id: u64,
    },
    /// Change the title, description or destination branch.
    Edit {
        /// Pull request ID.
        id: u64,
        /// Pull request title.
        #[arg(long, short = 't')]
        title: Option<String>,
        /// New description in Markdown.
        #[arg(long, short = 'b')]
        body: Option<String>,
        /// Read the description from a file, or - for stdin.
        #[arg(long)]
        body_file: Option<PathBuf>,
        /// New destination branch.
        #[arg(long)]
        to: Option<String>,
    },
    /// List reviewers; with --add or --remove, change them.
    Reviewers {
        /// Pull request ID.
        id: u64,
        /// Reviewer to add (Cloud: display name, nickname, account ID or {uuid}; Data Center: username, name or email); repeatable.
        #[arg(long)]
        add: Vec<String>,
        /// Reviewer to remove; repeatable.
        #[arg(long)]
        remove: Vec<String>,
    },
    /// Fetch the pull request's source branch and switch to it (git fetch + git switch).
    Checkout {
        /// Pull request ID.
        id: u64,
    },
}

#[derive(Subcommand, Debug)]
enum PipelineCommand {
    /// Latest runs, newest first.
    #[command(visible_alias = "ls")]
    List {
        /// Maximum number of results.
        #[arg(long, default_value_t = 20)]
        limit: usize,
    },
    /// One pipeline with its steps: build number or UUID.
    View {
        /// Build number or UUID.
        id: String,
    },
    /// Step logs: every step, or one with --step NAME.
    Logs {
        /// Build number or UUID.
        id: String,
        /// Step name or UUID.
        #[arg(long)]
        step: Option<String>,
    },
    /// Start the pipeline of a branch (default: the current git branch).
    Run {
        /// Branch to run the pipeline on.
        #[arg(long)]
        branch: Option<String>,
        /// Run this custom pipeline from bitbucket-pipelines.yml.
        #[arg(long)]
        custom: Option<String>,
    },
    /// Stop a running pipeline: build number or UUID.
    Stop {
        /// Build number or UUID.
        id: String,
    },
}

/// --repo, then .atl.toml, then the `origin` remote when it points at the
/// configured Bitbucket server.
fn repo(ctx: &Ctx<'_>, flag: Option<String>) -> Result<String, Error> {
    if let Some(repo) = ctx.repo.repo(flag) {
        return Ok(RepoRef::parse(&repo)?.to_string());
    }
    let host = url::Url::parse(&ctx.client.site_url(Family::Bitbucket))
        .ok()
        .and_then(|u| {
            u.host_str()
                .map(|h| h.trim_start_matches("api.").to_owned())
        })
        .unwrap_or_else(|| "bitbucket.org".into());
    git(&["remote", "get-url", "origin"])
        .filter(|remote| remote.contains(&host))
        .and_then(|remote| RepoRef::from_remote(&remote))
        .map(|repo| repo.to_string())
        .ok_or_else(|| {
            Error::Invalid(format!(
                "pass --repo WORKSPACE/REPO (or set [bitbucket] repo in .atl.toml, or run inside a clone whose origin is on {host})"
            ))
        })
}

fn current_branch(flag: Option<String>, what: &str) -> Result<String, Error> {
    flag.or_else(git_branch).ok_or_else(|| {
        Error::Invalid(format!(
            "pass {what} (not on a git branch, so it cannot default to the current one)"
        ))
    })
}

fn render_pr(pr: &Value, tty: bool) -> String {
    let list = |key: &str| -> String {
        pr[key]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .collect::<Vec<_>>()
            .join(", ")
    };
    let mut out = format!(
        "{}  {}  {}\n",
        bold(&format!("#{}", pr["id"]), tty),
        s(pr, "title"),
        s(pr, "state")
    );
    out.push_str(&dim(
        &format!(
            "{} → {} · by {} · updated {}",
            s(pr, "source"),
            s(pr, "destination"),
            s(pr, "author"),
            s(pr, "updated").chars().take(10).collect::<String>()
        ),
        tty,
    ));
    out.push('\n');
    if !list("reviewers").is_empty() {
        out.push_str(&dim(
            &format!(
                "Reviewers: {} · Approved: {}",
                list("reviewers"),
                list("approved_by")
            ),
            tty,
        ));
        out.push('\n');
    }
    out.push_str(&dim(s(pr, "url"), tty));
    out.push_str("\n\n");
    out.push_str(&match s(pr, "description") {
        "" => "(no description)".to_owned(),
        text => crate::output::markdown(text, tty),
    });
    out
}

const PR_COLUMNS: [crate::output::Col; 5] = [
    col("ID", "id"),
    col("STATE", "state"),
    col("AUTHOR", "author"),
    col("FROM", "source"),
    flex("TITLE", "title"),
];

pub async fn run(ctx: &Ctx<'_>, args: BitbucketArgs) -> Result<Exit> {
    let flag = args.repo;
    let repo = || repo(ctx, flag.clone());
    match args.command {
        BitbucketCommand::Repo(RepoCommand::List { owner, limit }) => {
            ctx.list(
                "bitbucket_repo_list",
                json!({ "owner": owner, "limit": limit }),
                &[
                    col("REPO", "repo"),
                    col("BRANCH", "default_branch"),
                    flex("DESCRIPTION", "description"),
                ],
            )
            .await
        }
        BitbucketCommand::Repo(RepoCommand::View { web }) => {
            let found = ctx
                .task("bitbucket_repo_view", json!({ "repo": repo()? }))
                .await?;
            if web {
                return open_url(ctx.out, s(&found, "url"));
            }
            ctx.out.item(found, Value::Null, |r| {
                format!(
                    "{}  default branch {}\n{}\n{}",
                    s(r, "repo"),
                    s(r, "default_branch"),
                    s(r, "url"),
                    s(r, "description")
                )
            })
        }
        BitbucketCommand::Branch(BranchCommand::List { filter, limit }) => {
            ctx.list(
                "bitbucket_branch_list",
                json!({ "repo": repo()?, "filter": filter, "limit": limit }),
                &[col("NAME", "name"), flex("COMMIT", "commit")],
            )
            .await
        }
        BitbucketCommand::Branch(BranchCommand::Create { name, from }) => {
            ctx.item(
                "bitbucket_branch_create",
                json!({ "repo": repo()?, "name": name, "from": from }),
                |b| format!("Created branch {} at {}", s(b, "name"), s(b, "commit")),
            )
            .await
        }
        BitbucketCommand::Branch(BranchCommand::Delete { name }) => {
            let repo = repo()?;
            confirm(ctx.yes, &format!("delete branch {name} in {repo}"))?;
            ctx.item(
                "bitbucket_branch_delete",
                json!({ "repo": repo, "name": name }),
                |_| format!("Deleted branch {name}"),
            )
            .await
        }
        BitbucketCommand::Commit(CommitCommand::List { branch, limit }) => {
            ctx.list(
                "bitbucket_commit_list",
                json!({ "repo": repo()?, "branch": branch, "limit": limit }),
                &[
                    col("HASH", "hash"),
                    col("AUTHOR", "author"),
                    col("DATE", "date"),
                    flex("MESSAGE", "message"),
                ],
            )
            .await
        }
        BitbucketCommand::Pipeline(PipelineCommand::Stop { id }) => {
            ctx.item(
                "bitbucket_pipeline_stop",
                json!({ "repo": repo()?, "id": id }),
                |p| {
                    format!(
                        "Pipeline #{} is {} {}",
                        p["build_number"],
                        s(p, "state"),
                        s(p, "url")
                    )
                },
            )
            .await
        }
        BitbucketCommand::Pr(command) => run_pr(ctx, repo()?, command).await,
        BitbucketCommand::Pipeline(PipelineCommand::List { limit }) => {
            ctx.list(
                "bitbucket_pipeline_list",
                json!({ "repo": repo()?, "limit": limit }),
                &[
                    col("#", "build_number"),
                    col("STATE", "state"),
                    col("RESULT", "result"),
                    col("BRANCH", "branch"),
                    col("CREATED", "created"),
                    flex("UUID", "uuid"),
                ],
            )
            .await
        }
        BitbucketCommand::Pipeline(PipelineCommand::View { id }) => {
            ctx.item(
                "bitbucket_pipeline_view",
                json!({ "repo": repo()?, "id": id }),
                |p| {
                    let mut out = format!(
                        "#{} {} {} {}\n{}\n",
                        p["build_number"],
                        s(p, "state"),
                        s(p, "result"),
                        s(p, "branch"),
                        s(p, "url")
                    );
                    for step in p["steps"].as_array().into_iter().flatten() {
                        let result = step["result"].as_str().unwrap_or(s(step, "state"));
                        out.push_str(&format!("  {result:<11} {}\n", s(step, "name")));
                    }
                    out
                },
            )
            .await
        }
        BitbucketCommand::Pipeline(PipelineCommand::Logs { id, step }) => {
            ctx.item(
                "bitbucket_pipeline_logs",
                json!({ "repo": repo()?, "id": id, "step": step }),
                |l| s(l, "log").trim_end().to_owned(),
            )
            .await
        }
        BitbucketCommand::Pipeline(PipelineCommand::Run { branch, custom }) => {
            let branch = current_branch(branch, "--branch")?;
            ctx.item(
                "bitbucket_pipeline_run",
                json!({ "repo": repo()?, "branch": branch, "custom": custom }),
                |p| format!("Started pipeline #{} {}", p["build_number"], s(p, "url")),
            )
            .await
        }
    }
}

async fn run_pr(ctx: &Ctx<'_>, repo: String, command: PrCommand) -> Result<Exit> {
    match command {
        PrCommand::List { state, limit } => {
            let rows = ctx
                .task(
                    "bitbucket_pr_list",
                    json!({ "repo": repo, "state": state, "limit": limit }),
                )
                .await?;
            ctx.out.list(rows, &PR_COLUMNS, json!({ "repo": repo }))
        }
        PrCommand::View { id, web } => {
            let pr = ctx
                .task("bitbucket_pr_view", json!({ "repo": repo, "id": id }))
                .await?;
            if web {
                return open_url(ctx.out, s(&pr, "url"));
            }
            let tty = ctx.out.tty;
            ctx.out.item(pr, Value::Null, |pr| render_pr(pr, tty))
        }
        PrCommand::Create {
            title,
            from,
            to,
            body,
            body_file,
            draft,
        } => {
            let source = current_branch(from, "--from <branch>")?;
            let description = read_text(body, body_file.as_deref())?;
            ctx.item(
                "bitbucket_pr_create",
                json!({
                    "repo": repo,
                    "title": title,
                    "source": source,
                    "destination": to,
                    "description": description,
                    "draft": draft,
                }),
                |pr| format!("Opened #{} {}", pr["id"], s(pr, "url")),
            )
            .await
        }
        PrCommand::Merge { id, strategy } => {
            confirm(ctx.yes, &format!("merge pull request #{id} in {repo}"))?;
            ctx.item(
                "bitbucket_pr_merge",
                json!({ "repo": repo, "id": id, "strategy": strategy }),
                |pr| format!("Merged #{id} ({})", s(pr, "state")),
            )
            .await
        }
        PrCommand::Approve { id } => {
            ctx.item(
                "bitbucket_pr_approve",
                json!({ "repo": repo, "id": id }),
                |_| format!("Approved #{id}"),
            )
            .await
        }
        PrCommand::Comment { args, body_file } => {
            let (id, text) = key_and_text(
                args,
                body_file.as_deref(),
                |_| true,
                |id| {
                    id.ok_or_else(|| {
                        Error::Invalid("pass `ID TEXT` or `ID --body-file <path>`".into())
                    })
                },
            )?;
            let id: u64 = id.parse().map_err(|_| {
                Error::Invalid(format!(
                    "`{id}` is not a pull request ID; pass a number, for example `atl bitbucket pr comment 12 \"Looks good\"`"
                ))
            })?;
            ctx.item(
                "bitbucket_pr_comment",
                json!({ "repo": repo, "id": id, "body": text }),
                |_| format!("Commented on #{id}"),
            )
            .await
        }
        PrCommand::Diff { id } => {
            ctx.item(
                "bitbucket_pr_diff",
                json!({ "repo": repo, "id": id }),
                |d| s(d, "diff").to_owned(),
            )
            .await
        }
        PrCommand::Decline { id, comment } => {
            confirm(ctx.yes, &format!("decline pull request #{id} in {repo}"))?;
            if let Some(text) = comment {
                ctx.task(
                    "bitbucket_pr_comment",
                    json!({ "repo": repo, "id": id, "body": text }),
                )
                .await?;
            }
            ctx.item(
                "bitbucket_pr_decline",
                json!({ "repo": repo, "id": id }),
                |pr| format!("Declined #{id} ({})", s(pr, "state")),
            )
            .await
        }
        PrCommand::Unapprove { id } => {
            ctx.item(
                "bitbucket_pr_unapprove",
                json!({ "repo": repo, "id": id }),
                |_| format!("Removed your approval from #{id}"),
            )
            .await
        }
        PrCommand::Ready { id, undo } => {
            ctx.item(
                "bitbucket_pr_ready",
                json!({ "repo": repo, "id": id, "draft": undo }),
                |_| match undo {
                    true => format!("#{id} is a draft again"),
                    false => format!("#{id} is ready for review"),
                },
            )
            .await
        }
        PrCommand::Comments { id, limit } => {
            let rows = ctx
                .task(
                    "bitbucket_pr_comments",
                    json!({ "repo": repo, "id": id, "limit": limit }),
                )
                .await?;
            let tty = ctx.out.tty;
            if !ctx.out.wants_text() {
                return ctx.out.list(
                    rows,
                    &[
                        col("ID", "id"),
                        col("AUTHOR", "author"),
                        col("PATH", "path"),
                        flex("BODY", "body"),
                    ],
                    json!({ "repo": repo, "id": id }),
                );
            }
            let mut text = String::new();
            for c in rows.as_array().into_iter().flatten() {
                let place = match (c["path"].as_str(), c["line"].as_u64()) {
                    (Some(path), Some(line)) => format!(" on {path}:{line}"),
                    (Some(path), None) => format!(" on {path}"),
                    _ => String::new(),
                };
                let reply = match c["parent"].as_str() {
                    Some(parent) => format!(" (reply to {parent})"),
                    None => String::new(),
                };
                let resolved = if c["resolved"] == true {
                    " · resolved"
                } else {
                    ""
                };
                text.push_str(&bold(
                    &format!(
                        "{} · {}{place}{reply}{resolved}",
                        s(c, "author"),
                        s(c, "created").chars().take(16).collect::<String>()
                    ),
                    tty,
                ));
                text.push_str(&dim(&format!("  #{}", s(c, "id")), tty));
                text.push('\n');
                text.push_str(&crate::output::markdown(s(c, "body"), tty));
                text.push_str("\n\n");
            }
            if text.is_empty() {
                text = format!("#{id} has no comments");
            }
            ctx.out.item(Value::Null, Value::Null, |_| text)
        }
        PrCommand::Checks { id } => {
            let result = ctx
                .task("bitbucket_pr_checks", json!({ "repo": repo, "id": id }))
                .await?;
            let failed = result["passed"] == false;
            let exit = if ctx.out.wants_text() {
                let rows = result["checks"].clone();
                let code = ctx.out.list(
                    rows,
                    &[
                        col("STATE", "state"),
                        col("NAME", "name"),
                        flex("URL", "url"),
                    ],
                    Value::Null,
                )?;
                if result["checks"].as_array().is_none_or(Vec::is_empty) {
                    eprintln!("#{id} has no build statuses");
                }
                code
            } else {
                ctx.out.item(result, Value::Null, |_| String::new())?
            };
            Ok(if failed { Exit::Api } else { exit })
        }
        PrCommand::RequestChanges { id } => {
            ctx.item(
                "bitbucket_pr_request_changes",
                json!({ "repo": repo, "id": id }),
                |_| format!("Requested changes on #{id}"),
            )
            .await
        }
        PrCommand::Edit {
            id,
            title,
            body,
            body_file,
            to,
        } => {
            let description = read_text(body, body_file.as_deref())?;
            ctx.item(
                "bitbucket_pr_edit",
                json!({ "repo": repo, "id": id, "title": title, "description": description, "destination": to }),
                |pr| format!("Updated #{id} {}", s(pr, "url")),
            )
            .await
        }
        PrCommand::Reviewers { id, add, remove } => {
            let pr = if add.is_empty() && remove.is_empty() {
                ctx.task("bitbucket_pr_view", json!({ "repo": repo, "id": id }))
                    .await?
            } else {
                ctx.task(
                    "bitbucket_pr_reviewers",
                    json!({ "repo": repo, "id": id, "add": add, "remove": remove }),
                )
                .await?
            };
            let value =
                json!({ "id": id, "reviewers": pr["reviewers"], "approved_by": pr["approved_by"] });
            ctx.out.item(value, Value::Null, |r| {
                let names = |key: &str| {
                    r[key]
                        .as_array()
                        .into_iter()
                        .flatten()
                        .filter_map(Value::as_str)
                        .collect::<Vec<_>>()
                        .join(", ")
                };
                format!(
                    "Reviewers: {}\nApproved: {}",
                    names("reviewers"),
                    names("approved_by")
                )
            })
        }
        PrCommand::Checkout { id } => {
            let pr = ctx
                .task("bitbucket_pr_view", json!({ "repo": repo, "id": id }))
                .await?;
            let cloud = ctx.client.deployment(Family::Bitbucket) == atl_sdk::Deployment::Cloud;
            let (branch, commands) = checkout_commands(&pr, &repo, cloud, id);
            if ctx.dry_run {
                return ctx.out.item(
                    json!({ "branch": branch, "commands": commands }),
                    Value::Null,
                    |_| String::new(),
                );
            }
            for args in &commands {
                if !ctx.out.agent {
                    eprintln!("git {}", args.join(" "));
                }
                let output = std::process::Command::new("git")
                    .args(args)
                    .output()
                    .map_err(|err| Error::Invalid(format!("cannot run git ({err}); install git and run this inside a clone of {repo}")))?;
                if !output.status.success() {
                    return Err(Error::Invalid(format!(
                        "`git {}` failed: {}",
                        args.join(" "),
                        String::from_utf8_lossy(&output.stderr).trim()
                    ))
                    .into());
                }
            }
            ctx.out.item(
                json!({ "branch": branch, "commands": commands }),
                Value::Null,
                |_| format!("Switched to {branch} (pull request #{id})"),
            )
        }
    }
}

/// The local branch for a pull request and the git commands that fetch it
/// and switch to it. Same-repository pull requests use the source branch;
/// pull requests from forks are fetched into `pr/ID`.
fn checkout_commands(pr: &Value, repo: &str, cloud: bool, id: u64) -> (String, Vec<Vec<String>>) {
    let source = s(pr, "source").to_owned();
    let from_fork = pr["source_repo"]
        .as_str()
        .is_some_and(|r| !r.eq_ignore_ascii_case(repo));
    let args = |list: &[&str]| list.iter().map(|a| (*a).to_owned()).collect::<Vec<_>>();
    if !from_fork {
        return (
            source.clone(),
            vec![
                args(&["fetch", "origin", &source]),
                args(&["switch", &source]),
            ],
        );
    }
    let branch = format!("pr/{id}");
    let fetch = if cloud {
        let url = format!("https://bitbucket.org/{}.git", s(pr, "source_repo"));
        args(&["fetch", &url, &format!("{source}:{branch}")])
    } else {
        args(&[
            "fetch",
            "origin",
            &format!("refs/pull-requests/{id}/from:{branch}"),
        ])
    };
    (branch.clone(), vec![fetch, args(&["switch", &branch])])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checkout_fetches_the_branch_or_the_fork_ref() {
        let same = json!({ "source": "feature/x", "source_repo": "acme/web" });
        assert_eq!(
            checkout_commands(&same, "acme/web", true, 7),
            (
                "feature/x".to_owned(),
                vec![
                    vec!["fetch".to_owned(), "origin".into(), "feature/x".into()],
                    vec!["switch".to_owned(), "feature/x".into()],
                ]
            )
        );
        let fork = json!({ "source": "fix", "source_repo": "ada/web" });
        let (branch, commands) = checkout_commands(&fork, "acme/web", true, 7);
        assert_eq!(branch, "pr/7");
        assert_eq!(
            commands[0],
            ["fetch", "https://bitbucket.org/ada/web.git", "fix:pr/7"]
        );
        let (_, commands) = checkout_commands(&fork, "acme/web", false, 7);
        assert_eq!(
            commands[0],
            ["fetch", "origin", "refs/pull-requests/7/from:pr/7"]
        );
    }
}
