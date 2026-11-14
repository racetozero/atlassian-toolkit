# Contributing

Bug reports, feature requests and pull requests are welcome. For a large change, open an issue first so we can agree on the shape.

## Set up

You need the Rust toolchain from `rust-toolchain.toml` and [just](https://github.com/casey/just). The SDKs also need [uv](https://docs.astral.sh/uv/) (Python), [Bun](https://bun.sh) and Deno (Node/WebAssembly), and `wasm-pack`.

```sh
just fmt          # every pre-commit hook on all files (prek): rustfmt, ruff, oxfmt, clippy
just check        # fmt, clippy with warnings as errors, every Rust test, the wasm build
just sdk-python   # Python SDK tests
just sdk-node     # Node, Bun and Deno tests
just sdk-wasm     # WebAssembly SDK tests
```

## Making a change

- A product command maps its flags to one SDK task in `crates/atl-sdk/src/tasks.rs`, so the CLI, the MCP server and every SDK get it. Add the task first, then the command.
- After adding or changing a task, regenerate the TypeScript and Python types: `ATL_UPDATE_SDK_TYPES=1 cargo test -p atl-sdk --test sdk_types`.
- Tests run against mock servers (`wiremock` in Rust); they never need a real Atlassian site. Cover Cloud and Data Center when they differ.
- Error messages say what the user did, what is accepted, and how to fix it.
- Update `docs/`, the skills in `.agents/skills/` and `CHANGELOG.md` when behavior changes. After you change a skill, run `ATL_UPDATE_SKILLS=1 cargo test -p atl --bin atl skills` to copy it into `crates/atl/skills/`, which `atl skill` embeds.

## Updating the API specs

The OpenAPI specs are copied into `crates/atl-sdk/specs/`, and `crates/atl-sdk/build.rs` turns them into the compiled-in catalogs. To get the latest specs:

```sh
scripts/update-specs.sh
cargo nextest run --workspace
```

## Pull requests

Keep one change per pull request, with a description of what changes for users. CI must pass.
