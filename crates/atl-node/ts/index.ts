/**
 * TypeScript SDK for the Atlassian suite (Node.js, Bun, Deno), built on the
 * `atl` Rust core through Node-API.
 *
 * ```ts
 * import { Client } from "atlassian-toolkit"
 * const client = Client.fromEnv()                      // credentials from `atl auth login`
 * const page = await client.jira.issue.list({ project: "ABC", status: ["In Progress"] })
 * const doc = await client.confluence.page.get({ page: "DOCS:Handbook" })
 * const raw = await client.call("jira", "getIssue", { params: { issueIdOrKey: "ABC-1" } })
 * ```
 */
import { existsSync } from "node:fs"
import path from "node:path"
import {
  type Connections,
  rethrow,
  type TaskArgs,
  type TaskDef,
  type TaskName,
  type TaskResults,
  Tasks,
} from "./common"

export * from "./common"

interface NativeClient {
  configured(readOnly?: boolean, maxRetries?: number): NativeClient
  task(name: string, args: string): Promise<string>
  call(service: string, operation: string, args: string): Promise<string>
  request(
    service: string,
    method: string,
    path: string,
    query: string,
    body?: string,
  ): Promise<string>
}

interface NativeBinding {
  NativeClient: {
    fromEnv(profile?: string): NativeClient
    fromConnections(json: string): NativeClient
  }
  taskList(): string
  markdownToAdf(markdown: string): string
  adfToMarkdown(adf: string): string
  markdownToStorage(markdown: string): string
  storageToMarkdown(storage: string): string
  markdownToWiki(markdown: string): string
  wikiToMarkdown(wiki: string): string
}

/** The platforms with a native addon in `native/<platform>/atl.node`. */
const PLATFORMS = [
  "linux-x64",
  "linux-arm64",
  "darwin-x64",
  "darwin-arm64",
  "win32-x64",
  "win32-arm64",
]

function load(): NativeBinding {
  const platform = `${process.platform}-${process.arch}`
  const file = path.join(__dirname, "..", "native", platform, "atl.node")
  let why = `there is no addon for ${platform}`
  if (existsSync(file)) {
    try {
      return require(file) as NativeBinding
    } catch (err) {
      // A glibc build fails to load on musl (Alpine); the report is slow, so
      // ask for it only here.
      const report = process.report?.getReport() as { header?: { glibcVersionRuntime?: string } }
      if (process.platform !== "linux" || report?.header?.glibcVersionRuntime) throw err
      why = "the Linux addons need glibc, and this system uses musl (for example Alpine)"
    }
  }
  throw new Error(
    `atlassian-toolkit cannot load its native addon: ${why}. It ships for ${PLATFORMS.join(", ")} (Linux with glibc); ` +
      `elsewhere, use the WebAssembly SDK, atlassian-toolkit-wasm.`,
  )
}

const native = load()

async function wrap<T>(promise: Promise<string>): Promise<T> {
  try {
    return JSON.parse(await promise) as T
  } catch (err) {
    rethrow(err)
  }
}

function sync<T>(fn: () => T): T {
  try {
    return fn()
  } catch (err) {
    rethrow(err)
  }
}

export class Client extends Tasks {
  private constructor(private readonly inner: NativeClient) {
    super()
  }

  /** Credentials from `atl auth login` and the ATL_* environment variables. */
  static fromEnv(profile?: string): Client {
    return new Client(sync(() => native.NativeClient.fromEnv(profile)))
  }

  static withConnections(connections: Connections): Client {
    return new Client(sync(() => native.NativeClient.fromConnections(JSON.stringify(connections))))
  }

  /** A copy that refuses writes and/or retries a different number of times. */
  configured(options: { readOnly?: boolean; maxRetries?: number }): Client {
    return new Client(this.inner.configured(options.readOnly, options.maxRetries))
  }

  /** Runs any task by name, for example `task("jira_issue_view", { key: "A-1" })`. */
  task<N extends TaskName>(name: N, args?: TaskArgs[N]): Promise<TaskResults[N]> {
    return wrap<TaskResults[N]>(this.inner.task(name, JSON.stringify(args ?? {})))
  }

  /** Runs any generated operation, for example `call("jira", "getIssue", { params: { issueIdOrKey: "A-1" } })`. */
  call<T = unknown>(
    service: string,
    operation: string,
    args: { params?: Record<string, unknown>; body?: unknown; files?: string[] } = {},
  ): Promise<T> {
    return wrap<T>(this.inner.call(service, operation, JSON.stringify(args)))
  }

  /** Sends a request to any path on a service's server. */
  request<T = unknown>(
    service: string,
    method: string,
    path: string,
    options: { query?: Record<string, unknown>; body?: unknown } = {},
  ): Promise<T> {
    const body = options.body === undefined ? undefined : JSON.stringify(options.body)
    return wrap<T>(
      this.inner.request(service, method, path, JSON.stringify(options.query ?? {}), body),
    )
  }
}

/** Every task with its description and the JSON Schema of its arguments. */
export function tasks(): TaskDef[] {
  return JSON.parse(native.taskList()) as TaskDef[]
}

export const markup = {
  markdownToAdf: (markdown: string): Record<string, unknown> =>
    JSON.parse(native.markdownToAdf(markdown)),
  adfToMarkdown: (adf: unknown): string => sync(() => native.adfToMarkdown(JSON.stringify(adf))),
  markdownToStorage: native.markdownToStorage,
  storageToMarkdown: native.storageToMarkdown,
  markdownToWiki: native.markdownToWiki,
  wikiToMarkdown: native.wikiToMarkdown,
}
