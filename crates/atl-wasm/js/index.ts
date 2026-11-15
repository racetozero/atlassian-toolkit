/**
 * Browser, worker and Deno SDK for the Atlassian suite, built from the `atl`
 * Rust core with wasm-bindgen. The API matches the Node SDK.
 *
 * ```ts
 * import { init, Client } from "./index.ts"
 * await init()
 * const client = Client.withConnections({ jira: { deployment: "cloud", base_url: "https://proxy.example/jira", auth: { kind: "bearer", token } } })
 * const { issues } = await client.jira.issue.list({ assignee: "me" })
 * ```
 *
 * Browsers enforce CORS: Atlassian site APIs do not allow arbitrary origins,
 * so point `base_url` at a proxy you control (or use this in a browser
 * extension, worker, or Deno, where CORS does not apply).
 */
import initWasm, * as native from "../pkg/atl_wasm.js"
import {
  type Connections,
  rethrow,
  type TaskArgs,
  type TaskDef,
  type TaskName,
  type TaskResults,
  Tasks,
} from "../../atl-node/ts/common.ts"

export * from "../../atl-node/ts/common.ts"

/** Loads the WebAssembly module once; pass a URL, Response or bytes to load it from elsewhere. */
let ready: Promise<unknown> | undefined
export function init(input?: Parameters<typeof initWasm>[0]): Promise<unknown> {
  ready ??= initWasm(input === undefined ? undefined : { module_or_path: input })
  return ready
}

async function run<T>(fn: () => Promise<unknown>): Promise<T> {
  try {
    return (await fn()) as T
  } catch (err) {
    rethrow(err)
  }
}

export class Client extends Tasks {
  private constructor(private readonly inner: native.Client) {
    super()
  }

  static withConnections(connections: Connections): Client {
    try {
      return new Client(native.Client.withConnections(connections))
    } catch (err) {
      rethrow(err)
    }
  }

  configured(options: { readOnly?: boolean; maxRetries?: number }): Client {
    return new Client(this.inner.configured(options.readOnly, options.maxRetries))
  }

  /** Runs any task by name, for example `task("jira_issue_view", { key: "A-1" })`. */
  task<N extends TaskName>(name: N, args?: TaskArgs[N]): Promise<TaskResults[N]> {
    return run<TaskResults[N]>(() => this.inner.task(name, args ?? {}))
  }

  call<T = unknown>(
    service: string,
    operation: string,
    args: { params?: Record<string, unknown>; body?: unknown } = {},
  ): Promise<T> {
    return run<T>(() => this.inner.call(service, operation, args))
  }

  request<T = unknown>(
    service: string,
    method: string,
    path: string,
    options: { query?: Record<string, unknown>; body?: unknown } = {},
  ): Promise<T> {
    return run<T>(() =>
      this.inner.request(service, method, path, options.query ?? {}, options.body ?? null),
    )
  }
}

export function tasks(): TaskDef[] {
  return native.taskList()
}

export const markup = {
  markdownToAdf: (markdown: string): Record<string, unknown> => native.markdownToAdf(markdown),
  adfToMarkdown: (adf: unknown): string => native.adfToMarkdown(adf),
  markdownToStorage: native.markdownToStorage,
  storageToMarkdown: native.storageToMarkdown,
  markdownToWiki: native.markdownToWiki,
  wikiToMarkdown: native.wikiToMarkdown,
}
