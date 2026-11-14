# atlassian-toolkit-wasm (TypeScript for browsers, workers and Deno)

The `atl` Rust core compiled to WebAssembly with wasm-bindgen. Same tasks, generated operations and Markdown converters as the Node and Python SDKs; HTTP goes through the platform's `fetch`.

```ts
import { init, Client, markup } from "atlassian-toolkit-wasm" // bun add atlassian-toolkit-wasm

await init()
const client = Client.withConnections({
  jira: {
    deployment: "cloud",
    base_url: "https://proxy.example.com/jira",
    auth: { kind: "bearer", token },
  },
})
const { issues } = await client.jiraIssueList({ assignee: "me" })
const page = await client.confluencePageGet({ page: "DOCS:Handbook" })
markup.markdownToAdf("**bold**")
```

**CORS.** Browsers only let a page call another origin when that server allows it, and Atlassian site APIs do not allow arbitrary origins. In a web page, set `base_url` to a proxy on your own origin that forwards to Atlassian. Browser extensions, web workers with host permissions, Deno, and Cloudflare-style workers are not limited this way. File uploads are not available (there is no local file system).

**Size.** `pkg/atl_wasm_bg.wasm` is about 8 MB (1.6 MB gzip), because every Atlassian API catalog is compiled in. `bun run test:size` checks the budget.

Build and test:

    bun install
    bun run build        # wasm-pack, then bun build and the bundled .d.ts
    bun run test
    deno run --allow-net --allow-read test/browser/serve.ts 8765   # then open http://127.0.0.1:8765/
