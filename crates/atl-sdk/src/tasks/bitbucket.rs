//! Bitbucket tasks.

use super::*;
use crate::bitbucket::{Bitbucket, RepoRef};
use crate::client::Client;
use crate::error::Result;
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::Value;

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
        RepoListArgs => Vec<crate::bitbucket::Repo>,
        "Bitbucket repositories of a workspace (Cloud) or project (Data Center), or the ones you belong to."
    ),
    task!(
        "bitbucket_repo_view",
        true,
        RepoArgs => crate::bitbucket::Repo,
        "One Bitbucket repository with its default branch and URL."
    ),
    task!(
        "bitbucket_branch_list",
        true,
        BranchListArgs => Vec<crate::bitbucket::Branch>,
        "Branches of a Bitbucket repository, most recently changed first."
    ),
    task!(
        "bitbucket_pr_list",
        true,
        PrListArgs => Vec<crate::bitbucket::PullRequest>,
        "Pull requests of a Bitbucket repository by state."
    ),
    task!(
        "bitbucket_pr_view",
        true,
        PrArgs => crate::bitbucket::PullRequest,
        "One pull request with description, reviewers and approvals."
    ),
    task!(
        "bitbucket_pr_create",
        false,
        PrCreateArgs => crate::bitbucket::PullRequest,
        "Open a pull request from a source branch."
    ),
    task!(
        "bitbucket_pr_merge",
        false,
        PrMergeArgs => crate::bitbucket::PullRequest,
        "Merge a pull request.",
        destructive
    ),
    task!(
        "bitbucket_pr_approve",
        false,
        PrArgs => PrApproval,
        "Approve a pull request as the signed-in user."
    ),
    task!(
        "bitbucket_pr_comment",
        false,
        PrCommentArgs => PrCommented,
        "Add a Markdown comment to a pull request."
    ),
    task!(
        "bitbucket_pr_diff",
        true,
        PrArgs => PrDiff,
        "The unified diff of a pull request."
    ),
    task!(
        "bitbucket_pr_comments",
        true,
        PrCommentsArgs => Vec<crate::bitbucket::PrComment>,
        "Comments of a pull request, oldest first, as Markdown: general and inline (path, line), replies flattened with `parent`."
    ),
    task!(
        "bitbucket_pr_checks",
        true,
        PrArgs => PrChecks,
        "Build statuses of a pull request's latest commit, with `passed` (none failed) and `pending` (some still running)."
    ),
    task!(
        "bitbucket_pr_unapprove",
        false,
        PrArgs => PrApproval,
        "Remove your approval from a pull request."
    ),
    task!(
        "bitbucket_pr_ready",
        false,
        PrReadyArgs => crate::bitbucket::PullRequest,
        "Mark a draft pull request as ready for review (or, with `draft: true`, as a draft again)."
    ),
    task!(
        "bitbucket_branch_create",
        false,
        BranchCreateArgs => crate::bitbucket::Branch,
        "Create a branch from another branch, a tag or a commit (default: the main branch)."
    ),
    task!(
        "bitbucket_branch_delete",
        false,
        BranchDeleteArgs => BranchDeleted,
        "Delete a branch. Unmerged commits on it can be lost.",
        destructive
    ),
    task!(
        "bitbucket_commit_list",
        true,
        CommitListArgs => Vec<crate::bitbucket::Commit>,
        "Recent commits of a branch, newest first: hash, author, date, first line of the message."
    ),
    task!(
        "bitbucket_pipeline_list",
        true,
        PipelineListArgs => Vec<crate::bitbucket::Pipeline>,
        "Latest Bitbucket Pipelines runs, newest first (Cloud only)."
    ),
    task!(
        "bitbucket_pipeline_run",
        false,
        PipelineRunArgs => crate::bitbucket::Pipeline,
        "Start the pipeline of a branch, or a custom pipeline (Cloud only)."
    ),
    task!(
        "bitbucket_pr_decline",
        false,
        PrArgs => crate::bitbucket::PullRequest,
        "Decline a Bitbucket pull request (Cloud cannot reopen it).",
        destructive
    ),
    task!(
        "bitbucket_pr_request_changes",
        false,
        PrArgs => ChangesRequested,
        "Mark a Bitbucket pull request as needing changes by you (Cloud: request changes; Data Center: needs work)."
    ),
    task!(
        "bitbucket_pr_edit",
        false,
        PrEditArgs => crate::bitbucket::PullRequest,
        "Change a Bitbucket pull request's title, description (Markdown) or destination branch."
    ),
    task!(
        "bitbucket_pr_reviewers",
        false,
        PrReviewersArgs => crate::bitbucket::PullRequest,
        "Add and remove reviewers of a Bitbucket pull request; other reviewers are kept."
    ),
    task!(
        "bitbucket_pipeline_stop",
        false,
        PipelineArgs => crate::bitbucket::Pipeline,
        "Stop a running Bitbucket pipeline by build number or UUID (Cloud only)."
    ),
    task!(
        "bitbucket_pipeline_view",
        true,
        PipelineArgs => crate::bitbucket::PipelineDetail,
        "One Bitbucket pipeline by build number or UUID, with each step's state and result (Cloud only)."
    ),
    task!(
        "bitbucket_pipeline_logs",
        true,
        PipelineLogArgs => PipelineLog,
        "Log text of one pipeline step (by name or UUID) or of every step (Cloud only)."
    ),
];

pub(super) async fn run(client: &Client, name: &str, args: Value) -> Result<Value> {
    let bitbucket = Bitbucket::new(client);
    match name {
        "bitbucket_repo_list" => {
            let a: RepoListArgs = parse(name, args)?;
            output(name, bitbucket.repos(a.owner.as_deref(), a.limit).await?)
        }
        "bitbucket_repo_view" => {
            let a: RepoArgs = parse(name, args)?;
            output(name, bitbucket.repo(&a.repo).await?)
        }
        "bitbucket_branch_list" => {
            let a: BranchListArgs = parse(name, args)?;
            output(
                name,
                bitbucket
                    .branches(&a.repo, a.filter.as_deref(), a.limit)
                    .await?,
            )
        }
        "bitbucket_pr_list" => {
            let a: PrListArgs = parse(name, args)?;
            output(
                name,
                bitbucket.pull_requests(&a.repo, &a.state, a.limit).await?,
            )
        }
        "bitbucket_pr_view" => {
            let a: PrArgs = parse(name, args)?;
            output(name, bitbucket.pull_request(&a.repo, a.id).await?)
        }
        "bitbucket_pr_create" => {
            let a: PrCreateArgs = parse(name, args)?;
            output(
                name,
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
            output(
                name,
                bitbucket
                    .merge(&a.repo, a.id, a.strategy.as_deref())
                    .await?,
            )
        }
        "bitbucket_pr_approve" => {
            let a: PrArgs = parse(name, args)?;
            bitbucket.approve(&a.repo, a.id).await?;
            output(
                name,
                PrApproval {
                    repo: a.repo,
                    id: a.id,
                    approved: true,
                },
            )
        }
        "bitbucket_pr_comment" => {
            let a: PrCommentArgs = parse(name, args)?;
            let id = bitbucket.comment(&a.repo, a.id, &a.body).await?;
            output(
                name,
                PrCommented {
                    repo: a.repo,
                    id: a.id,
                    comment_id: id,
                },
            )
        }
        "bitbucket_pr_diff" => {
            let a: PrArgs = parse(name, args)?;
            let diff = bitbucket.diff(&a.repo, a.id).await?;
            output(
                name,
                PrDiff {
                    repo: a.repo,
                    id: a.id,
                    diff,
                },
            )
        }
        "bitbucket_pr_comments" => {
            let a: PrCommentsArgs = parse(name, args)?;
            output(name, bitbucket.pr_comments(&a.repo, a.id, a.limit).await?)
        }
        "bitbucket_pr_checks" => {
            let a: PrArgs = parse(name, args)?;
            let checks = bitbucket.pr_checks(&a.repo, a.id).await?;
            let passed = !checks.iter().any(|c| c.failed());
            let pending = checks.iter().any(|c| c.pending());
            output(
                name,
                PrChecks {
                    repo: a.repo,
                    id: a.id,
                    passed,
                    pending,
                    checks,
                },
            )
        }
        "bitbucket_pr_unapprove" => {
            let a: PrArgs = parse(name, args)?;
            bitbucket.unapprove(&a.repo, a.id).await?;
            output(
                name,
                PrApproval {
                    repo: a.repo,
                    id: a.id,
                    approved: false,
                },
            )
        }
        "bitbucket_pr_ready" => {
            let a: PrReadyArgs = parse(name, args)?;
            output(name, bitbucket.set_draft(&a.repo, a.id, a.draft).await?)
        }
        "bitbucket_branch_create" => {
            let a: BranchCreateArgs = parse(name, args)?;
            output(
                name,
                bitbucket
                    .create_branch(&a.repo, &a.name, a.from.as_deref())
                    .await?,
            )
        }
        "bitbucket_branch_delete" => {
            let a: BranchDeleteArgs = parse(name, args)?;
            bitbucket.delete_branch(&a.repo, &a.name).await?;
            output(
                name,
                BranchDeleted {
                    repo: a.repo,
                    name: a.name,
                    deleted: true,
                },
            )
        }
        "bitbucket_commit_list" => {
            let a: CommitListArgs = parse(name, args)?;
            output(
                name,
                bitbucket
                    .commits(&a.repo, a.branch.as_deref(), a.limit)
                    .await?,
            )
        }
        "bitbucket_pipeline_stop" => {
            let a: PipelineArgs = parse(name, args)?;
            output(name, bitbucket.stop_pipeline(&a.repo, &a.id).await?)
        }
        "bitbucket_pipeline_list" => {
            let a: PipelineListArgs = parse(name, args)?;
            output(name, bitbucket.pipelines(&a.repo, a.limit).await?)
        }
        "bitbucket_pipeline_run" => {
            let a: PipelineRunArgs = parse(name, args)?;
            output(
                name,
                bitbucket
                    .run_pipeline(&a.repo, &a.branch, a.custom.as_deref())
                    .await?,
            )
        }
        "bitbucket_pr_decline" => {
            let a: PrArgs = parse(name, args)?;
            output(name, bitbucket.decline(&a.repo, a.id).await?)
        }
        "bitbucket_pr_request_changes" => {
            let a: PrArgs = parse(name, args)?;
            bitbucket.request_changes(&a.repo, a.id).await?;
            output(
                name,
                ChangesRequested {
                    repo: a.repo,
                    id: a.id,
                    changes_requested: true,
                },
            )
        }
        "bitbucket_pr_edit" => {
            let a: PrEditArgs = parse(name, args)?;
            output(
                name,
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
            output(
                name,
                bitbucket
                    .set_reviewers(&a.repo, a.id, &a.add, &a.remove)
                    .await?,
            )
        }
        "bitbucket_pipeline_view" => {
            let a: PipelineArgs = parse(name, args)?;
            output(name, bitbucket.pipeline(&a.repo, &a.id).await?)
        }
        "bitbucket_pipeline_logs" => {
            let a: PipelineLogArgs = parse(name, args)?;
            let log = bitbucket
                .pipeline_logs(&a.repo, &a.id, a.step.as_deref())
                .await?;
            output(
                name,
                PipelineLog {
                    repo: a.repo,
                    id: a.id,
                    log,
                },
            )
        }
        _ => Err(unknown(name)),
    }
}

#[derive(Serialize, JsonSchema)]
struct PrApproval {
    repo: RepoRef,
    id: u64,
    /// True after an approve, false after an unapprove.
    approved: bool,
}

#[derive(Serialize, JsonSchema)]
struct PrCommented {
    repo: RepoRef,
    id: u64,
    comment_id: String,
}

#[derive(Serialize, JsonSchema)]
struct PrDiff {
    repo: RepoRef,
    id: u64,
    /// The unified diff.
    diff: String,
}

#[derive(Serialize, JsonSchema)]
struct PrChecks {
    repo: RepoRef,
    id: u64,
    /// No check failed.
    passed: bool,
    /// At least one check is still running.
    pending: bool,
    checks: Vec<crate::bitbucket::BuildStatus>,
}

#[derive(Serialize, JsonSchema)]
struct BranchDeleted {
    repo: RepoRef,
    name: String,
    deleted: bool,
}

#[derive(Serialize, JsonSchema)]
struct ChangesRequested {
    repo: RepoRef,
    id: u64,
    changes_requested: bool,
}

#[derive(Serialize, JsonSchema)]
struct PipelineLog {
    repo: RepoRef,
    id: String,
    /// The step logs as text.
    log: String,
}
