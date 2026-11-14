# atlassian-toolkit (TypeScript for Node.js, Bun and Deno)

Jira, Confluence, Bitbucket, Trello and Atlassian Admin, on Cloud and Data Center, from TypeScript. A Node-API addon built from the `atl` Rust core with napi-rs; Bun and Deno load the same addon.

```ts
import { Client, AtlError, markup } from "atlassian-toolkit"

const client = Client.fromEnv() // uses `atl auth login`
const { issues } = await client.jiraIssueList({ project: "ABC", assignee: "me" })
await client.jiraIssueMove({ key: "ABC-12", status: "Done", comment: "Shipped in **v2**" })
const page = await client.confluencePageGet({ page: "DOCS:Handbook" }) // page.markdown
await client.task("jira_epic_add", { epic: "ABC-1", issues: ["ABC-12"] }) // any task
await client.call("jira", "getIssue", { params: { issueIdOrKey: "ABC-12" } }) // any generated operation
```

`Client.withConnections({ jira: { deployment: "cloud", base_url: "https://acme.atlassian.net", auth: { kind: "basic", username, secret } } })` uses explicit credentials; `.configured({ readOnly: true })` refuses writes. Errors are `AtlError` with `code`, `status`, `details`. `tasks()` lists every task with its JSON Schema.

Build and test: `bun install && bun run build && bun run test && bun run test:bun && bun run test:deno`.
