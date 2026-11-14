// End-to-end tests of the Node SDK against a local mock Jira (node:test).
const test = require("node:test")
const assert = require("node:assert/strict")
const http = require("node:http")
const { Client, AtlError, markup, tasks } = require("../dist/index.js")

const requests = []
const server = http.createServer((req, res) => {
  let body = ""
  req.on("data", (chunk) => (body += chunk))
  req.on("end", () => {
    requests.push({
      method: req.method,
      url: req.url,
      auth: req.headers.authorization,
      body: body ? JSON.parse(body) : null,
    })
    const reply = (status, data) => {
      res.writeHead(status, { "content-type": "application/json" })
      res.end(JSON.stringify(data))
    }
    if (req.method === "POST" && req.url === "/rest/api/3/search/jql") {
      reply(200, {
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
    } else if (req.url.startsWith("/rest/api/3/issue/A-1")) {
      reply(200, { key: "A-1", fields: { summary: "Fix login" } })
    } else {
      reply(404, { errorMessages: ["Issue does not exist"] })
    }
  })
})

let client
test.before(async () => {
  await new Promise((resolve) => server.listen(0, "127.0.0.1", resolve))
  const base_url = `http://127.0.0.1:${server.address().port}`
  client = Client.withConnections({
    jira: { deployment: "cloud", base_url, auth: { kind: "bearer", token: "t" } },
  }).configured({ maxRetries: 0 })
})
test.after(() => server.close())

test("tasks return small records and send credentials", async () => {
  const page = await client.jiraIssueList({ project: "A" })
  assert.equal(page.issues[0].key, "A-1")
  assert.equal(page.jql, 'project = "A" ORDER BY updated DESC')
  assert.equal(requests.at(-1).auth, "Bearer t")
})

test("generated operations run by operationId", async () => {
  const issue = await client.call("jira", "getIssue", { params: { issueIdOrKey: "A-1" } })
  assert.equal(issue.fields.summary, "Fix login")
})

test("errors become AtlError with code, status and details", async () => {
  await assert.rejects(client.jiraIssueView({ key: "NOPE-1" }), (err) => {
    assert.ok(err instanceof AtlError)
    assert.equal(err.code, "api")
    assert.equal(err.status, 404)
    assert.deepEqual(err.details, { errorMessages: ["Issue does not exist"] })
    return true
  })
  await assert.rejects(
    client.configured({ readOnly: true }).jiraIssueComment({ key: "A-1", body: "x" }),
    (err) => err.code === "read_only",
  )
})

test("markup and the task catalog", () => {
  assert.equal(markup.adfToMarkdown(markup.markdownToAdf("**hi**")), "**hi**")
  assert.equal(markup.markdownToWiki("**hi**"), "*hi*")
  assert.ok(tasks().some((t) => t.name === "confluence_page_get"))
})

const camel = (name) => name.replace(/_([a-z0-9])/g, (_, c) => c.toUpperCase())

test("every task has a typed method", () => {
  for (const task of tasks()) {
    assert.equal(typeof client[camel(task.name)], "function", task.name)
    assert.equal(task.schema.type, "object", task.name)
    assert.ok(task.description, task.name)
  }
})

test("task() by name matches the typed method", async () => {
  assert.deepEqual(
    await client.task("jira_issue_list", { project: "A" }),
    await client.jiraIssueList({ project: "A" }),
  )
})

test("concurrent calls share one client", async () => {
  const pages = await Promise.all(
    Array.from({ length: 64 }, () => client.jiraIssueList({ project: "A" })),
  )
  assert.deepEqual(new Set(pages.map((page) => page.issues[0].key)), new Set(["A-1"]))
})

test("Markdown survives ADF and storage format", () => {
  for (const markdown of [
    "# Title\n\nA paragraph with **bold**, *italic* and `code`.",
    "- one\n- two\n  - nested",
    "1. first\n2. second",
    "[a link](https://example.com)",
    '```python\nprint("hi")\n```',
    "> a quote",
  ]) {
    assert.equal(markup.adfToMarkdown(markup.markdownToAdf(markdown)).trim(), markdown)
    assert.equal(markup.storageToMarkdown(markup.markdownToStorage(markdown)).trim(), markdown)
  }
})

test("an unknown task name lists the choices", async () => {
  await assert.rejects(
    client.task("jira_issue_nope", {}),
    (err) => err instanceof AtlError && /jira_issue_list/.test(err.message),
  )
})
