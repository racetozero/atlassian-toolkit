// The WebAssembly SDK against a local mock Jira, in Deno (fetch, no CORS).
import { AtlError, Client, init, markup, tasks } from "../js/index.ts"

const seen: { method: string; path: string; auth: string | null }[] = []
const server = Deno.serve({ port: 0, hostname: "127.0.0.1", onListen() {} }, async (req) => {
  const url = new URL(req.url)
  seen.push({ method: req.method, path: url.pathname, auth: req.headers.get("authorization") })
  if (req.method === "POST" && url.pathname === "/rest/api/3/search/jql") {
    await req.json()
    return Response.json({
      issues: [
        {
          key: "A-1",
          fields: {
            summary: "Fix login",
            status: { name: "To Do", statusCategory: { key: "new" } },
            issuetype: { name: "Bug" },
          },
        },
      ],
    })
  }
  if (url.pathname === "/rest/api/3/issue/A-1")
    return Response.json({ key: "A-1", fields: { summary: "Fix login" } })
  return Response.json({ errorMessages: ["Issue does not exist"] }, { status: 404 })
})
const base_url = `http://127.0.0.1:${server.addr.port}`

await init(await Deno.readFile(new URL("../pkg/atl_wasm_bg.wasm", import.meta.url)))
const client = Client.withConnections({
  jira: { deployment: "cloud", base_url, auth: { kind: "basic", username: "ada", secret: "t" } },
}).configured({ maxRetries: 0 })

function assertEquals(actual: unknown, expected: unknown) {
  if (JSON.stringify(actual) !== JSON.stringify(expected))
    throw new Error(`expected ${JSON.stringify(expected)}, got ${JSON.stringify(actual)}`)
}

Deno.test("tasks run over fetch with credentials", async () => {
  const page = await client.jiraIssueList({ project: "A" })
  assertEquals(page.issues[0].key, "A-1")
  assertEquals(seen.at(-1)?.auth, "Basic YWRhOnQ=")
})

Deno.test("generated operations run by operationId", async () => {
  const issue = await client.call<{ fields: { summary: string } }>("jira", "getIssue", {
    params: { issueIdOrKey: "A-1" },
  })
  assertEquals(issue.fields.summary, "Fix login")
})

Deno.test("errors become AtlError", async () => {
  try {
    await client.jiraIssueView({ key: "NOPE-1" })
    throw new Error("should fail")
  } catch (err) {
    if (!(err instanceof AtlError)) throw err
    assertEquals([err.code, err.status], ["api", 404])
  }
})

Deno.test("markup and task catalog work without a network", () => {
  assertEquals(markup.adfToMarkdown(markup.markdownToAdf("**hi**")), "**hi**")
  assertEquals(markup.markdownToWiki("**hi**"), "*hi*")
  if (!tasks().some((t) => t.name === "confluence_page_get"))
    throw new Error("task list incomplete")
})

Deno.test({
  name: "stop the mock server",
  sanitizeOps: false,
  sanitizeResources: false,
  fn: () => server.shutdown(),
})
