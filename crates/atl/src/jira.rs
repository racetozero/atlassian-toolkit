//! `atl jira`: issues, projects, boards, sprints, epics, filters, JQL.

use crate::Exit;
use crate::output::{Col, bold, col, dim, flex, status};
use crate::prompt;
use crate::tasks::{
    Ctx, branch_key, confirm, edit_in_editor, key_and_text, key_and_value, key_or_branch, open_url,
    read_body, read_text, s, save_download,
};
use anyhow::Result;
use atl_sdk::error::Choice;
use atl_sdk::jira::{IssueQuery, Jira, is_issue_key};
use atl_sdk::{Deployment, Error, Family};
use clap::{Args, Subcommand};
use futures_util::StreamExt;
use serde_json::{Map, Value, json};
use std::path::{Path, PathBuf};

// Parsed once per process.
#[allow(clippy::large_enum_variant)]
#[derive(Subcommand, Debug)]
pub enum JiraCommand {
    /// Issues: list, view, create, edit, delete, assign, move, comment, history, watch, link, clone, worklog, worklogs, attach.
    #[command(subcommand)]
    Issue(IssueCommand),
    /// Issue comments: list, edit, delete (`issue comment` adds one).
    #[command(subcommand)]
    Comment(CommentCommand),
    /// Project components.
    #[command(subcommand)]
    Component(ComponentCommand),
    /// System and custom fields: names, IDs and types for --field.
    #[command(subcommand)]
    Field(FieldCommand),
    /// Projects.
    #[command(subcommand)]
    Project(ListOnly),
    /// Jira Software boards.
    #[command(subcommand)]
    Board(BoardCommand),
    /// Jira Software sprints: list, issues, add, create, start, close.
    #[command(subcommand)]
    Sprint(SprintCommand),
    /// Epics: list, add issues, remove issues.
    #[command(subcommand)]
    Epic(EpicCommand),
    /// Saved filters: list and run.
    #[command(subcommand)]
    Filter(FilterCommand),
    /// Validate JQL, list JQL fields, and get value suggestions.
    #[command(subcommand)]
    Jql(JqlCommand),
    /// Project versions (releases).
    #[command(subcommand, visible_alias = "version")]
    Release(ReleaseCommand),
    /// Download an attachment by ID (see `issue attachments`; -o chooses the file).
    #[command(subcommand)]
    Attachment(AttachmentCommand),
    /// The signed-in user.
    Me,
}

#[derive(Subcommand, Debug)]
pub enum IssueCommand {
    /// List issues: a table in a terminal (`--browse` opens the issue browser), JSON when piped or with --agent.
    #[command(visible_alias = "ls")]
    List {
        #[command(flatten)]
        query: IssueQuery,
        #[command(flatten)]
        view: ListView,
    },
    /// Show an issue with its description and recent comments as Markdown.
    View {
        /// Issue key; defaults to the key in the current git branch name.
        key: Option<String>,
        /// How many of the latest comments to include.
        #[arg(long, default_value_t = 5)]
        comments: usize,
        /// Open the issue in the browser instead.
        #[arg(long, short = 'w')]
        web: bool,
        /// Also show this field (name or ID); repeatable.
        #[arg(long = "field", short = 'f', value_name = "NAME")]
        field: Vec<String>,
    },
    /// Create an issue. The body is Markdown (converted to ADF on Cloud).
    Create(IssueCreateArgs),
    /// Change summary, body, priority, labels or any field (--field NAME=VALUE).
    Edit(IssueEditArgs),
    /// Delete an issue (asks first; --yes skips the question).
    Delete {
        /// Issue key.
        key: String,
        /// Also delete its sub-tasks (Jira refuses otherwise when there are some).
        #[arg(long)]
        subtasks: bool,
    },
    /// Watch an issue (you, or --user).
    Watch {
        /// Issue key (default: the key in the git branch name).
        key: Option<String>,
        /// `me`, an email, or a name.
        #[arg(long, short = 'u', default_value = "me")]
        user: String,
    },
    /// Stop watching an issue (you, or --user).
    Unwatch {
        /// Issue key (default: the key in the git branch name).
        key: Option<String>,
        /// User to remove (default: you).
        #[arg(long, short = 'u', default_value = "me")]
        user: String,
    },
    /// List an issue's watchers.
    Watchers {
        /// Issue key (default: the key in the git branch name).
        key: Option<String>,
    },
    /// Assign: `atl jira issue assign [KEY] <me|x|user>` (x = unassigned). Without USER, lists who can be assigned.
    Assign {
        /// `[KEY...] [USER]`; KEY defaults to the key in the git branch name. Several keys assign them all.
        #[arg(num_args = 0.., value_names = ["KEY", "USER"])]
        args: Vec<String>,
    },
    /// Move to another status: `atl jira issue move [KEY] "In Progress"`. Without STATUS, lists the choices.
    #[command(visible_alias = "transition")]
    Move {
        /// `[KEY...] [STATUS]`; STATUS matches a transition or target status name. Several keys move them all.
        #[arg(num_args = 0.., value_names = ["KEY", "STATUS"])]
        args: Vec<String>,
        /// Add a comment (Markdown) with the transition.
        #[arg(long)]
        comment: Option<String>,
        /// Set the resolution, for example Fixed (it must be on the transition screen).
        #[arg(short = 'R', long)]
        resolution: Option<String>,
        /// Assign with the transition: me, x, or a user.
        #[arg(short = 'a', long)]
        assignee: Option<String>,
    },
    /// Add a comment: `atl jira issue comment [KEY] "text"`, --body-file, or piped text.
    Comment {
        /// `[KEY] [TEXT]`.
        #[arg(num_args = 0..=2, value_names = ["KEY", "TEXT"])]
        args: Vec<String>,
        /// Read the comment (Markdown) from a file, or - for stdin.
        #[arg(long, visible_alias = "template")]
        body_file: Option<PathBuf>,
        /// A Jira Service Management internal note (customers do not see it).
        #[arg(long)]
        internal: bool,
    },
    /// List the statuses an issue can move to.
    Transitions {
        /// Issue key (default: the key in the git branch name).
        key: Option<String>,
    },
    /// Count issues matching the same filters as `list`.
    Count(IssueQuery),
    /// Link two issues: `atl jira issue link A-1 A-2 Blocks` means "A-1 blocks A-2".
    Link {
        /// Issue on the left of the link type, for example A-1 in "A-1 blocks A-2".
        from: String,
        /// Issue on the right of the link type, for example A-2 in "A-1 blocks A-2".
        to: String,
        /// Link type name, outward phrase ("blocks"), or inward phrase ("is blocked by").
        kind: String,
    },
    /// Remove the links between two issues: `atl jira issue unlink A-1 A-2`.
    Unlink {
        /// Issue on the left of the link type, for example A-1 in "A-1 blocks A-2".
        from: String,
        /// Issue on the right of the link type, for example A-2 in "A-1 blocks A-2".
        to: String,
        /// Only links of this type (name, outward or inward phrase).
        #[arg(long = "type")]
        kind: Option<String>,
    },
    /// Add a web link: `atl jira issue remote-link KEY URL --title "CI run"`.
    RemoteLink {
        /// Issue key.
        key: String,
        /// URL of the web page.
        url: String,
        /// Link text (default: the URL).
        #[arg(long)]
        title: Option<String>,
    },
    /// List the issue link types.
    LinkTypes,
    /// Copy an issue (summary, description, type, labels, priority, components, parent).
    Clone {
        /// Issue key (default: the key in the git branch name).
        key: Option<String>,
        /// Summary for the copy.
        #[arg(short = 's', long)]
        summary: Option<String>,
        /// Priority for the copy.
        #[arg(short = 'y', long)]
        priority: Option<String>,
        /// Assignee for the copy: me or a user.
        #[arg(short = 'a', long)]
        assignee: Option<String>,
        /// Labels for the copy (replace the original's); repeatable.
        #[arg(short = 'l', long)]
        label: Vec<String>,
        /// Replace text in the summary and description: FIND:REPLACE; repeatable.
        #[arg(short = 'H', long, value_name = "FIND:REPLACE")]
        replace: Vec<String>,
    },
    /// Log time: `atl jira issue worklog [KEY] "2h 30m"`.
    Worklog {
        /// Issue key (optional, default: the key in the git branch name) and the time spent, for example "2h 30m".
        #[arg(required = true, num_args = 1..=2, value_names = ["KEY", "TIME"])]
        args: Vec<String>,
        /// Comment for the worklog.
        #[arg(long)]
        comment: Option<String>,
        /// Start time, for example 2026-11-09T09:00:00.000+0000 (default: now).
        #[arg(long)]
        started: Option<String>,
    },
    /// Upload files: `atl jira issue attach [KEY] FILE...`.
    Attach {
        /// Issue key (optional, default: the key in the git branch name) and the files to attach.
        #[arg(required = true, value_names = ["KEY", "FILE"])]
        args: Vec<String>,
    },
    /// List an issue's attachments.
    Attachments {
        /// Issue key (default: the key in the git branch name).
        key: Option<String>,
    },
    /// List the time logged on an issue.
    Worklogs {
        /// Issue key (default: the key in the git branch name).
        key: Option<String>,
    },
    /// Field changes, newest first: when, who, field, from, to.
    History {
        /// Issue key (default: the key in the git branch name).
        key: Option<String>,
        /// Maximum number of results.
        #[arg(long, default_value_t = 100)]
        limit: usize,
    },
}

#[derive(Args, Debug)]
pub struct IssueCreateArgs {
    /// Project key (default: [jira] project in .atl.toml).
    #[arg(short = 'p', long)]
    project: Option<String>,
    /// Issue type, for example Task, Bug, Story (default Task; asked in a terminal).
    #[arg(short = 't', long = "type")]
    issue_type: Option<String>,
    /// One-line summary (asked in a terminal when missing).
    #[arg(short = 's', long)]
    summary: Option<String>,
    /// Description in Markdown (or pipe it on stdin).
    #[arg(short = 'b', long)]
    body: Option<String>,
    /// Read the description from a file, or - for stdin.
    #[arg(long, visible_alias = "template")]
    body_file: Option<PathBuf>,
    /// Assignee: me or a user.
    #[arg(short = 'a', long)]
    assignee: Option<String>,
    /// Label; repeatable.
    #[arg(short = 'l', long)]
    label: Vec<String>,
    /// Priority name.
    #[arg(short = 'y', long)]
    priority: Option<String>,
    /// Parent issue key (for sub-tasks and child issues).
    #[arg(long)]
    parent: Option<String>,
    /// Component; repeatable.
    #[arg(short = 'C', long)]
    component: Vec<String>,
    /// Fix version; repeatable.
    #[arg(long)]
    fix_version: Vec<String>,
    /// Version the problem affects; repeatable.
    #[arg(long)]
    affects_version: Vec<String>,
    /// Original estimate, for example 2h or "1d 4h".
    #[arg(long, visible_alias = "original-estimate")]
    estimate: Option<String>,
    /// Reporter: me or a user.
    #[arg(short = 'r', long)]
    reporter: Option<String>,
    /// Set a field by name or ID: --field "Story Points=3"; repeatable. See `atl jira field list`.
    #[arg(long = "field", short = 'f', value_name = "NAME=VALUE")]
    field: Vec<String>,
    /// Open the new issue in the browser.
    #[arg(short = 'w', long)]
    web: bool,
}

#[derive(Args, Debug)]
pub struct IssueEditArgs {
    /// Issue key; defaults to the key in the git branch name.
    key: Option<String>,
    /// New summary.
    #[arg(short = 's', long)]
    summary: Option<String>,
    /// New description in Markdown.
    #[arg(short = 'b', long)]
    body: Option<String>,
    /// Read the text from a file (`-` for stdin).
    #[arg(long)]
    body_file: Option<PathBuf>,
    /// New priority, for example High.
    #[arg(short = 'y', long)]
    priority: Option<String>,
    /// Assignee: me, x (unassigned), or a user.
    #[arg(short = 'a', long)]
    assignee: Option<String>,
    /// New parent issue key.
    #[arg(long)]
    parent: Option<String>,
    /// Reporter: me or a user.
    #[arg(short = 'r', long)]
    reporter: Option<String>,
    /// Original estimate, for example 2h.
    #[arg(long, visible_alias = "original-estimate")]
    estimate: Option<String>,
    /// Add a label, or remove it with a leading - (--label -old); repeatable.
    #[arg(short = 'l', long, allow_hyphen_values = true)]
    label: Vec<String>,
    /// Remove a label; repeatable.
    #[arg(long)]
    remove_label: Vec<String>,
    /// Add a component, or remove it with a leading -; repeatable.
    #[arg(short = 'C', long, allow_hyphen_values = true)]
    component: Vec<String>,
    /// Add a fix version, or remove it with a leading -; repeatable.
    #[arg(long, allow_hyphen_values = true)]
    fix_version: Vec<String>,
    /// Add an affected version, or remove it with a leading -; repeatable.
    #[arg(long, allow_hyphen_values = true)]
    affects_version: Vec<String>,
    /// Edit the description as Markdown in $VISUAL or $EDITOR.
    #[arg(long, short = 'e')]
    editor: bool,
    /// Set a field by name or ID: --field "Story Points=5"; repeatable. See `atl jira field list`.
    #[arg(long = "field", short = 'f', value_name = "NAME=VALUE")]
    field: Vec<String>,
    /// Do not email watchers about the change (Jira may need admin rights).
    #[arg(long)]
    skip_notify: bool,
    /// Open the issue in the browser afterwards.
    #[arg(short = 'w', long)]
    web: bool,
}

/// Values to add and (without their leading `-`) values to remove.
fn split_removals(values: Vec<String>) -> (Vec<String>, Vec<String>) {
    let (remove, add): (Vec<String>, Vec<String>) =
        values.into_iter().partition(|v| v.starts_with('-'));
    (add, remove.into_iter().map(|v| v[1..].to_owned()).collect())
}

#[derive(Subcommand, Debug)]
pub enum FieldCommand {
    /// Fields whose name or ID contains FILTER.
    #[command(visible_alias = "ls")]
    List {
        /// Only fields whose name or ID contains this text.
        filter: Option<String>,
        /// Only custom fields.
        #[arg(long)]
        custom: bool,
    },
}

/// `NAME=VALUE` flags as the `fields` task argument.
fn field_args(flags: Vec<String>) -> Result<Map<String, Value>, Error> {
    flags
        .into_iter()
        .map(|flag| match flag.split_once('=') {
            Some((name, value)) if !name.trim().is_empty() => {
                Ok((name.trim().to_owned(), Value::String(value.to_owned())))
            }
            _ => Err(Error::Invalid(format!(
                "`--field {flag}` has no `=`; pass NAME=VALUE, for example --field \"Story Points=3\""
            ))),
        })
        .collect()
}

#[derive(Subcommand, Debug)]
pub enum ListOnly {
    /// List them.
    #[command(visible_alias = "ls")]
    List,
}

#[derive(Subcommand, Debug)]
pub enum BoardCommand {
    /// List boards, optionally of one project.
    #[command(visible_alias = "ls")]
    List {
        /// Project key (default: `[jira] project` in .atl.toml).
        #[arg(short = 'p', long)]
        project: Option<String>,
    },
}

// Parsed once per process.
#[allow(clippy::large_enum_variant)]
#[derive(Subcommand, Debug)]
pub enum SprintCommand {
    /// List sprints of a board (default: the project's first board).
    #[command(visible_alias = "ls")]
    List {
        /// Board ID.
        #[arg(long)]
        board: Option<u64>,
        /// Project key (default: `[jira] project` in .atl.toml).
        #[arg(short = 'p', long)]
        project: Option<String>,
        /// active, future, closed, or a comma-separated mix.
        #[arg(long, default_value = "active,future")]
        state: String,
    },
    /// Create a future sprint: `atl jira sprint create "Sprint 12" --end 2026-11-27`.
    Create {
        /// Sprint name.
        name: String,
        /// Board ID (default: the project's first board).
        #[arg(long)]
        board: Option<u64>,
        /// Project key (default: `[jira] project` in .atl.toml).
        #[arg(short = 'p', long)]
        project: Option<String>,
        /// Start date, YYYY-MM-DD or ISO 8601.
        #[arg(long)]
        start: Option<String>,
        /// End date, YYYY-MM-DD or ISO 8601.
        #[arg(long)]
        end: Option<String>,
        /// Sprint goal.
        #[arg(long)]
        goal: Option<String>,
    },
    /// Start a future sprint (start defaults to its start date, else now; end to its end date).
    Start {
        /// Sprint ID.
        sprint: u64,
        /// Start date, YYYY-MM-DD or ISO 8601 (default: now).
        #[arg(long)]
        start: Option<String>,
        /// End date, YYYY-MM-DD or ISO 8601 (Jira needs one to start a sprint).
        #[arg(long)]
        end: Option<String>,
    },
    /// Complete an active sprint.
    Close {
        /// Sprint ID.
        sprint: u64,
    },
    /// Move issues into a sprint: `atl jira sprint add SPRINT_ID KEY...`.
    Add {
        /// Sprint ID.
        sprint: u64,
        /// Issue keys.
        #[arg(required = true)]
        issues: Vec<String>,
    },
    /// Issues of a sprint (default: the board's active sprint), with every `issue list` filter.
    #[command(group = clap::ArgGroup::new("which").args(["sprint_id", "current", "prev", "next"]))]
    Issues {
        /// Sprint ID.
        #[arg(id = "sprint_id", value_name = "SPRINT_ID")]
        sprint_id: Option<u64>,
        /// The board's active sprint (the default).
        #[arg(long)]
        current: bool,
        /// The board's most recently closed sprint.
        #[arg(long)]
        prev: bool,
        /// The board's next future sprint.
        #[arg(long)]
        next: bool,
        /// Board ID (default: the first board of the project).
        #[arg(long)]
        board: Option<u64>,
        #[command(flatten)]
        query: IssueQuery,
        #[command(flatten)]
        view: ListView,
    },
}

#[derive(Subcommand, Debug)]
pub enum CommentCommand {
    /// Every comment of an issue with its ID, oldest first.
    #[command(visible_alias = "ls")]
    List {
        /// Issue key (default: the key in the git branch name).
        key: Option<String>,
        /// Maximum number of results.
        #[arg(long, default_value_t = 500)]
        limit: usize,
    },
    /// Replace a comment's text: `atl jira comment edit KEY ID "text"`, --body-file, or $EDITOR.
    Edit {
        /// Issue key.
        key: String,
        /// Comment ID.
        id: String,
        /// New text (default: open the current text in $EDITOR).
        text: Option<String>,
        /// Read the new text (Markdown) from a file, or - for stdin.
        #[arg(long)]
        body_file: Option<PathBuf>,
    },
    /// Delete a comment (asks first; --yes skips the question).
    Delete {
        /// Issue key.
        key: String,
        /// Comment ID.
        id: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum ComponentCommand {
    /// Components of a project.
    #[command(visible_alias = "ls")]
    List {
        /// Project key (default: `[jira] project` in .atl.toml).
        #[arg(short = 'p', long)]
        project: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
pub enum JqlCommand {
    /// Validate JQL with Jira's parser (exit 3 and the errors when invalid).
    Check {
        /// JQL query.
        jql: String,
    },
    /// Fields (with operators) and functions usable in JQL.
    Fields,
    /// Value suggestions: `atl jira jql suggest status In`.
    Suggest {
        /// Field name.
        field: String,
        /// Start of the value, to narrow the suggestions.
        #[arg(default_value = "")]
        prefix: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum FilterCommand {
    /// Saved filters (Cloud: all you can see; Data Center: favourites).
    #[command(visible_alias = "ls")]
    List {
        /// Part of the filter name.
        name: Option<String>,
    },
    /// Run a saved filter by ID or exact name.
    Run {
        /// Filter ID or name.
        filter: String,
        #[command(flatten)]
        view: ListView,

        /// Maximum number of results.
        #[arg(long, default_value_t = 50)]
        limit: usize,
        /// Continue from the `next_page_token` of the previous page.
        #[arg(long)]
        page_token: Option<String>,
    },
}

// Parsed once per process.
#[allow(clippy::large_enum_variant)]
#[derive(Subcommand, Debug)]
pub enum EpicCommand {
    /// Open epics.
    #[command(visible_alias = "ls")]
    List {
        /// Project key (default: `[jira] project` in .atl.toml).
        #[arg(short = 'p', long)]
        project: Option<String>,
        /// Maximum number of results.
        #[arg(long, default_value_t = 50)]
        limit: usize,
    },
    /// Move issues into an epic: `atl jira epic add EPIC KEY...`.
    Add {
        /// Epic key.
        epic: String,
        /// Issue keys.
        #[arg(required = true)]
        issues: Vec<String>,
    },
    /// Remove issues from their epic.
    Remove {
        /// Issue keys.
        #[arg(required = true)]
        issues: Vec<String>,
    },
    /// Issues of an epic, with every `issue list` filter.
    Issues {
        /// Epic key.
        epic: String,
        #[command(flatten)]
        query: IssueQuery,
        #[command(flatten)]
        view: ListView,
    },
}

#[derive(Subcommand, Debug)]
pub enum ReleaseCommand {
    /// A project's versions.
    #[command(visible_alias = "ls")]
    List {
        /// Project key (default: [jira] project in .atl.toml).
        #[arg(short = 'p', long)]
        project: Option<String>,
        /// unreleased, released, archived, or all.
        #[arg(long, default_value = "all")]
        state: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum AttachmentCommand {
    /// Download an attachment by ID.
    Download {
        /// Attachment ID.
        id: String,
    },
}

pub const ISSUE_COLUMNS: [Col; 6] = [
    col("KEY", "key"),
    col("TYPE", "type"),
    col("STATUS", "status"),
    col("ASSIGNEE", "assignee"),
    col("UPDATED", "updated"),
    flex("SUMMARY", "summary"),
];

const ATTACHMENT_COLUMNS: [Col; 5] = [
    col("ID", "id"),
    col("SIZE", "size"),
    col("CREATED", "created"),
    col("AUTHOR", "author"),
    flex("FILE", "filename"),
];

fn date(value: &Value, key: &str) -> String {
    s(value, key).chars().take(10).collect()
}

pub fn render_issue(issue: &Value, tty: bool) -> String {
    let mut out = format!(
        "{}  {}  {}\n",
        bold(s(issue, "key"), tty),
        s(issue, "summary"),
        status(s(issue, "status"), s(issue, "status_category"), tty)
    );
    let mut facts = vec![format!("Type: {}", s(issue, "type"))];
    for (label, key) in [
        ("Priority", "priority"),
        ("Assignee", "assignee"),
        ("Reporter", "reporter"),
        ("Parent", "parent"),
    ] {
        if !s(issue, key).is_empty() {
            facts.push(format!("{label}: {}", s(issue, key)));
        }
    }
    out.push_str(&dim(&facts.join(" · "), tty));
    out.push('\n');
    let labels: Vec<&str> = issue
        .get("labels")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .collect();
    let mut line = vec![format!("Updated {}", date(issue, "updated"))];
    if !labels.is_empty() {
        line.push(format!("Labels: {}", labels.join(", ")));
    }
    out.push_str(&dim(&line.join(" · "), tty));
    out.push('\n');
    if let Some(fields) = issue.get("fields").and_then(Value::as_object) {
        for (name, value) in fields {
            let shown = match value {
                Value::String(text) => text.clone(),
                Value::Null => "-".into(),
                Value::Array(items) => items
                    .iter()
                    .map(|v| v.as_str().map_or_else(|| v.to_string(), str::to_owned))
                    .collect::<Vec<_>>()
                    .join(", "),
                other => other.to_string(),
            };
            out.push_str(&format!("{}: {shown}\n", bold(name, tty)));
        }
    }
    out.push_str(&dim(s(issue, "url"), tty));
    out.push_str("\n\n");
    out.push_str(&match s(issue, "description") {
        "" => "(no description)".to_owned(),
        description => crate::output::markdown(description, tty),
    });
    out.push('\n');
    if let Some(links) = issue
        .get("links")
        .and_then(Value::as_array)
        .filter(|l| !l.is_empty())
    {
        out.push_str(&format!("\n{}\n", bold("Links", tty)));
        for link in links {
            out.push_str(&format!(
                "  {} {}  {} {}\n",
                s(link, "relation"),
                bold(s(link, "key"), tty),
                s(link, "summary"),
                dim(&format!("({})", s(link, "status")), tty)
            ));
        }
    }
    if let Some(comments) = issue
        .get("comments")
        .and_then(Value::as_array)
        .filter(|c| !c.is_empty())
    {
        let total = issue
            .get("comment_count")
            .and_then(Value::as_u64)
            .unwrap_or(comments.len() as u64);
        out.push_str(&format!(
            "\n{}\n",
            bold(&format!("Comments ({} of {total})", comments.len()), tty)
        ));
        for comment in comments {
            let author = comment
                .get("author")
                .and_then(Value::as_str)
                .unwrap_or("unknown");
            out.push_str(&format!(
                "\n{} · {}\n{}\n",
                bold(author, tty),
                date(comment, "created"),
                crate::output::markdown(s(comment, "body"), tty)
            ));
        }
    }
    out
}

/// `[KEY...] [VALUE]` positionals: the last one is the value unless it is an
/// issue key; the others are keys (with several, each must look like one).
/// Without keys, the key in the git branch name.
fn keys_and_value(mut args: Vec<String>) -> Result<(Vec<String>, Option<String>), Error> {
    let value = match args.last() {
        Some(last) if !is_issue_key(last) => args.pop(),
        _ => None,
    };
    if args.len() > 1
        && let Some(bad) = args.iter().find(|a| !is_issue_key(a))
    {
        return Err(Error::Invalid(format!(
            "`{bad}` is not an issue key; pass the keys first and one value last, for example `AB-1 AB-2 \"In Progress\"` (quote values with spaces)"
        )));
    }
    let keys = match args.as_slice() {
        [] => vec![key_or_branch(None)?],
        keys => keys.iter().map(|k| k.to_ascii_uppercase()).collect(),
    };
    Ok((keys, value))
}

/// Requests a bulk command sends at the same time: fewer round trips in a
/// row without flooding the server.
const BULK_CONCURRENCY: usize = 8;

/// Runs `task` for every key and prints one row per key. A missing value is
/// asked for (or reported) on the first key and reused for the rest;
/// `answer` is the result field that holds it, shown under `header`.
async fn bulk(
    ctx: &Ctx<'_>,
    keys: &[String],
    mut value: Option<String>,
    task: &str,
    answer: &'static str,
    header: &'static str,
    args: impl Fn(&str, &str) -> Value,
) -> Result<Exit> {
    let mut rows = Vec::with_capacity(keys.len());
    let mut rest = keys;
    // Without a value, the first issue settles it (a prompt can choose it)
    // and its failure stops the command.
    if value.is_none()
        && let Some((first, others)) = keys.split_first()
    {
        let done = ctx.task(task, args(first, "")).await?;
        value = done[answer].as_str().map(str::to_owned);
        rows.push(json!({ "key": first, "ok": true, answer: done[answer] }));
        rest = others;
    }
    // The other issues are independent: send several requests at a time,
    // and keep the rows in the order of the keys.
    let value = value.unwrap_or_default();
    let results: Vec<_> = futures_util::stream::iter(rest)
        .map(|key| ctx.task(task, args(key, &value)))
        .buffered(BULK_CONCURRENCY)
        .collect()
        .await;
    let mut failed = false;
    for (key, result) in rest.iter().zip(results) {
        rows.push(match result {
            Ok(done) => json!({ "key": key, "ok": true, answer: done[answer] }),
            Err(err) => {
                failed = true;
                json!({ "key": key, "ok": false, "error": err.to_string() })
            }
        });
    }
    ctx.out
        .list(
            Value::Array(rows),
            &[
                col("KEY", "key"),
                col("OK", "ok"),
                col(header, answer),
                flex("ERROR", "error"),
            ],
            Value::Null,
        )
        .map(|exit| if failed { Exit::Api } else { exit })
}

/// `--browse` and `--web` of the issue lists.
#[derive(clap::Args, Debug)]
pub struct ListView {
    /// Open the full-screen issue browser instead of printing a table (terminal only).
    #[arg(long, short = 'i', conflicts_with = "web")]
    browse: bool,
    /// Open the same search in the web browser (with --agent, return its URL).
    #[arg(long)]
    web: bool,
}

/// `issue list`: one page of rows, the issue browser with `--browse`, or the
/// search page with `--web`.
async fn list_issues(ctx: &Ctx<'_>, query: IssueQuery, view: ListView) -> Result<Exit> {
    let jql = query.to_jql();
    if view.web {
        return web_search(ctx, &jql);
    }
    if ctx.dry_run {
        return ctx.out.item(json!({ "jql": jql }), Value::Null, |d| {
            s(d, "jql").to_owned()
        });
    }
    if view.browse {
        return browse_issues(ctx, jql).await;
    }
    let page = ctx
        .task("jira_issue_list", serde_json::to_value(&query)?)
        .await?;
    issue_page(ctx, page)
}

/// `--web`: Jira's issue search for `jql` in the browser.
fn web_search(ctx: &Ctx<'_>, jql: &str) -> Result<Exit> {
    let site = ctx.client.site_url(Family::Jira);
    let url = url::Url::parse_with_params(
        &format!("{}/issues/", site.trim_end_matches('/')),
        &[("jql", jql)],
    )
    .map_err(|err| {
        Error::Invalid(format!(
            "the Jira site URL `{site}` is not valid ({err}); check it with `atl auth status`"
        ))
    })?;
    crate::tasks::open_url(ctx.out, url.as_str())
}

/// `--browse`: the full-screen issue browser, which needs a terminal.
async fn browse_issues(ctx: &Ctx<'_>, jql: String) -> Result<Exit> {
    if !ctx.out.interactive() {
        return Err(Error::Invalid(
            "--browse opens a full-screen view and needs a terminal on stdin and stdout without --agent or --format; remove --browse to print the issues".into(),
        )
        .into());
    }
    crate::browse::issues(ctx.client, jql).await
}

/// Lists one page of issues, with the next-page hint.
fn issue_page(ctx: &Ctx<'_>, mut page: Value) -> Result<Exit> {
    let mut meta = json!({ "jql": page["jql"], "total": page["total"] });
    if let Some(token) = page["next_page_token"].as_str() {
        meta["next_page_token"] = token.into();
        meta["next"] = format!("rerun with --page-token {token}").into();
    }
    ctx.out.list(page["issues"].take(), &ISSUE_COLUMNS, meta)
}

/// `--web` after a write: opens the URL in a terminal (the URL is in the
/// output anyway, so a failure only prints a note).
fn open_quietly(ctx: &Ctx<'_>, url: &str) {
    if ctx.out.agent || !ctx.out.tty {
        return;
    }
    if let Err(err) = open::that_detached(url) {
        eprintln!("cannot open a browser ({err}); visit {url}");
    }
}

/// `[KEY] FILE...`: the first word is a key when it looks like one and is not
/// an existing file.
fn key_and_files(mut args: Vec<String>) -> Result<(String, Vec<String>), Error> {
    let first_is_key = args.len() > 1 && is_issue_key(&args[0]) && !Path::new(&args[0]).exists();
    let key = if first_is_key {
        args.remove(0).to_ascii_uppercase()
    } else {
        branch_key()?
    };
    Ok((key, args))
}

pub async fn run(ctx: &Ctx<'_>, command: JiraCommand) -> Result<Exit> {
    match command {
        JiraCommand::Issue(command) => run_issue(ctx, command).await,
        JiraCommand::Field(FieldCommand::List { filter, custom }) => {
            let mut rows = ctx
                .task("jira_field_list", json!({ "filter": filter }))
                .await?;
            if custom && let Value::Array(items) = &mut rows {
                items.retain(|f| f["custom"] == true);
            }
            ctx.out.list(
                rows,
                &[
                    col("ID", "id"),
                    col("TYPE", "type"),
                    col("ITEMS", "items"),
                    flex("NAME", "name"),
                ],
                Value::Null,
            )
        }
        JiraCommand::Project(ListOnly::List) => {
            ctx.list(
                "jira_project_list",
                json!({}),
                &[col("KEY", "key"), col("TYPE", "type"), flex("NAME", "name")],
            )
            .await
        }
        JiraCommand::Board(BoardCommand::List { project }) => {
            ctx.list(
                "jira_board_list",
                json!({ "project": ctx.repo.project(project) }),
                &[col("ID", "id"), col("TYPE", "type"), flex("NAME", "name")],
            )
            .await
        }
        JiraCommand::Sprint(SprintCommand::List {
            board,
            project,
            state,
        }) => {
            let mut result = ctx
                .task(
                    "jira_sprint_list",
                    json!({ "board": board, "project": ctx.repo.project(project), "state": state }),
                )
                .await?;
            ctx.out.list(
                result["sprints"].take(),
                &[
                    col("ID", "id"),
                    col("STATE", "state"),
                    col("START", "start"),
                    col("END", "end"),
                    flex("NAME", "name"),
                ],
                json!({ "board": result["board"] }),
            )
        }
        JiraCommand::Sprint(SprintCommand::Create {
            name,
            board,
            project,
            start,
            end,
            goal,
        }) => {
            let project = ctx.repo.project(project);
            ctx.item(
                "jira_sprint_create",
                json!({ "name": name, "board": board, "project": project, "start": start, "end": end, "goal": goal }),
                |sp| format!("Created sprint {} ({})", sp["id"], s(sp, "name")),
            )
            .await
        }
        JiraCommand::Sprint(SprintCommand::Start { sprint, start, end }) => {
            ctx.item(
                "jira_sprint_start",
                json!({ "sprint": sprint, "start": start, "end": end }),
                |sp| {
                    format!(
                        "Started sprint {sprint} ({}), ends {}",
                        s(sp, "name"),
                        s(sp, "end")
                    )
                },
            )
            .await
        }
        JiraCommand::Sprint(SprintCommand::Close { sprint }) => {
            ctx.item("jira_sprint_close", json!({ "sprint": sprint }), |sp| {
                format!("Closed sprint {sprint} ({})", s(sp, "name"))
            })
            .await
        }
        JiraCommand::Comment(CommentCommand::List { key, limit }) => {
            let key = key_or_branch(key)?;
            let rows = ctx
                .task("jira_comment_list", json!({ "key": key, "limit": limit }))
                .await?;
            if !ctx.out.wants_text() {
                return ctx.out.list(
                    rows,
                    &[
                        col("ID", "id"),
                        col("AUTHOR", "author"),
                        col("CREATED", "created"),
                        flex("BODY", "body"),
                    ],
                    json!({ "key": key }),
                );
            }
            let tty = ctx.out.tty;
            let mut text = String::new();
            for c in rows.as_array().into_iter().flatten() {
                text.push_str(&format!(
                    "{} · {} {}\n{}\n\n",
                    bold(c["author"].as_str().unwrap_or("unknown"), tty),
                    date(c, "created"),
                    dim(&format!("#{}", s(c, "id")), tty),
                    crate::output::markdown(s(c, "body"), tty)
                ));
            }
            if text.is_empty() {
                text = format!("{key} has no comments");
            }
            ctx.out.item(Value::Null, Value::Null, |_| text)
        }
        JiraCommand::Comment(CommentCommand::Edit {
            key,
            id,
            text,
            body_file,
        }) => {
            let key = key.to_ascii_uppercase();
            let text = match read_text(text, body_file.as_deref())? {
                Some(text) => text,
                None if crate::prompt::allowed() => {
                    let current = ctx.task("jira_comment_list", json!({ "key": key })).await?;
                    let body = current
                        .as_array()
                        .into_iter()
                        .flatten()
                        .find(|c| s(c, "id") == id)
                        .map(|c| s(c, "body").to_owned())
                        .ok_or_else(|| {
                            Error::Invalid(format!(
                                "{key} has no comment {id}; list the IDs with `atl jira comment list {key}`"
                            ))
                        })?;
                    edit_in_editor(&body)?.unwrap_or_default()
                }
                None => String::new(),
            };
            if text.trim().is_empty() {
                return Err(Error::Invalid(
                    "the new comment text is empty; pass it as TEXT, with --body-file <path> (- for stdin), or edit it in $EDITOR at a terminal".into(),
                )
                .into());
            }
            ctx.item(
                "jira_comment_edit",
                json!({ "key": key, "id": id, "body": text }),
                |_| format!("Updated comment {id} on {key}"),
            )
            .await
        }
        JiraCommand::Comment(CommentCommand::Delete { key, id }) => {
            let key = key.to_ascii_uppercase();
            confirm(ctx.yes, &format!("delete comment {id} on {key}"))?;
            ctx.item(
                "jira_comment_delete",
                json!({ "key": key, "id": id }),
                |_| format!("Deleted comment {id} on {key}"),
            )
            .await
        }
        JiraCommand::Component(ComponentCommand::List { project }) => {
            let project = ctx.repo.project(project).ok_or_else(|| {
                Error::Invalid("pass -p PROJECT (or set [jira] project in .atl.toml)".into())
            })?;
            ctx.list(
                "jira_component_list",
                json!({ "project": project }),
                &[
                    col("ID", "id"),
                    col("NAME", "name"),
                    col("LEAD", "lead"),
                    flex("DESCRIPTION", "description"),
                ],
            )
            .await
        }
        JiraCommand::Sprint(SprintCommand::Add { sprint, issues }) => {
            ctx.item(
                "jira_sprint_add",
                json!({ "sprint": sprint, "issues": issues }),
                |_| format!("Moved {} to sprint {sprint}", issues.join(", ")),
            )
            .await
        }
        JiraCommand::Sprint(SprintCommand::Issues {
            sprint_id,
            current: _,
            prev,
            next,
            board,
            mut query,
            view,
        }) => {
            let id = match sprint_id {
                Some(id) => id,
                None => {
                    let state = match (prev, next) {
                        (true, _) => "closed",
                        (_, true) => "future",
                        _ => "active",
                    };
                    // .atl.toml picks the board; it does not filter the issues.
                    let project = ctx.repo.project(query.project.clone());
                    let found = ctx
                        .task(
                            "jira_sprint_list",
                            json!({ "board": board, "project": project, "state": state }),
                        )
                        .await?;
                    let sprints = found["sprints"].as_array().cloned().unwrap_or_default();
                    let chosen = match prev {
                        true => sprints
                            .iter()
                            .max_by_key(|s| s["end"].as_str().unwrap_or("").to_owned()),
                        false => sprints.first(),
                    };
                    let board = &found["board"];
                    chosen.and_then(|s| s["id"].as_u64()).ok_or_else(|| {
                        Error::Invalid(format!(
                            "board {board} has no {state} sprint; list its sprints with `atl jira sprint list --board {board} --state active,future,closed`"
                        ))
                    })?
                }
            };
            query.sprint = Some(id.to_string());
            list_issues(ctx, query, view).await
        }
        JiraCommand::Epic(EpicCommand::Issues {
            epic,
            mut query,
            view,
        }) => {
            let epic = epic.to_ascii_uppercase();
            if ctx.client.deployment(Family::Jira) == Deployment::Cloud {
                query.parent = Some(epic);
            } else {
                // Data Center keeps the Epic Link field; keep a user ORDER BY last.
                let user = query.jql.take().unwrap_or_default();
                let (filter, order) = match user.to_ascii_lowercase().rfind("order by") {
                    Some(at) => (user[..at].trim().to_owned(), user[at..].trim().to_owned()),
                    None => (user.trim().to_owned(), String::new()),
                };
                let mut jql = format!("\"Epic Link\" = {epic}");
                if !filter.is_empty() {
                    jql.push_str(&format!(" AND ({filter})"));
                }
                if !order.is_empty() {
                    jql.push_str(&format!(" {order}"));
                }
                query.jql = Some(jql);
            }
            list_issues(ctx, query, view).await
        }
        JiraCommand::Epic(EpicCommand::List { project, limit }) => {
            ctx.list(
                "jira_epic_list",
                json!({ "project": ctx.repo.project(project), "limit": limit }),
                &ISSUE_COLUMNS,
            )
            .await
        }
        JiraCommand::Epic(EpicCommand::Add { epic, issues }) => {
            ctx.item(
                "jira_epic_add",
                json!({ "epic": epic, "issues": issues }),
                |_| format!("Moved {} into {epic}", issues.join(", ")),
            )
            .await
        }
        JiraCommand::Epic(EpicCommand::Remove { issues }) => {
            ctx.item("jira_epic_remove", json!({ "issues": issues }), |_| {
                format!("Removed {} from their epic", issues.join(", "))
            })
            .await
        }
        JiraCommand::Filter(FilterCommand::List { name }) => {
            ctx.list(
                "jira_filter_list",
                json!({ "name": name }),
                &[
                    col("ID", "id"),
                    col("NAME", "name"),
                    col("OWNER", "owner"),
                    flex("JQL", "jql"),
                ],
            )
            .await
        }
        JiraCommand::Filter(FilterCommand::Run {
            filter,
            limit,
            page_token,
            view,
        }) => {
            if view.browse || view.web {
                let filter = Jira::new(ctx.client).filter(&filter).await?;
                return match view.web {
                    true => web_search(ctx, &filter.jql),
                    false => browse_issues(ctx, filter.jql).await,
                };
            }
            let page = ctx
                .task(
                    "jira_filter_run",
                    json!({ "filter": filter, "limit": limit, "page_token": page_token }),
                )
                .await?;
            issue_page(ctx, page)
        }
        JiraCommand::Jql(JqlCommand::Check { jql }) => {
            let check = ctx.task("jira_jql_check", json!({ "jql": jql })).await?;
            if check["valid"] != true {
                let errors: Vec<&str> = check["errors"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter_map(Value::as_str)
                    .collect();
                return Err(
                    Error::Invalid(format!("the JQL is invalid: {}", errors.join("; "))).into(),
                );
            }
            ctx.out.item(check, Value::Null, |c| {
                let warnings: Vec<&str> = c["warnings"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter_map(Value::as_str)
                    .collect();
                if warnings.is_empty() {
                    "valid".into()
                } else {
                    format!("valid (warnings: {})", warnings.join("; "))
                }
            })
        }
        JiraCommand::Jql(JqlCommand::Fields) => {
            let mut reference = ctx.task("jira_jql_fields", json!({})).await?;
            ctx.out.list(
                reference["fields"].take(),
                &[
                    col("FIELD", "name"),
                    col("NAME", "display"),
                    flex("OPERATORS", "operators"),
                ],
                json!({ "functions": reference["functions"] }),
            )
        }
        JiraCommand::Jql(JqlCommand::Suggest { field, prefix }) => {
            ctx.list(
                "jira_jql_suggest",
                json!({ "field": field, "prefix": prefix }),
                &[col("VALUE", "value"), flex("SHOWN AS", "display")],
            )
            .await
        }
        JiraCommand::Release(ReleaseCommand::List { project, state }) => {
            let project = ctx.repo.project(project).ok_or_else(|| {
                Error::Invalid("pass --project KEY (or set [jira] project in .atl.toml)".into())
            })?;
            ctx.list(
                "jira_release_list",
                json!({ "project": project, "state": state }),
                &[
                    col("ID", "id"),
                    col("NAME", "name"),
                    col("RELEASED", "released"),
                    col("DATE", "release_date"),
                    flex("DESCRIPTION", "description"),
                ],
            )
            .await
        }
        JiraCommand::Attachment(AttachmentCommand::Download { id }) => {
            let (meta, bytes) = Jira::new(ctx.client).download(&id).await?;
            save_download(ctx, &meta.filename, &bytes)
        }
        JiraCommand::Me => {
            ctx.item("jira_me", json!({}), |m| {
                format!(
                    "{} <{}> ({})",
                    s(m, "name"),
                    m["email"].as_str().unwrap_or("-"),
                    s(m, "id")
                )
            })
            .await
        }
    }
}

async fn run_issue(ctx: &Ctx<'_>, command: IssueCommand) -> Result<Exit> {
    match command {
        IssueCommand::List { mut query, view } => {
            query.project = ctx.repo.project(query.project);
            list_issues(ctx, query, view).await
        }
        IssueCommand::Count(mut query) => {
            query.project = ctx.repo.project(query.project);
            ctx.item("jira_issue_count", serde_json::to_value(&query)?, |c| {
                c["count"].to_string()
            })
            .await
        }
        IssueCommand::View {
            key,
            comments,
            web,
            field,
        } => {
            let key = key_or_branch(key)?;
            if web {
                return open_url(ctx.out, &Jira::new(ctx.client).browse_url(&key));
            }
            let tty = ctx.out.tty;
            ctx.item(
                "jira_issue_view",
                json!({ "key": key, "comments": comments, "fields": field }),
                |issue| render_issue(issue, tty),
            )
            .await
        }
        IssueCommand::Create(args) => {
            let ask = prompt::allowed();
            let project = match ctx.repo.project(args.project) {
                Some(project) => project,
                None if ask => {
                    let projects = ctx.task("jira_project_list", json!({})).await?;
                    let choices: Vec<Choice> = projects
                        .as_array()
                        .into_iter()
                        .flatten()
                        .map(|p| {
                            Choice::new(s(p, "key"), format!("{}  {}", s(p, "key"), s(p, "name")))
                        })
                        .collect();
                    if choices.is_empty() {
                        return Err(Error::Invalid("you can see no Jira projects; check the login with `atl auth status --verify`".into()).into());
                    }
                    prompt::select("Project", &choices)?
                }
                None => {
                    return Err(Error::Invalid(
                        "pass --project KEY (or set [jira] project in .atl.toml); list keys with `atl jira project list`".into(),
                    )
                    .into());
                }
            };
            let issue_type = match args.issue_type {
                Some(kind) => kind,
                None if ask => {
                    let types = ctx
                        .task("jira_issue_types", json!({ "project": project }))
                        .await?;
                    let choices: Vec<Choice> = types
                        .as_array()
                        .into_iter()
                        .flatten()
                        .filter_map(Value::as_str)
                        .map(|t| Choice::new(t, t))
                        .collect();
                    match choices.is_empty() {
                        true => "Task".to_owned(),
                        false => prompt::select("Issue type", &choices)?,
                    }
                }
                None => "Task".to_owned(),
            };
            let summary = match args.summary {
                Some(summary) => summary,
                None if ask => prompt::text("Summary")?,
                None => {
                    return Err(Error::Invalid(
                        "pass --summary (-s) with a one-line summary".into(),
                    )
                    .into());
                }
            };
            let mut description = read_body(args.body, args.body_file.as_deref())?;
            if description.is_none()
                && ask
                && prompt::confirm("Write a description in $EDITOR?", false)?
            {
                description = edit_in_editor("")?;
            }
            let created = ctx
                .task(
                    "jira_issue_create",
                    json!({
                        "project": project,
                        "type": issue_type,
                        "summary": summary,
                        "description": description,
                        "assignee": args.assignee,
                        "labels": args.label,
                        "priority": args.priority,
                        "parent": args.parent,
                        "components": args.component,
                        "fix_versions": args.fix_version,
                        "affects_versions": args.affects_version,
                        "estimate": args.estimate,
                        "reporter": args.reporter,
                        "fields": field_args(args.field)?,
                    }),
                )
                .await?;
            if args.web {
                open_quietly(ctx, s(&created, "url"));
            }
            ctx.out.item(created, Value::Null, |c| {
                format!("Created {} {}", s(c, "key"), s(c, "url"))
            })
        }
        IssueCommand::Edit(args) => {
            let key = key_or_branch(args.key)?;
            let mut description = read_text(args.body, args.body_file.as_deref())?;
            let nothing = description.is_none()
                && args.summary.is_none()
                && args.priority.is_none()
                && args.assignee.is_none()
                && args.parent.is_none()
                && args.reporter.is_none()
                && args.estimate.is_none()
                && args.label.is_empty()
                && args.remove_label.is_empty()
                && args.component.is_empty()
                && args.fix_version.is_empty()
                && args.affects_version.is_empty()
                && args.field.is_empty();
            // `atl jira issue edit KEY` alone edits the description.
            if args.editor || (nothing && prompt::allowed()) {
                let current = ctx
                    .task("jira_issue_view", json!({ "key": key, "comments": 0 }))
                    .await?;
                match edit_in_editor(s(&current, "description"))? {
                    Some(text) => description = Some(text),
                    None if nothing => {
                        return ctx.out.item(
                            json!({ "key": key, "updated": false }),
                            Value::Null,
                            |_| format!("{key}: no changes"),
                        );
                    }
                    None => {}
                }
            }
            let (add_labels, mut remove_labels) = split_removals(args.label);
            remove_labels.extend(args.remove_label);
            let (add_components, remove_components) = split_removals(args.component);
            let (add_fix_versions, remove_fix_versions) = split_removals(args.fix_version);
            let (add_affects_versions, remove_affects_versions) =
                split_removals(args.affects_version);
            let updated = ctx
                .task(
                    "jira_issue_edit",
                    json!({
                        "key": key,
                        "summary": args.summary,
                        "description": description,
                        "priority": args.priority,
                        "assignee": args.assignee,
                        "parent": args.parent,
                        "reporter": args.reporter,
                        "estimate": args.estimate,
                        "add_labels": add_labels,
                        "remove_labels": remove_labels,
                        "add_components": add_components,
                        "remove_components": remove_components,
                        "add_fix_versions": add_fix_versions,
                        "remove_fix_versions": remove_fix_versions,
                        "add_affects_versions": add_affects_versions,
                        "remove_affects_versions": remove_affects_versions,
                        "fields": field_args(args.field)?,
                        "skip_notify": args.skip_notify,
                    }),
                )
                .await?;
            if args.web {
                open_quietly(ctx, s(&updated, "url"));
            }
            ctx.out
                .item(updated, Value::Null, |_| format!("Updated {key}"))
        }
        IssueCommand::Delete { key, subtasks } => {
            let key = key.to_ascii_uppercase();
            let what = if subtasks {
                format!("delete Jira issue {key} and its sub-tasks")
            } else {
                format!("delete Jira issue {key}")
            };
            confirm(ctx.yes, &what)?;
            ctx.item(
                "jira_issue_delete",
                json!({ "key": key, "subtasks": subtasks }),
                |_| format!("Deleted {key}"),
            )
            .await
        }
        IssueCommand::Watch { key, user } => {
            let key = key_or_branch(key)?;
            ctx.item(
                "jira_issue_watch",
                json!({ "key": key, "user": user }),
                |_| format!("{user} now watches {key}"),
            )
            .await
        }
        IssueCommand::Unwatch { key, user } => {
            let key = key_or_branch(key)?;
            ctx.item(
                "jira_issue_unwatch",
                json!({ "key": key, "user": user }),
                |_| format!("{user} no longer watches {key}"),
            )
            .await
        }
        IssueCommand::Watchers { key } => {
            let mut found = ctx
                .task("jira_issue_watchers", json!({ "key": key_or_branch(key)? }))
                .await?;
            let meta = json!({ "watching": found["watching"], "count": found["count"] });
            ctx.out.list(
                found["watchers"].take(),
                &[col("NAME", "name"), col("EMAIL", "email"), flex("ID", "id")],
                meta,
            )
        }
        IssueCommand::Assign { args } => {
            let (keys, user) = keys_and_value(args)?;
            if let [key] = keys.as_slice() {
                let user = user.unwrap_or_default();
                return ctx
                    .item(
                        "jira_issue_assign",
                        json!({ "key": key, "user": user }),
                        |_| format!("Assigned {key} to {user}"),
                    )
                    .await;
            }
            bulk(
                ctx,
                &keys,
                user,
                "jira_issue_assign",
                "assignee",
                "ASSIGNEE",
                |key, user| json!({ "key": key, "user": user }),
            )
            .await
        }
        IssueCommand::Move {
            args,
            comment,
            resolution,
            assignee,
        } => {
            let (keys, to) = keys_and_value(args)?;
            if let [key] = keys.as_slice() {
                let to = to.unwrap_or_default();
                return ctx
                    .item(
                        "jira_issue_move",
                        json!({ "key": key, "status": to, "comment": comment, "resolution": resolution, "assignee": assignee }),
                        |m| format!("Moved {key} to {}", s(m, "status")),
                    )
                    .await;
            }
            bulk(ctx, &keys, to, "jira_issue_move", "status", "STATUS", |key, to| {
                json!({ "key": key, "status": to, "comment": comment, "resolution": resolution, "assignee": assignee })
            })
            .await
        }
        IssueCommand::Worklogs { key } => {
            ctx.list(
                "jira_worklog_list",
                json!({ "key": key_or_branch(key)? }),
                &[
                    col("ID", "id"),
                    col("AUTHOR", "author"),
                    col("SPENT", "time_spent"),
                    col("STARTED", "started"),
                    flex("COMMENT", "comment"),
                ],
            )
            .await
        }
        IssueCommand::History { key, limit } => {
            ctx.list(
                "jira_issue_history",
                json!({ "key": key_or_branch(key)?, "limit": limit }),
                &[
                    col("WHEN", "created"),
                    col("WHO", "author"),
                    col("FIELD", "field"),
                    col("FROM", "from"),
                    flex("TO", "to"),
                ],
            )
            .await
        }
        IssueCommand::Comment {
            args,
            body_file,
            internal,
        } => {
            let (key, text) =
                key_and_text(args, body_file.as_deref(), is_issue_key, key_or_branch)?;
            ctx.item(
                "jira_issue_comment",
                json!({ "key": key, "body": text, "internal": internal }),
                |_| format!("Commented on {key}"),
            )
            .await
        }
        IssueCommand::Transitions { key } => {
            ctx.list(
                "jira_issue_transitions",
                json!({ "key": key_or_branch(key)? }),
                &[
                    col("TO STATUS", "to"),
                    col("TRANSITION", "name"),
                    col("ID", "id"),
                ],
            )
            .await
        }
        IssueCommand::Link { from, to, kind } => {
            let (from, to) = (from.to_ascii_uppercase(), to.to_ascii_uppercase());
            ctx.item(
                "jira_issue_link",
                json!({ "from": from, "to": to, "kind": kind }),
                |l| format!("Linked {from} to {to} ({})", s(l, "type")),
            )
            .await
        }
        IssueCommand::Unlink { from, to, kind } => {
            let (from, to) = (from.to_ascii_uppercase(), to.to_ascii_uppercase());
            ctx.item(
                "jira_issue_unlink",
                json!({ "from": from, "to": to, "kind": kind }),
                |u| {
                    format!(
                        "Unlinked {from} and {to} ({})",
                        u["removed"]
                            .as_array()
                            .map(|r| r
                                .iter()
                                .filter_map(Value::as_str)
                                .collect::<Vec<_>>()
                                .join(", "))
                            .unwrap_or_default()
                    )
                },
            )
            .await
        }
        IssueCommand::RemoteLink { key, url, title } => {
            let key = key.to_ascii_uppercase();
            ctx.item(
                "jira_issue_remote_link",
                json!({ "key": key, "url": url, "title": title }),
                |l| format!("Linked {key} to {}", s(l, "url")),
            )
            .await
        }
        IssueCommand::LinkTypes => {
            ctx.list(
                "jira_issue_link_types",
                json!({}),
                &[
                    col("NAME", "name"),
                    col("OUTWARD", "outward"),
                    flex("INWARD", "inward"),
                ],
            )
            .await
        }
        IssueCommand::Clone {
            key,
            summary,
            priority,
            assignee,
            label,
            replace,
        } => {
            let key = key_or_branch(key)?;
            ctx.item(
                "jira_issue_clone",
                json!({ "key": key, "summary": summary, "priority": priority, "assignee": assignee, "labels": label, "replace": replace }),
                |c| format!("Cloned {key} as {} {}", s(c, "key"), s(c, "url")),
            )
            .await
        }
        IssueCommand::Worklog {
            args,
            comment,
            started,
        } => {
            let (key, time) = key_and_value(args)?;
            ctx.item(
                "jira_issue_worklog",
                json!({ "key": key, "time": time, "comment": comment, "started": started }),
                |_| format!("Logged {time} on {key}"),
            )
            .await
        }
        IssueCommand::Attach { args } => {
            let (key, files) = key_and_files(args)?;
            ctx.list(
                "jira_issue_attach",
                json!({ "key": key, "files": files }),
                &ATTACHMENT_COLUMNS,
            )
            .await
        }
        IssueCommand::Attachments { key } => {
            ctx.list(
                "jira_issue_attachments",
                json!({ "key": key_or_branch(key)? }),
                &ATTACHMENT_COLUMNS,
            )
            .await
        }
    }
}
