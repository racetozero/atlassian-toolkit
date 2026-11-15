/**
 * Types and typed task methods shared by the Node SDK (`ts/index.ts`) and the
 * WebAssembly SDK (`crates/atl-wasm/js/index.ts`). Each SDK only supplies
 * `task()`, `call()` and `request()` over its own transport.
 */
import { type ErrorCode, GeneratedTasks } from "./tasks.ts"
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

export interface TaskDef {
  name: string
  description: string
  read_only: boolean
  /** JSON Schema of the arguments. */
  schema: Record<string, unknown>
  /** JSON Schema of the result. */
  output_schema: Record<string, unknown>
}

/** The product namespaces (generated in `tasks.ts`); `task()` runs any task by name. */
export { GeneratedTasks as Tasks }
