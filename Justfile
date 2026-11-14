default:
    @just --list

# Format all tracked files through prek.
fmt:
    @prek run --all-files

# Format, lint with warnings as errors, and run every test.
check:
    cargo fmt --all --check
    cargo clippy --workspace --all-targets -- -D warnings
    cargo nextest run --workspace
    cargo test --workspace --doc
    cargo check -p atl-sdk --target wasm32-unknown-unknown --no-default-features

# Build the release binary and run dry runs on Cloud and Data Center.
smoke:
    cargo build --release -p atl
    ATL_CONFIG_DIR=$(mktemp -d) target/release/atl api jira issues get-issue --issue-id-or-key ABC-1 --dry-run
    ATL_CONFIG_DIR=$(mktemp -d) ATL_JIRA_URL=https://jira.corp.example ATL_JIRA_TOKEN=x target/release/atl api jira issue get-issue --issue-id-or-key ABC-1 --dry-run

# Build and verify the release archive for this machine (versions and CHANGELOG must match).
release-check VERSION:
    scripts/release.sh check {{VERSION}}

# Test the shell installer against a local release.
installer-test:
    sh tests/installer_regression.sh
    shellcheck -x install.sh tests/installer_regression.sh scripts/release.sh

# Download the latest Atlassian OpenAPI documents.
update-specs:
    scripts/update-specs.sh

# Install atl into ~/.cargo/bin.
install:
    cargo install --path crates/atl

# Python SDK: build the extension with uv (maturin backend) and run pytest.
sdk-python:
    cd crates/atl-py && uv sync -q --python 3.11 && uv run --no-sync pytest -q

# Node/Bun/Deno SDK: build the napi addon and TypeScript, then test on all three runtimes.
sdk-node:
    cd crates/atl-node && bun install --silent && bun run build && bun run test && bun run test:bun && bun run test:deno

# WebAssembly SDK: build with wasm-pack and test in Deno.
sdk-wasm:
    cd crates/atl-wasm && bun install --silent && bun run build && bun run test && bun run test:size

# Every SDK.
sdk-all: sdk-python sdk-node sdk-wasm

# Test the cross-platform PGO build and benchmark helpers.
pgo-test:
    @python3 scripts/test_pgo.py

# Build one native PGO release binary.
pgo-build target="":
    @python3 scripts/build_atl_pgo.py --target-dir target/atl-pgo {{ if target == "" { "" } else { "--target=" + target } }}

# Build matched ordinary and PGO binaries, then compare held-out workloads.
pgo-bench target="":
    @python3 scripts/build_atl_pgo.py --baseline-and-pgo --target-dir target/atl-pgo {{ if target == "" { "" } else { "--target=" + target } }}
    @python3 scripts/benchmark_atl_pgo.py --target-dir target/atl-pgo {{ if target == "" { "" } else { "--target=" + target } }} --json target/atl-pgo/results.json
