//! Jira and Jira Software tasks.

use super::*;
use crate::client::Client;
use crate::error::{Error, Result};
use crate::jira::{CloneOptions, IssueEdit, IssueQuery, Jira, NewIssue, TransitionFields};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::{Value, json};

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct IssueDeleteArgs {
    key: String,
    /// Also delete the sub-tasks (Jira refuses otherwise when there are some).
    #[serde(default)]
    subtasks: bool,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct WatchArgs {
    key: String,
    /// `me`, an email, or a name.
    #[serde(default = "default_me")]
    user: String,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct CommentListArgs {
    key: String,
    #[serde(default = "default_issue_comment_limit")]
    limit: usize,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct CommentEditArgs {
    key: String,
    /// Comment ID (see jira_comment_list).
    id: String,
    /// New text in Markdown.
    body: String,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct CommentDeleteArgs {
    key: String,
    id: String,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct HistoryArgs {
    key: String,
    #[serde(default = "default_history_limit")]
    limit: usize,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct SprintCreateArgs {
    name: String,
    /// Board ID (default: the first board of `project`).
    board: Option<u64>,
    project: Option<String>,
    /// YYYY-MM-DD or ISO 8601.
    start: Option<String>,
    end: Option<String>,
    goal: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct SprintStartArgs {
    sprint: u64,
    /// YYYY-MM-DD or ISO 8601 (default: the sprint's start, else now).
    start: Option<String>,
    /// YYYY-MM-DD or ISO 8601 (default: the sprint's end; required when it has none).
    end: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct SprintArgs {
    sprint: u64,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct FieldListArgs {
    /// Part of the field name or ID.
    filter: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct IssueCreateArgs {
    project: String,
    /// Issue type name, for example Task, Bug, Story.
    #[serde(rename = "type", default = "default_type")]
    issue_type: String,
    summary: String,
    /// Description in Markdown.
    description: Option<String>,
    /// `me` or a user.
    assignee: Option<String>,
    #[serde(default)]
    labels: Vec<String>,
    priority: Option<String>,
    /// Parent issue key.
    parent: Option<String>,
    #[serde(default)]
    components: Vec<String>,
    #[serde(default)]
    fix_versions: Vec<String>,
    /// Versions the problem affects.
    #[serde(default)]
    affects_versions: Vec<String>,
    /// Original estimate, for example "2h".
    estimate: Option<String>,
    /// `me` or a user.
    reporter: Option<String>,
    /// Other fields by name or ID, for example {"Story Points": "3"}.
    #[serde(default)]
    fields: FieldMap,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct IssueEditArgs {
    key: String,
    summary: Option<String>,
    /// New description in Markdown.
    description: Option<String>,
    priority: Option<String>,
    /// `me`, `x` (unassigned), or a user.
    assignee: Option<String>,
    /// New parent issue key.
    parent: Option<String>,
    reporter: Option<String>,
    /// Original estimate, for example "2h".
    estimate: Option<String>,
    #[serde(default)]
    add_labels: Vec<String>,
    #[serde(default)]
    remove_labels: Vec<String>,
    #[serde(default)]
    add_components: Vec<String>,
    #[serde(default)]
    remove_components: Vec<String>,
    #[serde(default)]
    add_fix_versions: Vec<String>,
    #[serde(default)]
    remove_fix_versions: Vec<String>,
    #[serde(default)]
    add_affects_versions: Vec<String>,
    #[serde(default)]
    remove_affects_versions: Vec<String>,
    /// Other fields by name or ID, for example {"Story Points": "5"}.
    #[serde(default)]
    fields: FieldMap,
    /// Do not email watchers (Jira may need admin rights for it).
    #[serde(default)]
    skip_notify: bool,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct AssignArgs {
    key: String,
    /// `me`, `x` (unassigned), or a user (email or name).
    user: String,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct MoveArgs {
    key: String,
    /// Target status or transition name.
    status: String,
    /// Optional Markdown comment added with the transition.
    comment: Option<String>,
    /// Resolution to set, for example Fixed (must be on the transition screen).
    resolution: Option<String>,
    /// Assignee to set: `me`, `x`, or a user.
    assignee: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct CommentArgs {
    key: String,
    /// Comment in Markdown.
    body: String,
    /// A Jira Service Management internal note (customers do not see it).
    #[serde(default)]
    internal: bool,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct LinkArgs {
    from: String,
    to: String,
    /// Link type name ("Blocks"), outward phrase ("blocks"), or inward phrase ("is blocked by"); reads "from <kind> to".
    kind: String,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct UnlinkArgs {
    from: String,
    to: String,
    /// Only links of this type (name, outward or inward phrase).
    kind: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct RemoteLinkArgs {
    key: String,
    url: String,
    /// Link text (default: the URL).
    title: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct ReleaseListArgs {
    /// Project key.
    project: String,
    /// unreleased, released, archived, or all (default).
    #[serde(default = "default_release_state")]
    state: String,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct CloneArgs {
    key: String,
    /// Summary for the copy (default: the original's).
    summary: Option<String>,
    priority: Option<String>,
    /// `me` or a user.
    assignee: Option<String>,
    /// Labels for the copy (default: the original's).
    #[serde(default)]
    labels: Vec<String>,
    /// "FIND:REPLACE" pairs applied to the summary and description.
    #[serde(default)]
    replace: Vec<String>,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct WorklogArgs {
    key: String,
    /// Time spent, for example "2h 30m" or "1d".
    time: String,
    comment: Option<String>,
    /// Start time, for example 2026-11-09T09:00:00.000+0000.
    started: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct JqlArgs {
    jql: String,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct SuggestArgs {
    /// JQL field name, for example status, project, assignee.
    field: String,
    /// Start of the value.
    #[serde(default)]
    prefix: String,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct FilterListArgs {
    /// Part of the filter name.
    name: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct FilterRunArgs {
    /// Filter ID or exact name.
    filter: String,
    #[serde(default = "default_limit")]
    limit: usize,
    page_token: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct EpicListArgs {
    project: Option<String>,
    #[serde(default = "default_limit")]
    limit: usize,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct EpicAddArgs {
    /// Epic key.
    epic: String,
    issues: Vec<String>,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct IssuesArgs {
    issues: Vec<String>,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct ProjectKeyArgs {
    /// Project key, for example ABC.
    project: String,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct ProjectArgs {
    project: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct SprintListArgs {
    /// Board ID (default: the project's first board).
    board: Option<u64>,
    project: Option<String>,
    /// active, future, closed, or a comma-separated mix.
    #[serde(default = "default_sprint_state")]
    state: String,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct SprintAddArgs {
    sprint: u64,
    issues: Vec<String>,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct IssueAttachArgs {
    key: String,
    /// Local file paths to upload.
    files: Vec<std::path::PathBuf>,
}

fn default_me() -> String {
    "me".into()
}

fn default_issue_comment_limit() -> usize {
    500
}

fn default_history_limit() -> usize {
    100
}

fn default_type() -> String {
    "Task".into()
}

fn default_release_state() -> String {
    "all".into()
}

fn default_sprint_state() -> String {
    "active,future".into()
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct IssueViewArgs {
    key: String,
    /// How many of the latest comments to include.
    #[serde(default = "default_comments")]
    comments: usize,
    /// Extra fields to include, by name or ID (for example "Story Points").
    #[serde(default)]
    fields: Vec<String>,
}

/// Fields by name or ID. Text is converted by the field's type (number,
/// option, user, comma-separated arrays); other JSON is sent unchanged.
type FieldMap = std::collections::BTreeMap<String, Value>;

pub(super) static TASKS: &[TaskDef] = &[
    task!(
        "jira_issue_list",
        true,
        IssueQuery,
        "Search Jira issues with filters (or extra JQL). Returns small records: key, summary, status, type, assignee, priority, updated, url. With no filters: your unresolved issues."
    ),
    task!(
        "jira_issue_count",
        true,
        IssueQuery,
        "Count Jira issues matching the same filters as issue_list."
    ),
    task!(
        "jira_issue_view",
        true,
        IssueViewArgs,
        "One Jira issue with description and latest comments as Markdown, plus any `fields` asked for by name."
    ),
    task!(
        "jira_issue_delete",
        false,
        IssueDeleteArgs,
        "Delete a Jira issue (with `subtasks: true`, also its sub-tasks). Cannot be undone.",
        destructive
    ),
    task!(
        "jira_issue_watchers",
        true,
        KeyArgs,
        "Who watches a Jira issue, and whether you do."
    ),
    task!(
        "jira_issue_watch",
        false,
        WatchArgs,
        "Add a watcher to a Jira issue (default: you)."
    ),
    task!(
        "jira_issue_unwatch",
        false,
        WatchArgs,
        "Remove a watcher from a Jira issue (default: you)."
    ),
    task!(
        "jira_field_list",
        true,
        FieldListArgs,
        "Jira system and custom fields (ID, name, type), optionally filtered by part of the name. Use the names in the `fields` argument of issue create, edit and view."
    ),
    task!(
        "jira_issue_create",
        false,
        IssueCreateArgs,
        "Create a Jira issue; the description is Markdown."
    ),
    task!(
        "jira_issue_edit",
        false,
        IssueEditArgs,
        "Change a Jira issue's summary, description (Markdown), priority or labels."
    ),
    task!(
        "jira_issue_assign",
        false,
        AssignArgs,
        "Assign a Jira issue to me, nobody (x), or a user."
    ),
    task!(
        "jira_issue_transitions",
        true,
        KeyArgs,
        "Statuses a Jira issue can move to."
    ),
    task!(
        "jira_issue_move",
        false,
        MoveArgs,
        "Move a Jira issue to another status (by status or transition name)."
    ),
    task!(
        "jira_issue_comment",
        false,
        CommentArgs,
        "Add a Markdown comment to a Jira issue."
    ),
    task!(
        "jira_issue_link",
        false,
        LinkArgs,
        "Link two Jira issues so that 'from <kind> to' reads true, for example from A-1 kind blocks to A-2."
    ),
    task!(
        "jira_issue_unlink",
        false,
        UnlinkArgs,
        "Remove the links between two Jira issues (optionally only one link type)."
    ),
    task!(
        "jira_issue_remote_link",
        false,
        RemoteLinkArgs,
        "Add a web link (URL and title) to a Jira issue."
    ),
    task!(
        "jira_comment_list",
        true,
        CommentListArgs,
        "Every comment of a Jira issue, oldest first, with IDs, as Markdown."
    ),
    task!(
        "jira_comment_edit",
        false,
        CommentEditArgs,
        "Replace the text of a Jira comment (Markdown)."
    ),
    task!(
        "jira_comment_delete",
        false,
        CommentDeleteArgs,
        "Delete a Jira comment. Cannot be undone.",
        destructive
    ),
    task!(
        "jira_worklog_list",
        true,
        KeyArgs,
        "Time logged on a Jira issue: author, time spent, start, comment."
    ),
    task!(
        "jira_issue_history",
        true,
        HistoryArgs,
        "Field changes of a Jira issue, newest first: when, who, field, from, to."
    ),
    task!(
        "jira_component_list",
        true,
        ProjectKeyArgs,
        "Components of a Jira project with their leads."
    ),
    task!(
        "jira_sprint_create",
        false,
        SprintCreateArgs,
        "Create a future sprint on a board (default: the project's first board)."
    ),
    task!(
        "jira_sprint_start",
        false,
        SprintStartArgs,
        "Start a future sprint; Jira needs a start and an end date."
    ),
    task!(
        "jira_sprint_close",
        false,
        SprintArgs,
        "Complete an active sprint."
    ),
    task!(
        "jira_release_list",
        true,
        ReleaseListArgs,
        "Versions (releases) of a Jira project: name, released, archived, release date."
    ),
    task!(
        "jira_issue_link_types",
        true,
        Empty,
        "Jira issue link types with their outward and inward phrases."
    ),
    task!(
        "jira_issue_clone",
        false,
        CloneArgs,
        "Copy a Jira issue (summary, description, type, labels, priority, components, parent)."
    ),
    task!(
        "jira_issue_worklog",
        false,
        WorklogArgs,
        "Log time on a Jira issue."
    ),
    task!(
        "jira_issue_attach",
        false,
        IssueAttachArgs,
        "Upload local files to a Jira issue."
    ),
    task!(
        "jira_issue_attachments",
        true,
        KeyArgs,
        "List a Jira issue's attachments."
    ),
    task!(
        "jira_jql_check",
        true,
        JqlArgs,
        "Validate JQL with Jira's parser; returns valid, errors, warnings."
    ),
    task!(
        "jira_jql_fields",
        true,
        Empty,
        "Fields (with operators) and functions usable in JQL on this site."
    ),
    task!(
        "jira_jql_suggest",
        true,
        SuggestArgs,
        "Value suggestions for a JQL field, for example field=status prefix=In."
    ),
    task!(
        "jira_filter_list",
        true,
        FilterListArgs,
        "Saved Jira filters (Cloud: all visible, Data Center: favourites)."
    ),
    task!(
        "jira_filter_run",
        true,
        FilterRunArgs,
        "Run a saved Jira filter by ID or exact name; returns issue records."
    ),
    task!(
        "jira_epic_list",
        true,
        EpicListArgs,
        "Open epics, optionally of one project."
    ),
    task!(
        "jira_epic_add",
        false,
        EpicAddArgs,
        "Move issues into an epic."
    ),
    task!(
        "jira_epic_remove",
        false,
        IssuesArgs,
        "Remove issues from their epic."
    ),
    task!(
        "jira_board_list",
        true,
        ProjectArgs,
        "Jira Software boards, optionally of one project."
    ),
    task!(
        "jira_sprint_list",
        true,
        SprintListArgs,
        "Sprints of a board."
    ),
    task!(
        "jira_sprint_add",
        false,
        SprintAddArgs,
        "Move issues into a sprint."
    ),
    task!("jira_project_list", true, Empty, "Jira projects."),
    task!(
        "jira_issue_types",
        true,
        ProjectKeyArgs,
        "Issue type names of a Jira project (without sub-task types), for the `type` argument of jira_issue_create."
    ),
    task!("jira_me", true, Empty, "The signed-in Jira user."),
];

pub(super) async fn run(client: &Client, name: &str, args: Value) -> Result<Value> {
    let jira = Jira::new(client);
    match name {
        "jira_issue_list" => {
            let a: IssueQuery = parse(name, args)?;
            let jql = a.to_jql();
            page_json(
                jira.search(&jql, a.limit, a.page_token.as_deref()).await?,
                jql,
            )
        }
        "jira_issue_count" => {
            let a: IssueQuery = parse(name, args)?;
            let jql = a.to_jql();
            Ok(json!({ "count": jira.count(&jql).await?, "jql": jql }))
        }
        "jira_issue_view" => {
            let a: IssueViewArgs = parse(name, args)?;
            to_json(jira.issue(&a.key, a.comments, &a.fields).await?)
        }
        "jira_issue_delete" => {
            let a: IssueDeleteArgs = parse(name, args)?;
            jira.delete(&a.key, a.subtasks).await?;
            Ok(json!({ "key": a.key, "deleted": true }))
        }
        "jira_issue_watchers" => {
            let a: KeyArgs = parse(name, args)?;
            to_json(jira.watchers(&a.key).await?)
        }
        "jira_issue_watch" => {
            let a: WatchArgs = parse(name, args)?;
            jira.watch(&a.key, &a.user).await?;
            Ok(json!({ "key": a.key, "user": a.user, "watching": true }))
        }
        "jira_issue_unwatch" => {
            let a: WatchArgs = parse(name, args)?;
            jira.unwatch(&a.key, &a.user).await?;
            Ok(json!({ "key": a.key, "user": a.user, "watching": false }))
        }
        "jira_field_list" => {
            let a: FieldListArgs = parse(name, args)?;
            to_json(jira.fields(a.filter.as_deref()).await?)
        }
        "jira_issue_create" => {
            let a: IssueCreateArgs = parse(name, args)?;
            let new = NewIssue {
                project: a.project,
                issue_type: a.issue_type,
                summary: a.summary,
                description: a.description,
                assignee: a.assignee,
                labels: a.labels,
                priority: a.priority,
                parent: a.parent,
                components: a.components,
                fix_versions: a.fix_versions,
                affects_versions: a.affects_versions,
                estimate: a.estimate,
                reporter: a.reporter,
                fields: a.fields.into_iter().collect(),
            };
            to_json(jira.create(&new).await?)
        }
        "jira_issue_edit" => {
            let a: IssueEditArgs = parse(name, args)?;
            let edit = IssueEdit {
                summary: a.summary,
                description: a.description,
                priority: a.priority,
                assignee: a.assignee,
                parent: a.parent,
                reporter: a.reporter,
                estimate: a.estimate,
                add_labels: a.add_labels,
                remove_labels: a.remove_labels,
                add_components: a.add_components,
                remove_components: a.remove_components,
                add_fix_versions: a.add_fix_versions,
                remove_fix_versions: a.remove_fix_versions,
                add_affects_versions: a.add_affects_versions,
                remove_affects_versions: a.remove_affects_versions,
                fields: a.fields.into_iter().collect(),
                skip_notify: a.skip_notify,
            };
            jira.edit(&a.key, &edit).await?;
            Ok(json!({ "key": a.key, "updated": true, "url": jira.browse_url(&a.key) }))
        }
        "jira_issue_assign" => {
            let a: AssignArgs = parse(name, args)?;
            jira.assign(&a.key, &a.user).await?;
            Ok(json!({ "key": a.key, "assignee": a.user }))
        }
        "jira_issue_transitions" => {
            let a: KeyArgs = parse(name, args)?;
            to_json(jira.transitions(&a.key).await?)
        }
        "jira_issue_move" => {
            let a: MoveArgs = parse(name, args)?;
            let t = jira
                .transition(
                    &a.key,
                    &a.status,
                    a.comment.as_deref(),
                    &TransitionFields {
                        resolution: a.resolution,
                        assignee: a.assignee,
                    },
                )
                .await?;
            Ok(json!({ "key": a.key, "status": t.to, "transition": t.name }))
        }
        "jira_issue_comment" => {
            let a: CommentArgs = parse(name, args)?;
            to_json(jira.comment(&a.key, &a.body, a.internal).await?)
        }
        "jira_issue_link" => {
            let a: LinkArgs = parse(name, args)?;
            let kind = jira.link(&a.from, &a.to, &a.kind).await?;
            Ok(json!({ "from": a.from, "to": a.to, "type": kind.name }))
        }
        "jira_issue_unlink" => {
            let a: UnlinkArgs = parse(name, args)?;
            let removed = jira.unlink(&a.from, &a.to, a.kind.as_deref()).await?;
            Ok(json!({ "from": a.from, "to": a.to, "removed": removed }))
        }
        "jira_issue_remote_link" => {
            let a: RemoteLinkArgs = parse(name, args)?;
            jira.remote_link(&a.key, &a.url, a.title.as_deref()).await
        }
        "jira_comment_list" => {
            let a: CommentListArgs = parse(name, args)?;
            to_json(jira.comments(&a.key, a.limit).await?)
        }
        "jira_comment_edit" => {
            let a: CommentEditArgs = parse(name, args)?;
            to_json(jira.edit_comment(&a.key, &a.id, &a.body).await?)
        }
        "jira_comment_delete" => {
            let a: CommentDeleteArgs = parse(name, args)?;
            jira.delete_comment(&a.key, &a.id).await?;
            Ok(json!({ "key": a.key, "id": a.id, "deleted": true }))
        }
        "jira_worklog_list" => {
            let a: KeyArgs = parse(name, args)?;
            to_json(jira.worklogs(&a.key).await?)
        }
        "jira_issue_history" => {
            let a: HistoryArgs = parse(name, args)?;
            to_json(jira.history(&a.key, a.limit).await?)
        }
        "jira_component_list" => {
            let a: ProjectKeyArgs = parse(name, args)?;
            to_json(jira.components(&a.project).await?)
        }
        "jira_sprint_create" => {
            let a: SprintCreateArgs = parse(name, args)?;
            let board = jira.board_or_first(a.board, a.project.as_deref()).await?;
            to_json(
                jira.create_sprint(
                    board,
                    &a.name,
                    a.start.as_deref(),
                    a.end.as_deref(),
                    a.goal.as_deref(),
                )
                .await?,
            )
        }
        "jira_sprint_start" => {
            let a: SprintStartArgs = parse(name, args)?;
            to_json(
                jira.start_sprint(a.sprint, a.start.as_deref(), a.end.as_deref())
                    .await?,
            )
        }
        "jira_sprint_close" => {
            let a: SprintArgs = parse(name, args)?;
            to_json(jira.close_sprint(a.sprint).await?)
        }
        "jira_release_list" => {
            let a: ReleaseListArgs = parse(name, args)?;
            let keep: fn(&crate::jira::Release) -> bool = match a.state.as_str() {
                "all" => |_| true,
                "unreleased" => |r| !r.released && !r.archived,
                "released" => |r| r.released && !r.archived,
                "archived" => |r| r.archived,
                other => {
                    return Err(Error::Invalid(format!(
                        "release state `{other}` is unknown; use unreleased, released, archived or all"
                    )));
                }
            };
            let releases: Vec<_> = jira
                .releases(&a.project)
                .await?
                .into_iter()
                .filter(keep)
                .collect();
            to_json(releases)
        }
        "jira_issue_clone" => {
            let a: CloneArgs = parse(name, args)?;
            let replace = a
                .replace
                .iter()
                .map(|pair| {
                    pair.split_once(':')
                        .map(|(find, with)| (find.to_owned(), with.to_owned()))
                        .filter(|(find, _)| !find.is_empty())
                        .ok_or_else(|| {
                            Error::Invalid(format!(
                                "replace `{pair}` is not FIND:REPLACE; pass for example \"v1:v2\""
                            ))
                        })
                })
                .collect::<Result<Vec<_>>>()?;
            let options = CloneOptions {
                summary: a.summary,
                priority: a.priority,
                assignee: a.assignee,
                labels: a.labels,
                replace,
            };
            to_json(jira.clone_issue(&a.key, &options).await?)
        }
        "jira_issue_worklog" => {
            let a: WorklogArgs = parse(name, args)?;
            jira.worklog(&a.key, &a.time, a.comment.as_deref(), a.started.as_deref())
                .await
        }
        "jira_issue_attach" => {
            let a: IssueAttachArgs = parse(name, args)?;
            to_json(jira.attach(&a.key, &a.files).await?)
        }
        "jira_issue_attachments" => {
            let a: KeyArgs = parse(name, args)?;
            to_json(jira.attachments(&a.key).await?)
        }
        "jira_jql_check" => {
            let a: JqlArgs = parse(name, args)?;
            to_json(jira.check_jql(&a.jql).await?)
        }
        "jira_jql_fields" => {
            let _: Empty = parse(name, args)?;
            to_json(jira.jql_reference().await?)
        }
        "jira_jql_suggest" => {
            let a: SuggestArgs = parse(name, args)?;
            to_json(jira.suggest(&a.field, &a.prefix).await?)
        }
        "jira_filter_list" => {
            let a: FilterListArgs = parse(name, args)?;
            to_json(jira.filters(a.name.as_deref()).await?)
        }
        "jira_filter_run" => {
            let a: FilterRunArgs = parse(name, args)?;
            let filter = jira.filter(&a.filter).await?;
            page_json(
                jira.search(&filter.jql, a.limit, a.page_token.as_deref())
                    .await?,
                filter.jql,
            )
        }
        "jira_epic_list" => {
            let a: EpicListArgs = parse(name, args)?;
            to_json(jira.epics(a.project.as_deref(), a.limit).await?.issues)
        }
        "jira_epic_add" => {
            let a: EpicAddArgs = parse(name, args)?;
            jira.set_epic(Some(&a.epic), &a.issues).await?;
            Ok(json!({ "epic": a.epic, "issues": a.issues }))
        }
        "jira_epic_remove" => {
            let a: IssuesArgs = parse(name, args)?;
            jira.set_epic(None, &a.issues).await?;
            Ok(json!({ "epic": null, "issues": a.issues }))
        }
        "jira_board_list" => {
            let a: ProjectArgs = parse(name, args)?;
            to_json(jira.boards(a.project.as_deref()).await?)
        }
        "jira_sprint_list" => {
            let a: SprintListArgs = parse(name, args)?;
            let board = jira.board_or_first(a.board, a.project.as_deref()).await?;
            Ok(
                json!({ "board": board, "sprints": to_json(jira.sprints(board, Some(&a.state)).await?)? }),
            )
        }
        "jira_sprint_add" => {
            let a: SprintAddArgs = parse(name, args)?;
            jira.sprint_add(a.sprint, &a.issues).await?;
            Ok(json!({ "sprint": a.sprint, "issues": a.issues }))
        }
        "jira_project_list" => {
            let _: Empty = parse(name, args)?;
            to_json(jira.projects().await?)
        }
        "jira_issue_types" => {
            let a: ProjectKeyArgs = parse(name, args)?;
            to_json(jira.issue_types(&a.project).await?)
        }
        "jira_me" => {
            let _: Empty = parse(name, args)?;
            to_json(jira.me().await?)
        }
        "jira_issue_link_types" => {
            let _: Empty = parse(name, args)?;
            to_json(jira.link_types().await?)
        }
        _ => Err(unknown(name)),
    }
}

fn page_json(page: crate::jira::IssuePage, jql: String) -> Result<Value> {
    Ok(
        json!({ "issues": to_json(&page.issues)?, "next_page_token": page.next_page_token, "total": page.total, "jql": jql }),
    )
}
