# SDKs

All the SDKs use the same Rust core and the same tasks as the CLI. A task has the same path everywhere: `atl jira issue list` is `client.jira.issue.list(...)` in Python and TypeScript, and it returns the same data as `atl jira issue list --agent`.

| Language                                 | Install                          | Source            |
| ---------------------------------------- | -------------------------------- | ----------------- |
| Rust                                     | `cargo add atlassian-toolkit`    | `crates/atl-sdk`  |
| Python 3.11+                             | `uv add atlassian-toolkit`       | `crates/atl-py`   |
| Node.js, Bun, Deno                       | `bun add atlassian-toolkit`      | `crates/atl-node` |
| Browsers, workers and Deno (WebAssembly) | `bun add atlassian-toolkit-wasm` | `crates/atl-wasm` |

Each SDK has:

- one namespace per product (`jira`, `confluence`, `bitbucket`, `jsm`, `trello`, `admin`) with a typed method for each task
- typed results: pydantic models in Python, interfaces in TypeScript, structs in Rust
- `task(name, args)` to run any task by name
- `call(service, operation, ...)` for any of the 3,085 generated REST operations
- `request(service, method, path, ...)` for any path
- `tasks()`, which returns the name, description, and JSON Schemas of the arguments and the result of each task
- the Markdown converters
- read-only mode
- errors with `code`, `status` and `details`

The SDKs read your `atl auth login` config by default, or you can give explicit connections.

## Python

```python
import atl_sdk

client = atl_sdk.Client()
page = client.jira.issue.list(project='ABC', status=['In Progress'])
for issue in page.issues:
    print(issue.key, issue.summary)
client.jira.issue.move(key='ABC-1', status='Done')
```

The arguments are `TypedDict`s and the results are pydantic models, so your editor completes both and a type checker finds a wrong argument name before you run the code. Fields that a newer server adds are kept in `model_extra`.

`atl_sdk.AsyncClient` has the same methods as coroutines. Requests run on the SDK's own threads, so they do not block the event loop, and `asyncio.gather` runs them at the same time:

```python
import asyncio
import atl_sdk


async def main():
    client = atl_sdk.AsyncClient()
    issues = await asyncio.gather(
        *(client.jira.issue.view(key=key) for key in ['ABC-1', 'ABC-2', 'ABC-3'])
    )
    print([issue.status for issue in issues])


asyncio.run(main())
```

The sync client releases the GIL during a request, so threads also run requests in parallel, including on free-threaded Python 3.14t and 3.15t.

## TypeScript

```ts
import { Client } from "atlassian-toolkit"

const client = Client.fromEnv()
const { issues } = await client.jira.issue.list({ project: "ABC" })
```

The package works with `import` and `require`. npm installs only the native addon for your platform (macOS, Linux with glibc, or Windows, on x64 and arm64). On another platform, use the WebAssembly SDK.

## WebAssembly

```ts
import { init, Client } from "atlassian-toolkit-wasm"

await init()
const client = Client.withConnections({
  jira: {
    deployment: "cloud",
    base_url: "https://proxy.example.com/jira",
    auth: { kind: "bearer", token },
  },
})
await client.confluence.page.get({ page: "DOCS:Home" })
```

In a web page, the browser's CORS rule applies. Point `base_url` at a proxy on your own origin. See `crates/atl-wasm/README.md`.

## Rust

```rust
use atl_sdk::Client;

#[tokio::main]
async fn main() -> atl_sdk::Result<()> {
    let client = Client::from_env(None)?; // the config file and ATL_* variables
    let page = client.jira().search("project = ABC", 20, None).await?;
    for issue in &page.issues {
        println!("{} {}", issue.key, issue.summary);
    }
    Ok(())
}
```

`client.jira()`, `confluence()`, `bitbucket()`, `jsm()`, `trello()` and `admin()` hide the differences between Cloud and Data Center. `client.call(...)` runs any generated operation. `atl_sdk::markup` converts Markdown to and from ADF and Confluence storage format. See [docs.rs/atlassian-toolkit](https://docs.rs/atlassian-toolkit) for the full API.
