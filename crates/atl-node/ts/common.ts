/**
 * Types and typed task methods shared by the Node SDK (`ts/index.ts`) and the
 * WebAssembly SDK (`crates/atl-wasm/js/index.ts`). Each SDK only supplies
 * `task()`, `call()` and `request()` over its own transport.
 */
import { type ErrorCode, GeneratedTasks, type JiraIssueListArgs } from "./tasks.ts"
export type { ErrorCode }

export * from "./tasks.ts"

export type Family = "jira" | "confluence" | "bitbucket" | "trello" | "admin" | "admin-scim"
export type Auth =
  | { kind: "basic"; username: string; secret: string }
  | { kind: "bearer"; token: string }
  | { kind: "trello"; key: string; token: string }
  /** A browser session's Cookie header (Data Center behind SSO). */
  | { kind: "cookie"; cookie: string }
  /** Basic credentials from $NETRC or ~/.netrc for the server's host. */
  | { kind: "netrc" }
  /** Anonymous access. */
  | { kind: "none" }
export interface Connection {
  deployment: "cloud" | "data-center"
  base_url: string
  auth: Auth
  read_only?: boolean
  /** Browser URL when base_url is the api.atlassian.com gateway. */
  site_url?: string
  /** PEM client certificate for mutual TLS (Node only). */
  client_cert?: string
  /** PEM key of client_cert, when not in the same file. */
  client_key?: string
  /** PEM CA certificates to trust in addition to the system roots. */
  ca_cert?: string
}
export type Connections = Partial<Record<Family, Connection>>

/** For `input_required`: the argument to fill in and its valid values. */
export interface InputRequired {
  arg: string
  choices: { value: string; label: string }[]
}
export class AtlError extends Error {
  constructor(
    readonly code: ErrorCode,
    message: string,
    readonly status?: number,
    readonly details?: unknown,
    readonly input?: InputRequired,
  ) {
    super(message)
    this.name = "AtlError"
  }
}

/** Rethrows an SDK error (a JSON object in the message) as an AtlError. */
export function rethrow(err: unknown): never {
  const text = err instanceof Error ? err.message : String(err)
  let data:
    | {
        code?: ErrorCode
        message?: string
        status?: number
        details?: unknown
        input?: InputRequired
      }
    | undefined
  try {
    data = JSON.parse(text)
  } catch {
    // Not an SDK error object.
  }
  if (data?.code && data.message) {
    throw new AtlError(data.code, data.message, data.status, data.details, data.input)
  }
  throw err
}

/** Issue filters for jiraIssueList and jiraIssueCount; `~` excludes a value. */
export type IssueQuery = JiraIssueListArgs
export interface IssueSummary {
  key: string
  summary: string
  status: string
  status_category: string
  type: string
  assignee: string | null
  priority: string | null
  updated: string | null
  url: string
}
export interface IssuePage {
  issues: IssueSummary[]
  next_page_token: string | null
  total: number | null
  jql: string
}
export interface Comment {
  id: string
  author: string | null
  created: string | null
  body: string
}
export interface IssueDetail extends IssueSummary {
  reporter: string | null
  created: string | null
  labels: string[]
  components: string[]
  parent: string | null
  subtasks: string[]
  description: string
  comments: Comment[]
  comment_count: number
}
export interface Page {
  id: string
  title: string
  space: string | null
  version: number
  url: string
  markdown: string
  storage?: string
}
export interface PageSummary {
  id: string
  title: string
  url: string
}
export interface PullRequest {
  id: number
  title: string
  state: "OPEN" | "MERGED" | "DECLINED" | "SUPERSEDED"
  author: string | null
  source: string
  destination: string
  updated: string | null
  description: string
  reviewers: string[]
  approved_by: string[]
  url: string
}
export interface ServiceRequest {
  key: string
  summary: string
  status: string
  type: string | null
  reporter: string | null
  created: string | null
  url: string
}
export interface TaskDef {
  name: string
  description: string
  read_only: boolean
  schema: Record<string, unknown>
}

/** Result types of the tasks whose results have a hand-written type; other tasks return `unknown`. */
export interface TaskResults {
  jira_issue_list: IssuePage
  jira_issue_count: { count: number; jql: string }
  jira_issue_view: IssueDetail
  jira_issue_create: { key: string; url: string }
  jira_issue_edit: { key: string; updated: boolean; url: string }
  jira_issue_assign: { key: string; assignee: string }
  jira_issue_move: { key: string; status: string; transition: string }
  jira_issue_comment: Comment
  jira_jql_check: { valid: boolean; errors: string[]; warnings: string[] }
  jira_filter_run: IssuePage
  confluence_page_get: Page
  confluence_page_create: PageSummary
  confluence_page_update: PageSummary
  confluence_search: { cql: string; results: unknown[] }
  bitbucket_pr_list: PullRequest[]
  bitbucket_pr_create: PullRequest
  jsm_request_list: ServiceRequest[]
  jsm_request_create: ServiceRequest
}

/** A typed method for every task (generated in `tasks.ts`); `task()` runs any task by name. */
export { GeneratedTasks as Tasks }
