# SDKs

All the SDKs use the same Rust core and the same tasks as the CLI. `jira_issue_list` returns the same result in Python as in `atl jira issue list --agent`.

| Language                                 | Install                | Source            |
| ---------------------------------------- | ---------------------- | ----------------- |
| Rust                                     | `cargo add atl-sdk`    | `crates/atl-sdk`  |
| Python 3.11+                             | `uv add atl-sdk`       | `crates/atl-py`   |
| Node.js, Bun, Deno                       | `bun add atl-sdk`      | `crates/atl-node` |
| Browsers, workers and Deno (WebAssembly) | `bun add atl-sdk-wasm` | `crates/atl-wasm` |

Each SDK has:

- a typed method for each task, and `task(name, args)`
- `call(service, operation, {params, body})` for any generated operation
- `request(service, method, path, ...)` for any path
- `tasks()`, which returns the name, description and JSON Schema of each task
- the Markdown converters
- read-only mode
- errors with `code`, `status` and `details`

The SDKs read your `atl auth login` config by default, or you can give explicit connections.

## Python

```python
import atl_sdk

client = atl_sdk.Client()
issues = client.jira_issue_list(project='ABC', status=['In Progress'])['issues']
client.jira_issue_move(key='ABC-1', status='Done')
```

The method arguments are typed with `TypedDict`, so your editor completes them.

## TypeScript

```ts
import { Client } from "atl-sdk"

const client = Client.fromEnv()
const { issues } = await client.jiraIssueList({ project: "ABC" })
```

## WebAssembly

```ts
import { init, Client } from "atl-sdk-wasm"

await init()
const client = Client.withConnections({
  jira: {
    deployment: "cloud",
    base_url: "https://proxy.example.com/jira",
    auth: { kind: "bearer", token },
  },
})
await client.confluencePageGet({ page: "DOCS:Home" })
```

In a web page, the browser's CORS rule applies. Point `base_url` at a proxy on your own origin. See `crates/atl-wasm/README.md`.

## Rust

```rust
use atl_sdk::{CallArgs, Client};

#[tokio::main]
async fn main() -> atl_sdk::Result<()> {
    let client = Client::from_env(None)?; // the config file and ATL_* variables
    let issue = client
        .call("jira", "getIssue", &CallArgs::default().param("issueIdOrKey", "ABC-1").param("fields", "summary"))
        .await?;
    println!("{}", issue.json().unwrap_or_default()["fields"]["summary"]);
    Ok(())
}
```

The product modules (`atl_sdk::jira`, `confluence`, `bitbucket`, `jsm`, `trello`, `admin`) hide the differences between Cloud and Data Center. `atl_sdk::markup` converts Markdown to and from ADF and Confluence storage format. See [docs.rs/atl-sdk](https://docs.rs/atl-sdk) for the full API.
