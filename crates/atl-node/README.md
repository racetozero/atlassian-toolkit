# atlassian-toolkit (TypeScript for Node.js, Bun and Deno)

Jira, Confluence, Bitbucket, Trello and Atlassian Admin, on Cloud and Data Center, from TypeScript. A Node-API addon built from the `atl` Rust core with napi-rs; Bun and Deno load the same addon. Works with `import` and `require`; npm installs only the addon for your platform (`atlassian-toolkit-<platform>`: macOS, Linux with glibc, and Windows, on x64 and arm64).

```ts
import { Client, AtlError, markup } from "atlassian-toolkit"

const client = Client.fromEnv() // uses `atl auth login`
const { issues } = await client.jira.issue.list({ project: "ABC", assignee: "me" }) // IssueList
await client.jira.issue.move({ key: "ABC-12", status: "Done", comment: "Shipped in **v2**" })
const page = await client.confluence.page.get({ page: "DOCS:Handbook" }) // page.markdown
await client.task("jira_epic_add", { epic: "ABC-1", issues: ["ABC-12"] }) // any task, typed by name
await client.call("jira", "getIssue", { params: { issueIdOrKey: "ABC-12" } }) // any generated operation
```

`Client.withConnections({ jira: { deployment: "cloud", base_url: "https://acme.atlassian.net", auth: { kind: "basic", username, secret } } })` uses explicit credentials; `.configured({ readOnly: true })` refuses writes. Errors are `AtlError` with `code`, `status`, `details`. Every task is a method on its CLI path (`atl jira issue list` is `client.jira.issue.list`), with typed arguments and results. `tasks()` lists every task with the JSON Schemas of its arguments and result.

Build and test: `bun install && bun run build && bun run test && bun run test:bun && bun run test:deno`.
