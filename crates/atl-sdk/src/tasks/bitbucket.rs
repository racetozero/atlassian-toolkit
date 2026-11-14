//! Bitbucket tasks.

use super::*;
use crate::bitbucket::{Bitbucket, RepoRef};
use crate::client::Client;
use crate::error::Result;
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::{Value, json};

fn default_pr_state() -> String {
    "open".into()
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct RepoListArgs {
    /// Workspace (Cloud) or project key (Data Center); omit for your repositories.
    owner: Option<String>,
    #[serde(default = "default_limit")]
    limit: usize,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct RepoArgs {
    /// WORKSPACE/REPO (Cloud) or PROJECT/REPO (Data Center).
    repo: RepoRef,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct BranchListArgs {
    repo: RepoRef,
    /// Part of the branch name.
    filter: Option<String>,
    #[serde(default = "default_limit")]
    limit: usize,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct PrListArgs {
    repo: RepoRef,
    /// open, merged, declined, or all.
    #[serde(default = "default_pr_state")]
    state: String,
    #[serde(default = "default_limit")]
    limit: usize,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct PrArgs {
    repo: RepoRef,
    /// Pull request ID.
    id: u64,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct PrCreateArgs {
    repo: RepoRef,
    title: String,
    /// Source branch.
    source: String,
    /// Destination branch (default: the repository's main branch).
    destination: Option<String>,
    /// Description in Markdown.
    description: Option<String>,
    /// Open it as a draft (not ready for review).
    #[serde(default)]
    draft: bool,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct PrCommentsArgs {
    repo: RepoRef,
    id: u64,
    #[serde(default = "default_comment_limit")]
    limit: usize,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct PrReadyArgs {
    repo: RepoRef,
    id: u64,
    /// True turns it back into a draft.
    #[serde(default)]
    draft: bool,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct BranchCreateArgs {
    repo: RepoRef,
    name: String,
    /// Branch, tag or commit to start from (default: the main branch).
    from: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct BranchDeleteArgs {
    repo: RepoRef,
    name: String,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct CommitListArgs {
    repo: RepoRef,
    /// Branch (default: the main branch).
    branch: Option<String>,
    #[serde(default = "default_commit_limit")]
    limit: usize,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct PrMergeArgs {
    repo: RepoRef,
    id: u64,
    /// Cloud only: merge_commit, squash, or fast_forward.
    strategy: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct PrCommentArgs {
    repo: RepoRef,
    id: u64,
    /// Comment in Markdown.
    body: String,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct PrEditArgs {
    repo: RepoRef,
    id: u64,
    title: Option<String>,
    /// New description in Markdown.
    description: Option<String>,
    /// New destination branch.
    destination: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct PrReviewersArgs {
    repo: RepoRef,
    id: u64,
    /// Users to add: display name, nickname, account ID or {uuid} (Cloud); username, name or email (Data Center).
    #[serde(default)]
    add: Vec<String>,
    /// Reviewers to remove, matched the same way.
    #[serde(default)]
    remove: Vec<String>,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct PipelineArgs {
    repo: RepoRef,
    /// Build number or pipeline UUID.
    id: String,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct PipelineLogArgs {
    repo: RepoRef,
    /// Build number or pipeline UUID.
    id: String,
    /// Step name or UUID (default: every step).
    step: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct PipelineListArgs {
    repo: RepoRef,
    #[serde(default = "default_limit")]
    limit: usize,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct PipelineRunArgs {
    repo: RepoRef,
    branch: String,
    /// Name of a custom pipeline in bitbucket-pipelines.yml.
    custom: Option<String>,
}

fn default_comment_limit() -> usize {
    200
}

fn default_commit_limit() -> usize {
    20
}

pub(super) static TASKS: &[TaskDef] = &[
    task!(
        "bitbucket_repo_list",
        true,
        RepoListArgs,
        "Bitbucket repositories of a workspace (Cloud) or project (Data Center), or the ones you belong to."
    ),
    task!(
        "bitbucket_repo_view",
        true,
        RepoArgs,
        "One Bitbucket repository with its default branch and URL."
    ),
    task!(
        "bitbucket_branch_list",
        true,
        BranchListArgs,
        "Branches of a Bitbucket repository, most recently changed first."
    ),
    task!(
        "bitbucket_pr_list",
        true,
        PrListArgs,
        "Pull requests of a Bitbucket repository by state."
    ),
    task!(
        "bitbucket_pr_view",
        true,
        PrArgs,
        "One pull request with description, reviewers and approvals."
    ),
    task!(
        "bitbucket_pr_create",
        false,
        PrCreateArgs,
        "Open a pull request from a source branch."
    ),
    task!(
        "bitbucket_pr_merge",
        false,
        PrMergeArgs,
        "Merge a pull request.",
        destructive
    ),
    task!(
        "bitbucket_pr_approve",
        false,
        PrArgs,
        "Approve a pull request as the signed-in user."
    ),
    task!(
        "bitbucket_pr_comment",
        false,
        PrCommentArgs,
        "Add a Markdown comment to a pull request."
    ),
    task!(
        "bitbucket_pr_diff",
        true,
        PrArgs,
        "The unified diff of a pull request."
    ),
    task!(
        "bitbucket_pr_comments",
        true,
        PrCommentsArgs,
        "Comments of a pull request, oldest first, as Markdown: general and inline (path, line), replies flattened with `parent`."
    ),
    task!(
        "bitbucket_pr_checks",
        true,
        PrArgs,
        "Build statuses of a pull request's latest commit, with `passed` (none failed) and `pending` (some still running)."
    ),
    task!(
        "bitbucket_pr_unapprove",
        false,
        PrArgs,
        "Remove your approval from a pull request."
    ),
    task!(
        "bitbucket_pr_ready",
        false,
        PrReadyArgs,
        "Mark a draft pull request as ready for review (or, with `draft: true`, as a draft again)."
    ),
    task!(
        "bitbucket_branch_create",
        false,
        BranchCreateArgs,
        "Create a branch from another branch, a tag or a commit (default: the main branch)."
    ),
    task!(
        "bitbucket_branch_delete",
        false,
        BranchDeleteArgs,
        "Delete a branch. Unmerged commits on it can be lost.",
        destructive
    ),
    task!(
        "bitbucket_commit_list",
        true,
        CommitListArgs,
        "Recent commits of a branch, newest first: hash, author, date, first line of the message."
    ),
    task!(
        "bitbucket_pipeline_list",
        true,
        PipelineListArgs,
        "Latest Bitbucket Pipelines runs, newest first (Cloud only)."
    ),
    task!(
        "bitbucket_pipeline_run",
        false,
        PipelineRunArgs,
        "Start the pipeline of a branch, or a custom pipeline (Cloud only)."
    ),
    task!(
        "bitbucket_pr_decline",
        false,
        PrArgs,
        "Decline a Bitbucket pull request (Cloud cannot reopen it).",
        destructive
    ),
    task!(
        "bitbucket_pr_request_changes",
        false,
        PrArgs,
        "Mark a Bitbucket pull request as needing changes by you (Cloud: request changes; Data Center: needs work)."
    ),
    task!(
        "bitbucket_pr_edit",
        false,
        PrEditArgs,
        "Change a Bitbucket pull request's title, description (Markdown) or destination branch."
    ),
    task!(
        "bitbucket_pr_reviewers",
        false,
        PrReviewersArgs,
        "Add and remove reviewers of a Bitbucket pull request; other reviewers are kept."
    ),
    task!(
        "bitbucket_pipeline_stop",
        false,
        PipelineArgs,
        "Stop a running Bitbucket pipeline by build number or UUID (Cloud only)."
    ),
    task!(
        "bitbucket_pipeline_view",
        true,
        PipelineArgs,
        "One Bitbucket pipeline by build number or UUID, with each step's state and result (Cloud only)."
    ),
    task!(
        "bitbucket_pipeline_logs",
        true,
        PipelineLogArgs,
        "Log text of one pipeline step (by name or UUID) or of every step (Cloud only)."
    ),
];

pub(super) async fn run(client: &Client, name: &str, args: Value) -> Result<Value> {
    let bitbucket = Bitbucket::new(client);
    match name {
        "bitbucket_repo_list" => {
            let a: RepoListArgs = parse(name, args)?;
            to_json(bitbucket.repos(a.owner.as_deref(), a.limit).await?)
        }
        "bitbucket_repo_view" => {
            let a: RepoArgs = parse(name, args)?;
            to_json(bitbucket.repo(&a.repo).await?)
        }
        "bitbucket_branch_list" => {
            let a: BranchListArgs = parse(name, args)?;
            to_json(
                bitbucket
                    .branches(&a.repo, a.filter.as_deref(), a.limit)
                    .await?,
            )
        }
        "bitbucket_pr_list" => {
            let a: PrListArgs = parse(name, args)?;
            to_json(bitbucket.pull_requests(&a.repo, &a.state, a.limit).await?)
        }
        "bitbucket_pr_view" => {
            let a: PrArgs = parse(name, args)?;
            to_json(bitbucket.pull_request(&a.repo, a.id).await?)
        }
        "bitbucket_pr_create" => {
            let a: PrCreateArgs = parse(name, args)?;
            to_json(
                bitbucket
                    .create_pull_request(
                        &a.repo,
                        &a.title,
                        &a.source,
                        a.destination.as_deref(),
                        a.description.as_deref(),
                        a.draft,
                    )
                    .await?,
            )
        }
        "bitbucket_pr_merge" => {
            let a: PrMergeArgs = parse(name, args)?;
            to_json(
                bitbucket
                    .merge(&a.repo, a.id, a.strategy.as_deref())
                    .await?,
            )
        }
        "bitbucket_pr_approve" => {
            let a: PrArgs = parse(name, args)?;
            bitbucket.approve(&a.repo, a.id).await?;
            Ok(json!({ "repo": a.repo, "id": a.id, "approved": true }))
        }
        "bitbucket_pr_comment" => {
            let a: PrCommentArgs = parse(name, args)?;
            let id = bitbucket.comment(&a.repo, a.id, &a.body).await?;
            Ok(json!({ "repo": a.repo, "id": a.id, "comment_id": id }))
        }
        "bitbucket_pr_diff" => {
            let a: PrArgs = parse(name, args)?;
            let diff = bitbucket.diff(&a.repo, a.id).await?;
            Ok(json!({ "repo": a.repo, "id": a.id, "diff": diff }))
        }
        "bitbucket_pr_comments" => {
            let a: PrCommentsArgs = parse(name, args)?;
            to_json(bitbucket.pr_comments(&a.repo, a.id, a.limit).await?)
        }
        "bitbucket_pr_checks" => {
            let a: PrArgs = parse(name, args)?;
            let checks = bitbucket.pr_checks(&a.repo, a.id).await?;
            let passed = !checks.iter().any(|c| c.failed());
            let pending = checks.iter().any(|c| c.pending());
            Ok(
                json!({ "repo": a.repo, "id": a.id, "passed": passed, "pending": pending, "checks": to_json(&checks)? }),
            )
        }
        "bitbucket_pr_unapprove" => {
            let a: PrArgs = parse(name, args)?;
            bitbucket.unapprove(&a.repo, a.id).await?;
            Ok(json!({ "repo": a.repo, "id": a.id, "approved": false }))
        }
        "bitbucket_pr_ready" => {
            let a: PrReadyArgs = parse(name, args)?;
            to_json(bitbucket.set_draft(&a.repo, a.id, a.draft).await?)
        }
        "bitbucket_branch_create" => {
            let a: BranchCreateArgs = parse(name, args)?;
            to_json(
                bitbucket
                    .create_branch(&a.repo, &a.name, a.from.as_deref())
                    .await?,
            )
        }
        "bitbucket_branch_delete" => {
            let a: BranchDeleteArgs = parse(name, args)?;
            bitbucket.delete_branch(&a.repo, &a.name).await?;
            Ok(json!({ "repo": a.repo, "name": a.name, "deleted": true }))
        }
        "bitbucket_commit_list" => {
            let a: CommitListArgs = parse(name, args)?;
            to_json(
                bitbucket
                    .commits(&a.repo, a.branch.as_deref(), a.limit)
                    .await?,
            )
        }
        "bitbucket_pipeline_stop" => {
            let a: PipelineArgs = parse(name, args)?;
            to_json(bitbucket.stop_pipeline(&a.repo, &a.id).await?)
        }
        "bitbucket_pipeline_list" => {
            let a: PipelineListArgs = parse(name, args)?;
            to_json(bitbucket.pipelines(&a.repo, a.limit).await?)
        }
        "bitbucket_pipeline_run" => {
            let a: PipelineRunArgs = parse(name, args)?;
            to_json(
                bitbucket
                    .run_pipeline(&a.repo, &a.branch, a.custom.as_deref())
                    .await?,
            )
        }
        "bitbucket_pr_decline" => {
            let a: PrArgs = parse(name, args)?;
            to_json(bitbucket.decline(&a.repo, a.id).await?)
        }
        "bitbucket_pr_request_changes" => {
            let a: PrArgs = parse(name, args)?;
            bitbucket.request_changes(&a.repo, a.id).await?;
            Ok(json!({ "repo": a.repo, "id": a.id, "changes_requested": true }))
        }
        "bitbucket_pr_edit" => {
            let a: PrEditArgs = parse(name, args)?;
            to_json(
                bitbucket
                    .edit_pull_request(
                        &a.repo,
                        a.id,
                        a.title.as_deref(),
                        a.description.as_deref(),
                        a.destination.as_deref(),
                    )
                    .await?,
            )
        }
        "bitbucket_pr_reviewers" => {
            let a: PrReviewersArgs = parse(name, args)?;
            to_json(
                bitbucket
                    .set_reviewers(&a.repo, a.id, &a.add, &a.remove)
                    .await?,
            )
        }
        "bitbucket_pipeline_view" => {
            let a: PipelineArgs = parse(name, args)?;
            to_json(bitbucket.pipeline(&a.repo, &a.id).await?)
        }
        "bitbucket_pipeline_logs" => {
            let a: PipelineLogArgs = parse(name, args)?;
            let log = bitbucket
                .pipeline_logs(&a.repo, &a.id, a.step.as_deref())
                .await?;
            Ok(json!({ "repo": a.repo, "id": a.id, "log": log }))
        }
        _ => Err(unknown(name)),
    }
}
