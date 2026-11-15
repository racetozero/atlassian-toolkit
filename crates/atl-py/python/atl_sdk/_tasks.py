"""Generated from the task registry (crates/atl-sdk/src/tasks/) by
crates/atl-sdk/tests/sdk_types.rs. Do not edit; run
`ATL_UPDATE_SDK_TYPES=1 cargo test -p atlassian-toolkit --test sdk_types`.
"""

from __future__ import annotations

from collections.abc import Mapping
from enum import StrEnum
from typing import Any, NotRequired, Protocol, Required, TypedDict, Unpack

import pydantic

__all__ = ["AdminApi", "AdminDirectoryApi", "AdminDirectoryListArgs", "AdminDomainApi", "AdminDomainListArgs", "AdminEventApi", "AdminEventListArgs", "AdminGroupAddMemberArgs", "AdminGroupApi", "AdminGroupListArgs", "AdminGroupRemoveMemberArgs", "AdminOrgApi", "AdminOrgListArgs", "AdminUser", "AdminUserApi", "AdminUserListArgs", "AdminUserRestoreArgs", "AdminUserSuspendArgs", "Approval", "Assigned", "AsyncAdminApi", "AsyncAdminDirectoryApi", "AsyncAdminDomainApi", "AsyncAdminEventApi", "AsyncAdminGroupApi", "AsyncAdminOrgApi", "AsyncAdminUserApi", "AsyncBitbucketApi", "AsyncBitbucketBranchApi", "AsyncBitbucketCommitApi", "AsyncBitbucketPipelineApi", "AsyncBitbucketPrApi", "AsyncBitbucketRepoApi", "AsyncConfluenceApi", "AsyncConfluenceAttachmentApi", "AsyncConfluenceBlogApi", "AsyncConfluenceCommentApi", "AsyncConfluencePageApi", "AsyncConfluenceSpaceApi", "AsyncJiraApi", "AsyncJiraBoardApi", "AsyncJiraCommentApi", "AsyncJiraComponentApi", "AsyncJiraEpicApi", "AsyncJiraFieldApi", "AsyncJiraFilterApi", "AsyncJiraIssueApi", "AsyncJiraJqlApi", "AsyncJiraProjectApi", "AsyncJiraReleaseApi", "AsyncJiraSprintApi", "AsyncJiraWorklogApi", "AsyncJsmApi", "AsyncJsmDeskApi", "AsyncJsmQueueApi", "AsyncJsmRequestApi", "AsyncTrelloApi", "AsyncTrelloBoardApi", "AsyncTrelloCardApi", "AsyncTrelloChecklistApi", "AsyncTrelloListApi", "Attachment", "BitbucketApi", "BitbucketBranchApi", "BitbucketBranchCreateArgs", "BitbucketBranchDeleteArgs", "BitbucketBranchListArgs", "BitbucketCommitApi", "BitbucketCommitListArgs", "BitbucketPipelineApi", "BitbucketPipelineListArgs", "BitbucketPipelineLogsArgs", "BitbucketPipelineRunArgs", "BitbucketPipelineStopArgs", "BitbucketPipelineViewArgs", "BitbucketPrApi", "BitbucketPrApproveArgs", "BitbucketPrChecksArgs", "BitbucketPrCommentArgs", "BitbucketPrCommentsArgs", "BitbucketPrCreateArgs", "BitbucketPrDeclineArgs", "BitbucketPrDiffArgs", "BitbucketPrEditArgs", "BitbucketPrListArgs", "BitbucketPrMergeArgs", "BitbucketPrReadyArgs", "BitbucketPrRequestChangesArgs", "BitbucketPrReviewersArgs", "BitbucketPrUnapproveArgs", "BitbucketPrViewArgs", "BitbucketRepoApi", "BitbucketRepoListArgs", "BitbucketRepoViewArgs", "Board", "BoardSprints", "Branch", "BranchDeleted", "BuildStatus", "Card", "CardComment", "CardDetail", "Change", "ChangesRequested", "CheckItem", "Checklist", "Comment", "CommentDeleted", "Commit", "Component", "ConfluenceApi", "ConfluenceAttachmentApi", "ConfluenceAttachmentDeleteArgs", "ConfluenceBlogApi", "ConfluenceBlogCreateArgs", "ConfluenceBlogGetArgs", "ConfluenceBlogListArgs", "ConfluenceCommentApi", "ConfluenceCommentDeleteArgs", "ConfluencePageApi", "ConfluencePageAttachArgs", "ConfluencePageAttachmentsArgs", "ConfluencePageChildrenArgs", "ConfluencePageCommentArgs", "ConfluencePageCommentsArgs", "ConfluencePageCopyTreeArgs", "ConfluencePageCreateArgs", "ConfluencePageDeleteArgs", "ConfluencePageExportArgs", "ConfluencePageFindArgs", "ConfluencePageGetArgs", "ConfluencePageHistoryArgs", "ConfluencePageLabelArgs", "ConfluencePageLabelsArgs", "ConfluencePageMoveArgs", "ConfluencePagePushArgs", "ConfluencePageUpdateArgs", "ConfluenceSearchArgs", "ConfluenceSpaceApi", "ConfluenceSpaceCreateArgs", "ConfluenceSpaceListArgs", "ConfluenceSpacePagesArgs", "Copied", "Created", "Deleted", "Desk", "Directory", "Domain", "EpicIssues", "ErrorCode", "Event", "Exported", "Field", "Filter", "Group", "IssueCount", "IssueDeleted", "IssueDetail", "IssueLink", "IssueList", "IssueSummary", "IssueUpdated", "JiraApi", "JiraBoardApi", "JiraBoardListArgs", "JiraCommentApi", "JiraCommentDeleteArgs", "JiraCommentEditArgs", "JiraCommentListArgs", "JiraComponentApi", "JiraComponentListArgs", "JiraEpicAddArgs", "JiraEpicApi", "JiraEpicListArgs", "JiraEpicRemoveArgs", "JiraFieldApi", "JiraFieldListArgs", "JiraFilterApi", "JiraFilterListArgs", "JiraFilterRunArgs", "JiraIssueApi", "JiraIssueAssignArgs", "JiraIssueAttachArgs", "JiraIssueAttachmentsArgs", "JiraIssueCloneArgs", "JiraIssueCommentArgs", "JiraIssueCountArgs", "JiraIssueCreateArgs", "JiraIssueDeleteArgs", "JiraIssueEditArgs", "JiraIssueHistoryArgs", "JiraIssueLinkArgs", "JiraIssueLinkTypesArgs", "JiraIssueListArgs", "JiraIssueMoveArgs", "JiraIssueRemoteLinkArgs", "JiraIssueTransitionsArgs", "JiraIssueTypesArgs", "JiraIssueUnlinkArgs", "JiraIssueUnwatchArgs", "JiraIssueViewArgs", "JiraIssueWatchArgs", "JiraIssueWatchersArgs", "JiraIssueWorklogArgs", "JiraJqlApi", "JiraJqlCheckArgs", "JiraJqlFieldsArgs", "JiraJqlSuggestArgs", "JiraMeArgs", "JiraProjectApi", "JiraProjectListArgs", "JiraReleaseApi", "JiraReleaseListArgs", "JiraSprintAddArgs", "JiraSprintApi", "JiraSprintCloseArgs", "JiraSprintCreateArgs", "JiraSprintListArgs", "JiraSprintStartArgs", "JiraWorklogApi", "JiraWorklogListArgs", "JqlCheck", "JqlField", "JqlReference", "JsmApi", "JsmDeskApi", "JsmDeskListArgs", "JsmQueueApi", "JsmQueueIssuesArgs", "JsmQueueListArgs", "JsmRequestApi", "JsmRequestApprovalsArgs", "JsmRequestApproveArgs", "JsmRequestAttachArgs", "JsmRequestAttachmentsArgs", "JsmRequestCommentArgs", "JsmRequestCreateArgs", "JsmRequestListArgs", "JsmRequestMoveArgs", "JsmRequestParticipantsArgs", "JsmRequestSlaArgs", "JsmRequestTransitionsArgs", "JsmRequestTypesArgs", "JsmRequestViewArgs", "LinkType", "Linked", "Membership", "Moved", "Org", "Page", "PageAttachment", "PageComment", "PageSummary", "PageVersion", "PageView", "Participants", "Pipeline", "PipelineDetail", "PipelineLog", "PipelineStep", "PrApproval", "PrChecks", "PrComment", "PrCommented", "PrDiff", "Project", "PullRequest", "Pushed", "Queue", "Release", "RemoteLink", "Repo", "Request", "RequestAttachment", "RequestComment", "RequestDetail", "RequestMoved", "RequestTransition", "RequestType", "SearchHit", "SearchResults", "Sla", "Space", "Sprint", "SprintIssues", "Suggestion", "TaskName", "Transition", "TreeNode", "TrelloApi", "TrelloBoard", "TrelloBoardApi", "TrelloBoardListArgs", "TrelloCardApi", "TrelloCardArchiveArgs", "TrelloCardCommentArgs", "TrelloCardCreateArgs", "TrelloCardEditArgs", "TrelloCardListArgs", "TrelloCardMoveArgs", "TrelloCardViewArgs", "TrelloChecklistApi", "TrelloChecklistCheckArgs", "TrelloChecklistCreateArgs", "TrelloChecklistListArgs", "TrelloList", "TrelloListApi", "TrelloListListArgs", "Unlinked", "User", "Watchers", "Watching", "Worklog", "WorklogAdded"]

# Methods and fields can be named `list`, so annotations use this alias.
_list = list


class ErrorCode(StrEnum):
    """Every value of ``AtlError.code``."""

    AUTH = "auth"
    API = "api"
    INVALID = "invalid"
    UNKNOWN = "unknown"
    READ_ONLY = "read_only"
    DRY_RUN = "dry_run"
    INPUT_REQUIRED = "input_required"
    INTERNAL = "internal"


class TaskName(StrEnum):
    """Every task name."""

    JIRA_ISSUE_LIST = "jira_issue_list"
    JIRA_ISSUE_COUNT = "jira_issue_count"
    JIRA_ISSUE_VIEW = "jira_issue_view"
    JIRA_ISSUE_DELETE = "jira_issue_delete"
    JIRA_ISSUE_WATCHERS = "jira_issue_watchers"
    JIRA_ISSUE_WATCH = "jira_issue_watch"
    JIRA_ISSUE_UNWATCH = "jira_issue_unwatch"
    JIRA_FIELD_LIST = "jira_field_list"
    JIRA_ISSUE_CREATE = "jira_issue_create"
    JIRA_ISSUE_EDIT = "jira_issue_edit"
    JIRA_ISSUE_ASSIGN = "jira_issue_assign"
    JIRA_ISSUE_TRANSITIONS = "jira_issue_transitions"
    JIRA_ISSUE_MOVE = "jira_issue_move"
    JIRA_ISSUE_COMMENT = "jira_issue_comment"
    JIRA_ISSUE_LINK = "jira_issue_link"
    JIRA_ISSUE_UNLINK = "jira_issue_unlink"
    JIRA_ISSUE_REMOTE_LINK = "jira_issue_remote_link"
    JIRA_COMMENT_LIST = "jira_comment_list"
    JIRA_COMMENT_EDIT = "jira_comment_edit"
    JIRA_COMMENT_DELETE = "jira_comment_delete"
    JIRA_WORKLOG_LIST = "jira_worklog_list"
    JIRA_ISSUE_HISTORY = "jira_issue_history"
    JIRA_COMPONENT_LIST = "jira_component_list"
    JIRA_SPRINT_CREATE = "jira_sprint_create"
    JIRA_SPRINT_START = "jira_sprint_start"
    JIRA_SPRINT_CLOSE = "jira_sprint_close"
    JIRA_RELEASE_LIST = "jira_release_list"
    JIRA_ISSUE_LINK_TYPES = "jira_issue_link_types"
    JIRA_ISSUE_CLONE = "jira_issue_clone"
    JIRA_ISSUE_WORKLOG = "jira_issue_worklog"
    JIRA_ISSUE_ATTACH = "jira_issue_attach"
    JIRA_ISSUE_ATTACHMENTS = "jira_issue_attachments"
    JIRA_JQL_CHECK = "jira_jql_check"
    JIRA_JQL_FIELDS = "jira_jql_fields"
    JIRA_JQL_SUGGEST = "jira_jql_suggest"
    JIRA_FILTER_LIST = "jira_filter_list"
    JIRA_FILTER_RUN = "jira_filter_run"
    JIRA_EPIC_LIST = "jira_epic_list"
    JIRA_EPIC_ADD = "jira_epic_add"
    JIRA_EPIC_REMOVE = "jira_epic_remove"
    JIRA_BOARD_LIST = "jira_board_list"
    JIRA_SPRINT_LIST = "jira_sprint_list"
    JIRA_SPRINT_ADD = "jira_sprint_add"
    JIRA_PROJECT_LIST = "jira_project_list"
    JIRA_ISSUE_TYPES = "jira_issue_types"
    JIRA_ME = "jira_me"
    CONFLUENCE_PAGE_GET = "confluence_page_get"
    CONFLUENCE_PAGE_CREATE = "confluence_page_create"
    CONFLUENCE_BLOG_LIST = "confluence_blog_list"
    CONFLUENCE_BLOG_GET = "confluence_blog_get"
    CONFLUENCE_BLOG_CREATE = "confluence_blog_create"
    CONFLUENCE_SPACE_CREATE = "confluence_space_create"
    CONFLUENCE_PAGE_UPDATE = "confluence_page_update"
    CONFLUENCE_PAGE_DELETE = "confluence_page_delete"
    CONFLUENCE_PAGE_CHILDREN = "confluence_page_children"
    CONFLUENCE_PAGE_FIND = "confluence_page_find"
    CONFLUENCE_PAGE_ATTACH = "confluence_page_attach"
    CONFLUENCE_PAGE_ATTACHMENTS = "confluence_page_attachments"
    CONFLUENCE_PAGE_COMMENTS = "confluence_page_comments"
    CONFLUENCE_PAGE_COMMENT = "confluence_page_comment"
    CONFLUENCE_COMMENT_DELETE = "confluence_comment_delete"
    CONFLUENCE_ATTACHMENT_DELETE = "confluence_attachment_delete"
    CONFLUENCE_PAGE_EXPORT = "confluence_page_export"
    CONFLUENCE_PAGE_PUSH = "confluence_page_push"
    CONFLUENCE_PAGE_COPY_TREE = "confluence_page_copy_tree"
    CONFLUENCE_PAGE_LABELS = "confluence_page_labels"
    CONFLUENCE_PAGE_LABEL = "confluence_page_label"
    CONFLUENCE_PAGE_MOVE = "confluence_page_move"
    CONFLUENCE_PAGE_HISTORY = "confluence_page_history"
    CONFLUENCE_SPACE_LIST = "confluence_space_list"
    CONFLUENCE_SPACE_PAGES = "confluence_space_pages"
    CONFLUENCE_SEARCH = "confluence_search"
    BITBUCKET_REPO_LIST = "bitbucket_repo_list"
    BITBUCKET_REPO_VIEW = "bitbucket_repo_view"
    BITBUCKET_BRANCH_LIST = "bitbucket_branch_list"
    BITBUCKET_PR_LIST = "bitbucket_pr_list"
    BITBUCKET_PR_VIEW = "bitbucket_pr_view"
    BITBUCKET_PR_CREATE = "bitbucket_pr_create"
    BITBUCKET_PR_MERGE = "bitbucket_pr_merge"
    BITBUCKET_PR_APPROVE = "bitbucket_pr_approve"
    BITBUCKET_PR_COMMENT = "bitbucket_pr_comment"
    BITBUCKET_PR_DIFF = "bitbucket_pr_diff"
    BITBUCKET_PR_COMMENTS = "bitbucket_pr_comments"
    BITBUCKET_PR_CHECKS = "bitbucket_pr_checks"
    BITBUCKET_PR_UNAPPROVE = "bitbucket_pr_unapprove"
    BITBUCKET_PR_READY = "bitbucket_pr_ready"
    BITBUCKET_BRANCH_CREATE = "bitbucket_branch_create"
    BITBUCKET_BRANCH_DELETE = "bitbucket_branch_delete"
    BITBUCKET_COMMIT_LIST = "bitbucket_commit_list"
    BITBUCKET_PIPELINE_LIST = "bitbucket_pipeline_list"
    BITBUCKET_PIPELINE_RUN = "bitbucket_pipeline_run"
    BITBUCKET_PR_DECLINE = "bitbucket_pr_decline"
    BITBUCKET_PR_REQUEST_CHANGES = "bitbucket_pr_request_changes"
    BITBUCKET_PR_EDIT = "bitbucket_pr_edit"
    BITBUCKET_PR_REVIEWERS = "bitbucket_pr_reviewers"
    BITBUCKET_PIPELINE_STOP = "bitbucket_pipeline_stop"
    BITBUCKET_PIPELINE_VIEW = "bitbucket_pipeline_view"
    BITBUCKET_PIPELINE_LOGS = "bitbucket_pipeline_logs"
    JSM_DESK_LIST = "jsm_desk_list"
    JSM_REQUEST_TYPES = "jsm_request_types"
    JSM_QUEUE_LIST = "jsm_queue_list"
    JSM_QUEUE_ISSUES = "jsm_queue_issues"
    JSM_REQUEST_LIST = "jsm_request_list"
    JSM_REQUEST_VIEW = "jsm_request_view"
    JSM_REQUEST_CREATE = "jsm_request_create"
    JSM_REQUEST_COMMENT = "jsm_request_comment"
    JSM_REQUEST_TRANSITIONS = "jsm_request_transitions"
    JSM_REQUEST_MOVE = "jsm_request_move"
    JSM_REQUEST_APPROVALS = "jsm_request_approvals"
    JSM_REQUEST_APPROVE = "jsm_request_approve"
    JSM_REQUEST_PARTICIPANTS = "jsm_request_participants"
    JSM_REQUEST_SLA = "jsm_request_sla"
    JSM_REQUEST_ATTACH = "jsm_request_attach"
    JSM_REQUEST_ATTACHMENTS = "jsm_request_attachments"
    TRELLO_BOARD_LIST = "trello_board_list"
    TRELLO_LIST_LIST = "trello_list_list"
    TRELLO_CARD_LIST = "trello_card_list"
    TRELLO_CARD_VIEW = "trello_card_view"
    TRELLO_CARD_CREATE = "trello_card_create"
    TRELLO_CARD_MOVE = "trello_card_move"
    TRELLO_CARD_COMMENT = "trello_card_comment"
    TRELLO_CARD_ARCHIVE = "trello_card_archive"
    TRELLO_CARD_EDIT = "trello_card_edit"
    TRELLO_CHECKLIST_LIST = "trello_checklist_list"
    TRELLO_CHECKLIST_CREATE = "trello_checklist_create"
    TRELLO_CHECKLIST_CHECK = "trello_checklist_check"
    ADMIN_ORG_LIST = "admin_org_list"
    ADMIN_DIRECTORY_LIST = "admin_directory_list"
    ADMIN_USER_LIST = "admin_user_list"
    ADMIN_USER_SUSPEND = "admin_user_suspend"
    ADMIN_USER_RESTORE = "admin_user_restore"
    ADMIN_GROUP_ADD_MEMBER = "admin_group_add_member"
    ADMIN_GROUP_REMOVE_MEMBER = "admin_group_remove_member"
    ADMIN_GROUP_LIST = "admin_group_list"
    ADMIN_DOMAIN_LIST = "admin_domain_list"
    ADMIN_EVENT_LIST = "admin_event_list"


JiraIssueListArgs = TypedDict(
    "JiraIssueListArgs",
    {
        "project": NotRequired[str | None],
        "status": NotRequired[list[str]],
        "assignee": NotRequired[str | None],
        "reporter": NotRequired[str | None],
        "type": NotRequired[list[str]],
        "labels": NotRequired[list[str]],
        "priority": NotRequired[list[str]],
        "component": NotRequired[list[str]],
        "resolution": NotRequired[list[str]],
        "parent": NotRequired[str | None],
        "watching": NotRequired[bool],
        "history": NotRequired[bool],
        "sprint": NotRequired[str | None],
        "created": NotRequired[str | None],
        "created_before": NotRequired[str | None],
        "updated": NotRequired[str | None],
        "updated_before": NotRequired[str | None],
        "text": NotRequired[str | None],
        "jql": NotRequired[str | None],
        "order_by": NotRequired[str | None],
        "reverse": NotRequired[bool],
        "limit": NotRequired[int],
        "page_token": NotRequired[str | None],
    },
)
JiraIssueListArgs.__doc__ = """Arguments of ``jira_issue_list``: Search Jira issues with filters (or extra JQL). Returns small records: key, summary, status, type, assignee, priority, updated, url. With no filters: your unresolved issues."""


JiraIssueCountArgs = TypedDict(
    "JiraIssueCountArgs",
    {
        "project": NotRequired[str | None],
        "status": NotRequired[list[str]],
        "assignee": NotRequired[str | None],
        "reporter": NotRequired[str | None],
        "type": NotRequired[list[str]],
        "labels": NotRequired[list[str]],
        "priority": NotRequired[list[str]],
        "component": NotRequired[list[str]],
        "resolution": NotRequired[list[str]],
        "parent": NotRequired[str | None],
        "watching": NotRequired[bool],
        "history": NotRequired[bool],
        "sprint": NotRequired[str | None],
        "created": NotRequired[str | None],
        "created_before": NotRequired[str | None],
        "updated": NotRequired[str | None],
        "updated_before": NotRequired[str | None],
        "text": NotRequired[str | None],
        "jql": NotRequired[str | None],
        "order_by": NotRequired[str | None],
        "reverse": NotRequired[bool],
        "limit": NotRequired[int],
        "page_token": NotRequired[str | None],
    },
)
JiraIssueCountArgs.__doc__ = """Arguments of ``jira_issue_count``: Count Jira issues matching the same filters as issue_list."""


JiraIssueViewArgs = TypedDict(
    "JiraIssueViewArgs",
    {
        "key": Required[str],
        "comments": NotRequired[int],
        "fields": NotRequired[list[str]],
    },
)
JiraIssueViewArgs.__doc__ = """Arguments of ``jira_issue_view``: One Jira issue with description and latest comments as Markdown, plus any `fields` asked for by name."""


JiraIssueDeleteArgs = TypedDict(
    "JiraIssueDeleteArgs",
    {
        "key": Required[str],
        "subtasks": NotRequired[bool],
    },
)
JiraIssueDeleteArgs.__doc__ = """Arguments of ``jira_issue_delete``: Delete a Jira issue (with `subtasks: true`, also its sub-tasks). Cannot be undone."""


JiraIssueWatchersArgs = TypedDict(
    "JiraIssueWatchersArgs",
    {
        "key": Required[str],
    },
)
JiraIssueWatchersArgs.__doc__ = """Arguments of ``jira_issue_watchers``: Who watches a Jira issue, and whether you do."""


JiraIssueWatchArgs = TypedDict(
    "JiraIssueWatchArgs",
    {
        "key": Required[str],
        "user": NotRequired[str],
    },
)
JiraIssueWatchArgs.__doc__ = """Arguments of ``jira_issue_watch``: Add a watcher to a Jira issue (default: you)."""


JiraIssueUnwatchArgs = TypedDict(
    "JiraIssueUnwatchArgs",
    {
        "key": Required[str],
        "user": NotRequired[str],
    },
)
JiraIssueUnwatchArgs.__doc__ = """Arguments of ``jira_issue_unwatch``: Remove a watcher from a Jira issue (default: you)."""


JiraFieldListArgs = TypedDict(
    "JiraFieldListArgs",
    {
        "filter": NotRequired[str | None],
    },
)
JiraFieldListArgs.__doc__ = """Arguments of ``jira_field_list``: Jira system and custom fields (ID, name, type), optionally filtered by part of the name. Use the names in the `fields` argument of issue create, edit and view."""


JiraIssueCreateArgs = TypedDict(
    "JiraIssueCreateArgs",
    {
        "project": Required[str],
        "type": NotRequired[str],
        "summary": Required[str],
        "description": NotRequired[str | None],
        "assignee": NotRequired[str | None],
        "labels": NotRequired[list[str]],
        "priority": NotRequired[str | None],
        "parent": NotRequired[str | None],
        "components": NotRequired[list[str]],
        "fix_versions": NotRequired[list[str]],
        "affects_versions": NotRequired[list[str]],
        "estimate": NotRequired[str | None],
        "reporter": NotRequired[str | None],
        "fields": NotRequired[dict[str, Any]],
    },
)
JiraIssueCreateArgs.__doc__ = """Arguments of ``jira_issue_create``: Create a Jira issue; the description is Markdown."""


JiraIssueEditArgs = TypedDict(
    "JiraIssueEditArgs",
    {
        "key": Required[str],
        "summary": NotRequired[str | None],
        "description": NotRequired[str | None],
        "priority": NotRequired[str | None],
        "assignee": NotRequired[str | None],
        "parent": NotRequired[str | None],
        "reporter": NotRequired[str | None],
        "estimate": NotRequired[str | None],
        "add_labels": NotRequired[list[str]],
        "remove_labels": NotRequired[list[str]],
        "add_components": NotRequired[list[str]],
        "remove_components": NotRequired[list[str]],
        "add_fix_versions": NotRequired[list[str]],
        "remove_fix_versions": NotRequired[list[str]],
        "add_affects_versions": NotRequired[list[str]],
        "remove_affects_versions": NotRequired[list[str]],
        "fields": NotRequired[dict[str, Any]],
        "skip_notify": NotRequired[bool],
    },
)
JiraIssueEditArgs.__doc__ = """Arguments of ``jira_issue_edit``: Change a Jira issue's summary, description (Markdown), priority or labels."""


JiraIssueAssignArgs = TypedDict(
    "JiraIssueAssignArgs",
    {
        "key": Required[str],
        "user": Required[str],
    },
)
JiraIssueAssignArgs.__doc__ = """Arguments of ``jira_issue_assign``: Assign a Jira issue to me, nobody (x), or a user."""


JiraIssueTransitionsArgs = TypedDict(
    "JiraIssueTransitionsArgs",
    {
        "key": Required[str],
    },
)
JiraIssueTransitionsArgs.__doc__ = """Arguments of ``jira_issue_transitions``: Statuses a Jira issue can move to."""


JiraIssueMoveArgs = TypedDict(
    "JiraIssueMoveArgs",
    {
        "key": Required[str],
        "status": Required[str],
        "comment": NotRequired[str | None],
        "resolution": NotRequired[str | None],
        "assignee": NotRequired[str | None],
    },
)
JiraIssueMoveArgs.__doc__ = """Arguments of ``jira_issue_move``: Move a Jira issue to another status (by status or transition name)."""


JiraIssueCommentArgs = TypedDict(
    "JiraIssueCommentArgs",
    {
        "key": Required[str],
        "body": Required[str],
        "internal": NotRequired[bool],
    },
)
JiraIssueCommentArgs.__doc__ = """Arguments of ``jira_issue_comment``: Add a Markdown comment to a Jira issue."""


JiraIssueLinkArgs = TypedDict(
    "JiraIssueLinkArgs",
    {
        "from": Required[str],
        "to": Required[str],
        "kind": Required[str],
    },
)
JiraIssueLinkArgs.__doc__ = """Arguments of ``jira_issue_link``: Link two Jira issues so that 'from <kind> to' reads true, for example from A-1 kind blocks to A-2."""


JiraIssueUnlinkArgs = TypedDict(
    "JiraIssueUnlinkArgs",
    {
        "from": Required[str],
        "to": Required[str],
        "kind": NotRequired[str | None],
    },
)
JiraIssueUnlinkArgs.__doc__ = """Arguments of ``jira_issue_unlink``: Remove the links between two Jira issues (optionally only one link type)."""


JiraIssueRemoteLinkArgs = TypedDict(
    "JiraIssueRemoteLinkArgs",
    {
        "key": Required[str],
        "url": Required[str],
        "title": NotRequired[str | None],
    },
)
JiraIssueRemoteLinkArgs.__doc__ = """Arguments of ``jira_issue_remote_link``: Add a web link (URL and title) to a Jira issue."""


JiraCommentListArgs = TypedDict(
    "JiraCommentListArgs",
    {
        "key": Required[str],
        "limit": NotRequired[int],
    },
)
JiraCommentListArgs.__doc__ = """Arguments of ``jira_comment_list``: Every comment of a Jira issue, oldest first, with IDs, as Markdown."""


JiraCommentEditArgs = TypedDict(
    "JiraCommentEditArgs",
    {
        "key": Required[str],
        "id": Required[str],
        "body": Required[str],
    },
)
JiraCommentEditArgs.__doc__ = """Arguments of ``jira_comment_edit``: Replace the text of a Jira comment (Markdown)."""


JiraCommentDeleteArgs = TypedDict(
    "JiraCommentDeleteArgs",
    {
        "key": Required[str],
        "id": Required[str],
    },
)
JiraCommentDeleteArgs.__doc__ = """Arguments of ``jira_comment_delete``: Delete a Jira comment. Cannot be undone."""


JiraWorklogListArgs = TypedDict(
    "JiraWorklogListArgs",
    {
        "key": Required[str],
    },
)
JiraWorklogListArgs.__doc__ = """Arguments of ``jira_worklog_list``: Time logged on a Jira issue: author, time spent, start, comment."""


JiraIssueHistoryArgs = TypedDict(
    "JiraIssueHistoryArgs",
    {
        "key": Required[str],
        "limit": NotRequired[int],
    },
)
JiraIssueHistoryArgs.__doc__ = """Arguments of ``jira_issue_history``: Field changes of a Jira issue, newest first: when, who, field, from, to."""


JiraComponentListArgs = TypedDict(
    "JiraComponentListArgs",
    {
        "project": Required[str],
    },
)
JiraComponentListArgs.__doc__ = """Arguments of ``jira_component_list``: Components of a Jira project with their leads."""


JiraSprintCreateArgs = TypedDict(
    "JiraSprintCreateArgs",
    {
        "name": Required[str],
        "board": NotRequired[int | None],
        "project": NotRequired[str | None],
        "start": NotRequired[str | None],
        "end": NotRequired[str | None],
        "goal": NotRequired[str | None],
    },
)
JiraSprintCreateArgs.__doc__ = """Arguments of ``jira_sprint_create``: Create a future sprint on a board (default: the project's first board)."""


JiraSprintStartArgs = TypedDict(
    "JiraSprintStartArgs",
    {
        "sprint": Required[int],
        "start": NotRequired[str | None],
        "end": NotRequired[str | None],
    },
)
JiraSprintStartArgs.__doc__ = """Arguments of ``jira_sprint_start``: Start a future sprint; Jira needs a start and an end date."""


JiraSprintCloseArgs = TypedDict(
    "JiraSprintCloseArgs",
    {
        "sprint": Required[int],
    },
)
JiraSprintCloseArgs.__doc__ = """Arguments of ``jira_sprint_close``: Complete an active sprint."""


JiraReleaseListArgs = TypedDict(
    "JiraReleaseListArgs",
    {
        "project": Required[str],
        "state": NotRequired[str],
    },
)
JiraReleaseListArgs.__doc__ = """Arguments of ``jira_release_list``: Versions (releases) of a Jira project: name, released, archived, release date."""


JiraIssueLinkTypesArgs = TypedDict(
    "JiraIssueLinkTypesArgs",
    {
    },
)
JiraIssueLinkTypesArgs.__doc__ = """Arguments of ``jira_issue_link_types``: Jira issue link types with their outward and inward phrases."""


JiraIssueCloneArgs = TypedDict(
    "JiraIssueCloneArgs",
    {
        "key": Required[str],
        "summary": NotRequired[str | None],
        "priority": NotRequired[str | None],
        "assignee": NotRequired[str | None],
        "labels": NotRequired[list[str]],
        "replace": NotRequired[list[str]],
    },
)
JiraIssueCloneArgs.__doc__ = """Arguments of ``jira_issue_clone``: Copy a Jira issue (summary, description, type, labels, priority, components, parent)."""


JiraIssueWorklogArgs = TypedDict(
    "JiraIssueWorklogArgs",
    {
        "key": Required[str],
        "time": Required[str],
        "comment": NotRequired[str | None],
        "started": NotRequired[str | None],
    },
)
JiraIssueWorklogArgs.__doc__ = """Arguments of ``jira_issue_worklog``: Log time on a Jira issue."""


JiraIssueAttachArgs = TypedDict(
    "JiraIssueAttachArgs",
    {
        "key": Required[str],
        "files": Required[list[str]],
    },
)
JiraIssueAttachArgs.__doc__ = """Arguments of ``jira_issue_attach``: Upload local files to a Jira issue."""


JiraIssueAttachmentsArgs = TypedDict(
    "JiraIssueAttachmentsArgs",
    {
        "key": Required[str],
    },
)
JiraIssueAttachmentsArgs.__doc__ = """Arguments of ``jira_issue_attachments``: List a Jira issue's attachments."""


JiraJqlCheckArgs = TypedDict(
    "JiraJqlCheckArgs",
    {
        "jql": Required[str],
    },
)
JiraJqlCheckArgs.__doc__ = """Arguments of ``jira_jql_check``: Validate JQL with Jira's parser; returns valid, errors, warnings."""


JiraJqlFieldsArgs = TypedDict(
    "JiraJqlFieldsArgs",
    {
    },
)
JiraJqlFieldsArgs.__doc__ = """Arguments of ``jira_jql_fields``: Fields (with operators) and functions usable in JQL on this site."""


JiraJqlSuggestArgs = TypedDict(
    "JiraJqlSuggestArgs",
    {
        "field": Required[str],
        "prefix": NotRequired[str],
    },
)
JiraJqlSuggestArgs.__doc__ = """Arguments of ``jira_jql_suggest``: Value suggestions for a JQL field, for example field=status prefix=In."""


JiraFilterListArgs = TypedDict(
    "JiraFilterListArgs",
    {
        "name": NotRequired[str | None],
    },
)
JiraFilterListArgs.__doc__ = """Arguments of ``jira_filter_list``: Saved Jira filters (Cloud: all visible, Data Center: favourites)."""


JiraFilterRunArgs = TypedDict(
    "JiraFilterRunArgs",
    {
        "filter": Required[str],
        "limit": NotRequired[int],
        "page_token": NotRequired[str | None],
    },
)
JiraFilterRunArgs.__doc__ = """Arguments of ``jira_filter_run``: Run a saved Jira filter by ID or exact name; returns issue records."""


JiraEpicListArgs = TypedDict(
    "JiraEpicListArgs",
    {
        "project": NotRequired[str | None],
        "limit": NotRequired[int],
    },
)
JiraEpicListArgs.__doc__ = """Arguments of ``jira_epic_list``: Open epics, optionally of one project."""


JiraEpicAddArgs = TypedDict(
    "JiraEpicAddArgs",
    {
        "epic": Required[str],
        "issues": Required[list[str]],
    },
)
JiraEpicAddArgs.__doc__ = """Arguments of ``jira_epic_add``: Move issues into an epic."""


JiraEpicRemoveArgs = TypedDict(
    "JiraEpicRemoveArgs",
    {
        "issues": Required[list[str]],
    },
)
JiraEpicRemoveArgs.__doc__ = """Arguments of ``jira_epic_remove``: Remove issues from their epic."""


JiraBoardListArgs = TypedDict(
    "JiraBoardListArgs",
    {
        "project": NotRequired[str | None],
    },
)
JiraBoardListArgs.__doc__ = """Arguments of ``jira_board_list``: Jira Software boards, optionally of one project."""


JiraSprintListArgs = TypedDict(
    "JiraSprintListArgs",
    {
        "board": NotRequired[int | None],
        "project": NotRequired[str | None],
        "state": NotRequired[str],
    },
)
JiraSprintListArgs.__doc__ = """Arguments of ``jira_sprint_list``: Sprints of a board."""


JiraSprintAddArgs = TypedDict(
    "JiraSprintAddArgs",
    {
        "sprint": Required[int],
        "issues": Required[list[str]],
    },
)
JiraSprintAddArgs.__doc__ = """Arguments of ``jira_sprint_add``: Move issues into a sprint."""


JiraProjectListArgs = TypedDict(
    "JiraProjectListArgs",
    {
    },
)
JiraProjectListArgs.__doc__ = """Arguments of ``jira_project_list``: Jira projects."""


JiraIssueTypesArgs = TypedDict(
    "JiraIssueTypesArgs",
    {
        "project": Required[str],
    },
)
JiraIssueTypesArgs.__doc__ = """Arguments of ``jira_issue_types``: Issue type names of a Jira project (without sub-task types), for the `type` argument of jira_issue_create."""


JiraMeArgs = TypedDict(
    "JiraMeArgs",
    {
    },
)
JiraMeArgs.__doc__ = """Arguments of ``jira_me``: The signed-in Jira user."""


ConfluencePageGetArgs = TypedDict(
    "ConfluencePageGetArgs",
    {
        "page": Required[str],
        "front_matter": NotRequired[bool],
        "storage": NotRequired[bool],
        "version": NotRequired[int | None],
    },
)
ConfluencePageGetArgs.__doc__ = """Arguments of ``confluence_page_get``: A Confluence page as Markdown (with title, space, version, url)."""


ConfluencePageCreateArgs = TypedDict(
    "ConfluencePageCreateArgs",
    {
        "space": Required[str],
        "title": Required[str],
        "parent": NotRequired[str | None],
        "markdown": NotRequired[str | None],
        "storage": NotRequired[str | None],
    },
)
ConfluencePageCreateArgs.__doc__ = """Arguments of ``confluence_page_create``: Create a Confluence page from Markdown (or storage format)."""


ConfluenceBlogListArgs = TypedDict(
    "ConfluenceBlogListArgs",
    {
        "space": Required[str],
        "limit": NotRequired[int],
    },
)
ConfluenceBlogListArgs.__doc__ = """Arguments of ``confluence_blog_list``: Blog posts of a Confluence space, newest first."""


ConfluenceBlogGetArgs = TypedDict(
    "ConfluenceBlogGetArgs",
    {
        "id": Required[str],
    },
)
ConfluenceBlogGetArgs.__doc__ = """Arguments of ``confluence_blog_get``: One Confluence blog post with its body as Markdown."""


ConfluenceBlogCreateArgs = TypedDict(
    "ConfluenceBlogCreateArgs",
    {
        "space": Required[str],
        "title": Required[str],
        "markdown": NotRequired[str | None],
        "storage": NotRequired[str | None],
    },
)
ConfluenceBlogCreateArgs.__doc__ = """Arguments of ``confluence_blog_create``: Publish a Confluence blog post from Markdown."""


ConfluenceSpaceCreateArgs = TypedDict(
    "ConfluenceSpaceCreateArgs",
    {
        "key": Required[str],
        "name": Required[str],
        "description": NotRequired[str | None],
    },
)
ConfluenceSpaceCreateArgs.__doc__ = """Arguments of ``confluence_space_create``: Create a Confluence space with a key, a name and an optional plain-text description."""


ConfluencePageUpdateArgs = TypedDict(
    "ConfluencePageUpdateArgs",
    {
        "page": Required[str],
        "title": NotRequired[str | None],
        "markdown": NotRequired[str | None],
        "storage": NotRequired[str | None],
    },
)
ConfluencePageUpdateArgs.__doc__ = """Arguments of ``confluence_page_update``: Replace a Confluence page's title and/or body; the version is bumped automatically."""


ConfluencePageDeleteArgs = TypedDict(
    "ConfluencePageDeleteArgs",
    {
        "page": Required[str],
    },
)
ConfluencePageDeleteArgs.__doc__ = """Arguments of ``confluence_page_delete``: Delete a Confluence page."""


ConfluencePageChildrenArgs = TypedDict(
    "ConfluencePageChildrenArgs",
    {
        "page": Required[str],
        "recursive": NotRequired[bool],
        "max_depth": NotRequired[int | None],
    },
)
ConfluencePageChildrenArgs.__doc__ = """Arguments of ``confluence_page_children``: Children of a Confluence page (Cloud: also folders), or with `recursive` the whole tree, depth first, each row with type, depth and parent."""


ConfluencePageFindArgs = TypedDict(
    "ConfluencePageFindArgs",
    {
        "title": Required[str],
        "space": NotRequired[str | None],
        "limit": NotRequired[int],
    },
)
ConfluencePageFindArgs.__doc__ = """Arguments of ``confluence_page_find``: Confluence pages whose title contains the words, optionally in one space."""


ConfluencePageAttachArgs = TypedDict(
    "ConfluencePageAttachArgs",
    {
        "replace": NotRequired[bool],
        "page": Required[str],
        "files": Required[list[str]],
    },
)
ConfluencePageAttachArgs.__doc__ = """Arguments of ``confluence_page_attach``: Upload local files to a Confluence page."""


ConfluencePageAttachmentsArgs = TypedDict(
    "ConfluencePageAttachmentsArgs",
    {
        "page": Required[str],
        "pattern": NotRequired[str | None],
    },
)
ConfluencePageAttachmentsArgs.__doc__ = """Arguments of ``confluence_page_attachments``: Attachments of a Confluence page with download URLs."""


ConfluencePageCommentsArgs = TypedDict(
    "ConfluencePageCommentsArgs",
    {
        "page": Required[str],
        "limit": NotRequired[int],
        "inline": NotRequired[bool],
    },
)
ConfluencePageCommentsArgs.__doc__ = """Arguments of ``confluence_page_comments``: Footer (or with `inline`, inline) comments of a Confluence page, oldest first, as Markdown, with the replied-to `parent`."""


ConfluencePageCommentArgs = TypedDict(
    "ConfluencePageCommentArgs",
    {
        "page": Required[str],
        "body": Required[str],
        "parent": NotRequired[str | None],
    },
)
ConfluencePageCommentArgs.__doc__ = """Arguments of ``confluence_page_comment``: Add a footer comment (Markdown) to a Confluence page, or a reply with `parent`."""


ConfluenceCommentDeleteArgs = TypedDict(
    "ConfluenceCommentDeleteArgs",
    {
        "id": Required[str],
    },
)
ConfluenceCommentDeleteArgs.__doc__ = """Arguments of ``confluence_comment_delete``: Delete a Confluence comment by ID."""


ConfluenceAttachmentDeleteArgs = TypedDict(
    "ConfluenceAttachmentDeleteArgs",
    {
        "id": Required[str],
    },
)
ConfluenceAttachmentDeleteArgs.__doc__ = """Arguments of ``confluence_attachment_delete``: Delete a Confluence attachment by ID."""


ConfluencePageExportArgs = TypedDict(
    "ConfluencePageExportArgs",
    {
        "page": Required[str],
        "dest": Required[str],
        "recursive": NotRequired[bool],
        "max_depth": NotRequired[int | None],
        "exclude": NotRequired[list[str]],
        "attachments": NotRequired[bool],
        "dry_run": NotRequired[bool],
    },
)
ConfluencePageExportArgs.__doc__ = """Arguments of ``confluence_page_export``: Export a Confluence page (and with `recursive` its tree) to local Markdown folders: <title>/index.md with front matter and _attachments/. Changes nothing on the server."""


ConfluencePagePushArgs = TypedDict(
    "ConfluencePagePushArgs",
    {
        "paths": Required[list[str]],
        "space": NotRequired[str | None],
        "parent": NotRequired[str | None],
        "force": NotRequired[bool],
        "dry_run": NotRequired[bool],
    },
)
ConfluencePagePushArgs.__doc__ = """Arguments of ``confluence_page_push``: Publish local Markdown files: update the page in each file's front matter (refusing pages changed on the server unless `force`, skipping unchanged ones), create pages for new files under their folder's index.md, upload referenced images, and write ids and versions back."""


ConfluencePageCopyTreeArgs = TypedDict(
    "ConfluencePageCopyTreeArgs",
    {
        "page": Required[str],
        "target": Required[str],
        "suffix": NotRequired[str | None],
        "max_depth": NotRequired[int | None],
        "exclude": NotRequired[list[str]],
        "dry_run": NotRequired[bool],
    },
)
ConfluencePageCopyTreeArgs.__doc__ = """Arguments of ``confluence_page_copy_tree``: Copy a Confluence page and the pages below it under another page (titles, bodies, labels)."""


ConfluencePageLabelsArgs = TypedDict(
    "ConfluencePageLabelsArgs",
    {
        "page": Required[str],
    },
)
ConfluencePageLabelsArgs.__doc__ = """Arguments of ``confluence_page_labels``: Labels of a Confluence page."""


ConfluencePageLabelArgs = TypedDict(
    "ConfluencePageLabelArgs",
    {
        "page": Required[str],
        "add": NotRequired[list[str]],
        "remove": NotRequired[list[str]],
    },
)
ConfluencePageLabelArgs.__doc__ = """Arguments of ``confluence_page_label``: Add and remove labels on a Confluence page; returns the resulting labels."""


ConfluencePageMoveArgs = TypedDict(
    "ConfluencePageMoveArgs",
    {
        "page": Required[str],
        "target": Required[str],
        "position": NotRequired[str],
    },
)
ConfluencePageMoveArgs.__doc__ = """Arguments of ``confluence_page_move``: Move a Confluence page under another page (position append) or next to it (before/after, Cloud only)."""


ConfluencePageHistoryArgs = TypedDict(
    "ConfluencePageHistoryArgs",
    {
        "page": Required[str],
        "limit": NotRequired[int],
    },
)
ConfluencePageHistoryArgs.__doc__ = """Arguments of ``confluence_page_history``: Versions of a Confluence page, newest first: number, author, date, message. Read one with confluence_page_get and `version`."""


ConfluenceSpaceListArgs = TypedDict(
    "ConfluenceSpaceListArgs",
    {
        "limit": NotRequired[int],
    },
)
ConfluenceSpaceListArgs.__doc__ = """Arguments of ``confluence_space_list``: Confluence spaces."""


ConfluenceSpacePagesArgs = TypedDict(
    "ConfluenceSpacePagesArgs",
    {
        "space": Required[str],
        "limit": NotRequired[int],
    },
)
ConfluenceSpacePagesArgs.__doc__ = """Arguments of ``confluence_space_pages``: Top-level pages of a Confluence space."""


ConfluenceSearchArgs = TypedDict(
    "ConfluenceSearchArgs",
    {
        "query": NotRequired[str | None],
        "cql": NotRequired[str | None],
        "space": NotRequired[str | None],
        "type": NotRequired[str | None],
        "limit": NotRequired[int],
    },
)
ConfluenceSearchArgs.__doc__ = """Arguments of ``confluence_search``: Search Confluence by words or CQL; returns id, type, title, space, excerpt, url."""


BitbucketRepoListArgs = TypedDict(
    "BitbucketRepoListArgs",
    {
        "owner": NotRequired[str | None],
        "limit": NotRequired[int],
    },
)
BitbucketRepoListArgs.__doc__ = """Arguments of ``bitbucket_repo_list``: Bitbucket repositories of a workspace (Cloud) or project (Data Center), or the ones you belong to."""


BitbucketRepoViewArgs = TypedDict(
    "BitbucketRepoViewArgs",
    {
        "repo": Required[str],
    },
)
BitbucketRepoViewArgs.__doc__ = """Arguments of ``bitbucket_repo_view``: One Bitbucket repository with its default branch and URL."""


BitbucketBranchListArgs = TypedDict(
    "BitbucketBranchListArgs",
    {
        "repo": Required[str],
        "filter": NotRequired[str | None],
        "limit": NotRequired[int],
    },
)
BitbucketBranchListArgs.__doc__ = """Arguments of ``bitbucket_branch_list``: Branches of a Bitbucket repository, most recently changed first."""


BitbucketPrListArgs = TypedDict(
    "BitbucketPrListArgs",
    {
        "repo": Required[str],
        "state": NotRequired[str],
        "limit": NotRequired[int],
    },
)
BitbucketPrListArgs.__doc__ = """Arguments of ``bitbucket_pr_list``: Pull requests of a Bitbucket repository by state."""


BitbucketPrViewArgs = TypedDict(
    "BitbucketPrViewArgs",
    {
        "repo": Required[str],
        "id": Required[int],
    },
)
BitbucketPrViewArgs.__doc__ = """Arguments of ``bitbucket_pr_view``: One pull request with description, reviewers and approvals."""


BitbucketPrCreateArgs = TypedDict(
    "BitbucketPrCreateArgs",
    {
        "repo": Required[str],
        "title": Required[str],
        "source": Required[str],
        "destination": NotRequired[str | None],
        "description": NotRequired[str | None],
        "draft": NotRequired[bool],
    },
)
BitbucketPrCreateArgs.__doc__ = """Arguments of ``bitbucket_pr_create``: Open a pull request from a source branch."""


BitbucketPrMergeArgs = TypedDict(
    "BitbucketPrMergeArgs",
    {
        "repo": Required[str],
        "id": Required[int],
        "strategy": NotRequired[str | None],
    },
)
BitbucketPrMergeArgs.__doc__ = """Arguments of ``bitbucket_pr_merge``: Merge a pull request."""


BitbucketPrApproveArgs = TypedDict(
    "BitbucketPrApproveArgs",
    {
        "repo": Required[str],
        "id": Required[int],
    },
)
BitbucketPrApproveArgs.__doc__ = """Arguments of ``bitbucket_pr_approve``: Approve a pull request as the signed-in user."""


BitbucketPrCommentArgs = TypedDict(
    "BitbucketPrCommentArgs",
    {
        "repo": Required[str],
        "id": Required[int],
        "body": Required[str],
    },
)
BitbucketPrCommentArgs.__doc__ = """Arguments of ``bitbucket_pr_comment``: Add a Markdown comment to a pull request."""


BitbucketPrDiffArgs = TypedDict(
    "BitbucketPrDiffArgs",
    {
        "repo": Required[str],
        "id": Required[int],
    },
)
BitbucketPrDiffArgs.__doc__ = """Arguments of ``bitbucket_pr_diff``: The unified diff of a pull request."""


BitbucketPrCommentsArgs = TypedDict(
    "BitbucketPrCommentsArgs",
    {
        "repo": Required[str],
        "id": Required[int],
        "limit": NotRequired[int],
    },
)
BitbucketPrCommentsArgs.__doc__ = """Arguments of ``bitbucket_pr_comments``: Comments of a pull request, oldest first, as Markdown: general and inline (path, line), replies flattened with `parent`."""


BitbucketPrChecksArgs = TypedDict(
    "BitbucketPrChecksArgs",
    {
        "repo": Required[str],
        "id": Required[int],
    },
)
BitbucketPrChecksArgs.__doc__ = """Arguments of ``bitbucket_pr_checks``: Build statuses of a pull request's latest commit, with `passed` (none failed) and `pending` (some still running)."""


BitbucketPrUnapproveArgs = TypedDict(
    "BitbucketPrUnapproveArgs",
    {
        "repo": Required[str],
        "id": Required[int],
    },
)
BitbucketPrUnapproveArgs.__doc__ = """Arguments of ``bitbucket_pr_unapprove``: Remove your approval from a pull request."""


BitbucketPrReadyArgs = TypedDict(
    "BitbucketPrReadyArgs",
    {
        "repo": Required[str],
        "id": Required[int],
        "draft": NotRequired[bool],
    },
)
BitbucketPrReadyArgs.__doc__ = """Arguments of ``bitbucket_pr_ready``: Mark a draft pull request as ready for review (or, with `draft: true`, as a draft again)."""


BitbucketBranchCreateArgs = TypedDict(
    "BitbucketBranchCreateArgs",
    {
        "repo": Required[str],
        "name": Required[str],
        "from": NotRequired[str | None],
    },
)
BitbucketBranchCreateArgs.__doc__ = """Arguments of ``bitbucket_branch_create``: Create a branch from another branch, a tag or a commit (default: the main branch)."""


BitbucketBranchDeleteArgs = TypedDict(
    "BitbucketBranchDeleteArgs",
    {
        "repo": Required[str],
        "name": Required[str],
    },
)
BitbucketBranchDeleteArgs.__doc__ = """Arguments of ``bitbucket_branch_delete``: Delete a branch. Unmerged commits on it can be lost."""


BitbucketCommitListArgs = TypedDict(
    "BitbucketCommitListArgs",
    {
        "repo": Required[str],
        "branch": NotRequired[str | None],
        "limit": NotRequired[int],
    },
)
BitbucketCommitListArgs.__doc__ = """Arguments of ``bitbucket_commit_list``: Recent commits of a branch, newest first: hash, author, date, first line of the message."""


BitbucketPipelineListArgs = TypedDict(
    "BitbucketPipelineListArgs",
    {
        "repo": Required[str],
        "limit": NotRequired[int],
    },
)
BitbucketPipelineListArgs.__doc__ = """Arguments of ``bitbucket_pipeline_list``: Latest Bitbucket Pipelines runs, newest first (Cloud only)."""


BitbucketPipelineRunArgs = TypedDict(
    "BitbucketPipelineRunArgs",
    {
        "repo": Required[str],
        "branch": Required[str],
        "custom": NotRequired[str | None],
    },
)
BitbucketPipelineRunArgs.__doc__ = """Arguments of ``bitbucket_pipeline_run``: Start the pipeline of a branch, or a custom pipeline (Cloud only)."""


BitbucketPrDeclineArgs = TypedDict(
    "BitbucketPrDeclineArgs",
    {
        "repo": Required[str],
        "id": Required[int],
    },
)
BitbucketPrDeclineArgs.__doc__ = """Arguments of ``bitbucket_pr_decline``: Decline a Bitbucket pull request (Cloud cannot reopen it)."""


BitbucketPrRequestChangesArgs = TypedDict(
    "BitbucketPrRequestChangesArgs",
    {
        "repo": Required[str],
        "id": Required[int],
    },
)
BitbucketPrRequestChangesArgs.__doc__ = """Arguments of ``bitbucket_pr_request_changes``: Mark a Bitbucket pull request as needing changes by you (Cloud: request changes; Data Center: needs work)."""


BitbucketPrEditArgs = TypedDict(
    "BitbucketPrEditArgs",
    {
        "repo": Required[str],
        "id": Required[int],
        "title": NotRequired[str | None],
        "description": NotRequired[str | None],
        "destination": NotRequired[str | None],
    },
)
BitbucketPrEditArgs.__doc__ = """Arguments of ``bitbucket_pr_edit``: Change a Bitbucket pull request's title, description (Markdown) or destination branch."""


BitbucketPrReviewersArgs = TypedDict(
    "BitbucketPrReviewersArgs",
    {
        "repo": Required[str],
        "id": Required[int],
        "add": NotRequired[list[str]],
        "remove": NotRequired[list[str]],
    },
)
BitbucketPrReviewersArgs.__doc__ = """Arguments of ``bitbucket_pr_reviewers``: Add and remove reviewers of a Bitbucket pull request; other reviewers are kept."""


BitbucketPipelineStopArgs = TypedDict(
    "BitbucketPipelineStopArgs",
    {
        "repo": Required[str],
        "id": Required[str],
    },
)
BitbucketPipelineStopArgs.__doc__ = """Arguments of ``bitbucket_pipeline_stop``: Stop a running Bitbucket pipeline by build number or UUID (Cloud only)."""


BitbucketPipelineViewArgs = TypedDict(
    "BitbucketPipelineViewArgs",
    {
        "repo": Required[str],
        "id": Required[str],
    },
)
BitbucketPipelineViewArgs.__doc__ = """Arguments of ``bitbucket_pipeline_view``: One Bitbucket pipeline by build number or UUID, with each step's state and result (Cloud only)."""


BitbucketPipelineLogsArgs = TypedDict(
    "BitbucketPipelineLogsArgs",
    {
        "repo": Required[str],
        "id": Required[str],
        "step": NotRequired[str | None],
    },
)
BitbucketPipelineLogsArgs.__doc__ = """Arguments of ``bitbucket_pipeline_logs``: Log text of one pipeline step (by name or UUID) or of every step (Cloud only)."""


JsmDeskListArgs = TypedDict(
    "JsmDeskListArgs",
    {
    },
)
JsmDeskListArgs.__doc__ = """Arguments of ``jsm_desk_list``: Jira Service Management service desks (id, project key, name)."""


JsmRequestTypesArgs = TypedDict(
    "JsmRequestTypesArgs",
    {
        "desk": Required[str],
    },
)
JsmRequestTypesArgs.__doc__ = """Arguments of ``jsm_request_types``: Request types of a service desk."""


JsmQueueListArgs = TypedDict(
    "JsmQueueListArgs",
    {
        "desk": Required[str],
    },
)
JsmQueueListArgs.__doc__ = """Arguments of ``jsm_queue_list``: Queues of a service desk with issue counts."""


JsmQueueIssuesArgs = TypedDict(
    "JsmQueueIssuesArgs",
    {
        "desk": Required[str],
        "queue": Required[str],
        "limit": NotRequired[int],
    },
)
JsmQueueIssuesArgs.__doc__ = """Arguments of ``jsm_queue_issues``: Issues in a service desk queue (by queue ID or name)."""


JsmRequestListArgs = TypedDict(
    "JsmRequestListArgs",
    {
        "desk": NotRequired[str | None],
        "status": NotRequired[str],
        "search": NotRequired[str | None],
        "mine": NotRequired[bool],
        "limit": NotRequired[int],
    },
)
JsmRequestListArgs.__doc__ = """Arguments of ``jsm_request_list``: Customer requests by status, optionally of one service desk."""


JsmRequestViewArgs = TypedDict(
    "JsmRequestViewArgs",
    {
        "key": Required[str],
        "comments": NotRequired[int],
    },
)
JsmRequestViewArgs.__doc__ = """Arguments of ``jsm_request_view``: One customer request with its description and latest comments."""


JsmRequestCreateArgs = TypedDict(
    "JsmRequestCreateArgs",
    {
        "desk": Required[str],
        "type": Required[str],
        "summary": Required[str],
        "description": NotRequired[str | None],
        "on_behalf_of": NotRequired[str | None],
    },
)
JsmRequestCreateArgs.__doc__ = """Arguments of ``jsm_request_create``: Raise a customer request, optionally for a customer."""


JsmRequestCommentArgs = TypedDict(
    "JsmRequestCommentArgs",
    {
        "key": Required[str],
        "body": Required[str],
        "internal": NotRequired[bool],
    },
)
JsmRequestCommentArgs.__doc__ = """Arguments of ``jsm_request_comment``: Reply to a customer request, or add an internal note."""


JsmRequestTransitionsArgs = TypedDict(
    "JsmRequestTransitionsArgs",
    {
        "key": Required[str],
    },
)
JsmRequestTransitionsArgs.__doc__ = """Arguments of ``jsm_request_transitions``: Transitions available on a customer request."""


JsmRequestMoveArgs = TypedDict(
    "JsmRequestMoveArgs",
    {
        "key": Required[str],
        "transition": Required[str],
        "comment": NotRequired[str | None],
    },
)
JsmRequestMoveArgs.__doc__ = """Arguments of ``jsm_request_move``: Run a customer request transition by name or ID."""


JsmRequestApprovalsArgs = TypedDict(
    "JsmRequestApprovalsArgs",
    {
        "key": Required[str],
    },
)
JsmRequestApprovalsArgs.__doc__ = """Arguments of ``jsm_request_approvals``: Approvals of a JSM request: name, decision (pending, approved, declined), approvers, and whether you can answer."""


JsmRequestApproveArgs = TypedDict(
    "JsmRequestApproveArgs",
    {
        "key": Required[str],
        "approval": NotRequired[str | None],
        "decision": NotRequired[str],
    },
)
JsmRequestApproveArgs.__doc__ = """Arguments of ``jsm_request_approve``: Approve or decline (decision: decline) a JSM request's approval; without an ID, the one pending approval you can answer."""


JsmRequestParticipantsArgs = TypedDict(
    "JsmRequestParticipantsArgs",
    {
        "key": Required[str],
        "add": NotRequired[list[str]],
        "remove": NotRequired[list[str]],
    },
)
JsmRequestParticipantsArgs.__doc__ = """Arguments of ``jsm_request_participants``: List a JSM request's participants, after adding and removing any (me, an email or a name)."""


JsmRequestSlaArgs = TypedDict(
    "JsmRequestSlaArgs",
    {
        "key": Required[str],
    },
)
JsmRequestSlaArgs.__doc__ = """Arguments of ``jsm_request_sla``: SLAs of a JSM request: state (ongoing, paused, met, breached), time remaining and goal."""


JsmRequestAttachArgs = TypedDict(
    "JsmRequestAttachArgs",
    {
        "key": Required[str],
        "files": Required[list[str]],
        "internal": NotRequired[bool],
    },
)
JsmRequestAttachArgs.__doc__ = """Arguments of ``jsm_request_attach``: Attach local files to a JSM request (public unless internal)."""


JsmRequestAttachmentsArgs = TypedDict(
    "JsmRequestAttachmentsArgs",
    {
        "key": Required[str],
    },
)
JsmRequestAttachmentsArgs.__doc__ = """Arguments of ``jsm_request_attachments``: Attachments of a JSM request with download URLs."""


TrelloBoardListArgs = TypedDict(
    "TrelloBoardListArgs",
    {
    },
)
TrelloBoardListArgs.__doc__ = """Arguments of ``trello_board_list``: Your open Trello boards."""


TrelloListListArgs = TypedDict(
    "TrelloListListArgs",
    {
        "board": Required[str],
    },
)
TrelloListListArgs.__doc__ = """Arguments of ``trello_list_list``: Open lists of a Trello board (board by name, URL, short link or ID)."""


TrelloCardListArgs = TypedDict(
    "TrelloCardListArgs",
    {
        "board": NotRequired[str | None],
        "list": NotRequired[str | None],
        "mine": NotRequired[bool],
    },
)
TrelloCardListArgs.__doc__ = """Arguments of ``trello_card_list``: Cards of a Trello board or one of its lists, or your cards on every board (`mine`)."""


TrelloCardViewArgs = TypedDict(
    "TrelloCardViewArgs",
    {
        "card": Required[str],
    },
)
TrelloCardViewArgs.__doc__ = """Arguments of ``trello_card_view``: One Trello card with list, board, description and latest comments."""


TrelloCardCreateArgs = TypedDict(
    "TrelloCardCreateArgs",
    {
        "board": NotRequired[str | None],
        "list": Required[str],
        "name": Required[str],
        "description": NotRequired[str | None],
        "due": NotRequired[str | None],
    },
)
TrelloCardCreateArgs.__doc__ = """Arguments of ``trello_card_create``: Create a Trello card in a list (by name with `board`, or by list ID)."""


TrelloCardMoveArgs = TypedDict(
    "TrelloCardMoveArgs",
    {
        "card": Required[str],
        "list": Required[str],
        "board": NotRequired[str | None],
    },
)
TrelloCardMoveArgs.__doc__ = """Arguments of ``trello_card_move``: Move a Trello card to another list (by default on the card's own board)."""


TrelloCardCommentArgs = TypedDict(
    "TrelloCardCommentArgs",
    {
        "card": Required[str],
        "text": Required[str],
    },
)
TrelloCardCommentArgs.__doc__ = """Arguments of ``trello_card_comment``: Add a comment to a Trello card."""


TrelloCardArchiveArgs = TypedDict(
    "TrelloCardArchiveArgs",
    {
        "card": Required[str],
    },
)
TrelloCardArchiveArgs.__doc__ = """Arguments of ``trello_card_archive``: Archive (close) a Trello card."""


TrelloCardEditArgs = TypedDict(
    "TrelloCardEditArgs",
    {
        "card": Required[str],
        "name": NotRequired[str | None],
        "description": NotRequired[str | None],
        "due": NotRequired[str | None],
        "done": NotRequired[bool | None],
        "add_labels": NotRequired[list[str]],
        "remove_labels": NotRequired[list[str]],
        "add_members": NotRequired[list[str]],
        "remove_members": NotRequired[list[str]],
    },
)
TrelloCardEditArgs.__doc__ = """Arguments of ``trello_card_edit``: Change a Trello card: name, description, due date, done, labels (by name or color) and members (username, name or me)."""


TrelloChecklistListArgs = TypedDict(
    "TrelloChecklistListArgs",
    {
        "card": Required[str],
    },
)
TrelloChecklistListArgs.__doc__ = """Arguments of ``trello_checklist_list``: Checklists of a Trello card with their items and whether each is done."""


TrelloChecklistCreateArgs = TypedDict(
    "TrelloChecklistCreateArgs",
    {
        "card": Required[str],
        "name": Required[str],
        "items": NotRequired[list[str]],
    },
)
TrelloChecklistCreateArgs.__doc__ = """Arguments of ``trello_checklist_create``: Add a checklist with items to a Trello card."""


TrelloChecklistCheckArgs = TypedDict(
    "TrelloChecklistCheckArgs",
    {
        "card": Required[str],
        "item": Required[str],
        "done": NotRequired[bool],
    },
)
TrelloChecklistCheckArgs.__doc__ = """Arguments of ``trello_checklist_check``: Tick (or with done: false, untick) a Trello checklist item by name or ID."""


AdminOrgListArgs = TypedDict(
    "AdminOrgListArgs",
    {
    },
)
AdminOrgListArgs.__doc__ = """Arguments of ``admin_org_list``: Organizations the Atlassian Admin API key can see."""


AdminDirectoryListArgs = TypedDict(
    "AdminDirectoryListArgs",
    {
        "org": NotRequired[str | None],
    },
)
AdminDirectoryListArgs.__doc__ = """Arguments of ``admin_directory_list``: User directories of an Atlassian organization."""


AdminUserListArgs = TypedDict(
    "AdminUserListArgs",
    {
        "org": NotRequired[str | None],
        "search": NotRequired[str | None],
        "limit": NotRequired[int],
    },
)
AdminUserListArgs.__doc__ = """Arguments of ``admin_user_list``: Managed users of an Atlassian organization (every directory), optionally matching `search`."""


AdminUserSuspendArgs = TypedDict(
    "AdminUserSuspendArgs",
    {
        "org": NotRequired[str | None],
        "user": Required[str],
        "directory": NotRequired[str | None],
    },
)
AdminUserSuspendArgs.__doc__ = """Arguments of ``admin_user_suspend``: Suspend a managed user's access to every product (by account ID or email)."""


AdminUserRestoreArgs = TypedDict(
    "AdminUserRestoreArgs",
    {
        "org": NotRequired[str | None],
        "user": Required[str],
        "directory": NotRequired[str | None],
    },
)
AdminUserRestoreArgs.__doc__ = """Arguments of ``admin_user_restore``: Restore a suspended managed user's access (by account ID or email)."""


AdminGroupAddMemberArgs = TypedDict(
    "AdminGroupAddMemberArgs",
    {
        "org": NotRequired[str | None],
        "group": Required[str],
        "user": Required[str],
    },
)
AdminGroupAddMemberArgs.__doc__ = """Arguments of ``admin_group_add_member``: Add a managed user (email or account ID) to a group (name or ID) of an Atlassian organization."""


AdminGroupRemoveMemberArgs = TypedDict(
    "AdminGroupRemoveMemberArgs",
    {
        "org": NotRequired[str | None],
        "group": Required[str],
        "user": Required[str],
    },
)
AdminGroupRemoveMemberArgs.__doc__ = """Arguments of ``admin_group_remove_member``: Remove a managed user from a group of an Atlassian organization."""


AdminGroupListArgs = TypedDict(
    "AdminGroupListArgs",
    {
        "org": NotRequired[str | None],
        "search": NotRequired[str | None],
        "limit": NotRequired[int],
    },
)
AdminGroupListArgs.__doc__ = """Arguments of ``admin_group_list``: Groups of an Atlassian organization (every directory), optionally matching `search`."""


AdminDomainListArgs = TypedDict(
    "AdminDomainListArgs",
    {
        "org": NotRequired[str | None],
    },
)
AdminDomainListArgs.__doc__ = """Arguments of ``admin_domain_list``: Domains of an Atlassian organization with their verification status."""


AdminEventListArgs = TypedDict(
    "AdminEventListArgs",
    {
        "org": NotRequired[str | None],
        "search": NotRequired[str | None],
        "from": NotRequired[str | None],
        "to": NotRequired[str | None],
        "action": NotRequired[str | None],
        "limit": NotRequired[int],
    },
)
AdminEventListArgs.__doc__ = """Arguments of ``admin_event_list``: Audit log events of an Atlassian organization, newest first, filtered by words, dates and action."""


class _Model(pydantic.BaseModel):
    """A task result. Unknown fields are kept, so a newer server never breaks parsing."""

    model_config = pydantic.ConfigDict(
        extra="allow", defer_build=True, validate_by_name=True, serialize_by_alias=True
    )


class AdminUser(_Model):
    account_id: str
    name: str | None = None
    email: str | None = None
    status: str | None = None
    """Account status, for example active or suspended."""
    membership: str | None = None
    directory: str


class Approval(_Model):
    id: str
    name: str
    decision: str
    """pending, approved or declined."""
    can_answer: bool
    """True when you are an approver who has not answered yet."""
    approvers: _list[str]


class Assigned(_Model):
    key: str
    assignee: str


class Attachment(_Model):
    id: str
    filename: str
    size: int
    mime_type: str | None = None
    created: str | None = None
    author: str | None = None
    content: str
    """Download URL on the Jira server."""


class Board(_Model):
    id: int
    name: str
    type: str | None = None


class BoardSprints(_Model):
    board: int
    sprints: _list[Sprint]


class Branch(_Model):
    name: str
    commit: str | None = None


class BranchDeleted(_Model):
    repo: str
    name: str
    deleted: bool


class BuildStatus(_Model):
    name: str
    key: str | None = None
    state: str
    """SUCCESSFUL, FAILED, INPROGRESS, STOPPED (Cloud), CANCELLED or UNKNOWN (Data Center)."""
    url: str | None = None
    description: str | None = None


class Card(_Model):
    id: str
    name: str
    list: str | None = None
    """The list's name when known, else its ID."""
    due: str | None = None
    labels: _list[str]
    closed: bool
    short_link: str | None = None
    url: str | None = None


class CardComment(_Model):
    author: str | None = None
    date: str | None = None
    text: str


class CardDetail(_Model):
    id: str
    name: str
    list: str | None = None
    """The list's name when known, else its ID."""
    due: str | None = None
    labels: _list[str]
    closed: bool
    short_link: str | None = None
    url: str | None = None
    board: str | None = None
    description: str
    """The card description (Trello stores Markdown)."""
    comments: _list[CardComment]
    """Latest comments first."""


class Change(_Model):
    """One changed field of one history entry."""

    created: str | None = None
    author: str | None = None
    field: str
    from_: str | None = pydantic.Field(None, alias="from")
    to: str | None = None


class ChangesRequested(_Model):
    repo: str
    id: int
    changes_requested: bool


class CheckItem(_Model):
    id: str
    name: str
    done: bool


class Checklist(_Model):
    id: str
    name: str
    items: _list[CheckItem]


class Comment(_Model):
    id: str
    author: str | None = None
    created: str | None = None
    body: str
    """The comment as Markdown (Cloud) or wiki markup (Data Center)."""


class CommentDeleted(_Model):
    key: str
    id: str
    deleted: bool


class Commit(_Model):
    hash: str
    author: str | None = None
    date: str | None = None
    message: str
    """The first line of the message."""


class Component(_Model):
    id: str
    name: str
    lead: str | None = None
    description: str | None = None


class Copied(_Model):
    source: str
    id: str
    """The new page (empty in a dry run)."""
    title: str
    parent: str
    url: str


class Created(_Model):
    key: str
    url: str


class Deleted(_Model):
    id: str
    deleted: bool


class Desk(_Model):
    id: str
    project: str
    name: str


class Directory(_Model):
    id: str
    name: str


class Domain(_Model):
    id: str
    name: str
    status: str | None = None
    """Verification status of the domain claim."""


class EpicIssues(_Model):
    epic: str | None = None
    """The epic, or none after a remove."""
    issues: _list[str]


class Event(_Model):
    id: str
    time: str | None = None
    action: str | None = None
    actor: str | None = None
    message: str | None = None


class Exported(_Model):
    id: str
    title: str
    type: str
    path: str
    """The written `index.md` (or the folder, for a Cloud folder)."""
    attachments: int


class Field(_Model):
    """A system or custom field, from `GET /rest/api/{2|3}/field`."""

    id: str
    """For example `customfield_10016` or `duedate`."""
    name: str
    custom: bool
    type: str | None = None
    """JSON type: string, number, date, datetime, option, array, user, ..."""
    items: str | None = None
    """Item type of an array field."""


class Filter(_Model):
    id: str
    name: str
    jql: str
    owner: str | None = None
    favourite: bool
    url: str


class Group(_Model):
    id: str
    name: str
    description: str | None = None
    users: int | None = None
    directory: str


class IssueCount(_Model):
    count: int
    jql: str


class IssueDeleted(_Model):
    key: str
    deleted: bool


class IssueDetail(_Model):
    key: str
    summary: str
    status: str
    status_category: str
    """`new`, `indeterminate` (in progress), or `done`."""
    type: str
    assignee: str | None = None
    priority: str | None = None
    updated: str | None = None
    url: str
    reporter: str | None = None
    created: str | None = None
    labels: _list[str]
    components: _list[str]
    parent: str | None = None
    subtasks: _list[str]
    description: str
    """The description as Markdown (Cloud) or wiki markup (Data Center)."""
    comments: _list[Comment]
    comment_count: int
    links: _list[IssueLink]
    """Links to other issues, as "relation key" (for example "blocks ABC-2")."""
    fields: dict[str, Any] | None = None
    """Fields asked for by name (`issue view --field`), simplified: options
become their value, users their name, rich text Markdown."""


class IssueLink(_Model):
    relation: str
    """The phrase from this issue's side, for example "blocks" or "is blocked by"."""
    key: str
    summary: str | None = None
    status: str | None = None


class IssueList(_Model):
    """A page of issues and the JQL that found them."""

    issues: _list[IssueSummary]
    next_page_token: str | None = None
    """Pass back as `page_token` for the next page."""
    total: int | None = None
    """Total matches when the server reports it (Data Center)."""
    jql: str


class IssueSummary(_Model):
    key: str
    summary: str
    status: str
    status_category: str
    """`new`, `indeterminate` (in progress), or `done`."""
    type: str
    assignee: str | None = None
    priority: str | None = None
    updated: str | None = None
    url: str


class IssueUpdated(_Model):
    key: str
    updated: bool
    url: str


class JqlCheck(_Model):
    valid: bool
    errors: _list[str]
    warnings: _list[str]


class JqlField(_Model):
    name: str
    """The name to use in JQL."""
    display: str
    operators: _list[str]


class JqlReference(_Model):
    fields: _list[JqlField]
    functions: _list[str]


class LinkType(_Model):
    name: str
    outward: str
    """Phrase from the first issue to the second ("blocks")."""
    inward: str
    """Phrase from the second issue to the first ("is blocked by")."""


class Linked(_Model):
    from_: str = pydantic.Field(alias="from")
    to: str
    type: str
    """The link type name."""


class Membership(_Model):
    """A user's membership of a group after an add or a remove."""

    group: str
    group_id: str
    user: str | None = None
    """The user's email, or their name when the email is hidden."""
    account_id: str
    member: bool
    """True after an add, false after a remove."""


class Moved(_Model):
    key: str
    status: str
    """The status the issue is in now."""
    transition: str
    """The transition that moved it."""


class Org(_Model):
    id: str
    name: str


class Page(_Model):
    id: str
    title: str
    space: str | None = None
    parent: str | None = None
    """ID of the parent page (none for a page at the top of a space)."""
    version: int
    url: str
    markdown: str
    """The body as Markdown."""


class PageAttachment(_Model):
    id: str
    title: str
    size: int | None = None
    media_type: str | None = None
    download: str
    """Download URL on the Confluence server."""


class PageComment(_Model):
    id: str
    author: str | None = None
    """Display name (account ID on Cloud when the name lookup fails)."""
    created: str | None = None
    body: str
    """The comment as Markdown."""
    parent: str | None = None
    """The comment this one replies to."""
    selection: str | None = None
    """For inline comments, the page text they are attached to."""


class PageSummary(_Model):
    id: str
    title: str
    url: str


class PageVersion(_Model):
    number: int
    author: str | None = None
    created: str | None = None
    message: str | None = None
    minor_edit: bool


class PageView(_Model):
    """A page, with its storage-format body and its Markdown document (front
matter and body) when asked for."""

    id: str
    title: str
    space: str | None = None
    parent: str | None = None
    """ID of the parent page (none for a page at the top of a space)."""
    version: int
    url: str
    markdown: str
    """The body as Markdown."""
    document: str | None = None
    """The page as a Markdown file with front matter (with `front_matter: true`)."""
    storage: str | None = None
    """The body in storage format (with `storage: true`)."""


class Participants(_Model):
    key: str
    participants: _list[str]
    """Every participant after the change."""


class Pipeline(_Model):
    build_number: int
    uuid: str | None = None
    state: str
    """PENDING, IN_PROGRESS or COMPLETED."""
    result: str | None = None
    """SUCCESSFUL, FAILED, STOPPED or ERROR once completed."""
    branch: str | None = None
    created: str | None = None
    url: str


class PipelineDetail(_Model):
    build_number: int
    uuid: str | None = None
    state: str
    """PENDING, IN_PROGRESS or COMPLETED."""
    result: str | None = None
    """SUCCESSFUL, FAILED, STOPPED or ERROR once completed."""
    branch: str | None = None
    created: str | None = None
    url: str
    steps: _list[PipelineStep]


class PipelineLog(_Model):
    repo: str
    id: str
    log: str
    """The step logs as text."""


class PipelineStep(_Model):
    uuid: str
    name: str
    state: str
    """PENDING, IN_PROGRESS or COMPLETED."""
    result: str | None = None
    seconds: int | None = None


class PrApproval(_Model):
    repo: str
    id: int
    approved: bool
    """True after an approve, false after an unapprove."""


class PrChecks(_Model):
    repo: str
    id: int
    passed: bool
    """No check failed."""
    pending: bool
    """At least one check is still running."""
    checks: _list[BuildStatus]


class PrComment(_Model):
    id: str
    author: str | None = None
    created: str | None = None
    body: str
    """Markdown."""
    path: str | None = None
    """File of an inline comment."""
    line: int | None = None
    """Line of an inline comment (in the new version of the file when known)."""
    parent: str | None = None
    """ID of the comment this one answers."""
    resolved: bool


class PrCommented(_Model):
    repo: str
    id: int
    comment_id: str


class PrDiff(_Model):
    repo: str
    id: int
    diff: str
    """The unified diff."""


class Project(_Model):
    key: str
    name: str
    id: str
    type: str | None = None


class PullRequest(_Model):
    id: int
    title: str
    state: str
    """OPEN, MERGED, DECLINED or SUPERSEDED."""
    author: str | None = None
    source: str
    destination: str
    updated: str | None = None
    description: str
    reviewers: _list[str]
    approved_by: _list[str]
    url: str
    source_repo: str | None = None
    """`owner/slug` of the repository the source branch is in (differs
from the target repository for pull requests from forks)."""
    draft: bool
    """True for a draft (not yet ready for review)."""


class Pushed(_Model):
    file: str
    id: str
    title: str
    version: int
    action: str
    """created, updated, unchanged, or planned (dry run)."""


class Queue(_Model):
    id: str
    name: str
    jql: str | None = None
    issues: int | None = None


class Release(_Model):
    """A project version ("release")."""

    id: str
    name: str
    released: bool
    archived: bool
    release_date: str | None = None
    description: str | None = None


class RemoteLink(_Model):
    """A web link that `Jira::remote_link` added to an issue."""

    key: str
    id: int | None = None
    """Jira's ID of the link."""
    url: str
    title: str


class Repo(_Model):
    repo: str
    """`owner/slug`."""
    name: str
    description: str | None = None
    default_branch: str | None = None
    private: bool | None = None
    url: str


class Request(_Model):
    key: str
    summary: str
    status: str
    type: str | None = None
    reporter: str | None = None
    created: str | None = None
    url: str


class RequestAttachment(_Model):
    filename: str
    author: str | None = None
    created: str | None = None
    size: int | None = None
    mime_type: str | None = None
    url: str | None = None


class RequestComment(_Model):
    id: str
    author: str | None = None
    created: str | None = None
    public: bool
    """False for internal notes that customers cannot see."""
    body: str


class RequestDetail(_Model):
    key: str
    summary: str
    status: str
    type: str | None = None
    reporter: str | None = None
    created: str | None = None
    url: str
    description: str
    comments: _list[RequestComment]


class RequestMoved(_Model):
    key: str
    transition: str
    """The transition that ran."""
    id: str
    """Its ID."""


class RequestTransition(_Model):
    id: str
    name: str


class RequestType(_Model):
    id: str
    name: str
    description: str | None = None


class SearchHit(_Model):
    id: str
    type: str
    title: str
    space: str | None = None
    excerpt: str | None = None
    url: str


class SearchResults(_Model):
    cql: str
    """The CQL that ran."""
    results: _list[SearchHit]


class Sla(_Model):
    name: str
    state: str
    """ongoing, paused, met or breached."""
    breached: bool
    remaining: str | None = None
    """Time left (negative when breached), as Jira shows it."""
    goal: str | None = None


class Space(_Model):
    key: str
    name: str
    id: str | None = None
    type: str | None = None
    url: str


class Sprint(_Model):
    id: int
    name: str
    state: str
    start: str | None = None
    end: str | None = None
    goal: str | None = None


class SprintIssues(_Model):
    sprint: int
    issues: _list[str]


class Suggestion(_Model):
    value: str
    display: str


class Transition(_Model):
    id: str
    name: str
    to: str
    """The status the transition leads to."""


class TreeNode(_Model):
    """One node of a page tree (see [`Confluence::tree`])."""

    id: str
    title: str
    type: str
    """page, folder, whiteboard, database or embed (Data Center: page)."""
    depth: int
    """1 for direct children of the root."""
    parent: str
    """ID of the parent node."""
    url: str


class TrelloBoard(_Model):
    id: str
    name: str
    short_link: str | None = None
    url: str | None = None


class TrelloList(_Model):
    id: str
    name: str


class Unlinked(_Model):
    from_: str = pydantic.Field(alias="from")
    to: str
    removed: _list[str]
    """The link types that were removed."""


class User(_Model):
    id: str
    """Account ID (Cloud) or username (Data Center)."""
    name: str
    email: str | None = None


class Watchers(_Model):
    watching: bool
    """True when the signed-in user watches the issue."""
    count: int
    watchers: _list[User]


class Watching(_Model):
    key: str
    user: str
    watching: bool
    """True after a watch, false after an unwatch."""


class Worklog(_Model):
    id: str
    author: str | None = None
    time_spent: str | None = None
    """As Jira shows it, for example "2h 30m"."""
    seconds: int | None = None
    started: str | None = None
    comment: str | None = None
    """Markdown."""


class WorklogAdded(_Model):
    """A worklog that `Jira::worklog` added."""

    key: str
    worklog_id: str | None = None
    time_spent: str


class _Tasks(Protocol):
    def task(self, name: str, args: Mapping[str, Any] | None = None) -> Any: ...


class _AsyncTasks(Protocol):
    async def task(self, name: str, args: Mapping[str, Any] | None = None) -> Any: ...


class JiraBoardApi:
    """``atl jira board``"""

    def __init__(self, client: _Tasks) -> None:
        self._client = client

    def list(self, **kwargs: Unpack[JiraBoardListArgs]) -> _list[Board]:
        """Jira Software boards, optionally of one project."""
        return [Board.model_validate(item) for item in self._client.task(TaskName.JIRA_BOARD_LIST, kwargs)]


class JiraCommentApi:
    """``atl jira comment``"""

    def __init__(self, client: _Tasks) -> None:
        self._client = client

    def list(self, **kwargs: Unpack[JiraCommentListArgs]) -> _list[Comment]:
        """Every comment of a Jira issue, oldest first, with IDs, as Markdown."""
        return [Comment.model_validate(item) for item in self._client.task(TaskName.JIRA_COMMENT_LIST, kwargs)]

    def edit(self, **kwargs: Unpack[JiraCommentEditArgs]) -> Comment:
        """Replace the text of a Jira comment (Markdown)."""
        return Comment.model_validate(self._client.task(TaskName.JIRA_COMMENT_EDIT, kwargs))

    def delete(self, **kwargs: Unpack[JiraCommentDeleteArgs]) -> CommentDeleted:
        """Delete a Jira comment. Cannot be undone."""
        return CommentDeleted.model_validate(self._client.task(TaskName.JIRA_COMMENT_DELETE, kwargs))


class JiraComponentApi:
    """``atl jira component``"""

    def __init__(self, client: _Tasks) -> None:
        self._client = client

    def list(self, **kwargs: Unpack[JiraComponentListArgs]) -> _list[Component]:
        """Components of a Jira project with their leads."""
        return [Component.model_validate(item) for item in self._client.task(TaskName.JIRA_COMPONENT_LIST, kwargs)]


class JiraEpicApi:
    """``atl jira epic``"""

    def __init__(self, client: _Tasks) -> None:
        self._client = client

    def list(self, **kwargs: Unpack[JiraEpicListArgs]) -> _list[IssueSummary]:
        """Open epics, optionally of one project."""
        return [IssueSummary.model_validate(item) for item in self._client.task(TaskName.JIRA_EPIC_LIST, kwargs)]

    def add(self, **kwargs: Unpack[JiraEpicAddArgs]) -> EpicIssues:
        """Move issues into an epic."""
        return EpicIssues.model_validate(self._client.task(TaskName.JIRA_EPIC_ADD, kwargs))

    def remove(self, **kwargs: Unpack[JiraEpicRemoveArgs]) -> EpicIssues:
        """Remove issues from their epic."""
        return EpicIssues.model_validate(self._client.task(TaskName.JIRA_EPIC_REMOVE, kwargs))


class JiraFieldApi:
    """``atl jira field``"""

    def __init__(self, client: _Tasks) -> None:
        self._client = client

    def list(self, **kwargs: Unpack[JiraFieldListArgs]) -> _list[Field]:
        """Jira system and custom fields (ID, name, type), optionally filtered by part of the name. Use the names in the `fields` argument of issue create, edit and view."""
        return [Field.model_validate(item) for item in self._client.task(TaskName.JIRA_FIELD_LIST, kwargs)]


class JiraFilterApi:
    """``atl jira filter``"""

    def __init__(self, client: _Tasks) -> None:
        self._client = client

    def list(self, **kwargs: Unpack[JiraFilterListArgs]) -> _list[Filter]:
        """Saved Jira filters (Cloud: all visible, Data Center: favourites)."""
        return [Filter.model_validate(item) for item in self._client.task(TaskName.JIRA_FILTER_LIST, kwargs)]

    def run(self, **kwargs: Unpack[JiraFilterRunArgs]) -> IssueList:
        """Run a saved Jira filter by ID or exact name; returns issue records."""
        return IssueList.model_validate(self._client.task(TaskName.JIRA_FILTER_RUN, kwargs))


class JiraIssueApi:
    """``atl jira issue``"""

    def __init__(self, client: _Tasks) -> None:
        self._client = client

    def list(self, **kwargs: Unpack[JiraIssueListArgs]) -> IssueList:
        """Search Jira issues with filters (or extra JQL). Returns small records: key, summary, status, type, assignee, priority, updated, url. With no filters: your unresolved issues."""
        return IssueList.model_validate(self._client.task(TaskName.JIRA_ISSUE_LIST, kwargs))

    def count(self, **kwargs: Unpack[JiraIssueCountArgs]) -> IssueCount:
        """Count Jira issues matching the same filters as issue_list."""
        return IssueCount.model_validate(self._client.task(TaskName.JIRA_ISSUE_COUNT, kwargs))

    def view(self, **kwargs: Unpack[JiraIssueViewArgs]) -> IssueDetail:
        """One Jira issue with description and latest comments as Markdown, plus any `fields` asked for by name."""
        return IssueDetail.model_validate(self._client.task(TaskName.JIRA_ISSUE_VIEW, kwargs))

    def delete(self, **kwargs: Unpack[JiraIssueDeleteArgs]) -> IssueDeleted:
        """Delete a Jira issue (with `subtasks: true`, also its sub-tasks). Cannot be undone."""
        return IssueDeleted.model_validate(self._client.task(TaskName.JIRA_ISSUE_DELETE, kwargs))

    def watchers(self, **kwargs: Unpack[JiraIssueWatchersArgs]) -> Watchers:
        """Who watches a Jira issue, and whether you do."""
        return Watchers.model_validate(self._client.task(TaskName.JIRA_ISSUE_WATCHERS, kwargs))

    def watch(self, **kwargs: Unpack[JiraIssueWatchArgs]) -> Watching:
        """Add a watcher to a Jira issue (default: you)."""
        return Watching.model_validate(self._client.task(TaskName.JIRA_ISSUE_WATCH, kwargs))

    def unwatch(self, **kwargs: Unpack[JiraIssueUnwatchArgs]) -> Watching:
        """Remove a watcher from a Jira issue (default: you)."""
        return Watching.model_validate(self._client.task(TaskName.JIRA_ISSUE_UNWATCH, kwargs))

    def create(self, **kwargs: Unpack[JiraIssueCreateArgs]) -> Created:
        """Create a Jira issue; the description is Markdown."""
        return Created.model_validate(self._client.task(TaskName.JIRA_ISSUE_CREATE, kwargs))

    def edit(self, **kwargs: Unpack[JiraIssueEditArgs]) -> IssueUpdated:
        """Change a Jira issue's summary, description (Markdown), priority or labels."""
        return IssueUpdated.model_validate(self._client.task(TaskName.JIRA_ISSUE_EDIT, kwargs))

    def assign(self, **kwargs: Unpack[JiraIssueAssignArgs]) -> Assigned:
        """Assign a Jira issue to me, nobody (x), or a user."""
        return Assigned.model_validate(self._client.task(TaskName.JIRA_ISSUE_ASSIGN, kwargs))

    def transitions(self, **kwargs: Unpack[JiraIssueTransitionsArgs]) -> _list[Transition]:
        """Statuses a Jira issue can move to."""
        return [Transition.model_validate(item) for item in self._client.task(TaskName.JIRA_ISSUE_TRANSITIONS, kwargs)]

    def move(self, **kwargs: Unpack[JiraIssueMoveArgs]) -> Moved:
        """Move a Jira issue to another status (by status or transition name)."""
        return Moved.model_validate(self._client.task(TaskName.JIRA_ISSUE_MOVE, kwargs))

    def comment(self, **kwargs: Unpack[JiraIssueCommentArgs]) -> Comment:
        """Add a Markdown comment to a Jira issue."""
        return Comment.model_validate(self._client.task(TaskName.JIRA_ISSUE_COMMENT, kwargs))

    def link(self, **kwargs: Unpack[JiraIssueLinkArgs]) -> Linked:
        """Link two Jira issues so that 'from <kind> to' reads true, for example from A-1 kind blocks to A-2."""
        return Linked.model_validate(self._client.task(TaskName.JIRA_ISSUE_LINK, kwargs))

    def unlink(self, **kwargs: Unpack[JiraIssueUnlinkArgs]) -> Unlinked:
        """Remove the links between two Jira issues (optionally only one link type)."""
        return Unlinked.model_validate(self._client.task(TaskName.JIRA_ISSUE_UNLINK, kwargs))

    def remote_link(self, **kwargs: Unpack[JiraIssueRemoteLinkArgs]) -> RemoteLink:
        """Add a web link (URL and title) to a Jira issue."""
        return RemoteLink.model_validate(self._client.task(TaskName.JIRA_ISSUE_REMOTE_LINK, kwargs))

    def history(self, **kwargs: Unpack[JiraIssueHistoryArgs]) -> _list[Change]:
        """Field changes of a Jira issue, newest first: when, who, field, from, to."""
        return [Change.model_validate(item) for item in self._client.task(TaskName.JIRA_ISSUE_HISTORY, kwargs)]

    def link_types(self, **kwargs: Unpack[JiraIssueLinkTypesArgs]) -> _list[LinkType]:
        """Jira issue link types with their outward and inward phrases."""
        return [LinkType.model_validate(item) for item in self._client.task(TaskName.JIRA_ISSUE_LINK_TYPES, kwargs)]

    def clone(self, **kwargs: Unpack[JiraIssueCloneArgs]) -> Created:
        """Copy a Jira issue (summary, description, type, labels, priority, components, parent)."""
        return Created.model_validate(self._client.task(TaskName.JIRA_ISSUE_CLONE, kwargs))

    def worklog(self, **kwargs: Unpack[JiraIssueWorklogArgs]) -> WorklogAdded:
        """Log time on a Jira issue."""
        return WorklogAdded.model_validate(self._client.task(TaskName.JIRA_ISSUE_WORKLOG, kwargs))

    def attach(self, **kwargs: Unpack[JiraIssueAttachArgs]) -> _list[Attachment]:
        """Upload local files to a Jira issue."""
        return [Attachment.model_validate(item) for item in self._client.task(TaskName.JIRA_ISSUE_ATTACH, kwargs)]

    def attachments(self, **kwargs: Unpack[JiraIssueAttachmentsArgs]) -> _list[Attachment]:
        """List a Jira issue's attachments."""
        return [Attachment.model_validate(item) for item in self._client.task(TaskName.JIRA_ISSUE_ATTACHMENTS, kwargs)]

    def types(self, **kwargs: Unpack[JiraIssueTypesArgs]) -> _list[str]:
        """Issue type names of a Jira project (without sub-task types), for the `type` argument of jira_issue_create."""
        return self._client.task(TaskName.JIRA_ISSUE_TYPES, kwargs)


class JiraJqlApi:
    """``atl jira jql``"""

    def __init__(self, client: _Tasks) -> None:
        self._client = client

    def check(self, **kwargs: Unpack[JiraJqlCheckArgs]) -> JqlCheck:
        """Validate JQL with Jira's parser; returns valid, errors, warnings."""
        return JqlCheck.model_validate(self._client.task(TaskName.JIRA_JQL_CHECK, kwargs))

    def fields(self, **kwargs: Unpack[JiraJqlFieldsArgs]) -> JqlReference:
        """Fields (with operators) and functions usable in JQL on this site."""
        return JqlReference.model_validate(self._client.task(TaskName.JIRA_JQL_FIELDS, kwargs))

    def suggest(self, **kwargs: Unpack[JiraJqlSuggestArgs]) -> _list[Suggestion]:
        """Value suggestions for a JQL field, for example field=status prefix=In."""
        return [Suggestion.model_validate(item) for item in self._client.task(TaskName.JIRA_JQL_SUGGEST, kwargs)]


class JiraProjectApi:
    """``atl jira project``"""

    def __init__(self, client: _Tasks) -> None:
        self._client = client

    def list(self, **kwargs: Unpack[JiraProjectListArgs]) -> _list[Project]:
        """Jira projects."""
        return [Project.model_validate(item) for item in self._client.task(TaskName.JIRA_PROJECT_LIST, kwargs)]


class JiraReleaseApi:
    """``atl jira release``"""

    def __init__(self, client: _Tasks) -> None:
        self._client = client

    def list(self, **kwargs: Unpack[JiraReleaseListArgs]) -> _list[Release]:
        """Versions (releases) of a Jira project: name, released, archived, release date."""
        return [Release.model_validate(item) for item in self._client.task(TaskName.JIRA_RELEASE_LIST, kwargs)]


class JiraSprintApi:
    """``atl jira sprint``"""

    def __init__(self, client: _Tasks) -> None:
        self._client = client

    def create(self, **kwargs: Unpack[JiraSprintCreateArgs]) -> Sprint:
        """Create a future sprint on a board (default: the project's first board)."""
        return Sprint.model_validate(self._client.task(TaskName.JIRA_SPRINT_CREATE, kwargs))

    def start(self, **kwargs: Unpack[JiraSprintStartArgs]) -> Sprint:
        """Start a future sprint; Jira needs a start and an end date."""
        return Sprint.model_validate(self._client.task(TaskName.JIRA_SPRINT_START, kwargs))

    def close(self, **kwargs: Unpack[JiraSprintCloseArgs]) -> Sprint:
        """Complete an active sprint."""
        return Sprint.model_validate(self._client.task(TaskName.JIRA_SPRINT_CLOSE, kwargs))

    def list(self, **kwargs: Unpack[JiraSprintListArgs]) -> BoardSprints:
        """Sprints of a board."""
        return BoardSprints.model_validate(self._client.task(TaskName.JIRA_SPRINT_LIST, kwargs))

    def add(self, **kwargs: Unpack[JiraSprintAddArgs]) -> SprintIssues:
        """Move issues into a sprint."""
        return SprintIssues.model_validate(self._client.task(TaskName.JIRA_SPRINT_ADD, kwargs))


class JiraWorklogApi:
    """``atl jira worklog``"""

    def __init__(self, client: _Tasks) -> None:
        self._client = client

    def list(self, **kwargs: Unpack[JiraWorklogListArgs]) -> _list[Worklog]:
        """Time logged on a Jira issue: author, time spent, start, comment."""
        return [Worklog.model_validate(item) for item in self._client.task(TaskName.JIRA_WORKLOG_LIST, kwargs)]


class JiraApi:
    """Jira and Jira Software: issues, comments, sprints, boards, filters and JQL."""

    def __init__(self, client: _Tasks) -> None:
        self._client = client
        self.board = JiraBoardApi(client)
        """``atl jira board``"""
        self.comment = JiraCommentApi(client)
        """``atl jira comment``"""
        self.component = JiraComponentApi(client)
        """``atl jira component``"""
        self.epic = JiraEpicApi(client)
        """``atl jira epic``"""
        self.field = JiraFieldApi(client)
        """``atl jira field``"""
        self.filter = JiraFilterApi(client)
        """``atl jira filter``"""
        self.issue = JiraIssueApi(client)
        """``atl jira issue``"""
        self.jql = JiraJqlApi(client)
        """``atl jira jql``"""
        self.project = JiraProjectApi(client)
        """``atl jira project``"""
        self.release = JiraReleaseApi(client)
        """``atl jira release``"""
        self.sprint = JiraSprintApi(client)
        """``atl jira sprint``"""
        self.worklog = JiraWorklogApi(client)
        """``atl jira worklog``"""

    def me(self, **kwargs: Unpack[JiraMeArgs]) -> User:
        """The signed-in Jira user."""
        return User.model_validate(self._client.task(TaskName.JIRA_ME, kwargs))


class ConfluenceAttachmentApi:
    """``atl confluence attachment``"""

    def __init__(self, client: _Tasks) -> None:
        self._client = client

    def delete(self, **kwargs: Unpack[ConfluenceAttachmentDeleteArgs]) -> Deleted:
        """Delete a Confluence attachment by ID."""
        return Deleted.model_validate(self._client.task(TaskName.CONFLUENCE_ATTACHMENT_DELETE, kwargs))


class ConfluenceBlogApi:
    """``atl confluence blog``"""

    def __init__(self, client: _Tasks) -> None:
        self._client = client

    def list(self, **kwargs: Unpack[ConfluenceBlogListArgs]) -> _list[PageSummary]:
        """Blog posts of a Confluence space, newest first."""
        return [PageSummary.model_validate(item) for item in self._client.task(TaskName.CONFLUENCE_BLOG_LIST, kwargs)]

    def get(self, **kwargs: Unpack[ConfluenceBlogGetArgs]) -> Page:
        """One Confluence blog post with its body as Markdown."""
        return Page.model_validate(self._client.task(TaskName.CONFLUENCE_BLOG_GET, kwargs))

    def create(self, **kwargs: Unpack[ConfluenceBlogCreateArgs]) -> PageSummary:
        """Publish a Confluence blog post from Markdown."""
        return PageSummary.model_validate(self._client.task(TaskName.CONFLUENCE_BLOG_CREATE, kwargs))


class ConfluenceCommentApi:
    """``atl confluence comment``"""

    def __init__(self, client: _Tasks) -> None:
        self._client = client

    def delete(self, **kwargs: Unpack[ConfluenceCommentDeleteArgs]) -> Deleted:
        """Delete a Confluence comment by ID."""
        return Deleted.model_validate(self._client.task(TaskName.CONFLUENCE_COMMENT_DELETE, kwargs))


class ConfluencePageApi:
    """``atl confluence page``"""

    def __init__(self, client: _Tasks) -> None:
        self._client = client

    def get(self, **kwargs: Unpack[ConfluencePageGetArgs]) -> PageView:
        """A Confluence page as Markdown (with title, space, version, url)."""
        return PageView.model_validate(self._client.task(TaskName.CONFLUENCE_PAGE_GET, kwargs))

    def create(self, **kwargs: Unpack[ConfluencePageCreateArgs]) -> PageSummary:
        """Create a Confluence page from Markdown (or storage format)."""
        return PageSummary.model_validate(self._client.task(TaskName.CONFLUENCE_PAGE_CREATE, kwargs))

    def update(self, **kwargs: Unpack[ConfluencePageUpdateArgs]) -> PageSummary:
        """Replace a Confluence page's title and/or body; the version is bumped automatically."""
        return PageSummary.model_validate(self._client.task(TaskName.CONFLUENCE_PAGE_UPDATE, kwargs))

    def delete(self, **kwargs: Unpack[ConfluencePageDeleteArgs]) -> Deleted:
        """Delete a Confluence page."""
        return Deleted.model_validate(self._client.task(TaskName.CONFLUENCE_PAGE_DELETE, kwargs))

    def children(self, **kwargs: Unpack[ConfluencePageChildrenArgs]) -> _list[TreeNode]:
        """Children of a Confluence page (Cloud: also folders), or with `recursive` the whole tree, depth first, each row with type, depth and parent."""
        return [TreeNode.model_validate(item) for item in self._client.task(TaskName.CONFLUENCE_PAGE_CHILDREN, kwargs)]

    def find(self, **kwargs: Unpack[ConfluencePageFindArgs]) -> _list[SearchHit]:
        """Confluence pages whose title contains the words, optionally in one space."""
        return [SearchHit.model_validate(item) for item in self._client.task(TaskName.CONFLUENCE_PAGE_FIND, kwargs)]

    def attach(self, **kwargs: Unpack[ConfluencePageAttachArgs]) -> _list[PageAttachment]:
        """Upload local files to a Confluence page."""
        return [PageAttachment.model_validate(item) for item in self._client.task(TaskName.CONFLUENCE_PAGE_ATTACH, kwargs)]

    def attachments(self, **kwargs: Unpack[ConfluencePageAttachmentsArgs]) -> _list[PageAttachment]:
        """Attachments of a Confluence page with download URLs."""
        return [PageAttachment.model_validate(item) for item in self._client.task(TaskName.CONFLUENCE_PAGE_ATTACHMENTS, kwargs)]

    def comments(self, **kwargs: Unpack[ConfluencePageCommentsArgs]) -> _list[PageComment]:
        """Footer (or with `inline`, inline) comments of a Confluence page, oldest first, as Markdown, with the replied-to `parent`."""
        return [PageComment.model_validate(item) for item in self._client.task(TaskName.CONFLUENCE_PAGE_COMMENTS, kwargs)]

    def comment(self, **kwargs: Unpack[ConfluencePageCommentArgs]) -> PageComment:
        """Add a footer comment (Markdown) to a Confluence page, or a reply with `parent`."""
        return PageComment.model_validate(self._client.task(TaskName.CONFLUENCE_PAGE_COMMENT, kwargs))

    def export(self, **kwargs: Unpack[ConfluencePageExportArgs]) -> _list[Exported]:
        """Export a Confluence page (and with `recursive` its tree) to local Markdown folders: <title>/index.md with front matter and _attachments/. Changes nothing on the server."""
        return [Exported.model_validate(item) for item in self._client.task(TaskName.CONFLUENCE_PAGE_EXPORT, kwargs)]

    def push(self, **kwargs: Unpack[ConfluencePagePushArgs]) -> _list[Pushed]:
        """Publish local Markdown files: update the page in each file's front matter (refusing pages changed on the server unless `force`, skipping unchanged ones), create pages for new files under their folder's index.md, upload referenced images, and write ids and versions back."""
        return [Pushed.model_validate(item) for item in self._client.task(TaskName.CONFLUENCE_PAGE_PUSH, kwargs)]

    def copy_tree(self, **kwargs: Unpack[ConfluencePageCopyTreeArgs]) -> _list[Copied]:
        """Copy a Confluence page and the pages below it under another page (titles, bodies, labels)."""
        return [Copied.model_validate(item) for item in self._client.task(TaskName.CONFLUENCE_PAGE_COPY_TREE, kwargs)]

    def labels(self, **kwargs: Unpack[ConfluencePageLabelsArgs]) -> _list[str]:
        """Labels of a Confluence page."""
        return self._client.task(TaskName.CONFLUENCE_PAGE_LABELS, kwargs)

    def label(self, **kwargs: Unpack[ConfluencePageLabelArgs]) -> _list[str]:
        """Add and remove labels on a Confluence page; returns the resulting labels."""
        return self._client.task(TaskName.CONFLUENCE_PAGE_LABEL, kwargs)

    def move(self, **kwargs: Unpack[ConfluencePageMoveArgs]) -> PageSummary:
        """Move a Confluence page under another page (position append) or next to it (before/after, Cloud only)."""
        return PageSummary.model_validate(self._client.task(TaskName.CONFLUENCE_PAGE_MOVE, kwargs))

    def history(self, **kwargs: Unpack[ConfluencePageHistoryArgs]) -> _list[PageVersion]:
        """Versions of a Confluence page, newest first: number, author, date, message. Read one with confluence_page_get and `version`."""
        return [PageVersion.model_validate(item) for item in self._client.task(TaskName.CONFLUENCE_PAGE_HISTORY, kwargs)]


class ConfluenceSpaceApi:
    """``atl confluence space``"""

    def __init__(self, client: _Tasks) -> None:
        self._client = client

    def create(self, **kwargs: Unpack[ConfluenceSpaceCreateArgs]) -> Space:
        """Create a Confluence space with a key, a name and an optional plain-text description."""
        return Space.model_validate(self._client.task(TaskName.CONFLUENCE_SPACE_CREATE, kwargs))

    def list(self, **kwargs: Unpack[ConfluenceSpaceListArgs]) -> _list[Space]:
        """Confluence spaces."""
        return [Space.model_validate(item) for item in self._client.task(TaskName.CONFLUENCE_SPACE_LIST, kwargs)]

    def pages(self, **kwargs: Unpack[ConfluenceSpacePagesArgs]) -> _list[PageSummary]:
        """Top-level pages of a Confluence space."""
        return [PageSummary.model_validate(item) for item in self._client.task(TaskName.CONFLUENCE_SPACE_PAGES, kwargs)]


class ConfluenceApi:
    """Confluence pages, blogs, spaces, comments, labels and attachments."""

    def __init__(self, client: _Tasks) -> None:
        self._client = client
        self.attachment = ConfluenceAttachmentApi(client)
        """``atl confluence attachment``"""
        self.blog = ConfluenceBlogApi(client)
        """``atl confluence blog``"""
        self.comment = ConfluenceCommentApi(client)
        """``atl confluence comment``"""
        self.page = ConfluencePageApi(client)
        """``atl confluence page``"""
        self.space = ConfluenceSpaceApi(client)
        """``atl confluence space``"""

    def search(self, **kwargs: Unpack[ConfluenceSearchArgs]) -> SearchResults:
        """Search Confluence by words or CQL; returns id, type, title, space, excerpt, url."""
        return SearchResults.model_validate(self._client.task(TaskName.CONFLUENCE_SEARCH, kwargs))


class BitbucketBranchApi:
    """``atl bitbucket branch``"""

    def __init__(self, client: _Tasks) -> None:
        self._client = client

    def list(self, **kwargs: Unpack[BitbucketBranchListArgs]) -> _list[Branch]:
        """Branches of a Bitbucket repository, most recently changed first."""
        return [Branch.model_validate(item) for item in self._client.task(TaskName.BITBUCKET_BRANCH_LIST, kwargs)]

    def create(self, **kwargs: Unpack[BitbucketBranchCreateArgs]) -> Branch:
        """Create a branch from another branch, a tag or a commit (default: the main branch)."""
        return Branch.model_validate(self._client.task(TaskName.BITBUCKET_BRANCH_CREATE, kwargs))

    def delete(self, **kwargs: Unpack[BitbucketBranchDeleteArgs]) -> BranchDeleted:
        """Delete a branch. Unmerged commits on it can be lost."""
        return BranchDeleted.model_validate(self._client.task(TaskName.BITBUCKET_BRANCH_DELETE, kwargs))


class BitbucketCommitApi:
    """``atl bitbucket commit``"""

    def __init__(self, client: _Tasks) -> None:
        self._client = client

    def list(self, **kwargs: Unpack[BitbucketCommitListArgs]) -> _list[Commit]:
        """Recent commits of a branch, newest first: hash, author, date, first line of the message."""
        return [Commit.model_validate(item) for item in self._client.task(TaskName.BITBUCKET_COMMIT_LIST, kwargs)]


class BitbucketPipelineApi:
    """``atl bitbucket pipeline``"""

    def __init__(self, client: _Tasks) -> None:
        self._client = client

    def list(self, **kwargs: Unpack[BitbucketPipelineListArgs]) -> _list[Pipeline]:
        """Latest Bitbucket Pipelines runs, newest first (Cloud only)."""
        return [Pipeline.model_validate(item) for item in self._client.task(TaskName.BITBUCKET_PIPELINE_LIST, kwargs)]

    def run(self, **kwargs: Unpack[BitbucketPipelineRunArgs]) -> Pipeline:
        """Start the pipeline of a branch, or a custom pipeline (Cloud only)."""
        return Pipeline.model_validate(self._client.task(TaskName.BITBUCKET_PIPELINE_RUN, kwargs))

    def stop(self, **kwargs: Unpack[BitbucketPipelineStopArgs]) -> Pipeline:
        """Stop a running Bitbucket pipeline by build number or UUID (Cloud only)."""
        return Pipeline.model_validate(self._client.task(TaskName.BITBUCKET_PIPELINE_STOP, kwargs))

    def view(self, **kwargs: Unpack[BitbucketPipelineViewArgs]) -> PipelineDetail:
        """One Bitbucket pipeline by build number or UUID, with each step's state and result (Cloud only)."""
        return PipelineDetail.model_validate(self._client.task(TaskName.BITBUCKET_PIPELINE_VIEW, kwargs))

    def logs(self, **kwargs: Unpack[BitbucketPipelineLogsArgs]) -> PipelineLog:
        """Log text of one pipeline step (by name or UUID) or of every step (Cloud only)."""
        return PipelineLog.model_validate(self._client.task(TaskName.BITBUCKET_PIPELINE_LOGS, kwargs))


class BitbucketPrApi:
    """``atl bitbucket pr``"""

    def __init__(self, client: _Tasks) -> None:
        self._client = client

    def list(self, **kwargs: Unpack[BitbucketPrListArgs]) -> _list[PullRequest]:
        """Pull requests of a Bitbucket repository by state."""
        return [PullRequest.model_validate(item) for item in self._client.task(TaskName.BITBUCKET_PR_LIST, kwargs)]

    def view(self, **kwargs: Unpack[BitbucketPrViewArgs]) -> PullRequest:
        """One pull request with description, reviewers and approvals."""
        return PullRequest.model_validate(self._client.task(TaskName.BITBUCKET_PR_VIEW, kwargs))

    def create(self, **kwargs: Unpack[BitbucketPrCreateArgs]) -> PullRequest:
        """Open a pull request from a source branch."""
        return PullRequest.model_validate(self._client.task(TaskName.BITBUCKET_PR_CREATE, kwargs))

    def merge(self, **kwargs: Unpack[BitbucketPrMergeArgs]) -> PullRequest:
        """Merge a pull request."""
        return PullRequest.model_validate(self._client.task(TaskName.BITBUCKET_PR_MERGE, kwargs))

    def approve(self, **kwargs: Unpack[BitbucketPrApproveArgs]) -> PrApproval:
        """Approve a pull request as the signed-in user."""
        return PrApproval.model_validate(self._client.task(TaskName.BITBUCKET_PR_APPROVE, kwargs))

    def comment(self, **kwargs: Unpack[BitbucketPrCommentArgs]) -> PrCommented:
        """Add a Markdown comment to a pull request."""
        return PrCommented.model_validate(self._client.task(TaskName.BITBUCKET_PR_COMMENT, kwargs))

    def diff(self, **kwargs: Unpack[BitbucketPrDiffArgs]) -> PrDiff:
        """The unified diff of a pull request."""
        return PrDiff.model_validate(self._client.task(TaskName.BITBUCKET_PR_DIFF, kwargs))

    def comments(self, **kwargs: Unpack[BitbucketPrCommentsArgs]) -> _list[PrComment]:
        """Comments of a pull request, oldest first, as Markdown: general and inline (path, line), replies flattened with `parent`."""
        return [PrComment.model_validate(item) for item in self._client.task(TaskName.BITBUCKET_PR_COMMENTS, kwargs)]

    def checks(self, **kwargs: Unpack[BitbucketPrChecksArgs]) -> PrChecks:
        """Build statuses of a pull request's latest commit, with `passed` (none failed) and `pending` (some still running)."""
        return PrChecks.model_validate(self._client.task(TaskName.BITBUCKET_PR_CHECKS, kwargs))

    def unapprove(self, **kwargs: Unpack[BitbucketPrUnapproveArgs]) -> PrApproval:
        """Remove your approval from a pull request."""
        return PrApproval.model_validate(self._client.task(TaskName.BITBUCKET_PR_UNAPPROVE, kwargs))

    def ready(self, **kwargs: Unpack[BitbucketPrReadyArgs]) -> PullRequest:
        """Mark a draft pull request as ready for review (or, with `draft: true`, as a draft again)."""
        return PullRequest.model_validate(self._client.task(TaskName.BITBUCKET_PR_READY, kwargs))

    def decline(self, **kwargs: Unpack[BitbucketPrDeclineArgs]) -> PullRequest:
        """Decline a Bitbucket pull request (Cloud cannot reopen it)."""
        return PullRequest.model_validate(self._client.task(TaskName.BITBUCKET_PR_DECLINE, kwargs))

    def request_changes(self, **kwargs: Unpack[BitbucketPrRequestChangesArgs]) -> ChangesRequested:
        """Mark a Bitbucket pull request as needing changes by you (Cloud: request changes; Data Center: needs work)."""
        return ChangesRequested.model_validate(self._client.task(TaskName.BITBUCKET_PR_REQUEST_CHANGES, kwargs))

    def edit(self, **kwargs: Unpack[BitbucketPrEditArgs]) -> PullRequest:
        """Change a Bitbucket pull request's title, description (Markdown) or destination branch."""
        return PullRequest.model_validate(self._client.task(TaskName.BITBUCKET_PR_EDIT, kwargs))

    def reviewers(self, **kwargs: Unpack[BitbucketPrReviewersArgs]) -> PullRequest:
        """Add and remove reviewers of a Bitbucket pull request; other reviewers are kept."""
        return PullRequest.model_validate(self._client.task(TaskName.BITBUCKET_PR_REVIEWERS, kwargs))


class BitbucketRepoApi:
    """``atl bitbucket repo``"""

    def __init__(self, client: _Tasks) -> None:
        self._client = client

    def list(self, **kwargs: Unpack[BitbucketRepoListArgs]) -> _list[Repo]:
        """Bitbucket repositories of a workspace (Cloud) or project (Data Center), or the ones you belong to."""
        return [Repo.model_validate(item) for item in self._client.task(TaskName.BITBUCKET_REPO_LIST, kwargs)]

    def view(self, **kwargs: Unpack[BitbucketRepoViewArgs]) -> Repo:
        """One Bitbucket repository with its default branch and URL."""
        return Repo.model_validate(self._client.task(TaskName.BITBUCKET_REPO_VIEW, kwargs))


class BitbucketApi:
    """Bitbucket repositories, branches, pull requests and pipelines."""

    def __init__(self, client: _Tasks) -> None:
        self._client = client
        self.branch = BitbucketBranchApi(client)
        """``atl bitbucket branch``"""
        self.commit = BitbucketCommitApi(client)
        """``atl bitbucket commit``"""
        self.pipeline = BitbucketPipelineApi(client)
        """``atl bitbucket pipeline``"""
        self.pr = BitbucketPrApi(client)
        """``atl bitbucket pr``"""
        self.repo = BitbucketRepoApi(client)
        """``atl bitbucket repo``"""


class JsmDeskApi:
    """``atl jsm desk``"""

    def __init__(self, client: _Tasks) -> None:
        self._client = client

    def list(self, **kwargs: Unpack[JsmDeskListArgs]) -> _list[Desk]:
        """Jira Service Management service desks (id, project key, name)."""
        return [Desk.model_validate(item) for item in self._client.task(TaskName.JSM_DESK_LIST, kwargs)]


class JsmQueueApi:
    """``atl jsm queue``"""

    def __init__(self, client: _Tasks) -> None:
        self._client = client

    def list(self, **kwargs: Unpack[JsmQueueListArgs]) -> _list[Queue]:
        """Queues of a service desk with issue counts."""
        return [Queue.model_validate(item) for item in self._client.task(TaskName.JSM_QUEUE_LIST, kwargs)]

    def issues(self, **kwargs: Unpack[JsmQueueIssuesArgs]) -> _list[IssueSummary]:
        """Issues in a service desk queue (by queue ID or name)."""
        return [IssueSummary.model_validate(item) for item in self._client.task(TaskName.JSM_QUEUE_ISSUES, kwargs)]


class JsmRequestApi:
    """``atl jsm request``"""

    def __init__(self, client: _Tasks) -> None:
        self._client = client

    def types(self, **kwargs: Unpack[JsmRequestTypesArgs]) -> _list[RequestType]:
        """Request types of a service desk."""
        return [RequestType.model_validate(item) for item in self._client.task(TaskName.JSM_REQUEST_TYPES, kwargs)]

    def list(self, **kwargs: Unpack[JsmRequestListArgs]) -> _list[Request]:
        """Customer requests by status, optionally of one service desk."""
        return [Request.model_validate(item) for item in self._client.task(TaskName.JSM_REQUEST_LIST, kwargs)]

    def view(self, **kwargs: Unpack[JsmRequestViewArgs]) -> RequestDetail:
        """One customer request with its description and latest comments."""
        return RequestDetail.model_validate(self._client.task(TaskName.JSM_REQUEST_VIEW, kwargs))

    def create(self, **kwargs: Unpack[JsmRequestCreateArgs]) -> Request:
        """Raise a customer request, optionally for a customer."""
        return Request.model_validate(self._client.task(TaskName.JSM_REQUEST_CREATE, kwargs))

    def comment(self, **kwargs: Unpack[JsmRequestCommentArgs]) -> RequestComment:
        """Reply to a customer request, or add an internal note."""
        return RequestComment.model_validate(self._client.task(TaskName.JSM_REQUEST_COMMENT, kwargs))

    def transitions(self, **kwargs: Unpack[JsmRequestTransitionsArgs]) -> _list[RequestTransition]:
        """Transitions available on a customer request."""
        return [RequestTransition.model_validate(item) for item in self._client.task(TaskName.JSM_REQUEST_TRANSITIONS, kwargs)]

    def move(self, **kwargs: Unpack[JsmRequestMoveArgs]) -> RequestMoved:
        """Run a customer request transition by name or ID."""
        return RequestMoved.model_validate(self._client.task(TaskName.JSM_REQUEST_MOVE, kwargs))

    def approvals(self, **kwargs: Unpack[JsmRequestApprovalsArgs]) -> _list[Approval]:
        """Approvals of a JSM request: name, decision (pending, approved, declined), approvers, and whether you can answer."""
        return [Approval.model_validate(item) for item in self._client.task(TaskName.JSM_REQUEST_APPROVALS, kwargs)]

    def approve(self, **kwargs: Unpack[JsmRequestApproveArgs]) -> Approval:
        """Approve or decline (decision: decline) a JSM request's approval; without an ID, the one pending approval you can answer."""
        return Approval.model_validate(self._client.task(TaskName.JSM_REQUEST_APPROVE, kwargs))

    def participants(self, **kwargs: Unpack[JsmRequestParticipantsArgs]) -> Participants:
        """List a JSM request's participants, after adding and removing any (me, an email or a name)."""
        return Participants.model_validate(self._client.task(TaskName.JSM_REQUEST_PARTICIPANTS, kwargs))

    def sla(self, **kwargs: Unpack[JsmRequestSlaArgs]) -> _list[Sla]:
        """SLAs of a JSM request: state (ongoing, paused, met, breached), time remaining and goal."""
        return [Sla.model_validate(item) for item in self._client.task(TaskName.JSM_REQUEST_SLA, kwargs)]

    def attach(self, **kwargs: Unpack[JsmRequestAttachArgs]) -> _list[RequestAttachment]:
        """Attach local files to a JSM request (public unless internal)."""
        return [RequestAttachment.model_validate(item) for item in self._client.task(TaskName.JSM_REQUEST_ATTACH, kwargs)]

    def attachments(self, **kwargs: Unpack[JsmRequestAttachmentsArgs]) -> _list[RequestAttachment]:
        """Attachments of a JSM request with download URLs."""
        return [RequestAttachment.model_validate(item) for item in self._client.task(TaskName.JSM_REQUEST_ATTACHMENTS, kwargs)]


class JsmApi:
    """Jira Service Management desks, queues, requests and approvals."""

    def __init__(self, client: _Tasks) -> None:
        self._client = client
        self.desk = JsmDeskApi(client)
        """``atl jsm desk``"""
        self.queue = JsmQueueApi(client)
        """``atl jsm queue``"""
        self.request = JsmRequestApi(client)
        """``atl jsm request``"""


class TrelloBoardApi:
    """``atl trello board``"""

    def __init__(self, client: _Tasks) -> None:
        self._client = client

    def list(self, **kwargs: Unpack[TrelloBoardListArgs]) -> _list[TrelloBoard]:
        """Your open Trello boards."""
        return [TrelloBoard.model_validate(item) for item in self._client.task(TaskName.TRELLO_BOARD_LIST, kwargs)]


class TrelloCardApi:
    """``atl trello card``"""

    def __init__(self, client: _Tasks) -> None:
        self._client = client

    def list(self, **kwargs: Unpack[TrelloCardListArgs]) -> _list[Card]:
        """Cards of a Trello board or one of its lists, or your cards on every board (`mine`)."""
        return [Card.model_validate(item) for item in self._client.task(TaskName.TRELLO_CARD_LIST, kwargs)]

    def view(self, **kwargs: Unpack[TrelloCardViewArgs]) -> CardDetail:
        """One Trello card with list, board, description and latest comments."""
        return CardDetail.model_validate(self._client.task(TaskName.TRELLO_CARD_VIEW, kwargs))

    def create(self, **kwargs: Unpack[TrelloCardCreateArgs]) -> Card:
        """Create a Trello card in a list (by name with `board`, or by list ID)."""
        return Card.model_validate(self._client.task(TaskName.TRELLO_CARD_CREATE, kwargs))

    def move(self, **kwargs: Unpack[TrelloCardMoveArgs]) -> Card:
        """Move a Trello card to another list (by default on the card's own board)."""
        return Card.model_validate(self._client.task(TaskName.TRELLO_CARD_MOVE, kwargs))

    def comment(self, **kwargs: Unpack[TrelloCardCommentArgs]) -> CardComment:
        """Add a comment to a Trello card."""
        return CardComment.model_validate(self._client.task(TaskName.TRELLO_CARD_COMMENT, kwargs))

    def archive(self, **kwargs: Unpack[TrelloCardArchiveArgs]) -> Card:
        """Archive (close) a Trello card."""
        return Card.model_validate(self._client.task(TaskName.TRELLO_CARD_ARCHIVE, kwargs))

    def edit(self, **kwargs: Unpack[TrelloCardEditArgs]) -> Card:
        """Change a Trello card: name, description, due date, done, labels (by name or color) and members (username, name or me)."""
        return Card.model_validate(self._client.task(TaskName.TRELLO_CARD_EDIT, kwargs))


class TrelloChecklistApi:
    """``atl trello checklist``"""

    def __init__(self, client: _Tasks) -> None:
        self._client = client

    def list(self, **kwargs: Unpack[TrelloChecklistListArgs]) -> _list[Checklist]:
        """Checklists of a Trello card with their items and whether each is done."""
        return [Checklist.model_validate(item) for item in self._client.task(TaskName.TRELLO_CHECKLIST_LIST, kwargs)]

    def create(self, **kwargs: Unpack[TrelloChecklistCreateArgs]) -> Checklist:
        """Add a checklist with items to a Trello card."""
        return Checklist.model_validate(self._client.task(TaskName.TRELLO_CHECKLIST_CREATE, kwargs))

    def check(self, **kwargs: Unpack[TrelloChecklistCheckArgs]) -> CheckItem:
        """Tick (or with done: false, untick) a Trello checklist item by name or ID."""
        return CheckItem.model_validate(self._client.task(TaskName.TRELLO_CHECKLIST_CHECK, kwargs))


class TrelloListApi:
    """``atl trello list``"""

    def __init__(self, client: _Tasks) -> None:
        self._client = client

    def list(self, **kwargs: Unpack[TrelloListListArgs]) -> _list[TrelloList]:
        """Open lists of a Trello board (board by name, URL, short link or ID)."""
        return [TrelloList.model_validate(item) for item in self._client.task(TaskName.TRELLO_LIST_LIST, kwargs)]


class TrelloApi:
    """Trello boards, lists, cards and checklists."""

    def __init__(self, client: _Tasks) -> None:
        self._client = client
        self.board = TrelloBoardApi(client)
        """``atl trello board``"""
        self.card = TrelloCardApi(client)
        """``atl trello card``"""
        self.checklist = TrelloChecklistApi(client)
        """``atl trello checklist``"""
        self.list = TrelloListApi(client)
        """``atl trello list``"""


class AdminDirectoryApi:
    """``atl admin directory``"""

    def __init__(self, client: _Tasks) -> None:
        self._client = client

    def list(self, **kwargs: Unpack[AdminDirectoryListArgs]) -> _list[Directory]:
        """User directories of an Atlassian organization."""
        return [Directory.model_validate(item) for item in self._client.task(TaskName.ADMIN_DIRECTORY_LIST, kwargs)]


class AdminDomainApi:
    """``atl admin domain``"""

    def __init__(self, client: _Tasks) -> None:
        self._client = client

    def list(self, **kwargs: Unpack[AdminDomainListArgs]) -> _list[Domain]:
        """Domains of an Atlassian organization with their verification status."""
        return [Domain.model_validate(item) for item in self._client.task(TaskName.ADMIN_DOMAIN_LIST, kwargs)]


class AdminEventApi:
    """``atl admin event``"""

    def __init__(self, client: _Tasks) -> None:
        self._client = client

    def list(self, **kwargs: Unpack[AdminEventListArgs]) -> _list[Event]:
        """Audit log events of an Atlassian organization, newest first, filtered by words, dates and action."""
        return [Event.model_validate(item) for item in self._client.task(TaskName.ADMIN_EVENT_LIST, kwargs)]


class AdminGroupApi:
    """``atl admin group``"""

    def __init__(self, client: _Tasks) -> None:
        self._client = client

    def add_member(self, **kwargs: Unpack[AdminGroupAddMemberArgs]) -> Membership:
        """Add a managed user (email or account ID) to a group (name or ID) of an Atlassian organization."""
        return Membership.model_validate(self._client.task(TaskName.ADMIN_GROUP_ADD_MEMBER, kwargs))

    def remove_member(self, **kwargs: Unpack[AdminGroupRemoveMemberArgs]) -> Membership:
        """Remove a managed user from a group of an Atlassian organization."""
        return Membership.model_validate(self._client.task(TaskName.ADMIN_GROUP_REMOVE_MEMBER, kwargs))

    def list(self, **kwargs: Unpack[AdminGroupListArgs]) -> _list[Group]:
        """Groups of an Atlassian organization (every directory), optionally matching `search`."""
        return [Group.model_validate(item) for item in self._client.task(TaskName.ADMIN_GROUP_LIST, kwargs)]


class AdminOrgApi:
    """``atl admin org``"""

    def __init__(self, client: _Tasks) -> None:
        self._client = client

    def list(self, **kwargs: Unpack[AdminOrgListArgs]) -> _list[Org]:
        """Organizations the Atlassian Admin API key can see."""
        return [Org.model_validate(item) for item in self._client.task(TaskName.ADMIN_ORG_LIST, kwargs)]


class AdminUserApi:
    """``atl admin user``"""

    def __init__(self, client: _Tasks) -> None:
        self._client = client

    def list(self, **kwargs: Unpack[AdminUserListArgs]) -> _list[AdminUser]:
        """Managed users of an Atlassian organization (every directory), optionally matching `search`."""
        return [AdminUser.model_validate(item) for item in self._client.task(TaskName.ADMIN_USER_LIST, kwargs)]

    def suspend(self, **kwargs: Unpack[AdminUserSuspendArgs]) -> AdminUser:
        """Suspend a managed user's access to every product (by account ID or email)."""
        return AdminUser.model_validate(self._client.task(TaskName.ADMIN_USER_SUSPEND, kwargs))

    def restore(self, **kwargs: Unpack[AdminUserRestoreArgs]) -> AdminUser:
        """Restore a suspended managed user's access (by account ID or email)."""
        return AdminUser.model_validate(self._client.task(TaskName.ADMIN_USER_RESTORE, kwargs))


class AdminApi:
    """Atlassian Admin organizations, users, groups and audit events."""

    def __init__(self, client: _Tasks) -> None:
        self._client = client
        self.directory = AdminDirectoryApi(client)
        """``atl admin directory``"""
        self.domain = AdminDomainApi(client)
        """``atl admin domain``"""
        self.event = AdminEventApi(client)
        """``atl admin event``"""
        self.group = AdminGroupApi(client)
        """``atl admin group``"""
        self.org = AdminOrgApi(client)
        """``atl admin org``"""
        self.user = AdminUserApi(client)
        """``atl admin user``"""


class GeneratedClient:
    """The product namespaces of ``Client``; the client supplies ``task()``."""

    def task(self, name: str, args: Mapping[str, Any] | None = None) -> Any:
        raise NotImplementedError

    def __init__(self) -> None:
        self.jira = JiraApi(self)
        """Jira and Jira Software: issues, comments, sprints, boards, filters and JQL."""
        self.confluence = ConfluenceApi(self)
        """Confluence pages, blogs, spaces, comments, labels and attachments."""
        self.bitbucket = BitbucketApi(self)
        """Bitbucket repositories, branches, pull requests and pipelines."""
        self.jsm = JsmApi(self)
        """Jira Service Management desks, queues, requests and approvals."""
        self.trello = TrelloApi(self)
        """Trello boards, lists, cards and checklists."""
        self.admin = AdminApi(self)
        """Atlassian Admin organizations, users, groups and audit events."""


class AsyncJiraBoardApi:
    """``atl jira board``"""

    def __init__(self, client: _AsyncTasks) -> None:
        self._client = client

    async def list(self, **kwargs: Unpack[JiraBoardListArgs]) -> _list[Board]:
        """Jira Software boards, optionally of one project."""
        return [Board.model_validate(item) for item in await self._client.task(TaskName.JIRA_BOARD_LIST, kwargs)]


class AsyncJiraCommentApi:
    """``atl jira comment``"""

    def __init__(self, client: _AsyncTasks) -> None:
        self._client = client

    async def list(self, **kwargs: Unpack[JiraCommentListArgs]) -> _list[Comment]:
        """Every comment of a Jira issue, oldest first, with IDs, as Markdown."""
        return [Comment.model_validate(item) for item in await self._client.task(TaskName.JIRA_COMMENT_LIST, kwargs)]

    async def edit(self, **kwargs: Unpack[JiraCommentEditArgs]) -> Comment:
        """Replace the text of a Jira comment (Markdown)."""
        return Comment.model_validate(await self._client.task(TaskName.JIRA_COMMENT_EDIT, kwargs))

    async def delete(self, **kwargs: Unpack[JiraCommentDeleteArgs]) -> CommentDeleted:
        """Delete a Jira comment. Cannot be undone."""
        return CommentDeleted.model_validate(await self._client.task(TaskName.JIRA_COMMENT_DELETE, kwargs))


class AsyncJiraComponentApi:
    """``atl jira component``"""

    def __init__(self, client: _AsyncTasks) -> None:
        self._client = client

    async def list(self, **kwargs: Unpack[JiraComponentListArgs]) -> _list[Component]:
        """Components of a Jira project with their leads."""
        return [Component.model_validate(item) for item in await self._client.task(TaskName.JIRA_COMPONENT_LIST, kwargs)]


class AsyncJiraEpicApi:
    """``atl jira epic``"""

    def __init__(self, client: _AsyncTasks) -> None:
        self._client = client

    async def list(self, **kwargs: Unpack[JiraEpicListArgs]) -> _list[IssueSummary]:
        """Open epics, optionally of one project."""
        return [IssueSummary.model_validate(item) for item in await self._client.task(TaskName.JIRA_EPIC_LIST, kwargs)]

    async def add(self, **kwargs: Unpack[JiraEpicAddArgs]) -> EpicIssues:
        """Move issues into an epic."""
        return EpicIssues.model_validate(await self._client.task(TaskName.JIRA_EPIC_ADD, kwargs))

    async def remove(self, **kwargs: Unpack[JiraEpicRemoveArgs]) -> EpicIssues:
        """Remove issues from their epic."""
        return EpicIssues.model_validate(await self._client.task(TaskName.JIRA_EPIC_REMOVE, kwargs))


class AsyncJiraFieldApi:
    """``atl jira field``"""

    def __init__(self, client: _AsyncTasks) -> None:
        self._client = client

    async def list(self, **kwargs: Unpack[JiraFieldListArgs]) -> _list[Field]:
        """Jira system and custom fields (ID, name, type), optionally filtered by part of the name. Use the names in the `fields` argument of issue create, edit and view."""
        return [Field.model_validate(item) for item in await self._client.task(TaskName.JIRA_FIELD_LIST, kwargs)]


class AsyncJiraFilterApi:
    """``atl jira filter``"""

    def __init__(self, client: _AsyncTasks) -> None:
        self._client = client

    async def list(self, **kwargs: Unpack[JiraFilterListArgs]) -> _list[Filter]:
        """Saved Jira filters (Cloud: all visible, Data Center: favourites)."""
        return [Filter.model_validate(item) for item in await self._client.task(TaskName.JIRA_FILTER_LIST, kwargs)]

    async def run(self, **kwargs: Unpack[JiraFilterRunArgs]) -> IssueList:
        """Run a saved Jira filter by ID or exact name; returns issue records."""
        return IssueList.model_validate(await self._client.task(TaskName.JIRA_FILTER_RUN, kwargs))


class AsyncJiraIssueApi:
    """``atl jira issue``"""

    def __init__(self, client: _AsyncTasks) -> None:
        self._client = client

    async def list(self, **kwargs: Unpack[JiraIssueListArgs]) -> IssueList:
        """Search Jira issues with filters (or extra JQL). Returns small records: key, summary, status, type, assignee, priority, updated, url. With no filters: your unresolved issues."""
        return IssueList.model_validate(await self._client.task(TaskName.JIRA_ISSUE_LIST, kwargs))

    async def count(self, **kwargs: Unpack[JiraIssueCountArgs]) -> IssueCount:
        """Count Jira issues matching the same filters as issue_list."""
        return IssueCount.model_validate(await self._client.task(TaskName.JIRA_ISSUE_COUNT, kwargs))

    async def view(self, **kwargs: Unpack[JiraIssueViewArgs]) -> IssueDetail:
        """One Jira issue with description and latest comments as Markdown, plus any `fields` asked for by name."""
        return IssueDetail.model_validate(await self._client.task(TaskName.JIRA_ISSUE_VIEW, kwargs))

    async def delete(self, **kwargs: Unpack[JiraIssueDeleteArgs]) -> IssueDeleted:
        """Delete a Jira issue (with `subtasks: true`, also its sub-tasks). Cannot be undone."""
        return IssueDeleted.model_validate(await self._client.task(TaskName.JIRA_ISSUE_DELETE, kwargs))

    async def watchers(self, **kwargs: Unpack[JiraIssueWatchersArgs]) -> Watchers:
        """Who watches a Jira issue, and whether you do."""
        return Watchers.model_validate(await self._client.task(TaskName.JIRA_ISSUE_WATCHERS, kwargs))

    async def watch(self, **kwargs: Unpack[JiraIssueWatchArgs]) -> Watching:
        """Add a watcher to a Jira issue (default: you)."""
        return Watching.model_validate(await self._client.task(TaskName.JIRA_ISSUE_WATCH, kwargs))

    async def unwatch(self, **kwargs: Unpack[JiraIssueUnwatchArgs]) -> Watching:
        """Remove a watcher from a Jira issue (default: you)."""
        return Watching.model_validate(await self._client.task(TaskName.JIRA_ISSUE_UNWATCH, kwargs))

    async def create(self, **kwargs: Unpack[JiraIssueCreateArgs]) -> Created:
        """Create a Jira issue; the description is Markdown."""
        return Created.model_validate(await self._client.task(TaskName.JIRA_ISSUE_CREATE, kwargs))

    async def edit(self, **kwargs: Unpack[JiraIssueEditArgs]) -> IssueUpdated:
        """Change a Jira issue's summary, description (Markdown), priority or labels."""
        return IssueUpdated.model_validate(await self._client.task(TaskName.JIRA_ISSUE_EDIT, kwargs))

    async def assign(self, **kwargs: Unpack[JiraIssueAssignArgs]) -> Assigned:
        """Assign a Jira issue to me, nobody (x), or a user."""
        return Assigned.model_validate(await self._client.task(TaskName.JIRA_ISSUE_ASSIGN, kwargs))

    async def transitions(self, **kwargs: Unpack[JiraIssueTransitionsArgs]) -> _list[Transition]:
        """Statuses a Jira issue can move to."""
        return [Transition.model_validate(item) for item in await self._client.task(TaskName.JIRA_ISSUE_TRANSITIONS, kwargs)]

    async def move(self, **kwargs: Unpack[JiraIssueMoveArgs]) -> Moved:
        """Move a Jira issue to another status (by status or transition name)."""
        return Moved.model_validate(await self._client.task(TaskName.JIRA_ISSUE_MOVE, kwargs))

    async def comment(self, **kwargs: Unpack[JiraIssueCommentArgs]) -> Comment:
        """Add a Markdown comment to a Jira issue."""
        return Comment.model_validate(await self._client.task(TaskName.JIRA_ISSUE_COMMENT, kwargs))

    async def link(self, **kwargs: Unpack[JiraIssueLinkArgs]) -> Linked:
        """Link two Jira issues so that 'from <kind> to' reads true, for example from A-1 kind blocks to A-2."""
        return Linked.model_validate(await self._client.task(TaskName.JIRA_ISSUE_LINK, kwargs))

    async def unlink(self, **kwargs: Unpack[JiraIssueUnlinkArgs]) -> Unlinked:
        """Remove the links between two Jira issues (optionally only one link type)."""
        return Unlinked.model_validate(await self._client.task(TaskName.JIRA_ISSUE_UNLINK, kwargs))

    async def remote_link(self, **kwargs: Unpack[JiraIssueRemoteLinkArgs]) -> RemoteLink:
        """Add a web link (URL and title) to a Jira issue."""
        return RemoteLink.model_validate(await self._client.task(TaskName.JIRA_ISSUE_REMOTE_LINK, kwargs))

    async def history(self, **kwargs: Unpack[JiraIssueHistoryArgs]) -> _list[Change]:
        """Field changes of a Jira issue, newest first: when, who, field, from, to."""
        return [Change.model_validate(item) for item in await self._client.task(TaskName.JIRA_ISSUE_HISTORY, kwargs)]

    async def link_types(self, **kwargs: Unpack[JiraIssueLinkTypesArgs]) -> _list[LinkType]:
        """Jira issue link types with their outward and inward phrases."""
        return [LinkType.model_validate(item) for item in await self._client.task(TaskName.JIRA_ISSUE_LINK_TYPES, kwargs)]

    async def clone(self, **kwargs: Unpack[JiraIssueCloneArgs]) -> Created:
        """Copy a Jira issue (summary, description, type, labels, priority, components, parent)."""
        return Created.model_validate(await self._client.task(TaskName.JIRA_ISSUE_CLONE, kwargs))

    async def worklog(self, **kwargs: Unpack[JiraIssueWorklogArgs]) -> WorklogAdded:
        """Log time on a Jira issue."""
        return WorklogAdded.model_validate(await self._client.task(TaskName.JIRA_ISSUE_WORKLOG, kwargs))

    async def attach(self, **kwargs: Unpack[JiraIssueAttachArgs]) -> _list[Attachment]:
        """Upload local files to a Jira issue."""
        return [Attachment.model_validate(item) for item in await self._client.task(TaskName.JIRA_ISSUE_ATTACH, kwargs)]

    async def attachments(self, **kwargs: Unpack[JiraIssueAttachmentsArgs]) -> _list[Attachment]:
        """List a Jira issue's attachments."""
        return [Attachment.model_validate(item) for item in await self._client.task(TaskName.JIRA_ISSUE_ATTACHMENTS, kwargs)]

    async def types(self, **kwargs: Unpack[JiraIssueTypesArgs]) -> _list[str]:
        """Issue type names of a Jira project (without sub-task types), for the `type` argument of jira_issue_create."""
        return await self._client.task(TaskName.JIRA_ISSUE_TYPES, kwargs)


class AsyncJiraJqlApi:
    """``atl jira jql``"""

    def __init__(self, client: _AsyncTasks) -> None:
        self._client = client

    async def check(self, **kwargs: Unpack[JiraJqlCheckArgs]) -> JqlCheck:
        """Validate JQL with Jira's parser; returns valid, errors, warnings."""
        return JqlCheck.model_validate(await self._client.task(TaskName.JIRA_JQL_CHECK, kwargs))

    async def fields(self, **kwargs: Unpack[JiraJqlFieldsArgs]) -> JqlReference:
        """Fields (with operators) and functions usable in JQL on this site."""
        return JqlReference.model_validate(await self._client.task(TaskName.JIRA_JQL_FIELDS, kwargs))

    async def suggest(self, **kwargs: Unpack[JiraJqlSuggestArgs]) -> _list[Suggestion]:
        """Value suggestions for a JQL field, for example field=status prefix=In."""
        return [Suggestion.model_validate(item) for item in await self._client.task(TaskName.JIRA_JQL_SUGGEST, kwargs)]


class AsyncJiraProjectApi:
    """``atl jira project``"""

    def __init__(self, client: _AsyncTasks) -> None:
        self._client = client

    async def list(self, **kwargs: Unpack[JiraProjectListArgs]) -> _list[Project]:
        """Jira projects."""
        return [Project.model_validate(item) for item in await self._client.task(TaskName.JIRA_PROJECT_LIST, kwargs)]


class AsyncJiraReleaseApi:
    """``atl jira release``"""

    def __init__(self, client: _AsyncTasks) -> None:
        self._client = client

    async def list(self, **kwargs: Unpack[JiraReleaseListArgs]) -> _list[Release]:
        """Versions (releases) of a Jira project: name, released, archived, release date."""
        return [Release.model_validate(item) for item in await self._client.task(TaskName.JIRA_RELEASE_LIST, kwargs)]


class AsyncJiraSprintApi:
    """``atl jira sprint``"""

    def __init__(self, client: _AsyncTasks) -> None:
        self._client = client

    async def create(self, **kwargs: Unpack[JiraSprintCreateArgs]) -> Sprint:
        """Create a future sprint on a board (default: the project's first board)."""
        return Sprint.model_validate(await self._client.task(TaskName.JIRA_SPRINT_CREATE, kwargs))

    async def start(self, **kwargs: Unpack[JiraSprintStartArgs]) -> Sprint:
        """Start a future sprint; Jira needs a start and an end date."""
        return Sprint.model_validate(await self._client.task(TaskName.JIRA_SPRINT_START, kwargs))

    async def close(self, **kwargs: Unpack[JiraSprintCloseArgs]) -> Sprint:
        """Complete an active sprint."""
        return Sprint.model_validate(await self._client.task(TaskName.JIRA_SPRINT_CLOSE, kwargs))

    async def list(self, **kwargs: Unpack[JiraSprintListArgs]) -> BoardSprints:
        """Sprints of a board."""
        return BoardSprints.model_validate(await self._client.task(TaskName.JIRA_SPRINT_LIST, kwargs))

    async def add(self, **kwargs: Unpack[JiraSprintAddArgs]) -> SprintIssues:
        """Move issues into a sprint."""
        return SprintIssues.model_validate(await self._client.task(TaskName.JIRA_SPRINT_ADD, kwargs))


class AsyncJiraWorklogApi:
    """``atl jira worklog``"""

    def __init__(self, client: _AsyncTasks) -> None:
        self._client = client

    async def list(self, **kwargs: Unpack[JiraWorklogListArgs]) -> _list[Worklog]:
        """Time logged on a Jira issue: author, time spent, start, comment."""
        return [Worklog.model_validate(item) for item in await self._client.task(TaskName.JIRA_WORKLOG_LIST, kwargs)]


class AsyncJiraApi:
    """Jira and Jira Software: issues, comments, sprints, boards, filters and JQL."""

    def __init__(self, client: _AsyncTasks) -> None:
        self._client = client
        self.board = AsyncJiraBoardApi(client)
        """``atl jira board``"""
        self.comment = AsyncJiraCommentApi(client)
        """``atl jira comment``"""
        self.component = AsyncJiraComponentApi(client)
        """``atl jira component``"""
        self.epic = AsyncJiraEpicApi(client)
        """``atl jira epic``"""
        self.field = AsyncJiraFieldApi(client)
        """``atl jira field``"""
        self.filter = AsyncJiraFilterApi(client)
        """``atl jira filter``"""
        self.issue = AsyncJiraIssueApi(client)
        """``atl jira issue``"""
        self.jql = AsyncJiraJqlApi(client)
        """``atl jira jql``"""
        self.project = AsyncJiraProjectApi(client)
        """``atl jira project``"""
        self.release = AsyncJiraReleaseApi(client)
        """``atl jira release``"""
        self.sprint = AsyncJiraSprintApi(client)
        """``atl jira sprint``"""
        self.worklog = AsyncJiraWorklogApi(client)
        """``atl jira worklog``"""

    async def me(self, **kwargs: Unpack[JiraMeArgs]) -> User:
        """The signed-in Jira user."""
        return User.model_validate(await self._client.task(TaskName.JIRA_ME, kwargs))


class AsyncConfluenceAttachmentApi:
    """``atl confluence attachment``"""

    def __init__(self, client: _AsyncTasks) -> None:
        self._client = client

    async def delete(self, **kwargs: Unpack[ConfluenceAttachmentDeleteArgs]) -> Deleted:
        """Delete a Confluence attachment by ID."""
        return Deleted.model_validate(await self._client.task(TaskName.CONFLUENCE_ATTACHMENT_DELETE, kwargs))


class AsyncConfluenceBlogApi:
    """``atl confluence blog``"""

    def __init__(self, client: _AsyncTasks) -> None:
        self._client = client

    async def list(self, **kwargs: Unpack[ConfluenceBlogListArgs]) -> _list[PageSummary]:
        """Blog posts of a Confluence space, newest first."""
        return [PageSummary.model_validate(item) for item in await self._client.task(TaskName.CONFLUENCE_BLOG_LIST, kwargs)]

    async def get(self, **kwargs: Unpack[ConfluenceBlogGetArgs]) -> Page:
        """One Confluence blog post with its body as Markdown."""
        return Page.model_validate(await self._client.task(TaskName.CONFLUENCE_BLOG_GET, kwargs))

    async def create(self, **kwargs: Unpack[ConfluenceBlogCreateArgs]) -> PageSummary:
        """Publish a Confluence blog post from Markdown."""
        return PageSummary.model_validate(await self._client.task(TaskName.CONFLUENCE_BLOG_CREATE, kwargs))


class AsyncConfluenceCommentApi:
    """``atl confluence comment``"""

    def __init__(self, client: _AsyncTasks) -> None:
        self._client = client

    async def delete(self, **kwargs: Unpack[ConfluenceCommentDeleteArgs]) -> Deleted:
        """Delete a Confluence comment by ID."""
        return Deleted.model_validate(await self._client.task(TaskName.CONFLUENCE_COMMENT_DELETE, kwargs))


class AsyncConfluencePageApi:
    """``atl confluence page``"""

    def __init__(self, client: _AsyncTasks) -> None:
        self._client = client

    async def get(self, **kwargs: Unpack[ConfluencePageGetArgs]) -> PageView:
        """A Confluence page as Markdown (with title, space, version, url)."""
        return PageView.model_validate(await self._client.task(TaskName.CONFLUENCE_PAGE_GET, kwargs))

    async def create(self, **kwargs: Unpack[ConfluencePageCreateArgs]) -> PageSummary:
        """Create a Confluence page from Markdown (or storage format)."""
        return PageSummary.model_validate(await self._client.task(TaskName.CONFLUENCE_PAGE_CREATE, kwargs))

    async def update(self, **kwargs: Unpack[ConfluencePageUpdateArgs]) -> PageSummary:
        """Replace a Confluence page's title and/or body; the version is bumped automatically."""
        return PageSummary.model_validate(await self._client.task(TaskName.CONFLUENCE_PAGE_UPDATE, kwargs))

    async def delete(self, **kwargs: Unpack[ConfluencePageDeleteArgs]) -> Deleted:
        """Delete a Confluence page."""
        return Deleted.model_validate(await self._client.task(TaskName.CONFLUENCE_PAGE_DELETE, kwargs))

    async def children(self, **kwargs: Unpack[ConfluencePageChildrenArgs]) -> _list[TreeNode]:
        """Children of a Confluence page (Cloud: also folders), or with `recursive` the whole tree, depth first, each row with type, depth and parent."""
        return [TreeNode.model_validate(item) for item in await self._client.task(TaskName.CONFLUENCE_PAGE_CHILDREN, kwargs)]

    async def find(self, **kwargs: Unpack[ConfluencePageFindArgs]) -> _list[SearchHit]:
        """Confluence pages whose title contains the words, optionally in one space."""
        return [SearchHit.model_validate(item) for item in await self._client.task(TaskName.CONFLUENCE_PAGE_FIND, kwargs)]

    async def attach(self, **kwargs: Unpack[ConfluencePageAttachArgs]) -> _list[PageAttachment]:
        """Upload local files to a Confluence page."""
        return [PageAttachment.model_validate(item) for item in await self._client.task(TaskName.CONFLUENCE_PAGE_ATTACH, kwargs)]

    async def attachments(self, **kwargs: Unpack[ConfluencePageAttachmentsArgs]) -> _list[PageAttachment]:
        """Attachments of a Confluence page with download URLs."""
        return [PageAttachment.model_validate(item) for item in await self._client.task(TaskName.CONFLUENCE_PAGE_ATTACHMENTS, kwargs)]

    async def comments(self, **kwargs: Unpack[ConfluencePageCommentsArgs]) -> _list[PageComment]:
        """Footer (or with `inline`, inline) comments of a Confluence page, oldest first, as Markdown, with the replied-to `parent`."""
        return [PageComment.model_validate(item) for item in await self._client.task(TaskName.CONFLUENCE_PAGE_COMMENTS, kwargs)]

    async def comment(self, **kwargs: Unpack[ConfluencePageCommentArgs]) -> PageComment:
        """Add a footer comment (Markdown) to a Confluence page, or a reply with `parent`."""
        return PageComment.model_validate(await self._client.task(TaskName.CONFLUENCE_PAGE_COMMENT, kwargs))

    async def export(self, **kwargs: Unpack[ConfluencePageExportArgs]) -> _list[Exported]:
        """Export a Confluence page (and with `recursive` its tree) to local Markdown folders: <title>/index.md with front matter and _attachments/. Changes nothing on the server."""
        return [Exported.model_validate(item) for item in await self._client.task(TaskName.CONFLUENCE_PAGE_EXPORT, kwargs)]

    async def push(self, **kwargs: Unpack[ConfluencePagePushArgs]) -> _list[Pushed]:
        """Publish local Markdown files: update the page in each file's front matter (refusing pages changed on the server unless `force`, skipping unchanged ones), create pages for new files under their folder's index.md, upload referenced images, and write ids and versions back."""
        return [Pushed.model_validate(item) for item in await self._client.task(TaskName.CONFLUENCE_PAGE_PUSH, kwargs)]

    async def copy_tree(self, **kwargs: Unpack[ConfluencePageCopyTreeArgs]) -> _list[Copied]:
        """Copy a Confluence page and the pages below it under another page (titles, bodies, labels)."""
        return [Copied.model_validate(item) for item in await self._client.task(TaskName.CONFLUENCE_PAGE_COPY_TREE, kwargs)]

    async def labels(self, **kwargs: Unpack[ConfluencePageLabelsArgs]) -> _list[str]:
        """Labels of a Confluence page."""
        return await self._client.task(TaskName.CONFLUENCE_PAGE_LABELS, kwargs)

    async def label(self, **kwargs: Unpack[ConfluencePageLabelArgs]) -> _list[str]:
        """Add and remove labels on a Confluence page; returns the resulting labels."""
        return await self._client.task(TaskName.CONFLUENCE_PAGE_LABEL, kwargs)

    async def move(self, **kwargs: Unpack[ConfluencePageMoveArgs]) -> PageSummary:
        """Move a Confluence page under another page (position append) or next to it (before/after, Cloud only)."""
        return PageSummary.model_validate(await self._client.task(TaskName.CONFLUENCE_PAGE_MOVE, kwargs))

    async def history(self, **kwargs: Unpack[ConfluencePageHistoryArgs]) -> _list[PageVersion]:
        """Versions of a Confluence page, newest first: number, author, date, message. Read one with confluence_page_get and `version`."""
        return [PageVersion.model_validate(item) for item in await self._client.task(TaskName.CONFLUENCE_PAGE_HISTORY, kwargs)]


class AsyncConfluenceSpaceApi:
    """``atl confluence space``"""

    def __init__(self, client: _AsyncTasks) -> None:
        self._client = client

    async def create(self, **kwargs: Unpack[ConfluenceSpaceCreateArgs]) -> Space:
        """Create a Confluence space with a key, a name and an optional plain-text description."""
        return Space.model_validate(await self._client.task(TaskName.CONFLUENCE_SPACE_CREATE, kwargs))

    async def list(self, **kwargs: Unpack[ConfluenceSpaceListArgs]) -> _list[Space]:
        """Confluence spaces."""
        return [Space.model_validate(item) for item in await self._client.task(TaskName.CONFLUENCE_SPACE_LIST, kwargs)]

    async def pages(self, **kwargs: Unpack[ConfluenceSpacePagesArgs]) -> _list[PageSummary]:
        """Top-level pages of a Confluence space."""
        return [PageSummary.model_validate(item) for item in await self._client.task(TaskName.CONFLUENCE_SPACE_PAGES, kwargs)]


class AsyncConfluenceApi:
    """Confluence pages, blogs, spaces, comments, labels and attachments."""

    def __init__(self, client: _AsyncTasks) -> None:
        self._client = client
        self.attachment = AsyncConfluenceAttachmentApi(client)
        """``atl confluence attachment``"""
        self.blog = AsyncConfluenceBlogApi(client)
        """``atl confluence blog``"""
        self.comment = AsyncConfluenceCommentApi(client)
        """``atl confluence comment``"""
        self.page = AsyncConfluencePageApi(client)
        """``atl confluence page``"""
        self.space = AsyncConfluenceSpaceApi(client)
        """``atl confluence space``"""

    async def search(self, **kwargs: Unpack[ConfluenceSearchArgs]) -> SearchResults:
        """Search Confluence by words or CQL; returns id, type, title, space, excerpt, url."""
        return SearchResults.model_validate(await self._client.task(TaskName.CONFLUENCE_SEARCH, kwargs))


class AsyncBitbucketBranchApi:
    """``atl bitbucket branch``"""

    def __init__(self, client: _AsyncTasks) -> None:
        self._client = client

    async def list(self, **kwargs: Unpack[BitbucketBranchListArgs]) -> _list[Branch]:
        """Branches of a Bitbucket repository, most recently changed first."""
        return [Branch.model_validate(item) for item in await self._client.task(TaskName.BITBUCKET_BRANCH_LIST, kwargs)]

    async def create(self, **kwargs: Unpack[BitbucketBranchCreateArgs]) -> Branch:
        """Create a branch from another branch, a tag or a commit (default: the main branch)."""
        return Branch.model_validate(await self._client.task(TaskName.BITBUCKET_BRANCH_CREATE, kwargs))

    async def delete(self, **kwargs: Unpack[BitbucketBranchDeleteArgs]) -> BranchDeleted:
        """Delete a branch. Unmerged commits on it can be lost."""
        return BranchDeleted.model_validate(await self._client.task(TaskName.BITBUCKET_BRANCH_DELETE, kwargs))


class AsyncBitbucketCommitApi:
    """``atl bitbucket commit``"""

    def __init__(self, client: _AsyncTasks) -> None:
        self._client = client

    async def list(self, **kwargs: Unpack[BitbucketCommitListArgs]) -> _list[Commit]:
        """Recent commits of a branch, newest first: hash, author, date, first line of the message."""
        return [Commit.model_validate(item) for item in await self._client.task(TaskName.BITBUCKET_COMMIT_LIST, kwargs)]


class AsyncBitbucketPipelineApi:
    """``atl bitbucket pipeline``"""

    def __init__(self, client: _AsyncTasks) -> None:
        self._client = client

    async def list(self, **kwargs: Unpack[BitbucketPipelineListArgs]) -> _list[Pipeline]:
        """Latest Bitbucket Pipelines runs, newest first (Cloud only)."""
        return [Pipeline.model_validate(item) for item in await self._client.task(TaskName.BITBUCKET_PIPELINE_LIST, kwargs)]

    async def run(self, **kwargs: Unpack[BitbucketPipelineRunArgs]) -> Pipeline:
        """Start the pipeline of a branch, or a custom pipeline (Cloud only)."""
        return Pipeline.model_validate(await self._client.task(TaskName.BITBUCKET_PIPELINE_RUN, kwargs))

    async def stop(self, **kwargs: Unpack[BitbucketPipelineStopArgs]) -> Pipeline:
        """Stop a running Bitbucket pipeline by build number or UUID (Cloud only)."""
        return Pipeline.model_validate(await self._client.task(TaskName.BITBUCKET_PIPELINE_STOP, kwargs))

    async def view(self, **kwargs: Unpack[BitbucketPipelineViewArgs]) -> PipelineDetail:
        """One Bitbucket pipeline by build number or UUID, with each step's state and result (Cloud only)."""
        return PipelineDetail.model_validate(await self._client.task(TaskName.BITBUCKET_PIPELINE_VIEW, kwargs))

    async def logs(self, **kwargs: Unpack[BitbucketPipelineLogsArgs]) -> PipelineLog:
        """Log text of one pipeline step (by name or UUID) or of every step (Cloud only)."""
        return PipelineLog.model_validate(await self._client.task(TaskName.BITBUCKET_PIPELINE_LOGS, kwargs))


class AsyncBitbucketPrApi:
    """``atl bitbucket pr``"""

    def __init__(self, client: _AsyncTasks) -> None:
        self._client = client

    async def list(self, **kwargs: Unpack[BitbucketPrListArgs]) -> _list[PullRequest]:
        """Pull requests of a Bitbucket repository by state."""
        return [PullRequest.model_validate(item) for item in await self._client.task(TaskName.BITBUCKET_PR_LIST, kwargs)]

    async def view(self, **kwargs: Unpack[BitbucketPrViewArgs]) -> PullRequest:
        """One pull request with description, reviewers and approvals."""
        return PullRequest.model_validate(await self._client.task(TaskName.BITBUCKET_PR_VIEW, kwargs))

    async def create(self, **kwargs: Unpack[BitbucketPrCreateArgs]) -> PullRequest:
        """Open a pull request from a source branch."""
        return PullRequest.model_validate(await self._client.task(TaskName.BITBUCKET_PR_CREATE, kwargs))

    async def merge(self, **kwargs: Unpack[BitbucketPrMergeArgs]) -> PullRequest:
        """Merge a pull request."""
        return PullRequest.model_validate(await self._client.task(TaskName.BITBUCKET_PR_MERGE, kwargs))

    async def approve(self, **kwargs: Unpack[BitbucketPrApproveArgs]) -> PrApproval:
        """Approve a pull request as the signed-in user."""
        return PrApproval.model_validate(await self._client.task(TaskName.BITBUCKET_PR_APPROVE, kwargs))

    async def comment(self, **kwargs: Unpack[BitbucketPrCommentArgs]) -> PrCommented:
        """Add a Markdown comment to a pull request."""
        return PrCommented.model_validate(await self._client.task(TaskName.BITBUCKET_PR_COMMENT, kwargs))

    async def diff(self, **kwargs: Unpack[BitbucketPrDiffArgs]) -> PrDiff:
        """The unified diff of a pull request."""
        return PrDiff.model_validate(await self._client.task(TaskName.BITBUCKET_PR_DIFF, kwargs))

    async def comments(self, **kwargs: Unpack[BitbucketPrCommentsArgs]) -> _list[PrComment]:
        """Comments of a pull request, oldest first, as Markdown: general and inline (path, line), replies flattened with `parent`."""
        return [PrComment.model_validate(item) for item in await self._client.task(TaskName.BITBUCKET_PR_COMMENTS, kwargs)]

    async def checks(self, **kwargs: Unpack[BitbucketPrChecksArgs]) -> PrChecks:
        """Build statuses of a pull request's latest commit, with `passed` (none failed) and `pending` (some still running)."""
        return PrChecks.model_validate(await self._client.task(TaskName.BITBUCKET_PR_CHECKS, kwargs))

    async def unapprove(self, **kwargs: Unpack[BitbucketPrUnapproveArgs]) -> PrApproval:
        """Remove your approval from a pull request."""
        return PrApproval.model_validate(await self._client.task(TaskName.BITBUCKET_PR_UNAPPROVE, kwargs))

    async def ready(self, **kwargs: Unpack[BitbucketPrReadyArgs]) -> PullRequest:
        """Mark a draft pull request as ready for review (or, with `draft: true`, as a draft again)."""
        return PullRequest.model_validate(await self._client.task(TaskName.BITBUCKET_PR_READY, kwargs))

    async def decline(self, **kwargs: Unpack[BitbucketPrDeclineArgs]) -> PullRequest:
        """Decline a Bitbucket pull request (Cloud cannot reopen it)."""
        return PullRequest.model_validate(await self._client.task(TaskName.BITBUCKET_PR_DECLINE, kwargs))

    async def request_changes(self, **kwargs: Unpack[BitbucketPrRequestChangesArgs]) -> ChangesRequested:
        """Mark a Bitbucket pull request as needing changes by you (Cloud: request changes; Data Center: needs work)."""
        return ChangesRequested.model_validate(await self._client.task(TaskName.BITBUCKET_PR_REQUEST_CHANGES, kwargs))

    async def edit(self, **kwargs: Unpack[BitbucketPrEditArgs]) -> PullRequest:
        """Change a Bitbucket pull request's title, description (Markdown) or destination branch."""
        return PullRequest.model_validate(await self._client.task(TaskName.BITBUCKET_PR_EDIT, kwargs))

    async def reviewers(self, **kwargs: Unpack[BitbucketPrReviewersArgs]) -> PullRequest:
        """Add and remove reviewers of a Bitbucket pull request; other reviewers are kept."""
        return PullRequest.model_validate(await self._client.task(TaskName.BITBUCKET_PR_REVIEWERS, kwargs))


class AsyncBitbucketRepoApi:
    """``atl bitbucket repo``"""

    def __init__(self, client: _AsyncTasks) -> None:
        self._client = client

    async def list(self, **kwargs: Unpack[BitbucketRepoListArgs]) -> _list[Repo]:
        """Bitbucket repositories of a workspace (Cloud) or project (Data Center), or the ones you belong to."""
        return [Repo.model_validate(item) for item in await self._client.task(TaskName.BITBUCKET_REPO_LIST, kwargs)]

    async def view(self, **kwargs: Unpack[BitbucketRepoViewArgs]) -> Repo:
        """One Bitbucket repository with its default branch and URL."""
        return Repo.model_validate(await self._client.task(TaskName.BITBUCKET_REPO_VIEW, kwargs))


class AsyncBitbucketApi:
    """Bitbucket repositories, branches, pull requests and pipelines."""

    def __init__(self, client: _AsyncTasks) -> None:
        self._client = client
        self.branch = AsyncBitbucketBranchApi(client)
        """``atl bitbucket branch``"""
        self.commit = AsyncBitbucketCommitApi(client)
        """``atl bitbucket commit``"""
        self.pipeline = AsyncBitbucketPipelineApi(client)
        """``atl bitbucket pipeline``"""
        self.pr = AsyncBitbucketPrApi(client)
        """``atl bitbucket pr``"""
        self.repo = AsyncBitbucketRepoApi(client)
        """``atl bitbucket repo``"""


class AsyncJsmDeskApi:
    """``atl jsm desk``"""

    def __init__(self, client: _AsyncTasks) -> None:
        self._client = client

    async def list(self, **kwargs: Unpack[JsmDeskListArgs]) -> _list[Desk]:
        """Jira Service Management service desks (id, project key, name)."""
        return [Desk.model_validate(item) for item in await self._client.task(TaskName.JSM_DESK_LIST, kwargs)]


class AsyncJsmQueueApi:
    """``atl jsm queue``"""

    def __init__(self, client: _AsyncTasks) -> None:
        self._client = client

    async def list(self, **kwargs: Unpack[JsmQueueListArgs]) -> _list[Queue]:
        """Queues of a service desk with issue counts."""
        return [Queue.model_validate(item) for item in await self._client.task(TaskName.JSM_QUEUE_LIST, kwargs)]

    async def issues(self, **kwargs: Unpack[JsmQueueIssuesArgs]) -> _list[IssueSummary]:
        """Issues in a service desk queue (by queue ID or name)."""
        return [IssueSummary.model_validate(item) for item in await self._client.task(TaskName.JSM_QUEUE_ISSUES, kwargs)]


class AsyncJsmRequestApi:
    """``atl jsm request``"""

    def __init__(self, client: _AsyncTasks) -> None:
        self._client = client

    async def types(self, **kwargs: Unpack[JsmRequestTypesArgs]) -> _list[RequestType]:
        """Request types of a service desk."""
        return [RequestType.model_validate(item) for item in await self._client.task(TaskName.JSM_REQUEST_TYPES, kwargs)]

    async def list(self, **kwargs: Unpack[JsmRequestListArgs]) -> _list[Request]:
        """Customer requests by status, optionally of one service desk."""
        return [Request.model_validate(item) for item in await self._client.task(TaskName.JSM_REQUEST_LIST, kwargs)]

    async def view(self, **kwargs: Unpack[JsmRequestViewArgs]) -> RequestDetail:
        """One customer request with its description and latest comments."""
        return RequestDetail.model_validate(await self._client.task(TaskName.JSM_REQUEST_VIEW, kwargs))

    async def create(self, **kwargs: Unpack[JsmRequestCreateArgs]) -> Request:
        """Raise a customer request, optionally for a customer."""
        return Request.model_validate(await self._client.task(TaskName.JSM_REQUEST_CREATE, kwargs))

    async def comment(self, **kwargs: Unpack[JsmRequestCommentArgs]) -> RequestComment:
        """Reply to a customer request, or add an internal note."""
        return RequestComment.model_validate(await self._client.task(TaskName.JSM_REQUEST_COMMENT, kwargs))

    async def transitions(self, **kwargs: Unpack[JsmRequestTransitionsArgs]) -> _list[RequestTransition]:
        """Transitions available on a customer request."""
        return [RequestTransition.model_validate(item) for item in await self._client.task(TaskName.JSM_REQUEST_TRANSITIONS, kwargs)]

    async def move(self, **kwargs: Unpack[JsmRequestMoveArgs]) -> RequestMoved:
        """Run a customer request transition by name or ID."""
        return RequestMoved.model_validate(await self._client.task(TaskName.JSM_REQUEST_MOVE, kwargs))

    async def approvals(self, **kwargs: Unpack[JsmRequestApprovalsArgs]) -> _list[Approval]:
        """Approvals of a JSM request: name, decision (pending, approved, declined), approvers, and whether you can answer."""
        return [Approval.model_validate(item) for item in await self._client.task(TaskName.JSM_REQUEST_APPROVALS, kwargs)]

    async def approve(self, **kwargs: Unpack[JsmRequestApproveArgs]) -> Approval:
        """Approve or decline (decision: decline) a JSM request's approval; without an ID, the one pending approval you can answer."""
        return Approval.model_validate(await self._client.task(TaskName.JSM_REQUEST_APPROVE, kwargs))

    async def participants(self, **kwargs: Unpack[JsmRequestParticipantsArgs]) -> Participants:
        """List a JSM request's participants, after adding and removing any (me, an email or a name)."""
        return Participants.model_validate(await self._client.task(TaskName.JSM_REQUEST_PARTICIPANTS, kwargs))

    async def sla(self, **kwargs: Unpack[JsmRequestSlaArgs]) -> _list[Sla]:
        """SLAs of a JSM request: state (ongoing, paused, met, breached), time remaining and goal."""
        return [Sla.model_validate(item) for item in await self._client.task(TaskName.JSM_REQUEST_SLA, kwargs)]

    async def attach(self, **kwargs: Unpack[JsmRequestAttachArgs]) -> _list[RequestAttachment]:
        """Attach local files to a JSM request (public unless internal)."""
        return [RequestAttachment.model_validate(item) for item in await self._client.task(TaskName.JSM_REQUEST_ATTACH, kwargs)]

    async def attachments(self, **kwargs: Unpack[JsmRequestAttachmentsArgs]) -> _list[RequestAttachment]:
        """Attachments of a JSM request with download URLs."""
        return [RequestAttachment.model_validate(item) for item in await self._client.task(TaskName.JSM_REQUEST_ATTACHMENTS, kwargs)]


class AsyncJsmApi:
    """Jira Service Management desks, queues, requests and approvals."""

    def __init__(self, client: _AsyncTasks) -> None:
        self._client = client
        self.desk = AsyncJsmDeskApi(client)
        """``atl jsm desk``"""
        self.queue = AsyncJsmQueueApi(client)
        """``atl jsm queue``"""
        self.request = AsyncJsmRequestApi(client)
        """``atl jsm request``"""


class AsyncTrelloBoardApi:
    """``atl trello board``"""

    def __init__(self, client: _AsyncTasks) -> None:
        self._client = client

    async def list(self, **kwargs: Unpack[TrelloBoardListArgs]) -> _list[TrelloBoard]:
        """Your open Trello boards."""
        return [TrelloBoard.model_validate(item) for item in await self._client.task(TaskName.TRELLO_BOARD_LIST, kwargs)]


class AsyncTrelloCardApi:
    """``atl trello card``"""

    def __init__(self, client: _AsyncTasks) -> None:
        self._client = client

    async def list(self, **kwargs: Unpack[TrelloCardListArgs]) -> _list[Card]:
        """Cards of a Trello board or one of its lists, or your cards on every board (`mine`)."""
        return [Card.model_validate(item) for item in await self._client.task(TaskName.TRELLO_CARD_LIST, kwargs)]

    async def view(self, **kwargs: Unpack[TrelloCardViewArgs]) -> CardDetail:
        """One Trello card with list, board, description and latest comments."""
        return CardDetail.model_validate(await self._client.task(TaskName.TRELLO_CARD_VIEW, kwargs))

    async def create(self, **kwargs: Unpack[TrelloCardCreateArgs]) -> Card:
        """Create a Trello card in a list (by name with `board`, or by list ID)."""
        return Card.model_validate(await self._client.task(TaskName.TRELLO_CARD_CREATE, kwargs))

    async def move(self, **kwargs: Unpack[TrelloCardMoveArgs]) -> Card:
        """Move a Trello card to another list (by default on the card's own board)."""
        return Card.model_validate(await self._client.task(TaskName.TRELLO_CARD_MOVE, kwargs))

    async def comment(self, **kwargs: Unpack[TrelloCardCommentArgs]) -> CardComment:
        """Add a comment to a Trello card."""
        return CardComment.model_validate(await self._client.task(TaskName.TRELLO_CARD_COMMENT, kwargs))

    async def archive(self, **kwargs: Unpack[TrelloCardArchiveArgs]) -> Card:
        """Archive (close) a Trello card."""
        return Card.model_validate(await self._client.task(TaskName.TRELLO_CARD_ARCHIVE, kwargs))

    async def edit(self, **kwargs: Unpack[TrelloCardEditArgs]) -> Card:
        """Change a Trello card: name, description, due date, done, labels (by name or color) and members (username, name or me)."""
        return Card.model_validate(await self._client.task(TaskName.TRELLO_CARD_EDIT, kwargs))


class AsyncTrelloChecklistApi:
    """``atl trello checklist``"""

    def __init__(self, client: _AsyncTasks) -> None:
        self._client = client

    async def list(self, **kwargs: Unpack[TrelloChecklistListArgs]) -> _list[Checklist]:
        """Checklists of a Trello card with their items and whether each is done."""
        return [Checklist.model_validate(item) for item in await self._client.task(TaskName.TRELLO_CHECKLIST_LIST, kwargs)]

    async def create(self, **kwargs: Unpack[TrelloChecklistCreateArgs]) -> Checklist:
        """Add a checklist with items to a Trello card."""
        return Checklist.model_validate(await self._client.task(TaskName.TRELLO_CHECKLIST_CREATE, kwargs))

    async def check(self, **kwargs: Unpack[TrelloChecklistCheckArgs]) -> CheckItem:
        """Tick (or with done: false, untick) a Trello checklist item by name or ID."""
        return CheckItem.model_validate(await self._client.task(TaskName.TRELLO_CHECKLIST_CHECK, kwargs))


class AsyncTrelloListApi:
    """``atl trello list``"""

    def __init__(self, client: _AsyncTasks) -> None:
        self._client = client

    async def list(self, **kwargs: Unpack[TrelloListListArgs]) -> _list[TrelloList]:
        """Open lists of a Trello board (board by name, URL, short link or ID)."""
        return [TrelloList.model_validate(item) for item in await self._client.task(TaskName.TRELLO_LIST_LIST, kwargs)]


class AsyncTrelloApi:
    """Trello boards, lists, cards and checklists."""

    def __init__(self, client: _AsyncTasks) -> None:
        self._client = client
        self.board = AsyncTrelloBoardApi(client)
        """``atl trello board``"""
        self.card = AsyncTrelloCardApi(client)
        """``atl trello card``"""
        self.checklist = AsyncTrelloChecklistApi(client)
        """``atl trello checklist``"""
        self.list = AsyncTrelloListApi(client)
        """``atl trello list``"""


class AsyncAdminDirectoryApi:
    """``atl admin directory``"""

    def __init__(self, client: _AsyncTasks) -> None:
        self._client = client

    async def list(self, **kwargs: Unpack[AdminDirectoryListArgs]) -> _list[Directory]:
        """User directories of an Atlassian organization."""
        return [Directory.model_validate(item) for item in await self._client.task(TaskName.ADMIN_DIRECTORY_LIST, kwargs)]


class AsyncAdminDomainApi:
    """``atl admin domain``"""

    def __init__(self, client: _AsyncTasks) -> None:
        self._client = client

    async def list(self, **kwargs: Unpack[AdminDomainListArgs]) -> _list[Domain]:
        """Domains of an Atlassian organization with their verification status."""
        return [Domain.model_validate(item) for item in await self._client.task(TaskName.ADMIN_DOMAIN_LIST, kwargs)]


class AsyncAdminEventApi:
    """``atl admin event``"""

    def __init__(self, client: _AsyncTasks) -> None:
        self._client = client

    async def list(self, **kwargs: Unpack[AdminEventListArgs]) -> _list[Event]:
        """Audit log events of an Atlassian organization, newest first, filtered by words, dates and action."""
        return [Event.model_validate(item) for item in await self._client.task(TaskName.ADMIN_EVENT_LIST, kwargs)]


class AsyncAdminGroupApi:
    """``atl admin group``"""

    def __init__(self, client: _AsyncTasks) -> None:
        self._client = client

    async def add_member(self, **kwargs: Unpack[AdminGroupAddMemberArgs]) -> Membership:
        """Add a managed user (email or account ID) to a group (name or ID) of an Atlassian organization."""
        return Membership.model_validate(await self._client.task(TaskName.ADMIN_GROUP_ADD_MEMBER, kwargs))

    async def remove_member(self, **kwargs: Unpack[AdminGroupRemoveMemberArgs]) -> Membership:
        """Remove a managed user from a group of an Atlassian organization."""
        return Membership.model_validate(await self._client.task(TaskName.ADMIN_GROUP_REMOVE_MEMBER, kwargs))

    async def list(self, **kwargs: Unpack[AdminGroupListArgs]) -> _list[Group]:
        """Groups of an Atlassian organization (every directory), optionally matching `search`."""
        return [Group.model_validate(item) for item in await self._client.task(TaskName.ADMIN_GROUP_LIST, kwargs)]


class AsyncAdminOrgApi:
    """``atl admin org``"""

    def __init__(self, client: _AsyncTasks) -> None:
        self._client = client

    async def list(self, **kwargs: Unpack[AdminOrgListArgs]) -> _list[Org]:
        """Organizations the Atlassian Admin API key can see."""
        return [Org.model_validate(item) for item in await self._client.task(TaskName.ADMIN_ORG_LIST, kwargs)]


class AsyncAdminUserApi:
    """``atl admin user``"""

    def __init__(self, client: _AsyncTasks) -> None:
        self._client = client

    async def list(self, **kwargs: Unpack[AdminUserListArgs]) -> _list[AdminUser]:
        """Managed users of an Atlassian organization (every directory), optionally matching `search`."""
        return [AdminUser.model_validate(item) for item in await self._client.task(TaskName.ADMIN_USER_LIST, kwargs)]

    async def suspend(self, **kwargs: Unpack[AdminUserSuspendArgs]) -> AdminUser:
        """Suspend a managed user's access to every product (by account ID or email)."""
        return AdminUser.model_validate(await self._client.task(TaskName.ADMIN_USER_SUSPEND, kwargs))

    async def restore(self, **kwargs: Unpack[AdminUserRestoreArgs]) -> AdminUser:
        """Restore a suspended managed user's access (by account ID or email)."""
        return AdminUser.model_validate(await self._client.task(TaskName.ADMIN_USER_RESTORE, kwargs))


class AsyncAdminApi:
    """Atlassian Admin organizations, users, groups and audit events."""

    def __init__(self, client: _AsyncTasks) -> None:
        self._client = client
        self.directory = AsyncAdminDirectoryApi(client)
        """``atl admin directory``"""
        self.domain = AsyncAdminDomainApi(client)
        """``atl admin domain``"""
        self.event = AsyncAdminEventApi(client)
        """``atl admin event``"""
        self.group = AsyncAdminGroupApi(client)
        """``atl admin group``"""
        self.org = AsyncAdminOrgApi(client)
        """``atl admin org``"""
        self.user = AsyncAdminUserApi(client)
        """``atl admin user``"""


class GeneratedAsyncClient:
    """The product namespaces of ``AsyncClient``; the client supplies ``task()``."""

    async def task(self, name: str, args: Mapping[str, Any] | None = None) -> Any:
        raise NotImplementedError

    def __init__(self) -> None:
        self.jira = AsyncJiraApi(self)
        """Jira and Jira Software: issues, comments, sprints, boards, filters and JQL."""
        self.confluence = AsyncConfluenceApi(self)
        """Confluence pages, blogs, spaces, comments, labels and attachments."""
        self.bitbucket = AsyncBitbucketApi(self)
        """Bitbucket repositories, branches, pull requests and pipelines."""
        self.jsm = AsyncJsmApi(self)
        """Jira Service Management desks, queues, requests and approvals."""
        self.trello = AsyncTrelloApi(self)
        """Trello boards, lists, cards and checklists."""
        self.admin = AsyncAdminApi(self)
        """Atlassian Admin organizations, users, groups and audit events."""
