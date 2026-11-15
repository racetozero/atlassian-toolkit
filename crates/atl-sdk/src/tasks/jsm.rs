//! Jira Service Management tasks.

use super::*;
use crate::client::Client;
use crate::error::{Error, Result};
use crate::jsm::ServiceDesk;
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct DeskArgs {
    /// Service desk ID or project key.
    desk: String,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct QueueIssuesArgs {
    desk: String,
    /// Queue ID or name.
    queue: String,
    #[serde(default = "default_limit")]
    limit: usize,
}

fn default_request_status() -> String {
    "open".into()
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct RequestViewArgs {
    /// Request key, for example HELP-7.
    key: String,
    /// How many of the latest comments to include.
    #[serde(default = "default_comments")]
    comments: usize,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct RequestListArgs {
    /// Service desk ID or project key (default: all).
    desk: Option<String>,
    /// open, closed, or all.
    #[serde(default = "default_request_status")]
    status: String,
    /// Words to find.
    search: Option<String>,
    /// Only requests you raised.
    #[serde(default)]
    mine: bool,
    #[serde(default = "default_limit")]
    limit: usize,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct RequestCreateArgs {
    desk: String,
    /// Request type ID or name (see jsm_request_types).
    #[serde(rename = "type")]
    request_type: String,
    summary: String,
    description: Option<String>,
    /// Customer email or account ID to raise the request for.
    on_behalf_of: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct RequestCommentArgs {
    key: String,
    body: String,
    /// An internal note that customers cannot see.
    #[serde(default)]
    internal: bool,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct RequestMoveArgs {
    key: String,
    /// Transition name or ID (see jsm_request_transitions).
    transition: String,
    comment: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct ApproveArgs {
    key: String,
    /// Approval ID (default: the one pending approval you can answer).
    approval: Option<String>,
    /// approve or decline.
    #[serde(default = "default_decision")]
    decision: String,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct ParticipantsArgs {
    key: String,
    /// Users to add: me, an email or a name.
    #[serde(default)]
    add: Vec<String>,
    #[serde(default)]
    remove: Vec<String>,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct RequestAttachArgs {
    key: String,
    /// Local file paths to upload.
    files: Vec<std::path::PathBuf>,
    /// Keep the files internal (agents only); customers see them otherwise.
    #[serde(default)]
    internal: bool,
}

fn default_decision() -> String {
    "approve".into()
}

pub(super) static TASKS: &[TaskDef] = &[
    task!(
        "jsm_desk_list",
        true,
        Empty => Vec<crate::jsm::Desk>,
        "Jira Service Management service desks (id, project key, name)."
    ),
    task!(
        "jsm_request_types",
        true,
        DeskArgs => Vec<crate::jsm::RequestType>,
        "Request types of a service desk."
    ),
    task!(
        "jsm_queue_list",
        true,
        DeskArgs => Vec<crate::jsm::Queue>,
        "Queues of a service desk with issue counts."
    ),
    task!(
        "jsm_queue_issues",
        true,
        QueueIssuesArgs => Vec<crate::jira::IssueSummary>,
        "Issues in a service desk queue (by queue ID or name)."
    ),
    task!(
        "jsm_request_list",
        true,
        RequestListArgs => Vec<crate::jsm::Request>,
        "Customer requests by status, optionally of one service desk."
    ),
    task!(
        "jsm_request_view",
        true,
        RequestViewArgs => crate::jsm::RequestDetail,
        "One customer request with its description and latest comments."
    ),
    task!(
        "jsm_request_create",
        false,
        RequestCreateArgs => crate::jsm::Request,
        "Raise a customer request, optionally for a customer."
    ),
    task!(
        "jsm_request_comment",
        false,
        RequestCommentArgs => crate::jsm::RequestComment,
        "Reply to a customer request, or add an internal note."
    ),
    task!(
        "jsm_request_transitions",
        true,
        KeyArgs => Vec<crate::jsm::RequestTransition>,
        "Transitions available on a customer request."
    ),
    task!(
        "jsm_request_move",
        false,
        RequestMoveArgs => RequestMoved,
        "Run a customer request transition by name or ID."
    ),
    task!(
        "jsm_request_approvals",
        true,
        KeyArgs => Vec<crate::jsm::Approval>,
        "Approvals of a JSM request: name, decision (pending, approved, declined), approvers, and whether you can answer."
    ),
    task!(
        "jsm_request_approve",
        false,
        ApproveArgs => crate::jsm::Approval,
        "Approve or decline (decision: decline) a JSM request's approval; without an ID, the one pending approval you can answer."
    ),
    task!(
        "jsm_request_participants",
        false,
        ParticipantsArgs => Participants,
        "List a JSM request's participants, after adding and removing any (me, an email or a name)."
    ),
    task!(
        "jsm_request_sla",
        true,
        KeyArgs => Vec<crate::jsm::Sla>,
        "SLAs of a JSM request: state (ongoing, paused, met, breached), time remaining and goal."
    ),
    task!(
        "jsm_request_attach",
        false,
        RequestAttachArgs => Vec<crate::jsm::RequestAttachment>,
        "Attach local files to a JSM request (public unless internal)."
    ),
    task!(
        "jsm_request_attachments",
        true,
        KeyArgs => Vec<crate::jsm::RequestAttachment>,
        "Attachments of a JSM request with download URLs."
    ),
];

pub(super) async fn run(client: &Client, name: &str, args: Value) -> Result<Value> {
    let desk = ServiceDesk::new(client);
    match name {
        "jsm_request_approvals" => {
            let a: KeyArgs = parse(name, args)?;
            output(name, desk.approvals(&a.key).await?)
        }
        "jsm_request_approve" => {
            let a: ApproveArgs = parse(name, args)?;
            let approve = match a.decision.as_str() {
                "approve" => true,
                "decline" => false,
                other => {
                    return Err(Error::Invalid(format!(
                        "decision `{other}` is unknown; use approve or decline"
                    )));
                }
            };
            output(
                name,
                desk.answer_approval(&a.key, a.approval.as_deref(), approve)
                    .await?,
            )
        }
        "jsm_request_participants" => {
            let a: ParticipantsArgs = parse(name, args)?;
            let participants = desk.participants(&a.key, &a.add, &a.remove).await?;
            output(
                name,
                Participants {
                    key: a.key,
                    participants,
                },
            )
        }
        "jsm_request_sla" => {
            let a: KeyArgs = parse(name, args)?;
            output(name, desk.slas(&a.key).await?)
        }
        "jsm_request_attach" => {
            let a: RequestAttachArgs = parse(name, args)?;
            output(name, desk.attach(&a.key, &a.files, !a.internal).await?)
        }
        "jsm_request_attachments" => {
            let a: KeyArgs = parse(name, args)?;
            output(name, desk.attachments(&a.key).await?)
        }
        "jsm_desk_list" => {
            let _: Empty = parse(name, args)?;
            output(name, desk.desks().await?)
        }
        "jsm_request_types" => {
            let a: DeskArgs = parse(name, args)?;
            output(name, desk.request_types(&a.desk).await?)
        }
        "jsm_queue_list" => {
            let a: DeskArgs = parse(name, args)?;
            output(name, desk.queues(&a.desk).await?)
        }
        "jsm_queue_issues" => {
            let a: QueueIssuesArgs = parse(name, args)?;
            output(name, desk.queue_issues(&a.desk, &a.queue, a.limit).await?)
        }
        "jsm_request_list" => {
            let a: RequestListArgs = parse(name, args)?;
            output(
                name,
                desk.requests(
                    a.desk.as_deref(),
                    &a.status,
                    a.search.as_deref(),
                    a.mine,
                    a.limit,
                )
                .await?,
            )
        }
        "jsm_request_view" => {
            let a: RequestViewArgs = parse(name, args)?;
            output(name, desk.request(&a.key, a.comments).await?)
        }
        "jsm_request_create" => {
            let a: RequestCreateArgs = parse(name, args)?;
            output(
                name,
                desk.create(
                    &a.desk,
                    &a.request_type,
                    &a.summary,
                    a.description.as_deref(),
                    a.on_behalf_of.as_deref(),
                )
                .await?,
            )
        }
        "jsm_request_comment" => {
            let a: RequestCommentArgs = parse(name, args)?;
            output(name, desk.comment(&a.key, &a.body, a.internal).await?)
        }
        "jsm_request_transitions" => {
            let a: KeyArgs = parse(name, args)?;
            output(name, desk.transitions(&a.key).await?)
        }
        "jsm_request_move" => {
            let a: RequestMoveArgs = parse(name, args)?;
            let t = desk
                .transition(&a.key, &a.transition, a.comment.as_deref())
                .await?;
            output(
                name,
                RequestMoved {
                    key: a.key,
                    transition: t.name,
                    id: t.id,
                },
            )
        }
        _ => Err(unknown(name)),
    }
}

#[derive(Serialize, JsonSchema)]
struct Participants {
    key: String,
    /// Every participant after the change.
    participants: Vec<String>,
}

#[derive(Serialize, JsonSchema)]
struct RequestMoved {
    key: String,
    /// The transition that ran.
    transition: String,
    /// Its ID.
    id: String,
}
