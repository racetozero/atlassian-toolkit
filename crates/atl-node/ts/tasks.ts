// Generated from the task registry (crates/atl-sdk/src/tasks.rs) by
// crates/atl-sdk/tests/sdk_types.rs. Do not edit; run
// `ATL_UPDATE_SDK_TYPES=1 cargo test -p atlassian-toolkit --test sdk_types`.
import type { TaskResults } from "./common.ts"

/** The result type of a task: from `TaskResults` when known, else `unknown`. */
export type TaskResult<N extends string> = N extends keyof TaskResults ? TaskResults[N] : unknown

/** Arguments of `jira_issue_list`: Search Jira issues with filters (or extra JQL). Returns small records: key, summary, status, type, assignee, priority, updated, url. With no filters: your unresolved issues. */
export interface JiraIssueListArgs {
  /** Project key (CLI default: [jira] project in .atl.toml). */
  project?: string | null
  /** Statuses, for example `In Progress`; `~Done` excludes Done. Give several to match any of them. */
  status?: string[]
  /** `me`, `x` (unassigned), or a user; ~ negates. */
  assignee?: string | null
  /** `me` or a user; ~ negates. */
  reporter?: string | null
  /** Issue types, for example Bug. */
  type?: string[]
  /** Labels; ~ negates. */
  labels?: string[]
  /** Priorities, for example High; ~ negates. */
  priority?: string[]
  /** Components; ~ negates. */
  component?: string[]
  /** Resolutions, for example Fixed or Unresolved; ~ negates. */
  resolution?: string[]
  /** Parent issue key (an epic on Cloud, or the parent of sub-tasks). */
  parent?: string | null
  /** Only issues you watch. */
  watching?: boolean
  /** Issues you viewed recently, most recent first. */
  history?: boolean
  /** Sprint: current (open), next (future), closed, a sprint ID, or a sprint name. */
  sprint?: string | null
  /** Created since: -7d, -1h, today, week, month, or a date (2026-01-31). */
  created?: string | null
  /** Created before: -7d, today, week, month, or a date. */
  created_before?: string | null
  /** Updated since: -7d, -1h, today, week, month, or a date. */
  updated?: string | null
  /** Updated before: -7d, today, week, month, or a date. */
  updated_before?: string | null
  /** Full-text search over summary, description and comments. */
  text?: string | null
  /** Extra JQL combined with AND; may end with ORDER BY. */
  jql?: string | null
  /** Sort field (default updated), newest first unless `reverse`. */
  order_by?: string | null
  /** Oldest first. */
  reverse?: boolean
  /** Issues per page, 1-100 (default 50). */
  limit?: number
  /** `next_page_token` from the previous page. */
  page_token?: string | null
}

/** Arguments of `jira_issue_count`: Count Jira issues matching the same filters as issue_list. */
export interface JiraIssueCountArgs {
  /** Project key (CLI default: [jira] project in .atl.toml). */
  project?: string | null
  /** Statuses, for example `In Progress`; `~Done` excludes Done. Give several to match any of them. */
  status?: string[]
  /** `me`, `x` (unassigned), or a user; ~ negates. */
  assignee?: string | null
  /** `me` or a user; ~ negates. */
  reporter?: string | null
  /** Issue types, for example Bug. */
  type?: string[]
  /** Labels; ~ negates. */
  labels?: string[]
  /** Priorities, for example High; ~ negates. */
  priority?: string[]
  /** Components; ~ negates. */
  component?: string[]
  /** Resolutions, for example Fixed or Unresolved; ~ negates. */
  resolution?: string[]
  /** Parent issue key (an epic on Cloud, or the parent of sub-tasks). */
  parent?: string | null
  /** Only issues you watch. */
  watching?: boolean
  /** Issues you viewed recently, most recent first. */
  history?: boolean
  /** Sprint: current (open), next (future), closed, a sprint ID, or a sprint name. */
  sprint?: string | null
  /** Created since: -7d, -1h, today, week, month, or a date (2026-01-31). */
  created?: string | null
  /** Created before: -7d, today, week, month, or a date. */
  created_before?: string | null
  /** Updated since: -7d, -1h, today, week, month, or a date. */
  updated?: string | null
  /** Updated before: -7d, today, week, month, or a date. */
  updated_before?: string | null
  /** Full-text search over summary, description and comments. */
  text?: string | null
  /** Extra JQL combined with AND; may end with ORDER BY. */
  jql?: string | null
  /** Sort field (default updated), newest first unless `reverse`. */
  order_by?: string | null
  /** Oldest first. */
  reverse?: boolean
  /** Issues per page, 1-100 (default 50). */
  limit?: number
  /** `next_page_token` from the previous page. */
  page_token?: string | null
}

/** Arguments of `jira_issue_view`: One Jira issue with description and latest comments as Markdown, plus any `fields` asked for by name. */
export interface JiraIssueViewArgs {
  key: string
  /** How many of the latest comments to include. */
  comments?: number
  /** Extra fields to include, by name or ID (for example "Story Points"). */
  fields?: string[]
}

/** Arguments of `jira_issue_delete`: Delete a Jira issue (with `subtasks: true`, also its sub-tasks). Cannot be undone. */
export interface JiraIssueDeleteArgs {
  key: string
  /** Also delete the sub-tasks (Jira refuses otherwise when there are some). */
  subtasks?: boolean
}

/** Arguments of `jira_issue_watchers`: Who watches a Jira issue, and whether you do. */
export interface JiraIssueWatchersArgs {
  /** Issue key, for example ABC-123. */
  key: string
}

/** Arguments of `jira_issue_watch`: Add a watcher to a Jira issue (default: you). */
export interface JiraIssueWatchArgs {
  key: string
  /** `me`, an email, or a name. */
  user?: string
}

/** Arguments of `jira_issue_unwatch`: Remove a watcher from a Jira issue (default: you). */
export interface JiraIssueUnwatchArgs {
  key: string
  /** `me`, an email, or a name. */
  user?: string
}

/** Arguments of `jira_field_list`: Jira system and custom fields (ID, name, type), optionally filtered by part of the name. Use the names in the `fields` argument of issue create, edit and view. */
export interface JiraFieldListArgs {
  /** Part of the field name or ID. */
  filter?: string | null
}

/** Arguments of `jira_issue_create`: Create a Jira issue; the description is Markdown. */
export interface JiraIssueCreateArgs {
  project: string
  /** Issue type name, for example Task, Bug, Story. */
  type?: string
  summary: string
  /** Description in Markdown. */
  description?: string | null
  /** `me` or a user. */
  assignee?: string | null
  labels?: string[]
  priority?: string | null
  /** Parent issue key. */
  parent?: string | null
  components?: string[]
  fix_versions?: string[]
  /** Versions the problem affects. */
  affects_versions?: string[]
  /** Original estimate, for example "2h". */
  estimate?: string | null
  /** `me` or a user. */
  reporter?: string | null
  /** Other fields by name or ID, for example {"Story Points": "3"}. */
  fields?: Record<string, unknown>
}

/** Arguments of `jira_issue_edit`: Change a Jira issue's summary, description (Markdown), priority or labels. */
export interface JiraIssueEditArgs {
  key: string
  summary?: string | null
  /** New description in Markdown. */
  description?: string | null
  priority?: string | null
  /** `me`, `x` (unassigned), or a user. */
  assignee?: string | null
  /** New parent issue key. */
  parent?: string | null
  reporter?: string | null
  /** Original estimate, for example "2h". */
  estimate?: string | null
  add_labels?: string[]
  remove_labels?: string[]
  add_components?: string[]
  remove_components?: string[]
  add_fix_versions?: string[]
  remove_fix_versions?: string[]
  add_affects_versions?: string[]
  remove_affects_versions?: string[]
  /** Other fields by name or ID, for example {"Story Points": "5"}. */
  fields?: Record<string, unknown>
  /** Do not email watchers (Jira may need admin rights for it). */
  skip_notify?: boolean
}

/** Arguments of `jira_issue_assign`: Assign a Jira issue to me, nobody (x), or a user. */
export interface JiraIssueAssignArgs {
  key: string
  /** `me`, `x` (unassigned), or a user (email or name). */
  user: string
}

/** Arguments of `jira_issue_transitions`: Statuses a Jira issue can move to. */
export interface JiraIssueTransitionsArgs {
  /** Issue key, for example ABC-123. */
  key: string
}

/** Arguments of `jira_issue_move`: Move a Jira issue to another status (by status or transition name). */
export interface JiraIssueMoveArgs {
  key: string
  /** Target status or transition name. */
  status: string
  /** Optional Markdown comment added with the transition. */
  comment?: string | null
  /** Resolution to set, for example Fixed (must be on the transition screen). */
  resolution?: string | null
  /** Assignee to set: `me`, `x`, or a user. */
  assignee?: string | null
}

/** Arguments of `jira_issue_comment`: Add a Markdown comment to a Jira issue. */
export interface JiraIssueCommentArgs {
  key: string
  /** Comment in Markdown. */
  body: string
  /** A Jira Service Management internal note (customers do not see it). */
  internal?: boolean
}

/** Arguments of `jira_issue_link`: Link two Jira issues so that 'from <kind> to' reads true, for example from A-1 kind blocks to A-2. */
export interface JiraIssueLinkArgs {
  from: string
  to: string
  /** Link type name ("Blocks"), outward phrase ("blocks"), or inward phrase ("is blocked by"); reads "from <kind> to". */
  kind: string
}

/** Arguments of `jira_issue_unlink`: Remove the links between two Jira issues (optionally only one link type). */
export interface JiraIssueUnlinkArgs {
  from: string
  to: string
  /** Only links of this type (name, outward or inward phrase). */
  kind?: string | null
}

/** Arguments of `jira_issue_remote_link`: Add a web link (URL and title) to a Jira issue. */
export interface JiraIssueRemoteLinkArgs {
  key: string
  url: string
  /** Link text (default: the URL). */
  title?: string | null
}

/** Arguments of `jira_comment_list`: Every comment of a Jira issue, oldest first, with IDs, as Markdown. */
export interface JiraCommentListArgs {
  key: string
  limit?: number
}

/** Arguments of `jira_comment_edit`: Replace the text of a Jira comment (Markdown). */
export interface JiraCommentEditArgs {
  key: string
  /** Comment ID (see jira_comment_list). */
  id: string
  /** New text in Markdown. */
  body: string
}

/** Arguments of `jira_comment_delete`: Delete a Jira comment. Cannot be undone. */
export interface JiraCommentDeleteArgs {
  key: string
  id: string
}

/** Arguments of `jira_worklog_list`: Time logged on a Jira issue: author, time spent, start, comment. */
export interface JiraWorklogListArgs {
  /** Issue key, for example ABC-123. */
  key: string
}

/** Arguments of `jira_issue_history`: Field changes of a Jira issue, newest first: when, who, field, from, to. */
export interface JiraIssueHistoryArgs {
  key: string
  limit?: number
}

/** Arguments of `jira_component_list`: Components of a Jira project with their leads. */
export interface JiraComponentListArgs {
  /** Project key, for example ABC. */
  project: string
}

/** Arguments of `jira_sprint_create`: Create a future sprint on a board (default: the project's first board). */
export interface JiraSprintCreateArgs {
  name: string
  /** Board ID (default: the first board of `project`). */
  board?: number | null
  project?: string | null
  /** YYYY-MM-DD or ISO 8601. */
  start?: string | null
  end?: string | null
  goal?: string | null
}

/** Arguments of `jira_sprint_start`: Start a future sprint; Jira needs a start and an end date. */
export interface JiraSprintStartArgs {
  sprint: number
  /** YYYY-MM-DD or ISO 8601 (default: the sprint's start, else now). */
  start?: string | null
  /** YYYY-MM-DD or ISO 8601 (default: the sprint's end; required when it has none). */
  end?: string | null
}

/** Arguments of `jira_sprint_close`: Complete an active sprint. */
export interface JiraSprintCloseArgs {
  sprint: number
}

/** Arguments of `jira_release_list`: Versions (releases) of a Jira project: name, released, archived, release date. */
export interface JiraReleaseListArgs {
  /** Project key. */
  project: string
  /** unreleased, released, archived, or all (default). */
  state?: string
}

/** Arguments of `jira_issue_link_types`: Jira issue link types with their outward and inward phrases. */
export interface JiraIssueLinkTypesArgs {
}

/** Arguments of `jira_issue_clone`: Copy a Jira issue (summary, description, type, labels, priority, components, parent). */
export interface JiraIssueCloneArgs {
  key: string
  /** Summary for the copy (default: the original's). */
  summary?: string | null
  priority?: string | null
  /** `me` or a user. */
  assignee?: string | null
  /** Labels for the copy (default: the original's). */
  labels?: string[]
  /** "FIND:REPLACE" pairs applied to the summary and description. */
  replace?: string[]
}

/** Arguments of `jira_issue_worklog`: Log time on a Jira issue. */
export interface JiraIssueWorklogArgs {
  key: string
  /** Time spent, for example "2h 30m" or "1d". */
  time: string
  comment?: string | null
  /** Start time, for example 2026-11-09T09:00:00.000+0000. */
  started?: string | null
}

/** Arguments of `jira_issue_attach`: Upload local files to a Jira issue. */
export interface JiraIssueAttachArgs {
  key: string
  /** Local file paths to upload. */
  files: string[]
}

/** Arguments of `jira_issue_attachments`: List a Jira issue's attachments. */
export interface JiraIssueAttachmentsArgs {
  /** Issue key, for example ABC-123. */
  key: string
}

/** Arguments of `jira_jql_check`: Validate JQL with Jira's parser; returns valid, errors, warnings. */
export interface JiraJqlCheckArgs {
  jql: string
}

/** Arguments of `jira_jql_fields`: Fields (with operators) and functions usable in JQL on this site. */
export interface JiraJqlFieldsArgs {
}

/** Arguments of `jira_jql_suggest`: Value suggestions for a JQL field, for example field=status prefix=In. */
export interface JiraJqlSuggestArgs {
  /** JQL field name, for example status, project, assignee. */
  field: string
  /** Start of the value. */
  prefix?: string
}

/** Arguments of `jira_filter_list`: Saved Jira filters (Cloud: all visible, Data Center: favourites). */
export interface JiraFilterListArgs {
  /** Part of the filter name. */
  name?: string | null
}

/** Arguments of `jira_filter_run`: Run a saved Jira filter by ID or exact name; returns issue records. */
export interface JiraFilterRunArgs {
  /** Filter ID or exact name. */
  filter: string
  limit?: number
  page_token?: string | null
}

/** Arguments of `jira_epic_list`: Open epics, optionally of one project. */
export interface JiraEpicListArgs {
  project?: string | null
  limit?: number
}

/** Arguments of `jira_epic_add`: Move issues into an epic. */
export interface JiraEpicAddArgs {
  /** Epic key. */
  epic: string
  issues: string[]
}

/** Arguments of `jira_epic_remove`: Remove issues from their epic. */
export interface JiraEpicRemoveArgs {
  issues: string[]
}

/** Arguments of `jira_board_list`: Jira Software boards, optionally of one project. */
export interface JiraBoardListArgs {
  project?: string | null
}

/** Arguments of `jira_sprint_list`: Sprints of a board. */
export interface JiraSprintListArgs {
  /** Board ID (default: the project's first board). */
  board?: number | null
  project?: string | null
  /** active, future, closed, or a comma-separated mix. */
  state?: string
}

/** Arguments of `jira_sprint_add`: Move issues into a sprint. */
export interface JiraSprintAddArgs {
  sprint: number
  issues: string[]
}

/** Arguments of `jira_project_list`: Jira projects. */
export interface JiraProjectListArgs {
}

/** Arguments of `jira_issue_types`: Issue type names of a Jira project (without sub-task types), for the `type` argument of jira_issue_create. */
export interface JiraIssueTypesArgs {
  /** Project key, for example ABC. */
  project: string
}

/** Arguments of `jira_me`: The signed-in Jira user. */
export interface JiraMeArgs {
}

/** Arguments of `confluence_page_get`: A Confluence page as Markdown (with title, space, version, url). */
export interface ConfluencePageGetArgs {
  /** Page ID, page URL, or SPACE:Title. */
  page: string
  /** Also return `document`: the Markdown with front matter (id, title, space, parent, version), ready for confluence_page_push. */
  front_matter?: boolean
  /** Also return the storage-format XHTML. */
  storage?: boolean
  /** An older version number (see confluence_page_history). */
  version?: number | null
}

/** Arguments of `confluence_page_create`: Create a Confluence page from Markdown (or storage format). */
export interface ConfluencePageCreateArgs {
  /** Space key. */
  space: string
  title: string
  /** Parent page: ID, URL, or SPACE:Title. */
  parent?: string | null
  /** Body in Markdown. */
  markdown?: string | null
  /** Body in storage format (instead of markdown). */
  storage?: string | null
}

/** Arguments of `confluence_blog_list`: Blog posts of a Confluence space, newest first. */
export interface ConfluenceBlogListArgs {
  space: string
  limit?: number
}

/** Arguments of `confluence_blog_get`: One Confluence blog post with its body as Markdown. */
export interface ConfluenceBlogGetArgs {
  /** Blog post ID. */
  id: string
}

/** Arguments of `confluence_blog_create`: Publish a Confluence blog post from Markdown. */
export interface ConfluenceBlogCreateArgs {
  space: string
  title: string
  /** Body in Markdown. */
  markdown?: string | null
  /** Body in storage format (instead of markdown). */
  storage?: string | null
}

/** Arguments of `confluence_space_create`: Create a Confluence space with a key, a name and an optional plain-text description. */
export interface ConfluenceSpaceCreateArgs {
  key: string
  name: string
  /** Plain text. */
  description?: string | null
}

/** Arguments of `confluence_page_update`: Replace a Confluence page's title and/or body; the version is bumped automatically. */
export interface ConfluencePageUpdateArgs {
  page: string
  title?: string | null
  markdown?: string | null
  storage?: string | null
}

/** Arguments of `confluence_page_delete`: Delete a Confluence page. */
export interface ConfluencePageDeleteArgs {
  /** Page ID, page URL, or SPACE:Title. */
  page: string
}

/** Arguments of `confluence_page_children`: Children of a Confluence page (Cloud: also folders), or with `recursive` the whole tree, depth first, each row with type, depth and parent. */
export interface ConfluencePageChildrenArgs {
  /** Page ID, page URL, or SPACE:Title. */
  page: string
  /** The whole tree below the page, not only direct children. */
  recursive?: boolean
  /** With `recursive`, stop at this depth (1 = direct children). */
  max_depth?: number | null
}

/** Arguments of `confluence_page_find`: Confluence pages whose title contains the words, optionally in one space. */
export interface ConfluencePageFindArgs {
  /** Words the title contains. */
  title: string
  space?: string | null
  limit?: number
}

/** Arguments of `confluence_page_attach`: Upload local files to a Confluence page. */
export interface ConfluencePageAttachArgs {
  /** Upload new versions of attachments with the same name. */
  replace?: boolean
  /** Page ID, page URL, or SPACE:Title. */
  page: string
  /** Local file paths to upload. */
  files: string[]
}

/** Arguments of `confluence_page_attachments`: Attachments of a Confluence page with download URLs. */
export interface ConfluencePageAttachmentsArgs {
  page: string
  /** Only titles matching this glob, for example "*.png". */
  pattern?: string | null
}

/** Arguments of `confluence_page_comments`: Footer (or with `inline`, inline) comments of a Confluence page, oldest first, as Markdown, with the replied-to `parent`. */
export interface ConfluencePageCommentsArgs {
  page: string
  limit?: number
  /** Inline comments instead of footer comments. */
  inline?: boolean
}

/** Arguments of `confluence_page_comment`: Add a footer comment (Markdown) to a Confluence page, or a reply with `parent`. */
export interface ConfluencePageCommentArgs {
  page: string
  /** Comment in Markdown. */
  body: string
  /** Reply to this comment ID. */
  parent?: string | null
}

/** Arguments of `confluence_comment_delete`: Delete a Confluence comment by ID. */
export interface ConfluenceCommentDeleteArgs {
  /** Content ID. */
  id: string
}

/** Arguments of `confluence_attachment_delete`: Delete a Confluence attachment by ID. */
export interface ConfluenceAttachmentDeleteArgs {
  /** Content ID. */
  id: string
}

/** Arguments of `confluence_page_export`: Export a Confluence page (and with `recursive` its tree) to local Markdown folders: <title>/index.md with front matter and _attachments/. Changes nothing on the server. */
export interface ConfluencePageExportArgs {
  /** Page ID, page URL, or SPACE:Title. */
  page: string
  /** Folder to write into. */
  dest: string
  /** Also export the pages below. */
  recursive?: boolean
  max_depth?: number | null
  /** Title globs to skip (with their subtrees). */
  exclude?: string[]
  /** Download attachments into _attachments/ (default true). */
  attachments?: boolean
  /** List the files without writing. */
  dry_run?: boolean
}

/** Arguments of `confluence_page_push`: Publish local Markdown files: update the page in each file's front matter (refusing pages changed on the server unless `force`, skipping unchanged ones), create pages for new files under their folder's index.md, upload referenced images, and write ids and versions back. */
export interface ConfluencePagePushArgs {
  /** Markdown files or folders of them. */
  paths: string[]
  /** Space for new pages without one in their front matter. */
  space?: string | null
  /** Parent page for new pages without one. */
  parent?: string | null
  /** Overwrite pages that changed on the server since the file was written. */
  force?: boolean
  dry_run?: boolean
}

/** Arguments of `confluence_page_copy_tree`: Copy a Confluence page and the pages below it under another page (titles, bodies, labels). */
export interface ConfluencePageCopyTreeArgs {
  /** The page to copy with the pages below it. */
  page: string
  /** The new parent page. */
  target: string
  /** Appended to each title (default " (copy)" in the same space). */
  suffix?: string | null
  max_depth?: number | null
  exclude?: string[]
  dry_run?: boolean
}

/** Arguments of `confluence_page_labels`: Labels of a Confluence page. */
export interface ConfluencePageLabelsArgs {
  /** Page ID, page URL, or SPACE:Title. */
  page: string
}

/** Arguments of `confluence_page_label`: Add and remove labels on a Confluence page; returns the resulting labels. */
export interface ConfluencePageLabelArgs {
  page: string
  add?: string[]
  remove?: string[]
}

/** Arguments of `confluence_page_move`: Move a Confluence page under another page (position append) or next to it (before/after, Cloud only). */
export interface ConfluencePageMoveArgs {
  page: string
  /** The page to move next to or under. */
  target: string
  /** append (last child of target), before or after (sibling; Cloud only). */
  position?: string
}

/** Arguments of `confluence_page_history`: Versions of a Confluence page, newest first: number, author, date, message. Read one with confluence_page_get and `version`. */
export interface ConfluencePageHistoryArgs {
  /** Page ID, page URL, or SPACE:Title. */
  page: string
  limit?: number
}

/** Arguments of `confluence_space_list`: Confluence spaces. */
export interface ConfluenceSpaceListArgs {
  limit?: number
}

/** Arguments of `confluence_space_pages`: Top-level pages of a Confluence space. */
export interface ConfluenceSpacePagesArgs {
  space: string
  limit?: number
}

/** Arguments of `confluence_search`: Search Confluence by words or CQL; returns id, type, title, space, excerpt, url. */
export interface ConfluenceSearchArgs {
  /** Plain words (ignored when cql is given). */
  query?: string | null
  /** Raw CQL. */
  cql?: string | null
  space?: string | null
  /** page, blogpost, attachment, comment. */
  type?: string | null
  limit?: number
}

/** Arguments of `bitbucket_repo_list`: Bitbucket repositories of a workspace (Cloud) or project (Data Center), or the ones you belong to. */
export interface BitbucketRepoListArgs {
  /** Workspace (Cloud) or project key (Data Center); omit for your repositories. */
  owner?: string | null
  limit?: number
}

/** Arguments of `bitbucket_repo_view`: One Bitbucket repository with its default branch and URL. */
export interface BitbucketRepoViewArgs {
  /** WORKSPACE/REPO (Cloud) or PROJECT/REPO (Data Center). */
  repo: string
}

/** Arguments of `bitbucket_branch_list`: Branches of a Bitbucket repository, most recently changed first. */
export interface BitbucketBranchListArgs {
  repo: string
  /** Part of the branch name. */
  filter?: string | null
  limit?: number
}

/** Arguments of `bitbucket_pr_list`: Pull requests of a Bitbucket repository by state. */
export interface BitbucketPrListArgs {
  repo: string
  /** open, merged, declined, or all. */
  state?: string
  limit?: number
}

/** Arguments of `bitbucket_pr_view`: One pull request with description, reviewers and approvals. */
export interface BitbucketPrViewArgs {
  repo: string
  /** Pull request ID. */
  id: number
}

/** Arguments of `bitbucket_pr_create`: Open a pull request from a source branch. */
export interface BitbucketPrCreateArgs {
  repo: string
  title: string
  /** Source branch. */
  source: string
  /** Destination branch (default: the repository's main branch). */
  destination?: string | null
  /** Description in Markdown. */
  description?: string | null
  /** Open it as a draft (not ready for review). */
  draft?: boolean
}

/** Arguments of `bitbucket_pr_merge`: Merge a pull request. */
export interface BitbucketPrMergeArgs {
  repo: string
  id: number
  /** Cloud only: merge_commit, squash, or fast_forward. */
  strategy?: string | null
}

/** Arguments of `bitbucket_pr_approve`: Approve a pull request as the signed-in user. */
export interface BitbucketPrApproveArgs {
  repo: string
  /** Pull request ID. */
  id: number
}

/** Arguments of `bitbucket_pr_comment`: Add a Markdown comment to a pull request. */
export interface BitbucketPrCommentArgs {
  repo: string
  id: number
  /** Comment in Markdown. */
  body: string
}

/** Arguments of `bitbucket_pr_diff`: The unified diff of a pull request. */
export interface BitbucketPrDiffArgs {
  repo: string
  /** Pull request ID. */
  id: number
}

/** Arguments of `bitbucket_pr_comments`: Comments of a pull request, oldest first, as Markdown: general and inline (path, line), replies flattened with `parent`. */
export interface BitbucketPrCommentsArgs {
  repo: string
  id: number
  limit?: number
}

/** Arguments of `bitbucket_pr_checks`: Build statuses of a pull request's latest commit, with `passed` (none failed) and `pending` (some still running). */
export interface BitbucketPrChecksArgs {
  repo: string
  /** Pull request ID. */
  id: number
}

/** Arguments of `bitbucket_pr_unapprove`: Remove your approval from a pull request. */
export interface BitbucketPrUnapproveArgs {
  repo: string
  /** Pull request ID. */
  id: number
}

/** Arguments of `bitbucket_pr_ready`: Mark a draft pull request as ready for review (or, with `draft: true`, as a draft again). */
export interface BitbucketPrReadyArgs {
  repo: string
  id: number
  /** True turns it back into a draft. */
  draft?: boolean
}

/** Arguments of `bitbucket_branch_create`: Create a branch from another branch, a tag or a commit (default: the main branch). */
export interface BitbucketBranchCreateArgs {
  repo: string
  name: string
  /** Branch, tag or commit to start from (default: the main branch). */
  from?: string | null
}

/** Arguments of `bitbucket_branch_delete`: Delete a branch. Unmerged commits on it can be lost. */
export interface BitbucketBranchDeleteArgs {
  repo: string
  name: string
}

/** Arguments of `bitbucket_commit_list`: Recent commits of a branch, newest first: hash, author, date, first line of the message. */
export interface BitbucketCommitListArgs {
  repo: string
  /** Branch (default: the main branch). */
  branch?: string | null
  limit?: number
}

/** Arguments of `bitbucket_pipeline_list`: Latest Bitbucket Pipelines runs, newest first (Cloud only). */
export interface BitbucketPipelineListArgs {
  repo: string
  limit?: number
}

/** Arguments of `bitbucket_pipeline_run`: Start the pipeline of a branch, or a custom pipeline (Cloud only). */
export interface BitbucketPipelineRunArgs {
  repo: string
  branch: string
  /** Name of a custom pipeline in bitbucket-pipelines.yml. */
  custom?: string | null
}

/** Arguments of `bitbucket_pr_decline`: Decline a Bitbucket pull request (Cloud cannot reopen it). */
export interface BitbucketPrDeclineArgs {
  repo: string
  /** Pull request ID. */
  id: number
}

/** Arguments of `bitbucket_pr_request_changes`: Mark a Bitbucket pull request as needing changes by you (Cloud: request changes; Data Center: needs work). */
export interface BitbucketPrRequestChangesArgs {
  repo: string
  /** Pull request ID. */
  id: number
}

/** Arguments of `bitbucket_pr_edit`: Change a Bitbucket pull request's title, description (Markdown) or destination branch. */
export interface BitbucketPrEditArgs {
  repo: string
  id: number
  title?: string | null
  /** New description in Markdown. */
  description?: string | null
  /** New destination branch. */
  destination?: string | null
}

/** Arguments of `bitbucket_pr_reviewers`: Add and remove reviewers of a Bitbucket pull request; other reviewers are kept. */
export interface BitbucketPrReviewersArgs {
  repo: string
  id: number
  /** Users to add: display name, nickname, account ID or {uuid} (Cloud); username, name or email (Data Center). */
  add?: string[]
  /** Reviewers to remove, matched the same way. */
  remove?: string[]
}

/** Arguments of `bitbucket_pipeline_stop`: Stop a running Bitbucket pipeline by build number or UUID (Cloud only). */
export interface BitbucketPipelineStopArgs {
  repo: string
  /** Build number or pipeline UUID. */
  id: string
}

/** Arguments of `bitbucket_pipeline_view`: One Bitbucket pipeline by build number or UUID, with each step's state and result (Cloud only). */
export interface BitbucketPipelineViewArgs {
  repo: string
  /** Build number or pipeline UUID. */
  id: string
}

/** Arguments of `bitbucket_pipeline_logs`: Log text of one pipeline step (by name or UUID) or of every step (Cloud only). */
export interface BitbucketPipelineLogsArgs {
  repo: string
  /** Build number or pipeline UUID. */
  id: string
  /** Step name or UUID (default: every step). */
  step?: string | null
}

/** Arguments of `jsm_desk_list`: Jira Service Management service desks (id, project key, name). */
export interface JsmDeskListArgs {
}

/** Arguments of `jsm_request_types`: Request types of a service desk. */
export interface JsmRequestTypesArgs {
  /** Service desk ID or project key. */
  desk: string
}

/** Arguments of `jsm_queue_list`: Queues of a service desk with issue counts. */
export interface JsmQueueListArgs {
  /** Service desk ID or project key. */
  desk: string
}

/** Arguments of `jsm_queue_issues`: Issues in a service desk queue (by queue ID or name). */
export interface JsmQueueIssuesArgs {
  desk: string
  /** Queue ID or name. */
  queue: string
  limit?: number
}

/** Arguments of `jsm_request_list`: Customer requests by status, optionally of one service desk. */
export interface JsmRequestListArgs {
  /** Service desk ID or project key (default: all). */
  desk?: string | null
  /** open, closed, or all. */
  status?: string
  /** Words to find. */
  search?: string | null
  /** Only requests you raised. */
  mine?: boolean
  limit?: number
}

/** Arguments of `jsm_request_view`: One customer request with its description and latest comments. */
export interface JsmRequestViewArgs {
  /** Request key, for example HELP-7. */
  key: string
  /** How many of the latest comments to include. */
  comments?: number
}

/** Arguments of `jsm_request_create`: Raise a customer request, optionally for a customer. */
export interface JsmRequestCreateArgs {
  desk: string
  /** Request type ID or name (see jsm_request_types). */
  type: string
  summary: string
  description?: string | null
  /** Customer email or account ID to raise the request for. */
  on_behalf_of?: string | null
}

/** Arguments of `jsm_request_comment`: Reply to a customer request, or add an internal note. */
export interface JsmRequestCommentArgs {
  key: string
  body: string
  /** An internal note that customers cannot see. */
  internal?: boolean
}

/** Arguments of `jsm_request_transitions`: Transitions available on a customer request. */
export interface JsmRequestTransitionsArgs {
  /** Issue key, for example ABC-123. */
  key: string
}

/** Arguments of `jsm_request_move`: Run a customer request transition by name or ID. */
export interface JsmRequestMoveArgs {
  key: string
  /** Transition name or ID (see jsm_request_transitions). */
  transition: string
  comment?: string | null
}

/** Arguments of `jsm_request_approvals`: Approvals of a JSM request: name, decision (pending, approved, declined), approvers, and whether you can answer. */
export interface JsmRequestApprovalsArgs {
  /** Issue key, for example ABC-123. */
  key: string
}

/** Arguments of `jsm_request_approve`: Approve or decline (decision: decline) a JSM request's approval; without an ID, the one pending approval you can answer. */
export interface JsmRequestApproveArgs {
  key: string
  /** Approval ID (default: the one pending approval you can answer). */
  approval?: string | null
  /** approve or decline. */
  decision?: string
}

/** Arguments of `jsm_request_participants`: List a JSM request's participants, after adding and removing any (me, an email or a name). */
export interface JsmRequestParticipantsArgs {
  key: string
  /** Users to add: me, an email or a name. */
  add?: string[]
  remove?: string[]
}

/** Arguments of `jsm_request_sla`: SLAs of a JSM request: state (ongoing, paused, met, breached), time remaining and goal. */
export interface JsmRequestSlaArgs {
  /** Issue key, for example ABC-123. */
  key: string
}

/** Arguments of `jsm_request_attach`: Attach local files to a JSM request (public unless internal). */
export interface JsmRequestAttachArgs {
  key: string
  /** Local file paths to upload. */
  files: string[]
  /** Keep the files internal (agents only); customers see them otherwise. */
  internal?: boolean
}

/** Arguments of `jsm_request_attachments`: Attachments of a JSM request with download URLs. */
export interface JsmRequestAttachmentsArgs {
  /** Issue key, for example ABC-123. */
  key: string
}

/** Arguments of `trello_board_list`: Your open Trello boards. */
export interface TrelloBoardListArgs {
}

/** Arguments of `trello_list_list`: Open lists of a Trello board (board by name, URL, short link or ID). */
export interface TrelloListListArgs {
  /** Board name, URL, short link or ID. */
  board: string
}

/** Arguments of `trello_card_list`: Cards of a Trello board or one of its lists, or your cards on every board (`mine`). */
export interface TrelloCardListArgs {
  /** Board name, URL, short link or ID. */
  board?: string | null
  /** List name or ID on the board. */
  list?: string | null
  /** Your cards on every board instead. */
  mine?: boolean
}

/** Arguments of `trello_card_view`: One Trello card with list, board, description and latest comments. */
export interface TrelloCardViewArgs {
  /** Card URL, short link or ID. */
  card: string
}

/** Arguments of `trello_card_create`: Create a Trello card in a list (by name with `board`, or by list ID). */
export interface TrelloCardCreateArgs {
  /** Board name, URL, short link or ID (needed when `list` is a name). */
  board?: string | null
  /** List name or ID. */
  list: string
  name: string
  /** Description (Markdown). */
  description?: string | null
  /** Due date, for example 2026-12-01 or 2026-12-01T17:00:00Z. */
  due?: string | null
}

/** Arguments of `trello_card_move`: Move a Trello card to another list (by default on the card's own board). */
export interface TrelloCardMoveArgs {
  card: string
  /** Target list name or ID. */
  list: string
  /** Board of the list (default: the card's board). */
  board?: string | null
}

/** Arguments of `trello_card_comment`: Add a comment to a Trello card. */
export interface TrelloCardCommentArgs {
  card: string
  text: string
}

/** Arguments of `trello_card_archive`: Archive (close) a Trello card. */
export interface TrelloCardArchiveArgs {
  /** Card URL, short link or ID. */
  card: string
}

/** Arguments of `trello_card_edit`: Change a Trello card: name, description, due date, done, labels (by name or color) and members (username, name or me). */
export interface TrelloCardEditArgs {
  /** Card URL, short link or ID. */
  card: string
  name?: string | null
  /** Description (Markdown). */
  description?: string | null
  /** Due date or date-time; an empty string removes it. */
  due?: string | null
  /** Mark the due date complete (true) or not (false). */
  done?: boolean | null
  /** Label names or colors on the card's board. */
  add_labels?: string[]
  remove_labels?: string[]
  /** Members by username, full name, or me. */
  add_members?: string[]
  remove_members?: string[]
}

/** Arguments of `trello_checklist_list`: Checklists of a Trello card with their items and whether each is done. */
export interface TrelloChecklistListArgs {
  /** Card URL, short link or ID. */
  card: string
}

/** Arguments of `trello_checklist_create`: Add a checklist with items to a Trello card. */
export interface TrelloChecklistCreateArgs {
  card: string
  name: string
  /** Items to add, in order. */
  items?: string[]
}

/** Arguments of `trello_checklist_check`: Tick (or with done: false, untick) a Trello checklist item by name or ID. */
export interface TrelloChecklistCheckArgs {
  card: string
  /** Item name or ID. */
  item: string
  /** false unticks the item. */
  done?: boolean
}

/** Arguments of `admin_org_list`: Organizations the Atlassian Admin API key can see. */
export interface AdminOrgListArgs {
}

/** Arguments of `admin_directory_list`: User directories of an Atlassian organization. */
export interface AdminDirectoryListArgs {
  /** Organization ID or name (default: the only one the API key sees). */
  org?: string | null
}

/** Arguments of `admin_user_list`: Managed users of an Atlassian organization (every directory), optionally matching `search`. */
export interface AdminUserListArgs {
  org?: string | null
  /** Words to match in names and emails. */
  search?: string | null
  limit?: number
}

/** Arguments of `admin_user_suspend`: Suspend a managed user's access to every product (by account ID or email). */
export interface AdminUserSuspendArgs {
  org?: string | null
  /** Account ID or email. */
  user: string
  /** Directory ID (default: the directory where the user is found). */
  directory?: string | null
}

/** Arguments of `admin_user_restore`: Restore a suspended managed user's access (by account ID or email). */
export interface AdminUserRestoreArgs {
  org?: string | null
  /** Account ID or email. */
  user: string
  /** Directory ID (default: the directory where the user is found). */
  directory?: string | null
}

/** Arguments of `admin_group_add_member`: Add a managed user (email or account ID) to a group (name or ID) of an Atlassian organization. */
export interface AdminGroupAddMemberArgs {
  /** Organization ID or name (default: the only one). */
  org?: string | null
  /** Group ID or name. */
  group: string
  /** Email or account ID. */
  user: string
}

/** Arguments of `admin_group_remove_member`: Remove a managed user from a group of an Atlassian organization. */
export interface AdminGroupRemoveMemberArgs {
  /** Organization ID or name (default: the only one). */
  org?: string | null
  /** Group ID or name. */
  group: string
  /** Email or account ID. */
  user: string
}

/** Arguments of `admin_group_list`: Groups of an Atlassian organization (every directory), optionally matching `search`. */
export interface AdminGroupListArgs {
  org?: string | null
  /** Words to match in names and emails. */
  search?: string | null
  limit?: number
}

/** Arguments of `admin_domain_list`: Domains of an Atlassian organization with their verification status. */
export interface AdminDomainListArgs {
  /** Organization ID or name (default: the only one the API key sees). */
  org?: string | null
}

/** Arguments of `admin_event_list`: Audit log events of an Atlassian organization, newest first, filtered by words, dates and action. */
export interface AdminEventListArgs {
  org?: string | null
  /** Words to find in the events. */
  search?: string | null
  /** Start date or time, for example 2026-11-01. */
  from?: string | null
  /** End date or time. */
  to?: string | null
  /** Event action, for example user_added_to_group. */
  action?: string | null
  limit?: number
}

/** The arguments of every task, by task name. */
export interface TaskArgs {
  jira_issue_list: JiraIssueListArgs
  jira_issue_count: JiraIssueCountArgs
  jira_issue_view: JiraIssueViewArgs
  jira_issue_delete: JiraIssueDeleteArgs
  jira_issue_watchers: JiraIssueWatchersArgs
  jira_issue_watch: JiraIssueWatchArgs
  jira_issue_unwatch: JiraIssueUnwatchArgs
  jira_field_list: JiraFieldListArgs
  jira_issue_create: JiraIssueCreateArgs
  jira_issue_edit: JiraIssueEditArgs
  jira_issue_assign: JiraIssueAssignArgs
  jira_issue_transitions: JiraIssueTransitionsArgs
  jira_issue_move: JiraIssueMoveArgs
  jira_issue_comment: JiraIssueCommentArgs
  jira_issue_link: JiraIssueLinkArgs
  jira_issue_unlink: JiraIssueUnlinkArgs
  jira_issue_remote_link: JiraIssueRemoteLinkArgs
  jira_comment_list: JiraCommentListArgs
  jira_comment_edit: JiraCommentEditArgs
  jira_comment_delete: JiraCommentDeleteArgs
  jira_worklog_list: JiraWorklogListArgs
  jira_issue_history: JiraIssueHistoryArgs
  jira_component_list: JiraComponentListArgs
  jira_sprint_create: JiraSprintCreateArgs
  jira_sprint_start: JiraSprintStartArgs
  jira_sprint_close: JiraSprintCloseArgs
  jira_release_list: JiraReleaseListArgs
  jira_issue_link_types: JiraIssueLinkTypesArgs
  jira_issue_clone: JiraIssueCloneArgs
  jira_issue_worklog: JiraIssueWorklogArgs
  jira_issue_attach: JiraIssueAttachArgs
  jira_issue_attachments: JiraIssueAttachmentsArgs
  jira_jql_check: JiraJqlCheckArgs
  jira_jql_fields: JiraJqlFieldsArgs
  jira_jql_suggest: JiraJqlSuggestArgs
  jira_filter_list: JiraFilterListArgs
  jira_filter_run: JiraFilterRunArgs
  jira_epic_list: JiraEpicListArgs
  jira_epic_add: JiraEpicAddArgs
  jira_epic_remove: JiraEpicRemoveArgs
  jira_board_list: JiraBoardListArgs
  jira_sprint_list: JiraSprintListArgs
  jira_sprint_add: JiraSprintAddArgs
  jira_project_list: JiraProjectListArgs
  jira_issue_types: JiraIssueTypesArgs
  jira_me: JiraMeArgs
  confluence_page_get: ConfluencePageGetArgs
  confluence_page_create: ConfluencePageCreateArgs
  confluence_blog_list: ConfluenceBlogListArgs
  confluence_blog_get: ConfluenceBlogGetArgs
  confluence_blog_create: ConfluenceBlogCreateArgs
  confluence_space_create: ConfluenceSpaceCreateArgs
  confluence_page_update: ConfluencePageUpdateArgs
  confluence_page_delete: ConfluencePageDeleteArgs
  confluence_page_children: ConfluencePageChildrenArgs
  confluence_page_find: ConfluencePageFindArgs
  confluence_page_attach: ConfluencePageAttachArgs
  confluence_page_attachments: ConfluencePageAttachmentsArgs
  confluence_page_comments: ConfluencePageCommentsArgs
  confluence_page_comment: ConfluencePageCommentArgs
  confluence_comment_delete: ConfluenceCommentDeleteArgs
  confluence_attachment_delete: ConfluenceAttachmentDeleteArgs
  confluence_page_export: ConfluencePageExportArgs
  confluence_page_push: ConfluencePagePushArgs
  confluence_page_copy_tree: ConfluencePageCopyTreeArgs
  confluence_page_labels: ConfluencePageLabelsArgs
  confluence_page_label: ConfluencePageLabelArgs
  confluence_page_move: ConfluencePageMoveArgs
  confluence_page_history: ConfluencePageHistoryArgs
  confluence_space_list: ConfluenceSpaceListArgs
  confluence_space_pages: ConfluenceSpacePagesArgs
  confluence_search: ConfluenceSearchArgs
  bitbucket_repo_list: BitbucketRepoListArgs
  bitbucket_repo_view: BitbucketRepoViewArgs
  bitbucket_branch_list: BitbucketBranchListArgs
  bitbucket_pr_list: BitbucketPrListArgs
  bitbucket_pr_view: BitbucketPrViewArgs
  bitbucket_pr_create: BitbucketPrCreateArgs
  bitbucket_pr_merge: BitbucketPrMergeArgs
  bitbucket_pr_approve: BitbucketPrApproveArgs
  bitbucket_pr_comment: BitbucketPrCommentArgs
  bitbucket_pr_diff: BitbucketPrDiffArgs
  bitbucket_pr_comments: BitbucketPrCommentsArgs
  bitbucket_pr_checks: BitbucketPrChecksArgs
  bitbucket_pr_unapprove: BitbucketPrUnapproveArgs
  bitbucket_pr_ready: BitbucketPrReadyArgs
  bitbucket_branch_create: BitbucketBranchCreateArgs
  bitbucket_branch_delete: BitbucketBranchDeleteArgs
  bitbucket_commit_list: BitbucketCommitListArgs
  bitbucket_pipeline_list: BitbucketPipelineListArgs
  bitbucket_pipeline_run: BitbucketPipelineRunArgs
  bitbucket_pr_decline: BitbucketPrDeclineArgs
  bitbucket_pr_request_changes: BitbucketPrRequestChangesArgs
  bitbucket_pr_edit: BitbucketPrEditArgs
  bitbucket_pr_reviewers: BitbucketPrReviewersArgs
  bitbucket_pipeline_stop: BitbucketPipelineStopArgs
  bitbucket_pipeline_view: BitbucketPipelineViewArgs
  bitbucket_pipeline_logs: BitbucketPipelineLogsArgs
  jsm_desk_list: JsmDeskListArgs
  jsm_request_types: JsmRequestTypesArgs
  jsm_queue_list: JsmQueueListArgs
  jsm_queue_issues: JsmQueueIssuesArgs
  jsm_request_list: JsmRequestListArgs
  jsm_request_view: JsmRequestViewArgs
  jsm_request_create: JsmRequestCreateArgs
  jsm_request_comment: JsmRequestCommentArgs
  jsm_request_transitions: JsmRequestTransitionsArgs
  jsm_request_move: JsmRequestMoveArgs
  jsm_request_approvals: JsmRequestApprovalsArgs
  jsm_request_approve: JsmRequestApproveArgs
  jsm_request_participants: JsmRequestParticipantsArgs
  jsm_request_sla: JsmRequestSlaArgs
  jsm_request_attach: JsmRequestAttachArgs
  jsm_request_attachments: JsmRequestAttachmentsArgs
  trello_board_list: TrelloBoardListArgs
  trello_list_list: TrelloListListArgs
  trello_card_list: TrelloCardListArgs
  trello_card_view: TrelloCardViewArgs
  trello_card_create: TrelloCardCreateArgs
  trello_card_move: TrelloCardMoveArgs
  trello_card_comment: TrelloCardCommentArgs
  trello_card_archive: TrelloCardArchiveArgs
  trello_card_edit: TrelloCardEditArgs
  trello_checklist_list: TrelloChecklistListArgs
  trello_checklist_create: TrelloChecklistCreateArgs
  trello_checklist_check: TrelloChecklistCheckArgs
  admin_org_list: AdminOrgListArgs
  admin_directory_list: AdminDirectoryListArgs
  admin_user_list: AdminUserListArgs
  admin_user_suspend: AdminUserSuspendArgs
  admin_user_restore: AdminUserRestoreArgs
  admin_group_add_member: AdminGroupAddMemberArgs
  admin_group_remove_member: AdminGroupRemoveMemberArgs
  admin_group_list: AdminGroupListArgs
  admin_domain_list: AdminDomainListArgs
  admin_event_list: AdminEventListArgs
}
export type TaskName = keyof TaskArgs

/** Every value of `AtlError.code`. */
export type ErrorCode = "auth" | "api" | "invalid" | "unknown" | "read_only" | "dry_run" | "input_required" | "internal"

/** One typed method per task; each SDK supplies `task()` over its own transport. */
export abstract class GeneratedTasks {
  /** Runs any task from `tasks()`, for example `task("jira_issue_view", { key: "A-1" })`. */
  abstract task<T = unknown>(name: string, args?: Record<string, unknown>): Promise<T>

  /** Search Jira issues with filters (or extra JQL). Returns small records: key, summary, status, type, assignee, priority, updated, url. With no filters: your unresolved issues. */
  jiraIssueList(args: JiraIssueListArgs = {}): Promise<TaskResult<"jira_issue_list">> {
    return this.task("jira_issue_list", { ...args })
  }

  /** Count Jira issues matching the same filters as issue_list. */
  jiraIssueCount(args: JiraIssueCountArgs = {}): Promise<TaskResult<"jira_issue_count">> {
    return this.task("jira_issue_count", { ...args })
  }

  /** One Jira issue with description and latest comments as Markdown, plus any `fields` asked for by name. */
  jiraIssueView(args: JiraIssueViewArgs): Promise<TaskResult<"jira_issue_view">> {
    return this.task("jira_issue_view", { ...args })
  }

  /** Delete a Jira issue (with `subtasks: true`, also its sub-tasks). Cannot be undone. */
  jiraIssueDelete(args: JiraIssueDeleteArgs): Promise<TaskResult<"jira_issue_delete">> {
    return this.task("jira_issue_delete", { ...args })
  }

  /** Who watches a Jira issue, and whether you do. */
  jiraIssueWatchers(args: JiraIssueWatchersArgs): Promise<TaskResult<"jira_issue_watchers">> {
    return this.task("jira_issue_watchers", { ...args })
  }

  /** Add a watcher to a Jira issue (default: you). */
  jiraIssueWatch(args: JiraIssueWatchArgs): Promise<TaskResult<"jira_issue_watch">> {
    return this.task("jira_issue_watch", { ...args })
  }

  /** Remove a watcher from a Jira issue (default: you). */
  jiraIssueUnwatch(args: JiraIssueUnwatchArgs): Promise<TaskResult<"jira_issue_unwatch">> {
    return this.task("jira_issue_unwatch", { ...args })
  }

  /** Jira system and custom fields (ID, name, type), optionally filtered by part of the name. Use the names in the `fields` argument of issue create, edit and view. */
  jiraFieldList(args: JiraFieldListArgs = {}): Promise<TaskResult<"jira_field_list">> {
    return this.task("jira_field_list", { ...args })
  }

  /** Create a Jira issue; the description is Markdown. */
  jiraIssueCreate(args: JiraIssueCreateArgs): Promise<TaskResult<"jira_issue_create">> {
    return this.task("jira_issue_create", { ...args })
  }

  /** Change a Jira issue's summary, description (Markdown), priority or labels. */
  jiraIssueEdit(args: JiraIssueEditArgs): Promise<TaskResult<"jira_issue_edit">> {
    return this.task("jira_issue_edit", { ...args })
  }

  /** Assign a Jira issue to me, nobody (x), or a user. */
  jiraIssueAssign(args: JiraIssueAssignArgs): Promise<TaskResult<"jira_issue_assign">> {
    return this.task("jira_issue_assign", { ...args })
  }

  /** Statuses a Jira issue can move to. */
  jiraIssueTransitions(args: JiraIssueTransitionsArgs): Promise<TaskResult<"jira_issue_transitions">> {
    return this.task("jira_issue_transitions", { ...args })
  }

  /** Move a Jira issue to another status (by status or transition name). */
  jiraIssueMove(args: JiraIssueMoveArgs): Promise<TaskResult<"jira_issue_move">> {
    return this.task("jira_issue_move", { ...args })
  }

  /** Add a Markdown comment to a Jira issue. */
  jiraIssueComment(args: JiraIssueCommentArgs): Promise<TaskResult<"jira_issue_comment">> {
    return this.task("jira_issue_comment", { ...args })
  }

  /** Link two Jira issues so that 'from <kind> to' reads true, for example from A-1 kind blocks to A-2. */
  jiraIssueLink(args: JiraIssueLinkArgs): Promise<TaskResult<"jira_issue_link">> {
    return this.task("jira_issue_link", { ...args })
  }

  /** Remove the links between two Jira issues (optionally only one link type). */
  jiraIssueUnlink(args: JiraIssueUnlinkArgs): Promise<TaskResult<"jira_issue_unlink">> {
    return this.task("jira_issue_unlink", { ...args })
  }

  /** Add a web link (URL and title) to a Jira issue. */
  jiraIssueRemoteLink(args: JiraIssueRemoteLinkArgs): Promise<TaskResult<"jira_issue_remote_link">> {
    return this.task("jira_issue_remote_link", { ...args })
  }

  /** Every comment of a Jira issue, oldest first, with IDs, as Markdown. */
  jiraCommentList(args: JiraCommentListArgs): Promise<TaskResult<"jira_comment_list">> {
    return this.task("jira_comment_list", { ...args })
  }

  /** Replace the text of a Jira comment (Markdown). */
  jiraCommentEdit(args: JiraCommentEditArgs): Promise<TaskResult<"jira_comment_edit">> {
    return this.task("jira_comment_edit", { ...args })
  }

  /** Delete a Jira comment. Cannot be undone. */
  jiraCommentDelete(args: JiraCommentDeleteArgs): Promise<TaskResult<"jira_comment_delete">> {
    return this.task("jira_comment_delete", { ...args })
  }

  /** Time logged on a Jira issue: author, time spent, start, comment. */
  jiraWorklogList(args: JiraWorklogListArgs): Promise<TaskResult<"jira_worklog_list">> {
    return this.task("jira_worklog_list", { ...args })
  }

  /** Field changes of a Jira issue, newest first: when, who, field, from, to. */
  jiraIssueHistory(args: JiraIssueHistoryArgs): Promise<TaskResult<"jira_issue_history">> {
    return this.task("jira_issue_history", { ...args })
  }

  /** Components of a Jira project with their leads. */
  jiraComponentList(args: JiraComponentListArgs): Promise<TaskResult<"jira_component_list">> {
    return this.task("jira_component_list", { ...args })
  }

  /** Create a future sprint on a board (default: the project's first board). */
  jiraSprintCreate(args: JiraSprintCreateArgs): Promise<TaskResult<"jira_sprint_create">> {
    return this.task("jira_sprint_create", { ...args })
  }

  /** Start a future sprint; Jira needs a start and an end date. */
  jiraSprintStart(args: JiraSprintStartArgs): Promise<TaskResult<"jira_sprint_start">> {
    return this.task("jira_sprint_start", { ...args })
  }

  /** Complete an active sprint. */
  jiraSprintClose(args: JiraSprintCloseArgs): Promise<TaskResult<"jira_sprint_close">> {
    return this.task("jira_sprint_close", { ...args })
  }

  /** Versions (releases) of a Jira project: name, released, archived, release date. */
  jiraReleaseList(args: JiraReleaseListArgs): Promise<TaskResult<"jira_release_list">> {
    return this.task("jira_release_list", { ...args })
  }

  /** Jira issue link types with their outward and inward phrases. */
  jiraIssueLinkTypes(args: JiraIssueLinkTypesArgs = {}): Promise<TaskResult<"jira_issue_link_types">> {
    return this.task("jira_issue_link_types", { ...args })
  }

  /** Copy a Jira issue (summary, description, type, labels, priority, components, parent). */
  jiraIssueClone(args: JiraIssueCloneArgs): Promise<TaskResult<"jira_issue_clone">> {
    return this.task("jira_issue_clone", { ...args })
  }

  /** Log time on a Jira issue. */
  jiraIssueWorklog(args: JiraIssueWorklogArgs): Promise<TaskResult<"jira_issue_worklog">> {
    return this.task("jira_issue_worklog", { ...args })
  }

  /** Upload local files to a Jira issue. */
  jiraIssueAttach(args: JiraIssueAttachArgs): Promise<TaskResult<"jira_issue_attach">> {
    return this.task("jira_issue_attach", { ...args })
  }

  /** List a Jira issue's attachments. */
  jiraIssueAttachments(args: JiraIssueAttachmentsArgs): Promise<TaskResult<"jira_issue_attachments">> {
    return this.task("jira_issue_attachments", { ...args })
  }

  /** Validate JQL with Jira's parser; returns valid, errors, warnings. */
  jiraJqlCheck(args: JiraJqlCheckArgs): Promise<TaskResult<"jira_jql_check">> {
    return this.task("jira_jql_check", { ...args })
  }

  /** Fields (with operators) and functions usable in JQL on this site. */
  jiraJqlFields(args: JiraJqlFieldsArgs = {}): Promise<TaskResult<"jira_jql_fields">> {
    return this.task("jira_jql_fields", { ...args })
  }

  /** Value suggestions for a JQL field, for example field=status prefix=In. */
  jiraJqlSuggest(args: JiraJqlSuggestArgs): Promise<TaskResult<"jira_jql_suggest">> {
    return this.task("jira_jql_suggest", { ...args })
  }

  /** Saved Jira filters (Cloud: all visible, Data Center: favourites). */
  jiraFilterList(args: JiraFilterListArgs = {}): Promise<TaskResult<"jira_filter_list">> {
    return this.task("jira_filter_list", { ...args })
  }

  /** Run a saved Jira filter by ID or exact name; returns issue records. */
  jiraFilterRun(args: JiraFilterRunArgs): Promise<TaskResult<"jira_filter_run">> {
    return this.task("jira_filter_run", { ...args })
  }

  /** Open epics, optionally of one project. */
  jiraEpicList(args: JiraEpicListArgs = {}): Promise<TaskResult<"jira_epic_list">> {
    return this.task("jira_epic_list", { ...args })
  }

  /** Move issues into an epic. */
  jiraEpicAdd(args: JiraEpicAddArgs): Promise<TaskResult<"jira_epic_add">> {
    return this.task("jira_epic_add", { ...args })
  }

  /** Remove issues from their epic. */
  jiraEpicRemove(args: JiraEpicRemoveArgs): Promise<TaskResult<"jira_epic_remove">> {
    return this.task("jira_epic_remove", { ...args })
  }

  /** Jira Software boards, optionally of one project. */
  jiraBoardList(args: JiraBoardListArgs = {}): Promise<TaskResult<"jira_board_list">> {
    return this.task("jira_board_list", { ...args })
  }

  /** Sprints of a board. */
  jiraSprintList(args: JiraSprintListArgs = {}): Promise<TaskResult<"jira_sprint_list">> {
    return this.task("jira_sprint_list", { ...args })
  }

  /** Move issues into a sprint. */
  jiraSprintAdd(args: JiraSprintAddArgs): Promise<TaskResult<"jira_sprint_add">> {
    return this.task("jira_sprint_add", { ...args })
  }

  /** Jira projects. */
  jiraProjectList(args: JiraProjectListArgs = {}): Promise<TaskResult<"jira_project_list">> {
    return this.task("jira_project_list", { ...args })
  }

  /** Issue type names of a Jira project (without sub-task types), for the `type` argument of jira_issue_create. */
  jiraIssueTypes(args: JiraIssueTypesArgs): Promise<TaskResult<"jira_issue_types">> {
    return this.task("jira_issue_types", { ...args })
  }

  /** The signed-in Jira user. */
  jiraMe(args: JiraMeArgs = {}): Promise<TaskResult<"jira_me">> {
    return this.task("jira_me", { ...args })
  }

  /** A Confluence page as Markdown (with title, space, version, url). */
  confluencePageGet(args: ConfluencePageGetArgs): Promise<TaskResult<"confluence_page_get">> {
    return this.task("confluence_page_get", { ...args })
  }

  /** Create a Confluence page from Markdown (or storage format). */
  confluencePageCreate(args: ConfluencePageCreateArgs): Promise<TaskResult<"confluence_page_create">> {
    return this.task("confluence_page_create", { ...args })
  }

  /** Blog posts of a Confluence space, newest first. */
  confluenceBlogList(args: ConfluenceBlogListArgs): Promise<TaskResult<"confluence_blog_list">> {
    return this.task("confluence_blog_list", { ...args })
  }

  /** One Confluence blog post with its body as Markdown. */
  confluenceBlogGet(args: ConfluenceBlogGetArgs): Promise<TaskResult<"confluence_blog_get">> {
    return this.task("confluence_blog_get", { ...args })
  }

  /** Publish a Confluence blog post from Markdown. */
  confluenceBlogCreate(args: ConfluenceBlogCreateArgs): Promise<TaskResult<"confluence_blog_create">> {
    return this.task("confluence_blog_create", { ...args })
  }

  /** Create a Confluence space with a key, a name and an optional plain-text description. */
  confluenceSpaceCreate(args: ConfluenceSpaceCreateArgs): Promise<TaskResult<"confluence_space_create">> {
    return this.task("confluence_space_create", { ...args })
  }

  /** Replace a Confluence page's title and/or body; the version is bumped automatically. */
  confluencePageUpdate(args: ConfluencePageUpdateArgs): Promise<TaskResult<"confluence_page_update">> {
    return this.task("confluence_page_update", { ...args })
  }

  /** Delete a Confluence page. */
  confluencePageDelete(args: ConfluencePageDeleteArgs): Promise<TaskResult<"confluence_page_delete">> {
    return this.task("confluence_page_delete", { ...args })
  }

  /** Children of a Confluence page (Cloud: also folders), or with `recursive` the whole tree, depth first, each row with type, depth and parent. */
  confluencePageChildren(args: ConfluencePageChildrenArgs): Promise<TaskResult<"confluence_page_children">> {
    return this.task("confluence_page_children", { ...args })
  }

  /** Confluence pages whose title contains the words, optionally in one space. */
  confluencePageFind(args: ConfluencePageFindArgs): Promise<TaskResult<"confluence_page_find">> {
    return this.task("confluence_page_find", { ...args })
  }

  /** Upload local files to a Confluence page. */
  confluencePageAttach(args: ConfluencePageAttachArgs): Promise<TaskResult<"confluence_page_attach">> {
    return this.task("confluence_page_attach", { ...args })
  }

  /** Attachments of a Confluence page with download URLs. */
  confluencePageAttachments(args: ConfluencePageAttachmentsArgs): Promise<TaskResult<"confluence_page_attachments">> {
    return this.task("confluence_page_attachments", { ...args })
  }

  /** Footer (or with `inline`, inline) comments of a Confluence page, oldest first, as Markdown, with the replied-to `parent`. */
  confluencePageComments(args: ConfluencePageCommentsArgs): Promise<TaskResult<"confluence_page_comments">> {
    return this.task("confluence_page_comments", { ...args })
  }

  /** Add a footer comment (Markdown) to a Confluence page, or a reply with `parent`. */
  confluencePageComment(args: ConfluencePageCommentArgs): Promise<TaskResult<"confluence_page_comment">> {
    return this.task("confluence_page_comment", { ...args })
  }

  /** Delete a Confluence comment by ID. */
  confluenceCommentDelete(args: ConfluenceCommentDeleteArgs): Promise<TaskResult<"confluence_comment_delete">> {
    return this.task("confluence_comment_delete", { ...args })
  }

  /** Delete a Confluence attachment by ID. */
  confluenceAttachmentDelete(args: ConfluenceAttachmentDeleteArgs): Promise<TaskResult<"confluence_attachment_delete">> {
    return this.task("confluence_attachment_delete", { ...args })
  }

  /** Export a Confluence page (and with `recursive` its tree) to local Markdown folders: <title>/index.md with front matter and _attachments/. Changes nothing on the server. */
  confluencePageExport(args: ConfluencePageExportArgs): Promise<TaskResult<"confluence_page_export">> {
    return this.task("confluence_page_export", { ...args })
  }

  /** Publish local Markdown files: update the page in each file's front matter (refusing pages changed on the server unless `force`, skipping unchanged ones), create pages for new files under their folder's index.md, upload referenced images, and write ids and versions back. */
  confluencePagePush(args: ConfluencePagePushArgs): Promise<TaskResult<"confluence_page_push">> {
    return this.task("confluence_page_push", { ...args })
  }

  /** Copy a Confluence page and the pages below it under another page (titles, bodies, labels). */
  confluencePageCopyTree(args: ConfluencePageCopyTreeArgs): Promise<TaskResult<"confluence_page_copy_tree">> {
    return this.task("confluence_page_copy_tree", { ...args })
  }

  /** Labels of a Confluence page. */
  confluencePageLabels(args: ConfluencePageLabelsArgs): Promise<TaskResult<"confluence_page_labels">> {
    return this.task("confluence_page_labels", { ...args })
  }

  /** Add and remove labels on a Confluence page; returns the resulting labels. */
  confluencePageLabel(args: ConfluencePageLabelArgs): Promise<TaskResult<"confluence_page_label">> {
    return this.task("confluence_page_label", { ...args })
  }

  /** Move a Confluence page under another page (position append) or next to it (before/after, Cloud only). */
  confluencePageMove(args: ConfluencePageMoveArgs): Promise<TaskResult<"confluence_page_move">> {
    return this.task("confluence_page_move", { ...args })
  }

  /** Versions of a Confluence page, newest first: number, author, date, message. Read one with confluence_page_get and `version`. */
  confluencePageHistory(args: ConfluencePageHistoryArgs): Promise<TaskResult<"confluence_page_history">> {
    return this.task("confluence_page_history", { ...args })
  }

  /** Confluence spaces. */
  confluenceSpaceList(args: ConfluenceSpaceListArgs = {}): Promise<TaskResult<"confluence_space_list">> {
    return this.task("confluence_space_list", { ...args })
  }

  /** Top-level pages of a Confluence space. */
  confluenceSpacePages(args: ConfluenceSpacePagesArgs): Promise<TaskResult<"confluence_space_pages">> {
    return this.task("confluence_space_pages", { ...args })
  }

  /** Search Confluence by words or CQL; returns id, type, title, space, excerpt, url. */
  confluenceSearch(args: ConfluenceSearchArgs = {}): Promise<TaskResult<"confluence_search">> {
    return this.task("confluence_search", { ...args })
  }

  /** Bitbucket repositories of a workspace (Cloud) or project (Data Center), or the ones you belong to. */
  bitbucketRepoList(args: BitbucketRepoListArgs = {}): Promise<TaskResult<"bitbucket_repo_list">> {
    return this.task("bitbucket_repo_list", { ...args })
  }

  /** One Bitbucket repository with its default branch and URL. */
  bitbucketRepoView(args: BitbucketRepoViewArgs): Promise<TaskResult<"bitbucket_repo_view">> {
    return this.task("bitbucket_repo_view", { ...args })
  }

  /** Branches of a Bitbucket repository, most recently changed first. */
  bitbucketBranchList(args: BitbucketBranchListArgs): Promise<TaskResult<"bitbucket_branch_list">> {
    return this.task("bitbucket_branch_list", { ...args })
  }

  /** Pull requests of a Bitbucket repository by state. */
  bitbucketPrList(args: BitbucketPrListArgs): Promise<TaskResult<"bitbucket_pr_list">> {
    return this.task("bitbucket_pr_list", { ...args })
  }

  /** One pull request with description, reviewers and approvals. */
  bitbucketPrView(args: BitbucketPrViewArgs): Promise<TaskResult<"bitbucket_pr_view">> {
    return this.task("bitbucket_pr_view", { ...args })
  }

  /** Open a pull request from a source branch. */
  bitbucketPrCreate(args: BitbucketPrCreateArgs): Promise<TaskResult<"bitbucket_pr_create">> {
    return this.task("bitbucket_pr_create", { ...args })
  }

  /** Merge a pull request. */
  bitbucketPrMerge(args: BitbucketPrMergeArgs): Promise<TaskResult<"bitbucket_pr_merge">> {
    return this.task("bitbucket_pr_merge", { ...args })
  }

  /** Approve a pull request as the signed-in user. */
  bitbucketPrApprove(args: BitbucketPrApproveArgs): Promise<TaskResult<"bitbucket_pr_approve">> {
    return this.task("bitbucket_pr_approve", { ...args })
  }

  /** Add a Markdown comment to a pull request. */
  bitbucketPrComment(args: BitbucketPrCommentArgs): Promise<TaskResult<"bitbucket_pr_comment">> {
    return this.task("bitbucket_pr_comment", { ...args })
  }

  /** The unified diff of a pull request. */
  bitbucketPrDiff(args: BitbucketPrDiffArgs): Promise<TaskResult<"bitbucket_pr_diff">> {
    return this.task("bitbucket_pr_diff", { ...args })
  }

  /** Comments of a pull request, oldest first, as Markdown: general and inline (path, line), replies flattened with `parent`. */
  bitbucketPrComments(args: BitbucketPrCommentsArgs): Promise<TaskResult<"bitbucket_pr_comments">> {
    return this.task("bitbucket_pr_comments", { ...args })
  }

  /** Build statuses of a pull request's latest commit, with `passed` (none failed) and `pending` (some still running). */
  bitbucketPrChecks(args: BitbucketPrChecksArgs): Promise<TaskResult<"bitbucket_pr_checks">> {
    return this.task("bitbucket_pr_checks", { ...args })
  }

  /** Remove your approval from a pull request. */
  bitbucketPrUnapprove(args: BitbucketPrUnapproveArgs): Promise<TaskResult<"bitbucket_pr_unapprove">> {
    return this.task("bitbucket_pr_unapprove", { ...args })
  }

  /** Mark a draft pull request as ready for review (or, with `draft: true`, as a draft again). */
  bitbucketPrReady(args: BitbucketPrReadyArgs): Promise<TaskResult<"bitbucket_pr_ready">> {
    return this.task("bitbucket_pr_ready", { ...args })
  }

  /** Create a branch from another branch, a tag or a commit (default: the main branch). */
  bitbucketBranchCreate(args: BitbucketBranchCreateArgs): Promise<TaskResult<"bitbucket_branch_create">> {
    return this.task("bitbucket_branch_create", { ...args })
  }

  /** Delete a branch. Unmerged commits on it can be lost. */
  bitbucketBranchDelete(args: BitbucketBranchDeleteArgs): Promise<TaskResult<"bitbucket_branch_delete">> {
    return this.task("bitbucket_branch_delete", { ...args })
  }

  /** Recent commits of a branch, newest first: hash, author, date, first line of the message. */
  bitbucketCommitList(args: BitbucketCommitListArgs): Promise<TaskResult<"bitbucket_commit_list">> {
    return this.task("bitbucket_commit_list", { ...args })
  }

  /** Latest Bitbucket Pipelines runs, newest first (Cloud only). */
  bitbucketPipelineList(args: BitbucketPipelineListArgs): Promise<TaskResult<"bitbucket_pipeline_list">> {
    return this.task("bitbucket_pipeline_list", { ...args })
  }

  /** Start the pipeline of a branch, or a custom pipeline (Cloud only). */
  bitbucketPipelineRun(args: BitbucketPipelineRunArgs): Promise<TaskResult<"bitbucket_pipeline_run">> {
    return this.task("bitbucket_pipeline_run", { ...args })
  }

  /** Decline a Bitbucket pull request (Cloud cannot reopen it). */
  bitbucketPrDecline(args: BitbucketPrDeclineArgs): Promise<TaskResult<"bitbucket_pr_decline">> {
    return this.task("bitbucket_pr_decline", { ...args })
  }

  /** Mark a Bitbucket pull request as needing changes by you (Cloud: request changes; Data Center: needs work). */
  bitbucketPrRequestChanges(args: BitbucketPrRequestChangesArgs): Promise<TaskResult<"bitbucket_pr_request_changes">> {
    return this.task("bitbucket_pr_request_changes", { ...args })
  }

  /** Change a Bitbucket pull request's title, description (Markdown) or destination branch. */
  bitbucketPrEdit(args: BitbucketPrEditArgs): Promise<TaskResult<"bitbucket_pr_edit">> {
    return this.task("bitbucket_pr_edit", { ...args })
  }

  /** Add and remove reviewers of a Bitbucket pull request; other reviewers are kept. */
  bitbucketPrReviewers(args: BitbucketPrReviewersArgs): Promise<TaskResult<"bitbucket_pr_reviewers">> {
    return this.task("bitbucket_pr_reviewers", { ...args })
  }

  /** Stop a running Bitbucket pipeline by build number or UUID (Cloud only). */
  bitbucketPipelineStop(args: BitbucketPipelineStopArgs): Promise<TaskResult<"bitbucket_pipeline_stop">> {
    return this.task("bitbucket_pipeline_stop", { ...args })
  }

  /** One Bitbucket pipeline by build number or UUID, with each step's state and result (Cloud only). */
  bitbucketPipelineView(args: BitbucketPipelineViewArgs): Promise<TaskResult<"bitbucket_pipeline_view">> {
    return this.task("bitbucket_pipeline_view", { ...args })
  }

  /** Log text of one pipeline step (by name or UUID) or of every step (Cloud only). */
  bitbucketPipelineLogs(args: BitbucketPipelineLogsArgs): Promise<TaskResult<"bitbucket_pipeline_logs">> {
    return this.task("bitbucket_pipeline_logs", { ...args })
  }

  /** Jira Service Management service desks (id, project key, name). */
  jsmDeskList(args: JsmDeskListArgs = {}): Promise<TaskResult<"jsm_desk_list">> {
    return this.task("jsm_desk_list", { ...args })
  }

  /** Request types of a service desk. */
  jsmRequestTypes(args: JsmRequestTypesArgs): Promise<TaskResult<"jsm_request_types">> {
    return this.task("jsm_request_types", { ...args })
  }

  /** Queues of a service desk with issue counts. */
  jsmQueueList(args: JsmQueueListArgs): Promise<TaskResult<"jsm_queue_list">> {
    return this.task("jsm_queue_list", { ...args })
  }

  /** Issues in a service desk queue (by queue ID or name). */
  jsmQueueIssues(args: JsmQueueIssuesArgs): Promise<TaskResult<"jsm_queue_issues">> {
    return this.task("jsm_queue_issues", { ...args })
  }

  /** Customer requests by status, optionally of one service desk. */
  jsmRequestList(args: JsmRequestListArgs = {}): Promise<TaskResult<"jsm_request_list">> {
    return this.task("jsm_request_list", { ...args })
  }

  /** One customer request with its description and latest comments. */
  jsmRequestView(args: JsmRequestViewArgs): Promise<TaskResult<"jsm_request_view">> {
    return this.task("jsm_request_view", { ...args })
  }

  /** Raise a customer request, optionally for a customer. */
  jsmRequestCreate(args: JsmRequestCreateArgs): Promise<TaskResult<"jsm_request_create">> {
    return this.task("jsm_request_create", { ...args })
  }

  /** Reply to a customer request, or add an internal note. */
  jsmRequestComment(args: JsmRequestCommentArgs): Promise<TaskResult<"jsm_request_comment">> {
    return this.task("jsm_request_comment", { ...args })
  }

  /** Transitions available on a customer request. */
  jsmRequestTransitions(args: JsmRequestTransitionsArgs): Promise<TaskResult<"jsm_request_transitions">> {
    return this.task("jsm_request_transitions", { ...args })
  }

  /** Run a customer request transition by name or ID. */
  jsmRequestMove(args: JsmRequestMoveArgs): Promise<TaskResult<"jsm_request_move">> {
    return this.task("jsm_request_move", { ...args })
  }

  /** Approvals of a JSM request: name, decision (pending, approved, declined), approvers, and whether you can answer. */
  jsmRequestApprovals(args: JsmRequestApprovalsArgs): Promise<TaskResult<"jsm_request_approvals">> {
    return this.task("jsm_request_approvals", { ...args })
  }

  /** Approve or decline (decision: decline) a JSM request's approval; without an ID, the one pending approval you can answer. */
  jsmRequestApprove(args: JsmRequestApproveArgs): Promise<TaskResult<"jsm_request_approve">> {
    return this.task("jsm_request_approve", { ...args })
  }

  /** List a JSM request's participants, after adding and removing any (me, an email or a name). */
  jsmRequestParticipants(args: JsmRequestParticipantsArgs): Promise<TaskResult<"jsm_request_participants">> {
    return this.task("jsm_request_participants", { ...args })
  }

  /** SLAs of a JSM request: state (ongoing, paused, met, breached), time remaining and goal. */
  jsmRequestSla(args: JsmRequestSlaArgs): Promise<TaskResult<"jsm_request_sla">> {
    return this.task("jsm_request_sla", { ...args })
  }

  /** Attach local files to a JSM request (public unless internal). */
  jsmRequestAttach(args: JsmRequestAttachArgs): Promise<TaskResult<"jsm_request_attach">> {
    return this.task("jsm_request_attach", { ...args })
  }

  /** Attachments of a JSM request with download URLs. */
  jsmRequestAttachments(args: JsmRequestAttachmentsArgs): Promise<TaskResult<"jsm_request_attachments">> {
    return this.task("jsm_request_attachments", { ...args })
  }

  /** Your open Trello boards. */
  trelloBoardList(args: TrelloBoardListArgs = {}): Promise<TaskResult<"trello_board_list">> {
    return this.task("trello_board_list", { ...args })
  }

  /** Open lists of a Trello board (board by name, URL, short link or ID). */
  trelloListList(args: TrelloListListArgs): Promise<TaskResult<"trello_list_list">> {
    return this.task("trello_list_list", { ...args })
  }

  /** Cards of a Trello board or one of its lists, or your cards on every board (`mine`). */
  trelloCardList(args: TrelloCardListArgs = {}): Promise<TaskResult<"trello_card_list">> {
    return this.task("trello_card_list", { ...args })
  }

  /** One Trello card with list, board, description and latest comments. */
  trelloCardView(args: TrelloCardViewArgs): Promise<TaskResult<"trello_card_view">> {
    return this.task("trello_card_view", { ...args })
  }

  /** Create a Trello card in a list (by name with `board`, or by list ID). */
  trelloCardCreate(args: TrelloCardCreateArgs): Promise<TaskResult<"trello_card_create">> {
    return this.task("trello_card_create", { ...args })
  }

  /** Move a Trello card to another list (by default on the card's own board). */
  trelloCardMove(args: TrelloCardMoveArgs): Promise<TaskResult<"trello_card_move">> {
    return this.task("trello_card_move", { ...args })
  }

  /** Add a comment to a Trello card. */
  trelloCardComment(args: TrelloCardCommentArgs): Promise<TaskResult<"trello_card_comment">> {
    return this.task("trello_card_comment", { ...args })
  }

  /** Archive (close) a Trello card. */
  trelloCardArchive(args: TrelloCardArchiveArgs): Promise<TaskResult<"trello_card_archive">> {
    return this.task("trello_card_archive", { ...args })
  }

  /** Change a Trello card: name, description, due date, done, labels (by name or color) and members (username, name or me). */
  trelloCardEdit(args: TrelloCardEditArgs): Promise<TaskResult<"trello_card_edit">> {
    return this.task("trello_card_edit", { ...args })
  }

  /** Checklists of a Trello card with their items and whether each is done. */
  trelloChecklistList(args: TrelloChecklistListArgs): Promise<TaskResult<"trello_checklist_list">> {
    return this.task("trello_checklist_list", { ...args })
  }

  /** Add a checklist with items to a Trello card. */
  trelloChecklistCreate(args: TrelloChecklistCreateArgs): Promise<TaskResult<"trello_checklist_create">> {
    return this.task("trello_checklist_create", { ...args })
  }

  /** Tick (or with done: false, untick) a Trello checklist item by name or ID. */
  trelloChecklistCheck(args: TrelloChecklistCheckArgs): Promise<TaskResult<"trello_checklist_check">> {
    return this.task("trello_checklist_check", { ...args })
  }

  /** Organizations the Atlassian Admin API key can see. */
  adminOrgList(args: AdminOrgListArgs = {}): Promise<TaskResult<"admin_org_list">> {
    return this.task("admin_org_list", { ...args })
  }

  /** User directories of an Atlassian organization. */
  adminDirectoryList(args: AdminDirectoryListArgs = {}): Promise<TaskResult<"admin_directory_list">> {
    return this.task("admin_directory_list", { ...args })
  }

  /** Managed users of an Atlassian organization (every directory), optionally matching `search`. */
  adminUserList(args: AdminUserListArgs = {}): Promise<TaskResult<"admin_user_list">> {
    return this.task("admin_user_list", { ...args })
  }

  /** Suspend a managed user's access to every product (by account ID or email). */
  adminUserSuspend(args: AdminUserSuspendArgs): Promise<TaskResult<"admin_user_suspend">> {
    return this.task("admin_user_suspend", { ...args })
  }

  /** Restore a suspended managed user's access (by account ID or email). */
  adminUserRestore(args: AdminUserRestoreArgs): Promise<TaskResult<"admin_user_restore">> {
    return this.task("admin_user_restore", { ...args })
  }

  /** Add a managed user (email or account ID) to a group (name or ID) of an Atlassian organization. */
  adminGroupAddMember(args: AdminGroupAddMemberArgs): Promise<TaskResult<"admin_group_add_member">> {
    return this.task("admin_group_add_member", { ...args })
  }

  /** Remove a managed user from a group of an Atlassian organization. */
  adminGroupRemoveMember(args: AdminGroupRemoveMemberArgs): Promise<TaskResult<"admin_group_remove_member">> {
    return this.task("admin_group_remove_member", { ...args })
  }

  /** Groups of an Atlassian organization (every directory), optionally matching `search`. */
  adminGroupList(args: AdminGroupListArgs = {}): Promise<TaskResult<"admin_group_list">> {
    return this.task("admin_group_list", { ...args })
  }

  /** Domains of an Atlassian organization with their verification status. */
  adminDomainList(args: AdminDomainListArgs = {}): Promise<TaskResult<"admin_domain_list">> {
    return this.task("admin_domain_list", { ...args })
  }

  /** Audit log events of an Atlassian organization, newest first, filtered by words, dates and action. */
  adminEventList(args: AdminEventListArgs = {}): Promise<TaskResult<"admin_event_list">> {
    return this.task("admin_event_list", { ...args })
  }
}
