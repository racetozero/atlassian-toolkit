"""Generated from the task registry (crates/atl-sdk/src/tasks.rs) by
crates/atl-sdk/tests/sdk_types.rs. Do not edit; run
`ATL_UPDATE_SDK_TYPES=1 cargo test -p atlassian-toolkit --test sdk_types`.
"""

from __future__ import annotations

from collections.abc import Mapping
from enum import StrEnum
from typing import Any, NotRequired, Required, TypedDict, Unpack

__all__ = ["AdminDirectoryListArgs", "AdminDomainListArgs", "AdminEventListArgs", "AdminGroupAddMemberArgs", "AdminGroupListArgs", "AdminGroupRemoveMemberArgs", "AdminOrgListArgs", "AdminUserListArgs", "AdminUserRestoreArgs", "AdminUserSuspendArgs", "BitbucketBranchCreateArgs", "BitbucketBranchDeleteArgs", "BitbucketBranchListArgs", "BitbucketCommitListArgs", "BitbucketPipelineListArgs", "BitbucketPipelineLogsArgs", "BitbucketPipelineRunArgs", "BitbucketPipelineStopArgs", "BitbucketPipelineViewArgs", "BitbucketPrApproveArgs", "BitbucketPrChecksArgs", "BitbucketPrCommentArgs", "BitbucketPrCommentsArgs", "BitbucketPrCreateArgs", "BitbucketPrDeclineArgs", "BitbucketPrDiffArgs", "BitbucketPrEditArgs", "BitbucketPrListArgs", "BitbucketPrMergeArgs", "BitbucketPrReadyArgs", "BitbucketPrRequestChangesArgs", "BitbucketPrReviewersArgs", "BitbucketPrUnapproveArgs", "BitbucketPrViewArgs", "BitbucketRepoListArgs", "BitbucketRepoViewArgs", "ConfluenceAttachmentDeleteArgs", "ConfluenceBlogCreateArgs", "ConfluenceBlogGetArgs", "ConfluenceBlogListArgs", "ConfluenceCommentDeleteArgs", "ConfluencePageAttachArgs", "ConfluencePageAttachmentsArgs", "ConfluencePageChildrenArgs", "ConfluencePageCommentArgs", "ConfluencePageCommentsArgs", "ConfluencePageCopyTreeArgs", "ConfluencePageCreateArgs", "ConfluencePageDeleteArgs", "ConfluencePageExportArgs", "ConfluencePageFindArgs", "ConfluencePageGetArgs", "ConfluencePageHistoryArgs", "ConfluencePageLabelArgs", "ConfluencePageLabelsArgs", "ConfluencePageMoveArgs", "ConfluencePagePushArgs", "ConfluencePageUpdateArgs", "ConfluenceSearchArgs", "ConfluenceSpaceCreateArgs", "ConfluenceSpaceListArgs", "ConfluenceSpacePagesArgs", "ErrorCode", "GeneratedTasks", "JiraBoardListArgs", "JiraCommentDeleteArgs", "JiraCommentEditArgs", "JiraCommentListArgs", "JiraComponentListArgs", "JiraEpicAddArgs", "JiraEpicListArgs", "JiraEpicRemoveArgs", "JiraFieldListArgs", "JiraFilterListArgs", "JiraFilterRunArgs", "JiraIssueAssignArgs", "JiraIssueAttachArgs", "JiraIssueAttachmentsArgs", "JiraIssueCloneArgs", "JiraIssueCommentArgs", "JiraIssueCountArgs", "JiraIssueCreateArgs", "JiraIssueDeleteArgs", "JiraIssueEditArgs", "JiraIssueHistoryArgs", "JiraIssueLinkArgs", "JiraIssueLinkTypesArgs", "JiraIssueListArgs", "JiraIssueMoveArgs", "JiraIssueRemoteLinkArgs", "JiraIssueTransitionsArgs", "JiraIssueTypesArgs", "JiraIssueUnlinkArgs", "JiraIssueUnwatchArgs", "JiraIssueViewArgs", "JiraIssueWatchArgs", "JiraIssueWatchersArgs", "JiraIssueWorklogArgs", "JiraJqlCheckArgs", "JiraJqlFieldsArgs", "JiraJqlSuggestArgs", "JiraMeArgs", "JiraProjectListArgs", "JiraReleaseListArgs", "JiraSprintAddArgs", "JiraSprintCloseArgs", "JiraSprintCreateArgs", "JiraSprintListArgs", "JiraSprintStartArgs", "JiraWorklogListArgs", "JsmDeskListArgs", "JsmQueueIssuesArgs", "JsmQueueListArgs", "JsmRequestApprovalsArgs", "JsmRequestApproveArgs", "JsmRequestAttachArgs", "JsmRequestAttachmentsArgs", "JsmRequestCommentArgs", "JsmRequestCreateArgs", "JsmRequestListArgs", "JsmRequestMoveArgs", "JsmRequestParticipantsArgs", "JsmRequestSlaArgs", "JsmRequestTransitionsArgs", "JsmRequestTypesArgs", "JsmRequestViewArgs", "TaskName", "TrelloBoardListArgs", "TrelloCardArchiveArgs", "TrelloCardCommentArgs", "TrelloCardCreateArgs", "TrelloCardEditArgs", "TrelloCardListArgs", "TrelloCardMoveArgs", "TrelloCardViewArgs", "TrelloChecklistCheckArgs", "TrelloChecklistCreateArgs", "TrelloChecklistListArgs", "TrelloListListArgs"]


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


class GeneratedTasks:
    """One typed method per task; the client supplies ``task()``."""

    def task(self, name: str, args: Mapping[str, Any] | None = None, **kwargs: Any) -> Any:
        raise NotImplementedError

    def jira_issue_list(self, **kwargs: Unpack[JiraIssueListArgs]) -> Any:
        """Search Jira issues with filters (or extra JQL). Returns small records: key, summary, status, type, assignee, priority, updated, url. With no filters: your unresolved issues."""
        return self.task(TaskName.JIRA_ISSUE_LIST, kwargs)

    def jira_issue_count(self, **kwargs: Unpack[JiraIssueCountArgs]) -> Any:
        """Count Jira issues matching the same filters as issue_list."""
        return self.task(TaskName.JIRA_ISSUE_COUNT, kwargs)

    def jira_issue_view(self, **kwargs: Unpack[JiraIssueViewArgs]) -> Any:
        """One Jira issue with description and latest comments as Markdown, plus any `fields` asked for by name."""
        return self.task(TaskName.JIRA_ISSUE_VIEW, kwargs)

    def jira_issue_delete(self, **kwargs: Unpack[JiraIssueDeleteArgs]) -> Any:
        """Delete a Jira issue (with `subtasks: true`, also its sub-tasks). Cannot be undone."""
        return self.task(TaskName.JIRA_ISSUE_DELETE, kwargs)

    def jira_issue_watchers(self, **kwargs: Unpack[JiraIssueWatchersArgs]) -> Any:
        """Who watches a Jira issue, and whether you do."""
        return self.task(TaskName.JIRA_ISSUE_WATCHERS, kwargs)

    def jira_issue_watch(self, **kwargs: Unpack[JiraIssueWatchArgs]) -> Any:
        """Add a watcher to a Jira issue (default: you)."""
        return self.task(TaskName.JIRA_ISSUE_WATCH, kwargs)

    def jira_issue_unwatch(self, **kwargs: Unpack[JiraIssueUnwatchArgs]) -> Any:
        """Remove a watcher from a Jira issue (default: you)."""
        return self.task(TaskName.JIRA_ISSUE_UNWATCH, kwargs)

    def jira_field_list(self, **kwargs: Unpack[JiraFieldListArgs]) -> Any:
        """Jira system and custom fields (ID, name, type), optionally filtered by part of the name. Use the names in the `fields` argument of issue create, edit and view."""
        return self.task(TaskName.JIRA_FIELD_LIST, kwargs)

    def jira_issue_create(self, **kwargs: Unpack[JiraIssueCreateArgs]) -> Any:
        """Create a Jira issue; the description is Markdown."""
        return self.task(TaskName.JIRA_ISSUE_CREATE, kwargs)

    def jira_issue_edit(self, **kwargs: Unpack[JiraIssueEditArgs]) -> Any:
        """Change a Jira issue's summary, description (Markdown), priority or labels."""
        return self.task(TaskName.JIRA_ISSUE_EDIT, kwargs)

    def jira_issue_assign(self, **kwargs: Unpack[JiraIssueAssignArgs]) -> Any:
        """Assign a Jira issue to me, nobody (x), or a user."""
        return self.task(TaskName.JIRA_ISSUE_ASSIGN, kwargs)

    def jira_issue_transitions(self, **kwargs: Unpack[JiraIssueTransitionsArgs]) -> Any:
        """Statuses a Jira issue can move to."""
        return self.task(TaskName.JIRA_ISSUE_TRANSITIONS, kwargs)

    def jira_issue_move(self, **kwargs: Unpack[JiraIssueMoveArgs]) -> Any:
        """Move a Jira issue to another status (by status or transition name)."""
        return self.task(TaskName.JIRA_ISSUE_MOVE, kwargs)

    def jira_issue_comment(self, **kwargs: Unpack[JiraIssueCommentArgs]) -> Any:
        """Add a Markdown comment to a Jira issue."""
        return self.task(TaskName.JIRA_ISSUE_COMMENT, kwargs)

    def jira_issue_link(self, **kwargs: Unpack[JiraIssueLinkArgs]) -> Any:
        """Link two Jira issues so that 'from <kind> to' reads true, for example from A-1 kind blocks to A-2."""
        return self.task(TaskName.JIRA_ISSUE_LINK, kwargs)

    def jira_issue_unlink(self, **kwargs: Unpack[JiraIssueUnlinkArgs]) -> Any:
        """Remove the links between two Jira issues (optionally only one link type)."""
        return self.task(TaskName.JIRA_ISSUE_UNLINK, kwargs)

    def jira_issue_remote_link(self, **kwargs: Unpack[JiraIssueRemoteLinkArgs]) -> Any:
        """Add a web link (URL and title) to a Jira issue."""
        return self.task(TaskName.JIRA_ISSUE_REMOTE_LINK, kwargs)

    def jira_comment_list(self, **kwargs: Unpack[JiraCommentListArgs]) -> Any:
        """Every comment of a Jira issue, oldest first, with IDs, as Markdown."""
        return self.task(TaskName.JIRA_COMMENT_LIST, kwargs)

    def jira_comment_edit(self, **kwargs: Unpack[JiraCommentEditArgs]) -> Any:
        """Replace the text of a Jira comment (Markdown)."""
        return self.task(TaskName.JIRA_COMMENT_EDIT, kwargs)

    def jira_comment_delete(self, **kwargs: Unpack[JiraCommentDeleteArgs]) -> Any:
        """Delete a Jira comment. Cannot be undone."""
        return self.task(TaskName.JIRA_COMMENT_DELETE, kwargs)

    def jira_worklog_list(self, **kwargs: Unpack[JiraWorklogListArgs]) -> Any:
        """Time logged on a Jira issue: author, time spent, start, comment."""
        return self.task(TaskName.JIRA_WORKLOG_LIST, kwargs)

    def jira_issue_history(self, **kwargs: Unpack[JiraIssueHistoryArgs]) -> Any:
        """Field changes of a Jira issue, newest first: when, who, field, from, to."""
        return self.task(TaskName.JIRA_ISSUE_HISTORY, kwargs)

    def jira_component_list(self, **kwargs: Unpack[JiraComponentListArgs]) -> Any:
        """Components of a Jira project with their leads."""
        return self.task(TaskName.JIRA_COMPONENT_LIST, kwargs)

    def jira_sprint_create(self, **kwargs: Unpack[JiraSprintCreateArgs]) -> Any:
        """Create a future sprint on a board (default: the project's first board)."""
        return self.task(TaskName.JIRA_SPRINT_CREATE, kwargs)

    def jira_sprint_start(self, **kwargs: Unpack[JiraSprintStartArgs]) -> Any:
        """Start a future sprint; Jira needs a start and an end date."""
        return self.task(TaskName.JIRA_SPRINT_START, kwargs)

    def jira_sprint_close(self, **kwargs: Unpack[JiraSprintCloseArgs]) -> Any:
        """Complete an active sprint."""
        return self.task(TaskName.JIRA_SPRINT_CLOSE, kwargs)

    def jira_release_list(self, **kwargs: Unpack[JiraReleaseListArgs]) -> Any:
        """Versions (releases) of a Jira project: name, released, archived, release date."""
        return self.task(TaskName.JIRA_RELEASE_LIST, kwargs)

    def jira_issue_link_types(self, **kwargs: Unpack[JiraIssueLinkTypesArgs]) -> Any:
        """Jira issue link types with their outward and inward phrases."""
        return self.task(TaskName.JIRA_ISSUE_LINK_TYPES, kwargs)

    def jira_issue_clone(self, **kwargs: Unpack[JiraIssueCloneArgs]) -> Any:
        """Copy a Jira issue (summary, description, type, labels, priority, components, parent)."""
        return self.task(TaskName.JIRA_ISSUE_CLONE, kwargs)

    def jira_issue_worklog(self, **kwargs: Unpack[JiraIssueWorklogArgs]) -> Any:
        """Log time on a Jira issue."""
        return self.task(TaskName.JIRA_ISSUE_WORKLOG, kwargs)

    def jira_issue_attach(self, **kwargs: Unpack[JiraIssueAttachArgs]) -> Any:
        """Upload local files to a Jira issue."""
        return self.task(TaskName.JIRA_ISSUE_ATTACH, kwargs)

    def jira_issue_attachments(self, **kwargs: Unpack[JiraIssueAttachmentsArgs]) -> Any:
        """List a Jira issue's attachments."""
        return self.task(TaskName.JIRA_ISSUE_ATTACHMENTS, kwargs)

    def jira_jql_check(self, **kwargs: Unpack[JiraJqlCheckArgs]) -> Any:
        """Validate JQL with Jira's parser; returns valid, errors, warnings."""
        return self.task(TaskName.JIRA_JQL_CHECK, kwargs)

    def jira_jql_fields(self, **kwargs: Unpack[JiraJqlFieldsArgs]) -> Any:
        """Fields (with operators) and functions usable in JQL on this site."""
        return self.task(TaskName.JIRA_JQL_FIELDS, kwargs)

    def jira_jql_suggest(self, **kwargs: Unpack[JiraJqlSuggestArgs]) -> Any:
        """Value suggestions for a JQL field, for example field=status prefix=In."""
        return self.task(TaskName.JIRA_JQL_SUGGEST, kwargs)

    def jira_filter_list(self, **kwargs: Unpack[JiraFilterListArgs]) -> Any:
        """Saved Jira filters (Cloud: all visible, Data Center: favourites)."""
        return self.task(TaskName.JIRA_FILTER_LIST, kwargs)

    def jira_filter_run(self, **kwargs: Unpack[JiraFilterRunArgs]) -> Any:
        """Run a saved Jira filter by ID or exact name; returns issue records."""
        return self.task(TaskName.JIRA_FILTER_RUN, kwargs)

    def jira_epic_list(self, **kwargs: Unpack[JiraEpicListArgs]) -> Any:
        """Open epics, optionally of one project."""
        return self.task(TaskName.JIRA_EPIC_LIST, kwargs)

    def jira_epic_add(self, **kwargs: Unpack[JiraEpicAddArgs]) -> Any:
        """Move issues into an epic."""
        return self.task(TaskName.JIRA_EPIC_ADD, kwargs)

    def jira_epic_remove(self, **kwargs: Unpack[JiraEpicRemoveArgs]) -> Any:
        """Remove issues from their epic."""
        return self.task(TaskName.JIRA_EPIC_REMOVE, kwargs)

    def jira_board_list(self, **kwargs: Unpack[JiraBoardListArgs]) -> Any:
        """Jira Software boards, optionally of one project."""
        return self.task(TaskName.JIRA_BOARD_LIST, kwargs)

    def jira_sprint_list(self, **kwargs: Unpack[JiraSprintListArgs]) -> Any:
        """Sprints of a board."""
        return self.task(TaskName.JIRA_SPRINT_LIST, kwargs)

    def jira_sprint_add(self, **kwargs: Unpack[JiraSprintAddArgs]) -> Any:
        """Move issues into a sprint."""
        return self.task(TaskName.JIRA_SPRINT_ADD, kwargs)

    def jira_project_list(self, **kwargs: Unpack[JiraProjectListArgs]) -> Any:
        """Jira projects."""
        return self.task(TaskName.JIRA_PROJECT_LIST, kwargs)

    def jira_issue_types(self, **kwargs: Unpack[JiraIssueTypesArgs]) -> Any:
        """Issue type names of a Jira project (without sub-task types), for the `type` argument of jira_issue_create."""
        return self.task(TaskName.JIRA_ISSUE_TYPES, kwargs)

    def jira_me(self, **kwargs: Unpack[JiraMeArgs]) -> Any:
        """The signed-in Jira user."""
        return self.task(TaskName.JIRA_ME, kwargs)

    def confluence_page_get(self, **kwargs: Unpack[ConfluencePageGetArgs]) -> Any:
        """A Confluence page as Markdown (with title, space, version, url)."""
        return self.task(TaskName.CONFLUENCE_PAGE_GET, kwargs)

    def confluence_page_create(self, **kwargs: Unpack[ConfluencePageCreateArgs]) -> Any:
        """Create a Confluence page from Markdown (or storage format)."""
        return self.task(TaskName.CONFLUENCE_PAGE_CREATE, kwargs)

    def confluence_blog_list(self, **kwargs: Unpack[ConfluenceBlogListArgs]) -> Any:
        """Blog posts of a Confluence space, newest first."""
        return self.task(TaskName.CONFLUENCE_BLOG_LIST, kwargs)

    def confluence_blog_get(self, **kwargs: Unpack[ConfluenceBlogGetArgs]) -> Any:
        """One Confluence blog post with its body as Markdown."""
        return self.task(TaskName.CONFLUENCE_BLOG_GET, kwargs)

    def confluence_blog_create(self, **kwargs: Unpack[ConfluenceBlogCreateArgs]) -> Any:
        """Publish a Confluence blog post from Markdown."""
        return self.task(TaskName.CONFLUENCE_BLOG_CREATE, kwargs)

    def confluence_space_create(self, **kwargs: Unpack[ConfluenceSpaceCreateArgs]) -> Any:
        """Create a Confluence space with a key, a name and an optional plain-text description."""
        return self.task(TaskName.CONFLUENCE_SPACE_CREATE, kwargs)

    def confluence_page_update(self, **kwargs: Unpack[ConfluencePageUpdateArgs]) -> Any:
        """Replace a Confluence page's title and/or body; the version is bumped automatically."""
        return self.task(TaskName.CONFLUENCE_PAGE_UPDATE, kwargs)

    def confluence_page_delete(self, **kwargs: Unpack[ConfluencePageDeleteArgs]) -> Any:
        """Delete a Confluence page."""
        return self.task(TaskName.CONFLUENCE_PAGE_DELETE, kwargs)

    def confluence_page_children(self, **kwargs: Unpack[ConfluencePageChildrenArgs]) -> Any:
        """Children of a Confluence page (Cloud: also folders), or with `recursive` the whole tree, depth first, each row with type, depth and parent."""
        return self.task(TaskName.CONFLUENCE_PAGE_CHILDREN, kwargs)

    def confluence_page_find(self, **kwargs: Unpack[ConfluencePageFindArgs]) -> Any:
        """Confluence pages whose title contains the words, optionally in one space."""
        return self.task(TaskName.CONFLUENCE_PAGE_FIND, kwargs)

    def confluence_page_attach(self, **kwargs: Unpack[ConfluencePageAttachArgs]) -> Any:
        """Upload local files to a Confluence page."""
        return self.task(TaskName.CONFLUENCE_PAGE_ATTACH, kwargs)

    def confluence_page_attachments(self, **kwargs: Unpack[ConfluencePageAttachmentsArgs]) -> Any:
        """Attachments of a Confluence page with download URLs."""
        return self.task(TaskName.CONFLUENCE_PAGE_ATTACHMENTS, kwargs)

    def confluence_page_comments(self, **kwargs: Unpack[ConfluencePageCommentsArgs]) -> Any:
        """Footer (or with `inline`, inline) comments of a Confluence page, oldest first, as Markdown, with the replied-to `parent`."""
        return self.task(TaskName.CONFLUENCE_PAGE_COMMENTS, kwargs)

    def confluence_page_comment(self, **kwargs: Unpack[ConfluencePageCommentArgs]) -> Any:
        """Add a footer comment (Markdown) to a Confluence page, or a reply with `parent`."""
        return self.task(TaskName.CONFLUENCE_PAGE_COMMENT, kwargs)

    def confluence_comment_delete(self, **kwargs: Unpack[ConfluenceCommentDeleteArgs]) -> Any:
        """Delete a Confluence comment by ID."""
        return self.task(TaskName.CONFLUENCE_COMMENT_DELETE, kwargs)

    def confluence_attachment_delete(self, **kwargs: Unpack[ConfluenceAttachmentDeleteArgs]) -> Any:
        """Delete a Confluence attachment by ID."""
        return self.task(TaskName.CONFLUENCE_ATTACHMENT_DELETE, kwargs)

    def confluence_page_export(self, **kwargs: Unpack[ConfluencePageExportArgs]) -> Any:
        """Export a Confluence page (and with `recursive` its tree) to local Markdown folders: <title>/index.md with front matter and _attachments/. Changes nothing on the server."""
        return self.task(TaskName.CONFLUENCE_PAGE_EXPORT, kwargs)

    def confluence_page_push(self, **kwargs: Unpack[ConfluencePagePushArgs]) -> Any:
        """Publish local Markdown files: update the page in each file's front matter (refusing pages changed on the server unless `force`, skipping unchanged ones), create pages for new files under their folder's index.md, upload referenced images, and write ids and versions back."""
        return self.task(TaskName.CONFLUENCE_PAGE_PUSH, kwargs)

    def confluence_page_copy_tree(self, **kwargs: Unpack[ConfluencePageCopyTreeArgs]) -> Any:
        """Copy a Confluence page and the pages below it under another page (titles, bodies, labels)."""
        return self.task(TaskName.CONFLUENCE_PAGE_COPY_TREE, kwargs)

    def confluence_page_labels(self, **kwargs: Unpack[ConfluencePageLabelsArgs]) -> Any:
        """Labels of a Confluence page."""
        return self.task(TaskName.CONFLUENCE_PAGE_LABELS, kwargs)

    def confluence_page_label(self, **kwargs: Unpack[ConfluencePageLabelArgs]) -> Any:
        """Add and remove labels on a Confluence page; returns the resulting labels."""
        return self.task(TaskName.CONFLUENCE_PAGE_LABEL, kwargs)

    def confluence_page_move(self, **kwargs: Unpack[ConfluencePageMoveArgs]) -> Any:
        """Move a Confluence page under another page (position append) or next to it (before/after, Cloud only)."""
        return self.task(TaskName.CONFLUENCE_PAGE_MOVE, kwargs)

    def confluence_page_history(self, **kwargs: Unpack[ConfluencePageHistoryArgs]) -> Any:
        """Versions of a Confluence page, newest first: number, author, date, message. Read one with confluence_page_get and `version`."""
        return self.task(TaskName.CONFLUENCE_PAGE_HISTORY, kwargs)

    def confluence_space_list(self, **kwargs: Unpack[ConfluenceSpaceListArgs]) -> Any:
        """Confluence spaces."""
        return self.task(TaskName.CONFLUENCE_SPACE_LIST, kwargs)

    def confluence_space_pages(self, **kwargs: Unpack[ConfluenceSpacePagesArgs]) -> Any:
        """Top-level pages of a Confluence space."""
        return self.task(TaskName.CONFLUENCE_SPACE_PAGES, kwargs)

    def confluence_search(self, **kwargs: Unpack[ConfluenceSearchArgs]) -> Any:
        """Search Confluence by words or CQL; returns id, type, title, space, excerpt, url."""
        return self.task(TaskName.CONFLUENCE_SEARCH, kwargs)

    def bitbucket_repo_list(self, **kwargs: Unpack[BitbucketRepoListArgs]) -> Any:
        """Bitbucket repositories of a workspace (Cloud) or project (Data Center), or the ones you belong to."""
        return self.task(TaskName.BITBUCKET_REPO_LIST, kwargs)

    def bitbucket_repo_view(self, **kwargs: Unpack[BitbucketRepoViewArgs]) -> Any:
        """One Bitbucket repository with its default branch and URL."""
        return self.task(TaskName.BITBUCKET_REPO_VIEW, kwargs)

    def bitbucket_branch_list(self, **kwargs: Unpack[BitbucketBranchListArgs]) -> Any:
        """Branches of a Bitbucket repository, most recently changed first."""
        return self.task(TaskName.BITBUCKET_BRANCH_LIST, kwargs)

    def bitbucket_pr_list(self, **kwargs: Unpack[BitbucketPrListArgs]) -> Any:
        """Pull requests of a Bitbucket repository by state."""
        return self.task(TaskName.BITBUCKET_PR_LIST, kwargs)

    def bitbucket_pr_view(self, **kwargs: Unpack[BitbucketPrViewArgs]) -> Any:
        """One pull request with description, reviewers and approvals."""
        return self.task(TaskName.BITBUCKET_PR_VIEW, kwargs)

    def bitbucket_pr_create(self, **kwargs: Unpack[BitbucketPrCreateArgs]) -> Any:
        """Open a pull request from a source branch."""
        return self.task(TaskName.BITBUCKET_PR_CREATE, kwargs)

    def bitbucket_pr_merge(self, **kwargs: Unpack[BitbucketPrMergeArgs]) -> Any:
        """Merge a pull request."""
        return self.task(TaskName.BITBUCKET_PR_MERGE, kwargs)

    def bitbucket_pr_approve(self, **kwargs: Unpack[BitbucketPrApproveArgs]) -> Any:
        """Approve a pull request as the signed-in user."""
        return self.task(TaskName.BITBUCKET_PR_APPROVE, kwargs)

    def bitbucket_pr_comment(self, **kwargs: Unpack[BitbucketPrCommentArgs]) -> Any:
        """Add a Markdown comment to a pull request."""
        return self.task(TaskName.BITBUCKET_PR_COMMENT, kwargs)

    def bitbucket_pr_diff(self, **kwargs: Unpack[BitbucketPrDiffArgs]) -> Any:
        """The unified diff of a pull request."""
        return self.task(TaskName.BITBUCKET_PR_DIFF, kwargs)

    def bitbucket_pr_comments(self, **kwargs: Unpack[BitbucketPrCommentsArgs]) -> Any:
        """Comments of a pull request, oldest first, as Markdown: general and inline (path, line), replies flattened with `parent`."""
        return self.task(TaskName.BITBUCKET_PR_COMMENTS, kwargs)

    def bitbucket_pr_checks(self, **kwargs: Unpack[BitbucketPrChecksArgs]) -> Any:
        """Build statuses of a pull request's latest commit, with `passed` (none failed) and `pending` (some still running)."""
        return self.task(TaskName.BITBUCKET_PR_CHECKS, kwargs)

    def bitbucket_pr_unapprove(self, **kwargs: Unpack[BitbucketPrUnapproveArgs]) -> Any:
        """Remove your approval from a pull request."""
        return self.task(TaskName.BITBUCKET_PR_UNAPPROVE, kwargs)

    def bitbucket_pr_ready(self, **kwargs: Unpack[BitbucketPrReadyArgs]) -> Any:
        """Mark a draft pull request as ready for review (or, with `draft: true`, as a draft again)."""
        return self.task(TaskName.BITBUCKET_PR_READY, kwargs)

    def bitbucket_branch_create(self, **kwargs: Unpack[BitbucketBranchCreateArgs]) -> Any:
        """Create a branch from another branch, a tag or a commit (default: the main branch)."""
        return self.task(TaskName.BITBUCKET_BRANCH_CREATE, kwargs)

    def bitbucket_branch_delete(self, **kwargs: Unpack[BitbucketBranchDeleteArgs]) -> Any:
        """Delete a branch. Unmerged commits on it can be lost."""
        return self.task(TaskName.BITBUCKET_BRANCH_DELETE, kwargs)

    def bitbucket_commit_list(self, **kwargs: Unpack[BitbucketCommitListArgs]) -> Any:
        """Recent commits of a branch, newest first: hash, author, date, first line of the message."""
        return self.task(TaskName.BITBUCKET_COMMIT_LIST, kwargs)

    def bitbucket_pipeline_list(self, **kwargs: Unpack[BitbucketPipelineListArgs]) -> Any:
        """Latest Bitbucket Pipelines runs, newest first (Cloud only)."""
        return self.task(TaskName.BITBUCKET_PIPELINE_LIST, kwargs)

    def bitbucket_pipeline_run(self, **kwargs: Unpack[BitbucketPipelineRunArgs]) -> Any:
        """Start the pipeline of a branch, or a custom pipeline (Cloud only)."""
        return self.task(TaskName.BITBUCKET_PIPELINE_RUN, kwargs)

    def bitbucket_pr_decline(self, **kwargs: Unpack[BitbucketPrDeclineArgs]) -> Any:
        """Decline a Bitbucket pull request (Cloud cannot reopen it)."""
        return self.task(TaskName.BITBUCKET_PR_DECLINE, kwargs)

    def bitbucket_pr_request_changes(self, **kwargs: Unpack[BitbucketPrRequestChangesArgs]) -> Any:
        """Mark a Bitbucket pull request as needing changes by you (Cloud: request changes; Data Center: needs work)."""
        return self.task(TaskName.BITBUCKET_PR_REQUEST_CHANGES, kwargs)

    def bitbucket_pr_edit(self, **kwargs: Unpack[BitbucketPrEditArgs]) -> Any:
        """Change a Bitbucket pull request's title, description (Markdown) or destination branch."""
        return self.task(TaskName.BITBUCKET_PR_EDIT, kwargs)

    def bitbucket_pr_reviewers(self, **kwargs: Unpack[BitbucketPrReviewersArgs]) -> Any:
        """Add and remove reviewers of a Bitbucket pull request; other reviewers are kept."""
        return self.task(TaskName.BITBUCKET_PR_REVIEWERS, kwargs)

    def bitbucket_pipeline_stop(self, **kwargs: Unpack[BitbucketPipelineStopArgs]) -> Any:
        """Stop a running Bitbucket pipeline by build number or UUID (Cloud only)."""
        return self.task(TaskName.BITBUCKET_PIPELINE_STOP, kwargs)

    def bitbucket_pipeline_view(self, **kwargs: Unpack[BitbucketPipelineViewArgs]) -> Any:
        """One Bitbucket pipeline by build number or UUID, with each step's state and result (Cloud only)."""
        return self.task(TaskName.BITBUCKET_PIPELINE_VIEW, kwargs)

    def bitbucket_pipeline_logs(self, **kwargs: Unpack[BitbucketPipelineLogsArgs]) -> Any:
        """Log text of one pipeline step (by name or UUID) or of every step (Cloud only)."""
        return self.task(TaskName.BITBUCKET_PIPELINE_LOGS, kwargs)

    def jsm_desk_list(self, **kwargs: Unpack[JsmDeskListArgs]) -> Any:
        """Jira Service Management service desks (id, project key, name)."""
        return self.task(TaskName.JSM_DESK_LIST, kwargs)

    def jsm_request_types(self, **kwargs: Unpack[JsmRequestTypesArgs]) -> Any:
        """Request types of a service desk."""
        return self.task(TaskName.JSM_REQUEST_TYPES, kwargs)

    def jsm_queue_list(self, **kwargs: Unpack[JsmQueueListArgs]) -> Any:
        """Queues of a service desk with issue counts."""
        return self.task(TaskName.JSM_QUEUE_LIST, kwargs)

    def jsm_queue_issues(self, **kwargs: Unpack[JsmQueueIssuesArgs]) -> Any:
        """Issues in a service desk queue (by queue ID or name)."""
        return self.task(TaskName.JSM_QUEUE_ISSUES, kwargs)

    def jsm_request_list(self, **kwargs: Unpack[JsmRequestListArgs]) -> Any:
        """Customer requests by status, optionally of one service desk."""
        return self.task(TaskName.JSM_REQUEST_LIST, kwargs)

    def jsm_request_view(self, **kwargs: Unpack[JsmRequestViewArgs]) -> Any:
        """One customer request with its description and latest comments."""
        return self.task(TaskName.JSM_REQUEST_VIEW, kwargs)

    def jsm_request_create(self, **kwargs: Unpack[JsmRequestCreateArgs]) -> Any:
        """Raise a customer request, optionally for a customer."""
        return self.task(TaskName.JSM_REQUEST_CREATE, kwargs)

    def jsm_request_comment(self, **kwargs: Unpack[JsmRequestCommentArgs]) -> Any:
        """Reply to a customer request, or add an internal note."""
        return self.task(TaskName.JSM_REQUEST_COMMENT, kwargs)

    def jsm_request_transitions(self, **kwargs: Unpack[JsmRequestTransitionsArgs]) -> Any:
        """Transitions available on a customer request."""
        return self.task(TaskName.JSM_REQUEST_TRANSITIONS, kwargs)

    def jsm_request_move(self, **kwargs: Unpack[JsmRequestMoveArgs]) -> Any:
        """Run a customer request transition by name or ID."""
        return self.task(TaskName.JSM_REQUEST_MOVE, kwargs)

    def jsm_request_approvals(self, **kwargs: Unpack[JsmRequestApprovalsArgs]) -> Any:
        """Approvals of a JSM request: name, decision (pending, approved, declined), approvers, and whether you can answer."""
        return self.task(TaskName.JSM_REQUEST_APPROVALS, kwargs)

    def jsm_request_approve(self, **kwargs: Unpack[JsmRequestApproveArgs]) -> Any:
        """Approve or decline (decision: decline) a JSM request's approval; without an ID, the one pending approval you can answer."""
        return self.task(TaskName.JSM_REQUEST_APPROVE, kwargs)

    def jsm_request_participants(self, **kwargs: Unpack[JsmRequestParticipantsArgs]) -> Any:
        """List a JSM request's participants, after adding and removing any (me, an email or a name)."""
        return self.task(TaskName.JSM_REQUEST_PARTICIPANTS, kwargs)

    def jsm_request_sla(self, **kwargs: Unpack[JsmRequestSlaArgs]) -> Any:
        """SLAs of a JSM request: state (ongoing, paused, met, breached), time remaining and goal."""
        return self.task(TaskName.JSM_REQUEST_SLA, kwargs)

    def jsm_request_attach(self, **kwargs: Unpack[JsmRequestAttachArgs]) -> Any:
        """Attach local files to a JSM request (public unless internal)."""
        return self.task(TaskName.JSM_REQUEST_ATTACH, kwargs)

    def jsm_request_attachments(self, **kwargs: Unpack[JsmRequestAttachmentsArgs]) -> Any:
        """Attachments of a JSM request with download URLs."""
        return self.task(TaskName.JSM_REQUEST_ATTACHMENTS, kwargs)

    def trello_board_list(self, **kwargs: Unpack[TrelloBoardListArgs]) -> Any:
        """Your open Trello boards."""
        return self.task(TaskName.TRELLO_BOARD_LIST, kwargs)

    def trello_list_list(self, **kwargs: Unpack[TrelloListListArgs]) -> Any:
        """Open lists of a Trello board (board by name, URL, short link or ID)."""
        return self.task(TaskName.TRELLO_LIST_LIST, kwargs)

    def trello_card_list(self, **kwargs: Unpack[TrelloCardListArgs]) -> Any:
        """Cards of a Trello board or one of its lists, or your cards on every board (`mine`)."""
        return self.task(TaskName.TRELLO_CARD_LIST, kwargs)

    def trello_card_view(self, **kwargs: Unpack[TrelloCardViewArgs]) -> Any:
        """One Trello card with list, board, description and latest comments."""
        return self.task(TaskName.TRELLO_CARD_VIEW, kwargs)

    def trello_card_create(self, **kwargs: Unpack[TrelloCardCreateArgs]) -> Any:
        """Create a Trello card in a list (by name with `board`, or by list ID)."""
        return self.task(TaskName.TRELLO_CARD_CREATE, kwargs)

    def trello_card_move(self, **kwargs: Unpack[TrelloCardMoveArgs]) -> Any:
        """Move a Trello card to another list (by default on the card's own board)."""
        return self.task(TaskName.TRELLO_CARD_MOVE, kwargs)

    def trello_card_comment(self, **kwargs: Unpack[TrelloCardCommentArgs]) -> Any:
        """Add a comment to a Trello card."""
        return self.task(TaskName.TRELLO_CARD_COMMENT, kwargs)

    def trello_card_archive(self, **kwargs: Unpack[TrelloCardArchiveArgs]) -> Any:
        """Archive (close) a Trello card."""
        return self.task(TaskName.TRELLO_CARD_ARCHIVE, kwargs)

    def trello_card_edit(self, **kwargs: Unpack[TrelloCardEditArgs]) -> Any:
        """Change a Trello card: name, description, due date, done, labels (by name or color) and members (username, name or me)."""
        return self.task(TaskName.TRELLO_CARD_EDIT, kwargs)

    def trello_checklist_list(self, **kwargs: Unpack[TrelloChecklistListArgs]) -> Any:
        """Checklists of a Trello card with their items and whether each is done."""
        return self.task(TaskName.TRELLO_CHECKLIST_LIST, kwargs)

    def trello_checklist_create(self, **kwargs: Unpack[TrelloChecklistCreateArgs]) -> Any:
        """Add a checklist with items to a Trello card."""
        return self.task(TaskName.TRELLO_CHECKLIST_CREATE, kwargs)

    def trello_checklist_check(self, **kwargs: Unpack[TrelloChecklistCheckArgs]) -> Any:
        """Tick (or with done: false, untick) a Trello checklist item by name or ID."""
        return self.task(TaskName.TRELLO_CHECKLIST_CHECK, kwargs)

    def admin_org_list(self, **kwargs: Unpack[AdminOrgListArgs]) -> Any:
        """Organizations the Atlassian Admin API key can see."""
        return self.task(TaskName.ADMIN_ORG_LIST, kwargs)

    def admin_directory_list(self, **kwargs: Unpack[AdminDirectoryListArgs]) -> Any:
        """User directories of an Atlassian organization."""
        return self.task(TaskName.ADMIN_DIRECTORY_LIST, kwargs)

    def admin_user_list(self, **kwargs: Unpack[AdminUserListArgs]) -> Any:
        """Managed users of an Atlassian organization (every directory), optionally matching `search`."""
        return self.task(TaskName.ADMIN_USER_LIST, kwargs)

    def admin_user_suspend(self, **kwargs: Unpack[AdminUserSuspendArgs]) -> Any:
        """Suspend a managed user's access to every product (by account ID or email)."""
        return self.task(TaskName.ADMIN_USER_SUSPEND, kwargs)

    def admin_user_restore(self, **kwargs: Unpack[AdminUserRestoreArgs]) -> Any:
        """Restore a suspended managed user's access (by account ID or email)."""
        return self.task(TaskName.ADMIN_USER_RESTORE, kwargs)

    def admin_group_add_member(self, **kwargs: Unpack[AdminGroupAddMemberArgs]) -> Any:
        """Add a managed user (email or account ID) to a group (name or ID) of an Atlassian organization."""
        return self.task(TaskName.ADMIN_GROUP_ADD_MEMBER, kwargs)

    def admin_group_remove_member(self, **kwargs: Unpack[AdminGroupRemoveMemberArgs]) -> Any:
        """Remove a managed user from a group of an Atlassian organization."""
        return self.task(TaskName.ADMIN_GROUP_REMOVE_MEMBER, kwargs)

    def admin_group_list(self, **kwargs: Unpack[AdminGroupListArgs]) -> Any:
        """Groups of an Atlassian organization (every directory), optionally matching `search`."""
        return self.task(TaskName.ADMIN_GROUP_LIST, kwargs)

    def admin_domain_list(self, **kwargs: Unpack[AdminDomainListArgs]) -> Any:
        """Domains of an Atlassian organization with their verification status."""
        return self.task(TaskName.ADMIN_DOMAIN_LIST, kwargs)

    def admin_event_list(self, **kwargs: Unpack[AdminEventListArgs]) -> Any:
        """Audit log events of an Atlassian organization, newest first, filtered by words, dates and action."""
        return self.task(TaskName.ADMIN_EVENT_LIST, kwargs)
