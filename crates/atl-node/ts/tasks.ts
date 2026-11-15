// Generated from the task registry (crates/atl-sdk/src/tasks/) by
// crates/atl-sdk/tests/sdk_types.rs. Do not edit; run
// `ATL_UPDATE_SDK_TYPES=1 cargo test -p atlassian-toolkit --test sdk_types`.

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

export interface AdminUser {
  account_id: string
  name: string | null
  email: string | null
  /** Account status, for example active or suspended. */
  status: string | null
  membership: string | null
  directory: string
}

export interface Approval {
  id: string
  name: string
  /** pending, approved or declined. */
  decision: string
  /** True when you are an approver who has not answered yet. */
  can_answer: boolean
  approvers: string[]
}

export interface Assigned {
  key: string
  assignee: string
}

export interface Attachment {
  id: string
  filename: string
  size: number
  mime_type: string | null
  created: string | null
  author: string | null
  /** Download URL on the Jira server. */
  content: string
}

export interface Board {
  id: number
  name: string
  type: string | null
}

export interface BoardSprints {
  board: number
  sprints: Sprint[]
}

export interface Branch {
  name: string
  commit: string | null
}

export interface BranchDeleted {
  repo: string
  name: string
  deleted: boolean
}

export interface BuildStatus {
  name: string
  key: string | null
  /** SUCCESSFUL, FAILED, INPROGRESS, STOPPED (Cloud), CANCELLED or UNKNOWN (Data Center). */
  state: string
  url: string | null
  description: string | null
}

export interface Card {
  id: string
  name: string
  /** The list's name when known, else its ID. */
  list: string | null
  due: string | null
  labels: string[]
  closed: boolean
  short_link: string | null
  url: string | null
}

export interface CardComment {
  author: string | null
  date: string | null
  text: string
}

export interface CardDetail {
  id: string
  name: string
  /** The list's name when known, else its ID. */
  list: string | null
  due: string | null
  labels: string[]
  closed: boolean
  short_link: string | null
  url: string | null
  board: string | null
  /** The card description (Trello stores Markdown). */
  description: string
  /** Latest comments first. */
  comments: CardComment[]
}

/** One changed field of one history entry. */
export interface Change {
  created: string | null
  author: string | null
  field: string
  from: string | null
  to: string | null
}

export interface ChangesRequested {
  repo: string
  id: number
  changes_requested: boolean
}

export interface CheckItem {
  id: string
  name: string
  done: boolean
}

export interface Checklist {
  id: string
  name: string
  items: CheckItem[]
}

export interface Comment {
  id: string
  author: string | null
  created: string | null
  /** The comment as Markdown (Cloud) or wiki markup (Data Center). */
  body: string
}

export interface CommentDeleted {
  key: string
  id: string
  deleted: boolean
}

export interface Commit {
  hash: string
  author: string | null
  date: string | null
  /** The first line of the message. */
  message: string
}

export interface Component {
  id: string
  name: string
  lead: string | null
  description: string | null
}

export interface Copied {
  source: string
  /** The new page (empty in a dry run). */
  id: string
  title: string
  parent: string
  url: string
}

export interface Created {
  key: string
  url: string
}

export interface Deleted {
  id: string
  deleted: boolean
}

export interface Desk {
  id: string
  project: string
  name: string
}

export interface Directory {
  id: string
  name: string
}

export interface Domain {
  id: string
  name: string
  /** Verification status of the domain claim. */
  status: string | null
}

export interface EpicIssues {
  /** The epic, or none after a remove. */
  epic: string | null
  issues: string[]
}

export interface Event {
  id: string
  time: string | null
  action: string | null
  actor: string | null
  message: string | null
}

export interface Exported {
  id: string
  title: string
  type: string
  /** The written `index.md` (or the folder, for a Cloud folder). */
  path: string
  attachments: number
}

/** A system or custom field, from `GET /rest/api/{2|3}/field`. */
export interface Field {
  /** For example `customfield_10016` or `duedate`. */
  id: string
  name: string
  custom: boolean
  /** JSON type: string, number, date, datetime, option, array, user, ... */
  type: string | null
  /** Item type of an array field. */
  items: string | null
}

export interface Filter {
  id: string
  name: string
  jql: string
  owner: string | null
  favourite: boolean
  url: string
}

export interface Group {
  id: string
  name: string
  description: string | null
  users: number | null
  directory: string
}

export interface IssueCount {
  count: number
  jql: string
}

export interface IssueDeleted {
  key: string
  deleted: boolean
}

export interface IssueDetail {
  key: string
  summary: string
  status: string
  /** `new`, `indeterminate` (in progress), or `done`. */
  status_category: string
  type: string
  assignee: string | null
  priority: string | null
  updated: string | null
  url: string
  reporter: string | null
  created: string | null
  labels: string[]
  components: string[]
  parent: string | null
  subtasks: string[]
  /** The description as Markdown (Cloud) or wiki markup (Data Center). */
  description: string
  comments: Comment[]
  comment_count: number
  /** Links to other issues, as "relation key" (for example "blocks ABC-2"). */
  links: IssueLink[]
  /**
   * Fields asked for by name (`issue view --field`), simplified: options
   * become their value, users their name, rich text Markdown.
   */
  fields?: Record<string, unknown>
}

export interface IssueLink {
  /** The phrase from this issue's side, for example "blocks" or "is blocked by". */
  relation: string
  key: string
  summary: string | null
  status: string | null
}

/** A page of issues and the JQL that found them. */
export interface IssueList {
  issues: IssueSummary[]
  /** Pass back as `page_token` for the next page. */
  next_page_token: string | null
  /** Total matches when the server reports it (Data Center). */
  total: number | null
  jql: string
}

export interface IssueSummary {
  key: string
  summary: string
  status: string
  /** `new`, `indeterminate` (in progress), or `done`. */
  status_category: string
  type: string
  assignee: string | null
  priority: string | null
  updated: string | null
  url: string
}

export interface IssueUpdated {
  key: string
  updated: boolean
  url: string
}

export interface JqlCheck {
  valid: boolean
  errors: string[]
  warnings: string[]
}

export interface JqlField {
  /** The name to use in JQL. */
  name: string
  display: string
  operators: string[]
}

export interface JqlReference {
  fields: JqlField[]
  functions: string[]
}

export interface LinkType {
  name: string
  /** Phrase from the first issue to the second ("blocks"). */
  outward: string
  /** Phrase from the second issue to the first ("is blocked by"). */
  inward: string
}

export interface Linked {
  from: string
  to: string
  /** The link type name. */
  type: string
}

/** A user's membership of a group after an add or a remove. */
export interface Membership {
  group: string
  group_id: string
  /** The user's email, or their name when the email is hidden. */
  user: string | null
  account_id: string
  /** True after an add, false after a remove. */
  member: boolean
}

export interface Moved {
  key: string
  /** The status the issue is in now. */
  status: string
  /** The transition that moved it. */
  transition: string
}

export interface Org {
  id: string
  name: string
}

export interface Page {
  id: string
  title: string
  space: string | null
  /** ID of the parent page (none for a page at the top of a space). */
  parent: string | null
  version: number
  url: string
  /** The body as Markdown. */
  markdown: string
}

export interface PageAttachment {
  id: string
  title: string
  size: number | null
  media_type: string | null
  /** Download URL on the Confluence server. */
  download: string
}

export interface PageComment {
  id: string
  /** Display name (account ID on Cloud when the name lookup fails). */
  author: string | null
  created: string | null
  /** The comment as Markdown. */
  body: string
  /** The comment this one replies to. */
  parent?: string | null
  /** For inline comments, the page text they are attached to. */
  selection?: string | null
}

export interface PageSummary {
  id: string
  title: string
  url: string
}

export interface PageVersion {
  number: number
  author: string | null
  created: string | null
  message: string | null
  minor_edit: boolean
}

/**
 * A page, with its storage-format body and its Markdown document (front
 * matter and body) when asked for.
 */
export interface PageView {
  id: string
  title: string
  space: string | null
  /** ID of the parent page (none for a page at the top of a space). */
  parent: string | null
  version: number
  url: string
  /** The body as Markdown. */
  markdown: string
  /** The page as a Markdown file with front matter (with `front_matter: true`). */
  document?: string | null
  /** The body in storage format (with `storage: true`). */
  storage?: string | null
}

export interface Participants {
  key: string
  /** Every participant after the change. */
  participants: string[]
}

export interface Pipeline {
  build_number: number
  uuid: string | null
  /** PENDING, IN_PROGRESS or COMPLETED. */
  state: string
  /** SUCCESSFUL, FAILED, STOPPED or ERROR once completed. */
  result: string | null
  branch: string | null
  created: string | null
  url: string
}

export interface PipelineDetail {
  build_number: number
  uuid: string | null
  /** PENDING, IN_PROGRESS or COMPLETED. */
  state: string
  /** SUCCESSFUL, FAILED, STOPPED or ERROR once completed. */
  result: string | null
  branch: string | null
  created: string | null
  url: string
  steps: PipelineStep[]
}

export interface PipelineLog {
  repo: string
  id: string
  /** The step logs as text. */
  log: string
}

export interface PipelineStep {
  uuid: string
  name: string
  /** PENDING, IN_PROGRESS or COMPLETED. */
  state: string
  result: string | null
  seconds: number | null
}

export interface PrApproval {
  repo: string
  id: number
  /** True after an approve, false after an unapprove. */
  approved: boolean
}

export interface PrChecks {
  repo: string
  id: number
  /** No check failed. */
  passed: boolean
  /** At least one check is still running. */
  pending: boolean
  checks: BuildStatus[]
}

export interface PrComment {
  id: string
  author: string | null
  created: string | null
  /** Markdown. */
  body: string
  /** File of an inline comment. */
  path: string | null
  /** Line of an inline comment (in the new version of the file when known). */
  line: number | null
  /** ID of the comment this one answers. */
  parent: string | null
  resolved: boolean
}

export interface PrCommented {
  repo: string
  id: number
  comment_id: string
}

export interface PrDiff {
  repo: string
  id: number
  /** The unified diff. */
  diff: string
}

export interface Project {
  key: string
  name: string
  id: string
  type: string | null
}

export interface PullRequest {
  id: number
  title: string
  /** OPEN, MERGED, DECLINED or SUPERSEDED. */
  state: string
  author: string | null
  source: string
  destination: string
  updated: string | null
  description: string
  reviewers: string[]
  approved_by: string[]
  url: string
  /**
   * `owner/slug` of the repository the source branch is in (differs
   * from the target repository for pull requests from forks).
   */
  source_repo: string | null
  /** True for a draft (not yet ready for review). */
  draft: boolean
}

export interface Pushed {
  file: string
  id: string
  title: string
  version: number
  /** created, updated, unchanged, or planned (dry run). */
  action: string
}

export interface Queue {
  id: string
  name: string
  jql: string | null
  issues: number | null
}

/** A project version ("release"). */
export interface Release {
  id: string
  name: string
  released: boolean
  archived: boolean
  release_date: string | null
  description: string | null
}

/** A web link that `Jira::remote_link` added to an issue. */
export interface RemoteLink {
  key: string
  /** Jira's ID of the link. */
  id: number | null
  url: string
  title: string
}

export interface Repo {
  /** `owner/slug`. */
  repo: string
  name: string
  description: string | null
  default_branch: string | null
  private: boolean | null
  url: string
}

export interface Request {
  key: string
  summary: string
  status: string
  type: string | null
  reporter: string | null
  created: string | null
  url: string
}

export interface RequestAttachment {
  filename: string
  author: string | null
  created: string | null
  size: number | null
  mime_type: string | null
  url: string | null
}

export interface RequestComment {
  id: string
  author: string | null
  created: string | null
  /** False for internal notes that customers cannot see. */
  public: boolean
  body: string
}

export interface RequestDetail {
  key: string
  summary: string
  status: string
  type: string | null
  reporter: string | null
  created: string | null
  url: string
  description: string
  comments: RequestComment[]
}

export interface RequestMoved {
  key: string
  /** The transition that ran. */
  transition: string
  /** Its ID. */
  id: string
}

export interface RequestTransition {
  id: string
  name: string
}

export interface RequestType {
  id: string
  name: string
  description: string | null
}

export interface SearchHit {
  id: string
  type: string
  title: string
  space: string | null
  excerpt: string | null
  url: string
}

export interface SearchResults {
  /** The CQL that ran. */
  cql: string
  results: SearchHit[]
}

export interface Sla {
  name: string
  /** ongoing, paused, met or breached. */
  state: string
  breached: boolean
  /** Time left (negative when breached), as Jira shows it. */
  remaining: string | null
  goal: string | null
}

export interface Space {
  key: string
  name: string
  id: string | null
  type: string | null
  url: string
}

export interface Sprint {
  id: number
  name: string
  state: string
  start: string | null
  end: string | null
  goal: string | null
}

export interface SprintIssues {
  sprint: number
  issues: string[]
}

export interface Suggestion {
  value: string
  display: string
}

export interface Transition {
  id: string
  name: string
  /** The status the transition leads to. */
  to: string
}

/** One node of a page tree (see [`Confluence::tree`]). */
export interface TreeNode {
  id: string
  title: string
  /** page, folder, whiteboard, database or embed (Data Center: page). */
  type: string
  /** 1 for direct children of the root. */
  depth: number
  /** ID of the parent node. */
  parent: string
  url: string
}

export interface TrelloBoard {
  id: string
  name: string
  short_link: string | null
  url: string | null
}

export interface TrelloList {
  id: string
  name: string
}

export interface Unlinked {
  from: string
  to: string
  /** The link types that were removed. */
  removed: string[]
}

export interface User {
  /** Account ID (Cloud) or username (Data Center). */
  id: string
  name: string
  email: string | null
}

export interface Watchers {
  /** True when the signed-in user watches the issue. */
  watching: boolean
  count: number
  watchers: User[]
}

export interface Watching {
  key: string
  user: string
  /** True after a watch, false after an unwatch. */
  watching: boolean
}

export interface Worklog {
  id: string
  author: string | null
  /** As Jira shows it, for example "2h 30m". */
  time_spent: string | null
  seconds: number | null
  started: string | null
  /** Markdown. */
  comment: string | null
}

/** A worklog that `Jira::worklog` added. */
export interface WorklogAdded {
  key: string
  worklog_id: string | null
  time_spent: string
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

/** The result of every task, by task name. */
export interface TaskResults {
  jira_issue_list: IssueList
  jira_issue_count: IssueCount
  jira_issue_view: IssueDetail
  jira_issue_delete: IssueDeleted
  jira_issue_watchers: Watchers
  jira_issue_watch: Watching
  jira_issue_unwatch: Watching
  jira_field_list: Field[]
  jira_issue_create: Created
  jira_issue_edit: IssueUpdated
  jira_issue_assign: Assigned
  jira_issue_transitions: Transition[]
  jira_issue_move: Moved
  jira_issue_comment: Comment
  jira_issue_link: Linked
  jira_issue_unlink: Unlinked
  jira_issue_remote_link: RemoteLink
  jira_comment_list: Comment[]
  jira_comment_edit: Comment
  jira_comment_delete: CommentDeleted
  jira_worklog_list: Worklog[]
  jira_issue_history: Change[]
  jira_component_list: Component[]
  jira_sprint_create: Sprint
  jira_sprint_start: Sprint
  jira_sprint_close: Sprint
  jira_release_list: Release[]
  jira_issue_link_types: LinkType[]
  jira_issue_clone: Created
  jira_issue_worklog: WorklogAdded
  jira_issue_attach: Attachment[]
  jira_issue_attachments: Attachment[]
  jira_jql_check: JqlCheck
  jira_jql_fields: JqlReference
  jira_jql_suggest: Suggestion[]
  jira_filter_list: Filter[]
  jira_filter_run: IssueList
  jira_epic_list: IssueSummary[]
  jira_epic_add: EpicIssues
  jira_epic_remove: EpicIssues
  jira_board_list: Board[]
  jira_sprint_list: BoardSprints
  jira_sprint_add: SprintIssues
  jira_project_list: Project[]
  jira_issue_types: string[]
  jira_me: User
  confluence_page_get: PageView
  confluence_page_create: PageSummary
  confluence_blog_list: PageSummary[]
  confluence_blog_get: Page
  confluence_blog_create: PageSummary
  confluence_space_create: Space
  confluence_page_update: PageSummary
  confluence_page_delete: Deleted
  confluence_page_children: TreeNode[]
  confluence_page_find: SearchHit[]
  confluence_page_attach: PageAttachment[]
  confluence_page_attachments: PageAttachment[]
  confluence_page_comments: PageComment[]
  confluence_page_comment: PageComment
  confluence_comment_delete: Deleted
  confluence_attachment_delete: Deleted
  confluence_page_export: Exported[]
  confluence_page_push: Pushed[]
  confluence_page_copy_tree: Copied[]
  confluence_page_labels: string[]
  confluence_page_label: string[]
  confluence_page_move: PageSummary
  confluence_page_history: PageVersion[]
  confluence_space_list: Space[]
  confluence_space_pages: PageSummary[]
  confluence_search: SearchResults
  bitbucket_repo_list: Repo[]
  bitbucket_repo_view: Repo
  bitbucket_branch_list: Branch[]
  bitbucket_pr_list: PullRequest[]
  bitbucket_pr_view: PullRequest
  bitbucket_pr_create: PullRequest
  bitbucket_pr_merge: PullRequest
  bitbucket_pr_approve: PrApproval
  bitbucket_pr_comment: PrCommented
  bitbucket_pr_diff: PrDiff
  bitbucket_pr_comments: PrComment[]
  bitbucket_pr_checks: PrChecks
  bitbucket_pr_unapprove: PrApproval
  bitbucket_pr_ready: PullRequest
  bitbucket_branch_create: Branch
  bitbucket_branch_delete: BranchDeleted
  bitbucket_commit_list: Commit[]
  bitbucket_pipeline_list: Pipeline[]
  bitbucket_pipeline_run: Pipeline
  bitbucket_pr_decline: PullRequest
  bitbucket_pr_request_changes: ChangesRequested
  bitbucket_pr_edit: PullRequest
  bitbucket_pr_reviewers: PullRequest
  bitbucket_pipeline_stop: Pipeline
  bitbucket_pipeline_view: PipelineDetail
  bitbucket_pipeline_logs: PipelineLog
  jsm_desk_list: Desk[]
  jsm_request_types: RequestType[]
  jsm_queue_list: Queue[]
  jsm_queue_issues: IssueSummary[]
  jsm_request_list: Request[]
  jsm_request_view: RequestDetail
  jsm_request_create: Request
  jsm_request_comment: RequestComment
  jsm_request_transitions: RequestTransition[]
  jsm_request_move: RequestMoved
  jsm_request_approvals: Approval[]
  jsm_request_approve: Approval
  jsm_request_participants: Participants
  jsm_request_sla: Sla[]
  jsm_request_attach: RequestAttachment[]
  jsm_request_attachments: RequestAttachment[]
  trello_board_list: TrelloBoard[]
  trello_list_list: TrelloList[]
  trello_card_list: Card[]
  trello_card_view: CardDetail
  trello_card_create: Card
  trello_card_move: Card
  trello_card_comment: CardComment
  trello_card_archive: Card
  trello_card_edit: Card
  trello_checklist_list: Checklist[]
  trello_checklist_create: Checklist
  trello_checklist_check: CheckItem
  admin_org_list: Org[]
  admin_directory_list: Directory[]
  admin_user_list: AdminUser[]
  admin_user_suspend: AdminUser
  admin_user_restore: AdminUser
  admin_group_add_member: Membership
  admin_group_remove_member: Membership
  admin_group_list: Group[]
  admin_domain_list: Domain[]
  admin_event_list: Event[]
}

export type TaskName = keyof TaskArgs

/** Every value of `AtlError.code`. */
export type ErrorCode = "auth" | "api" | "invalid" | "unknown" | "read_only" | "dry_run" | "input_required" | "internal"

/** The product namespaces; each SDK supplies `task()` over its own transport. */
export abstract class GeneratedTasks {
  /** Runs any task by name, for example `task("jira_issue_view", { key: "A-1" })`. */
  abstract task<N extends TaskName>(name: N, args?: TaskArgs[N]): Promise<TaskResults[N]>

  /** Jira and Jira Software: issues, comments, sprints, boards, filters and JQL. */
  readonly jira = {
    /** The signed-in Jira user. */
    me: (args: JiraMeArgs = {}): Promise<User> => this.task("jira_me", args),
    /** `atl jira board` */
    board: {
      /** Jira Software boards, optionally of one project. */
      list: (args: JiraBoardListArgs = {}): Promise<Board[]> => this.task("jira_board_list", args),
    },
    /** `atl jira comment` */
    comment: {
      /** Every comment of a Jira issue, oldest first, with IDs, as Markdown. */
      list: (args: JiraCommentListArgs): Promise<Comment[]> => this.task("jira_comment_list", args),
      /** Replace the text of a Jira comment (Markdown). */
      edit: (args: JiraCommentEditArgs): Promise<Comment> => this.task("jira_comment_edit", args),
      /** Delete a Jira comment. Cannot be undone. */
      delete: (args: JiraCommentDeleteArgs): Promise<CommentDeleted> => this.task("jira_comment_delete", args),
    },
    /** `atl jira component` */
    component: {
      /** Components of a Jira project with their leads. */
      list: (args: JiraComponentListArgs): Promise<Component[]> => this.task("jira_component_list", args),
    },
    /** `atl jira epic` */
    epic: {
      /** Open epics, optionally of one project. */
      list: (args: JiraEpicListArgs = {}): Promise<IssueSummary[]> => this.task("jira_epic_list", args),
      /** Move issues into an epic. */
      add: (args: JiraEpicAddArgs): Promise<EpicIssues> => this.task("jira_epic_add", args),
      /** Remove issues from their epic. */
      remove: (args: JiraEpicRemoveArgs): Promise<EpicIssues> => this.task("jira_epic_remove", args),
    },
    /** `atl jira field` */
    field: {
      /** Jira system and custom fields (ID, name, type), optionally filtered by part of the name. Use the names in the `fields` argument of issue create, edit and view. */
      list: (args: JiraFieldListArgs = {}): Promise<Field[]> => this.task("jira_field_list", args),
    },
    /** `atl jira filter` */
    filter: {
      /** Saved Jira filters (Cloud: all visible, Data Center: favourites). */
      list: (args: JiraFilterListArgs = {}): Promise<Filter[]> => this.task("jira_filter_list", args),
      /** Run a saved Jira filter by ID or exact name; returns issue records. */
      run: (args: JiraFilterRunArgs): Promise<IssueList> => this.task("jira_filter_run", args),
    },
    /** `atl jira issue` */
    issue: {
      /** Search Jira issues with filters (or extra JQL). Returns small records: key, summary, status, type, assignee, priority, updated, url. With no filters: your unresolved issues. */
      list: (args: JiraIssueListArgs = {}): Promise<IssueList> => this.task("jira_issue_list", args),
      /** Count Jira issues matching the same filters as issue_list. */
      count: (args: JiraIssueCountArgs = {}): Promise<IssueCount> => this.task("jira_issue_count", args),
      /** One Jira issue with description and latest comments as Markdown, plus any `fields` asked for by name. */
      view: (args: JiraIssueViewArgs): Promise<IssueDetail> => this.task("jira_issue_view", args),
      /** Delete a Jira issue (with `subtasks: true`, also its sub-tasks). Cannot be undone. */
      delete: (args: JiraIssueDeleteArgs): Promise<IssueDeleted> => this.task("jira_issue_delete", args),
      /** Who watches a Jira issue, and whether you do. */
      watchers: (args: JiraIssueWatchersArgs): Promise<Watchers> => this.task("jira_issue_watchers", args),
      /** Add a watcher to a Jira issue (default: you). */
      watch: (args: JiraIssueWatchArgs): Promise<Watching> => this.task("jira_issue_watch", args),
      /** Remove a watcher from a Jira issue (default: you). */
      unwatch: (args: JiraIssueUnwatchArgs): Promise<Watching> => this.task("jira_issue_unwatch", args),
      /** Create a Jira issue; the description is Markdown. */
      create: (args: JiraIssueCreateArgs): Promise<Created> => this.task("jira_issue_create", args),
      /** Change a Jira issue's summary, description (Markdown), priority or labels. */
      edit: (args: JiraIssueEditArgs): Promise<IssueUpdated> => this.task("jira_issue_edit", args),
      /** Assign a Jira issue to me, nobody (x), or a user. */
      assign: (args: JiraIssueAssignArgs): Promise<Assigned> => this.task("jira_issue_assign", args),
      /** Statuses a Jira issue can move to. */
      transitions: (args: JiraIssueTransitionsArgs): Promise<Transition[]> => this.task("jira_issue_transitions", args),
      /** Move a Jira issue to another status (by status or transition name). */
      move: (args: JiraIssueMoveArgs): Promise<Moved> => this.task("jira_issue_move", args),
      /** Add a Markdown comment to a Jira issue. */
      comment: (args: JiraIssueCommentArgs): Promise<Comment> => this.task("jira_issue_comment", args),
      /** Link two Jira issues so that 'from <kind> to' reads true, for example from A-1 kind blocks to A-2. */
      link: (args: JiraIssueLinkArgs): Promise<Linked> => this.task("jira_issue_link", args),
      /** Remove the links between two Jira issues (optionally only one link type). */
      unlink: (args: JiraIssueUnlinkArgs): Promise<Unlinked> => this.task("jira_issue_unlink", args),
      /** Add a web link (URL and title) to a Jira issue. */
      remoteLink: (args: JiraIssueRemoteLinkArgs): Promise<RemoteLink> => this.task("jira_issue_remote_link", args),
      /** Field changes of a Jira issue, newest first: when, who, field, from, to. */
      history: (args: JiraIssueHistoryArgs): Promise<Change[]> => this.task("jira_issue_history", args),
      /** Jira issue link types with their outward and inward phrases. */
      linkTypes: (args: JiraIssueLinkTypesArgs = {}): Promise<LinkType[]> => this.task("jira_issue_link_types", args),
      /** Copy a Jira issue (summary, description, type, labels, priority, components, parent). */
      clone: (args: JiraIssueCloneArgs): Promise<Created> => this.task("jira_issue_clone", args),
      /** Log time on a Jira issue. */
      worklog: (args: JiraIssueWorklogArgs): Promise<WorklogAdded> => this.task("jira_issue_worklog", args),
      /** Upload local files to a Jira issue. */
      attach: (args: JiraIssueAttachArgs): Promise<Attachment[]> => this.task("jira_issue_attach", args),
      /** List a Jira issue's attachments. */
      attachments: (args: JiraIssueAttachmentsArgs): Promise<Attachment[]> => this.task("jira_issue_attachments", args),
      /** Issue type names of a Jira project (without sub-task types), for the `type` argument of jira_issue_create. */
      types: (args: JiraIssueTypesArgs): Promise<string[]> => this.task("jira_issue_types", args),
    },
    /** `atl jira jql` */
    jql: {
      /** Validate JQL with Jira's parser; returns valid, errors, warnings. */
      check: (args: JiraJqlCheckArgs): Promise<JqlCheck> => this.task("jira_jql_check", args),
      /** Fields (with operators) and functions usable in JQL on this site. */
      fields: (args: JiraJqlFieldsArgs = {}): Promise<JqlReference> => this.task("jira_jql_fields", args),
      /** Value suggestions for a JQL field, for example field=status prefix=In. */
      suggest: (args: JiraJqlSuggestArgs): Promise<Suggestion[]> => this.task("jira_jql_suggest", args),
    },
    /** `atl jira project` */
    project: {
      /** Jira projects. */
      list: (args: JiraProjectListArgs = {}): Promise<Project[]> => this.task("jira_project_list", args),
    },
    /** `atl jira release` */
    release: {
      /** Versions (releases) of a Jira project: name, released, archived, release date. */
      list: (args: JiraReleaseListArgs): Promise<Release[]> => this.task("jira_release_list", args),
    },
    /** `atl jira sprint` */
    sprint: {
      /** Create a future sprint on a board (default: the project's first board). */
      create: (args: JiraSprintCreateArgs): Promise<Sprint> => this.task("jira_sprint_create", args),
      /** Start a future sprint; Jira needs a start and an end date. */
      start: (args: JiraSprintStartArgs): Promise<Sprint> => this.task("jira_sprint_start", args),
      /** Complete an active sprint. */
      close: (args: JiraSprintCloseArgs): Promise<Sprint> => this.task("jira_sprint_close", args),
      /** Sprints of a board. */
      list: (args: JiraSprintListArgs = {}): Promise<BoardSprints> => this.task("jira_sprint_list", args),
      /** Move issues into a sprint. */
      add: (args: JiraSprintAddArgs): Promise<SprintIssues> => this.task("jira_sprint_add", args),
    },
    /** `atl jira worklog` */
    worklog: {
      /** Time logged on a Jira issue: author, time spent, start, comment. */
      list: (args: JiraWorklogListArgs): Promise<Worklog[]> => this.task("jira_worklog_list", args),
    },
  }

  /** Confluence pages, blogs, spaces, comments, labels and attachments. */
  readonly confluence = {
    /** Search Confluence by words or CQL; returns id, type, title, space, excerpt, url. */
    search: (args: ConfluenceSearchArgs = {}): Promise<SearchResults> => this.task("confluence_search", args),
    /** `atl confluence attachment` */
    attachment: {
      /** Delete a Confluence attachment by ID. */
      delete: (args: ConfluenceAttachmentDeleteArgs): Promise<Deleted> => this.task("confluence_attachment_delete", args),
    },
    /** `atl confluence blog` */
    blog: {
      /** Blog posts of a Confluence space, newest first. */
      list: (args: ConfluenceBlogListArgs): Promise<PageSummary[]> => this.task("confluence_blog_list", args),
      /** One Confluence blog post with its body as Markdown. */
      get: (args: ConfluenceBlogGetArgs): Promise<Page> => this.task("confluence_blog_get", args),
      /** Publish a Confluence blog post from Markdown. */
      create: (args: ConfluenceBlogCreateArgs): Promise<PageSummary> => this.task("confluence_blog_create", args),
    },
    /** `atl confluence comment` */
    comment: {
      /** Delete a Confluence comment by ID. */
      delete: (args: ConfluenceCommentDeleteArgs): Promise<Deleted> => this.task("confluence_comment_delete", args),
    },
    /** `atl confluence page` */
    page: {
      /** A Confluence page as Markdown (with title, space, version, url). */
      get: (args: ConfluencePageGetArgs): Promise<PageView> => this.task("confluence_page_get", args),
      /** Create a Confluence page from Markdown (or storage format). */
      create: (args: ConfluencePageCreateArgs): Promise<PageSummary> => this.task("confluence_page_create", args),
      /** Replace a Confluence page's title and/or body; the version is bumped automatically. */
      update: (args: ConfluencePageUpdateArgs): Promise<PageSummary> => this.task("confluence_page_update", args),
      /** Delete a Confluence page. */
      delete: (args: ConfluencePageDeleteArgs): Promise<Deleted> => this.task("confluence_page_delete", args),
      /** Children of a Confluence page (Cloud: also folders), or with `recursive` the whole tree, depth first, each row with type, depth and parent. */
      children: (args: ConfluencePageChildrenArgs): Promise<TreeNode[]> => this.task("confluence_page_children", args),
      /** Confluence pages whose title contains the words, optionally in one space. */
      find: (args: ConfluencePageFindArgs): Promise<SearchHit[]> => this.task("confluence_page_find", args),
      /** Upload local files to a Confluence page. */
      attach: (args: ConfluencePageAttachArgs): Promise<PageAttachment[]> => this.task("confluence_page_attach", args),
      /** Attachments of a Confluence page with download URLs. */
      attachments: (args: ConfluencePageAttachmentsArgs): Promise<PageAttachment[]> => this.task("confluence_page_attachments", args),
      /** Footer (or with `inline`, inline) comments of a Confluence page, oldest first, as Markdown, with the replied-to `parent`. */
      comments: (args: ConfluencePageCommentsArgs): Promise<PageComment[]> => this.task("confluence_page_comments", args),
      /** Add a footer comment (Markdown) to a Confluence page, or a reply with `parent`. */
      comment: (args: ConfluencePageCommentArgs): Promise<PageComment> => this.task("confluence_page_comment", args),
      /** Export a Confluence page (and with `recursive` its tree) to local Markdown folders: <title>/index.md with front matter and _attachments/. Changes nothing on the server. */
      export: (args: ConfluencePageExportArgs): Promise<Exported[]> => this.task("confluence_page_export", args),
      /** Publish local Markdown files: update the page in each file's front matter (refusing pages changed on the server unless `force`, skipping unchanged ones), create pages for new files under their folder's index.md, upload referenced images, and write ids and versions back. */
      push: (args: ConfluencePagePushArgs): Promise<Pushed[]> => this.task("confluence_page_push", args),
      /** Copy a Confluence page and the pages below it under another page (titles, bodies, labels). */
      copyTree: (args: ConfluencePageCopyTreeArgs): Promise<Copied[]> => this.task("confluence_page_copy_tree", args),
      /** Labels of a Confluence page. */
      labels: (args: ConfluencePageLabelsArgs): Promise<string[]> => this.task("confluence_page_labels", args),
      /** Add and remove labels on a Confluence page; returns the resulting labels. */
      label: (args: ConfluencePageLabelArgs): Promise<string[]> => this.task("confluence_page_label", args),
      /** Move a Confluence page under another page (position append) or next to it (before/after, Cloud only). */
      move: (args: ConfluencePageMoveArgs): Promise<PageSummary> => this.task("confluence_page_move", args),
      /** Versions of a Confluence page, newest first: number, author, date, message. Read one with confluence_page_get and `version`. */
      history: (args: ConfluencePageHistoryArgs): Promise<PageVersion[]> => this.task("confluence_page_history", args),
    },
    /** `atl confluence space` */
    space: {
      /** Create a Confluence space with a key, a name and an optional plain-text description. */
      create: (args: ConfluenceSpaceCreateArgs): Promise<Space> => this.task("confluence_space_create", args),
      /** Confluence spaces. */
      list: (args: ConfluenceSpaceListArgs = {}): Promise<Space[]> => this.task("confluence_space_list", args),
      /** Top-level pages of a Confluence space. */
      pages: (args: ConfluenceSpacePagesArgs): Promise<PageSummary[]> => this.task("confluence_space_pages", args),
    },
  }

  /** Bitbucket repositories, branches, pull requests and pipelines. */
  readonly bitbucket = {
    /** `atl bitbucket branch` */
    branch: {
      /** Branches of a Bitbucket repository, most recently changed first. */
      list: (args: BitbucketBranchListArgs): Promise<Branch[]> => this.task("bitbucket_branch_list", args),
      /** Create a branch from another branch, a tag or a commit (default: the main branch). */
      create: (args: BitbucketBranchCreateArgs): Promise<Branch> => this.task("bitbucket_branch_create", args),
      /** Delete a branch. Unmerged commits on it can be lost. */
      delete: (args: BitbucketBranchDeleteArgs): Promise<BranchDeleted> => this.task("bitbucket_branch_delete", args),
    },
    /** `atl bitbucket commit` */
    commit: {
      /** Recent commits of a branch, newest first: hash, author, date, first line of the message. */
      list: (args: BitbucketCommitListArgs): Promise<Commit[]> => this.task("bitbucket_commit_list", args),
    },
    /** `atl bitbucket pipeline` */
    pipeline: {
      /** Latest Bitbucket Pipelines runs, newest first (Cloud only). */
      list: (args: BitbucketPipelineListArgs): Promise<Pipeline[]> => this.task("bitbucket_pipeline_list", args),
      /** Start the pipeline of a branch, or a custom pipeline (Cloud only). */
      run: (args: BitbucketPipelineRunArgs): Promise<Pipeline> => this.task("bitbucket_pipeline_run", args),
      /** Stop a running Bitbucket pipeline by build number or UUID (Cloud only). */
      stop: (args: BitbucketPipelineStopArgs): Promise<Pipeline> => this.task("bitbucket_pipeline_stop", args),
      /** One Bitbucket pipeline by build number or UUID, with each step's state and result (Cloud only). */
      view: (args: BitbucketPipelineViewArgs): Promise<PipelineDetail> => this.task("bitbucket_pipeline_view", args),
      /** Log text of one pipeline step (by name or UUID) or of every step (Cloud only). */
      logs: (args: BitbucketPipelineLogsArgs): Promise<PipelineLog> => this.task("bitbucket_pipeline_logs", args),
    },
    /** `atl bitbucket pr` */
    pr: {
      /** Pull requests of a Bitbucket repository by state. */
      list: (args: BitbucketPrListArgs): Promise<PullRequest[]> => this.task("bitbucket_pr_list", args),
      /** One pull request with description, reviewers and approvals. */
      view: (args: BitbucketPrViewArgs): Promise<PullRequest> => this.task("bitbucket_pr_view", args),
      /** Open a pull request from a source branch. */
      create: (args: BitbucketPrCreateArgs): Promise<PullRequest> => this.task("bitbucket_pr_create", args),
      /** Merge a pull request. */
      merge: (args: BitbucketPrMergeArgs): Promise<PullRequest> => this.task("bitbucket_pr_merge", args),
      /** Approve a pull request as the signed-in user. */
      approve: (args: BitbucketPrApproveArgs): Promise<PrApproval> => this.task("bitbucket_pr_approve", args),
      /** Add a Markdown comment to a pull request. */
      comment: (args: BitbucketPrCommentArgs): Promise<PrCommented> => this.task("bitbucket_pr_comment", args),
      /** The unified diff of a pull request. */
      diff: (args: BitbucketPrDiffArgs): Promise<PrDiff> => this.task("bitbucket_pr_diff", args),
      /** Comments of a pull request, oldest first, as Markdown: general and inline (path, line), replies flattened with `parent`. */
      comments: (args: BitbucketPrCommentsArgs): Promise<PrComment[]> => this.task("bitbucket_pr_comments", args),
      /** Build statuses of a pull request's latest commit, with `passed` (none failed) and `pending` (some still running). */
      checks: (args: BitbucketPrChecksArgs): Promise<PrChecks> => this.task("bitbucket_pr_checks", args),
      /** Remove your approval from a pull request. */
      unapprove: (args: BitbucketPrUnapproveArgs): Promise<PrApproval> => this.task("bitbucket_pr_unapprove", args),
      /** Mark a draft pull request as ready for review (or, with `draft: true`, as a draft again). */
      ready: (args: BitbucketPrReadyArgs): Promise<PullRequest> => this.task("bitbucket_pr_ready", args),
      /** Decline a Bitbucket pull request (Cloud cannot reopen it). */
      decline: (args: BitbucketPrDeclineArgs): Promise<PullRequest> => this.task("bitbucket_pr_decline", args),
      /** Mark a Bitbucket pull request as needing changes by you (Cloud: request changes; Data Center: needs work). */
      requestChanges: (args: BitbucketPrRequestChangesArgs): Promise<ChangesRequested> => this.task("bitbucket_pr_request_changes", args),
      /** Change a Bitbucket pull request's title, description (Markdown) or destination branch. */
      edit: (args: BitbucketPrEditArgs): Promise<PullRequest> => this.task("bitbucket_pr_edit", args),
      /** Add and remove reviewers of a Bitbucket pull request; other reviewers are kept. */
      reviewers: (args: BitbucketPrReviewersArgs): Promise<PullRequest> => this.task("bitbucket_pr_reviewers", args),
    },
    /** `atl bitbucket repo` */
    repo: {
      /** Bitbucket repositories of a workspace (Cloud) or project (Data Center), or the ones you belong to. */
      list: (args: BitbucketRepoListArgs = {}): Promise<Repo[]> => this.task("bitbucket_repo_list", args),
      /** One Bitbucket repository with its default branch and URL. */
      view: (args: BitbucketRepoViewArgs): Promise<Repo> => this.task("bitbucket_repo_view", args),
    },
  }

  /** Jira Service Management desks, queues, requests and approvals. */
  readonly jsm = {
    /** `atl jsm desk` */
    desk: {
      /** Jira Service Management service desks (id, project key, name). */
      list: (args: JsmDeskListArgs = {}): Promise<Desk[]> => this.task("jsm_desk_list", args),
    },
    /** `atl jsm queue` */
    queue: {
      /** Queues of a service desk with issue counts. */
      list: (args: JsmQueueListArgs): Promise<Queue[]> => this.task("jsm_queue_list", args),
      /** Issues in a service desk queue (by queue ID or name). */
      issues: (args: JsmQueueIssuesArgs): Promise<IssueSummary[]> => this.task("jsm_queue_issues", args),
    },
    /** `atl jsm request` */
    request: {
      /** Request types of a service desk. */
      types: (args: JsmRequestTypesArgs): Promise<RequestType[]> => this.task("jsm_request_types", args),
      /** Customer requests by status, optionally of one service desk. */
      list: (args: JsmRequestListArgs = {}): Promise<Request[]> => this.task("jsm_request_list", args),
      /** One customer request with its description and latest comments. */
      view: (args: JsmRequestViewArgs): Promise<RequestDetail> => this.task("jsm_request_view", args),
      /** Raise a customer request, optionally for a customer. */
      create: (args: JsmRequestCreateArgs): Promise<Request> => this.task("jsm_request_create", args),
      /** Reply to a customer request, or add an internal note. */
      comment: (args: JsmRequestCommentArgs): Promise<RequestComment> => this.task("jsm_request_comment", args),
      /** Transitions available on a customer request. */
      transitions: (args: JsmRequestTransitionsArgs): Promise<RequestTransition[]> => this.task("jsm_request_transitions", args),
      /** Run a customer request transition by name or ID. */
      move: (args: JsmRequestMoveArgs): Promise<RequestMoved> => this.task("jsm_request_move", args),
      /** Approvals of a JSM request: name, decision (pending, approved, declined), approvers, and whether you can answer. */
      approvals: (args: JsmRequestApprovalsArgs): Promise<Approval[]> => this.task("jsm_request_approvals", args),
      /** Approve or decline (decision: decline) a JSM request's approval; without an ID, the one pending approval you can answer. */
      approve: (args: JsmRequestApproveArgs): Promise<Approval> => this.task("jsm_request_approve", args),
      /** List a JSM request's participants, after adding and removing any (me, an email or a name). */
      participants: (args: JsmRequestParticipantsArgs): Promise<Participants> => this.task("jsm_request_participants", args),
      /** SLAs of a JSM request: state (ongoing, paused, met, breached), time remaining and goal. */
      sla: (args: JsmRequestSlaArgs): Promise<Sla[]> => this.task("jsm_request_sla", args),
      /** Attach local files to a JSM request (public unless internal). */
      attach: (args: JsmRequestAttachArgs): Promise<RequestAttachment[]> => this.task("jsm_request_attach", args),
      /** Attachments of a JSM request with download URLs. */
      attachments: (args: JsmRequestAttachmentsArgs): Promise<RequestAttachment[]> => this.task("jsm_request_attachments", args),
    },
  }

  /** Trello boards, lists, cards and checklists. */
  readonly trello = {
    /** `atl trello board` */
    board: {
      /** Your open Trello boards. */
      list: (args: TrelloBoardListArgs = {}): Promise<TrelloBoard[]> => this.task("trello_board_list", args),
    },
    /** `atl trello card` */
    card: {
      /** Cards of a Trello board or one of its lists, or your cards on every board (`mine`). */
      list: (args: TrelloCardListArgs = {}): Promise<Card[]> => this.task("trello_card_list", args),
      /** One Trello card with list, board, description and latest comments. */
      view: (args: TrelloCardViewArgs): Promise<CardDetail> => this.task("trello_card_view", args),
      /** Create a Trello card in a list (by name with `board`, or by list ID). */
      create: (args: TrelloCardCreateArgs): Promise<Card> => this.task("trello_card_create", args),
      /** Move a Trello card to another list (by default on the card's own board). */
      move: (args: TrelloCardMoveArgs): Promise<Card> => this.task("trello_card_move", args),
      /** Add a comment to a Trello card. */
      comment: (args: TrelloCardCommentArgs): Promise<CardComment> => this.task("trello_card_comment", args),
      /** Archive (close) a Trello card. */
      archive: (args: TrelloCardArchiveArgs): Promise<Card> => this.task("trello_card_archive", args),
      /** Change a Trello card: name, description, due date, done, labels (by name or color) and members (username, name or me). */
      edit: (args: TrelloCardEditArgs): Promise<Card> => this.task("trello_card_edit", args),
    },
    /** `atl trello checklist` */
    checklist: {
      /** Checklists of a Trello card with their items and whether each is done. */
      list: (args: TrelloChecklistListArgs): Promise<Checklist[]> => this.task("trello_checklist_list", args),
      /** Add a checklist with items to a Trello card. */
      create: (args: TrelloChecklistCreateArgs): Promise<Checklist> => this.task("trello_checklist_create", args),
      /** Tick (or with done: false, untick) a Trello checklist item by name or ID. */
      check: (args: TrelloChecklistCheckArgs): Promise<CheckItem> => this.task("trello_checklist_check", args),
    },
    /** `atl trello list` */
    list: {
      /** Open lists of a Trello board (board by name, URL, short link or ID). */
      list: (args: TrelloListListArgs): Promise<TrelloList[]> => this.task("trello_list_list", args),
    },
  }

  /** Atlassian Admin organizations, users, groups and audit events. */
  readonly admin = {
    /** `atl admin directory` */
    directory: {
      /** User directories of an Atlassian organization. */
      list: (args: AdminDirectoryListArgs = {}): Promise<Directory[]> => this.task("admin_directory_list", args),
    },
    /** `atl admin domain` */
    domain: {
      /** Domains of an Atlassian organization with their verification status. */
      list: (args: AdminDomainListArgs = {}): Promise<Domain[]> => this.task("admin_domain_list", args),
    },
    /** `atl admin event` */
    event: {
      /** Audit log events of an Atlassian organization, newest first, filtered by words, dates and action. */
      list: (args: AdminEventListArgs = {}): Promise<Event[]> => this.task("admin_event_list", args),
    },
    /** `atl admin group` */
    group: {
      /** Add a managed user (email or account ID) to a group (name or ID) of an Atlassian organization. */
      addMember: (args: AdminGroupAddMemberArgs): Promise<Membership> => this.task("admin_group_add_member", args),
      /** Remove a managed user from a group of an Atlassian organization. */
      removeMember: (args: AdminGroupRemoveMemberArgs): Promise<Membership> => this.task("admin_group_remove_member", args),
      /** Groups of an Atlassian organization (every directory), optionally matching `search`. */
      list: (args: AdminGroupListArgs = {}): Promise<Group[]> => this.task("admin_group_list", args),
    },
    /** `atl admin org` */
    org: {
      /** Organizations the Atlassian Admin API key can see. */
      list: (args: AdminOrgListArgs = {}): Promise<Org[]> => this.task("admin_org_list", args),
    },
    /** `atl admin user` */
    user: {
      /** Managed users of an Atlassian organization (every directory), optionally matching `search`. */
      list: (args: AdminUserListArgs = {}): Promise<AdminUser[]> => this.task("admin_user_list", args),
      /** Suspend a managed user's access to every product (by account ID or email). */
      suspend: (args: AdminUserSuspendArgs): Promise<AdminUser> => this.task("admin_user_suspend", args),
      /** Restore a suspended managed user's access (by account ID or email). */
      restore: (args: AdminUserRestoreArgs): Promise<AdminUser> => this.task("admin_user_restore", args),
    },
  }
}
