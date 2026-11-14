# Releasing

Pushing a `v*` tag starts `.github/workflows/release.yml`, which uses [cargo-dist](https://github.com/axodotdev/cargo-dist) 0.32. It builds and publishes:

- archives for eight targets, with checksums and build attestations, and the shell and PowerShell installers, on the GitHub release. Every target except Windows on ARM is built with profile-guided optimization (PGO): the build job trains an instrumented binary on the workloads in `scripts/pgo_common.py`, then builds the release with that profile.
- crates.io: `atlassian-toolkit`, `atlassian-toolkit-mcp` and `atl`
- npm: `atlassian-toolkit`, with native add-ons for six platforms, and `atlassian-toolkit-wasm` for browsers, workers and Deno
- PyPI: `atlassian-toolkit` (imported as `atl_sdk`), one wheel per platform and interpreter (CPython 3.11 to 3.15, and the free-threaded 3.14t and 3.15t)
- the Homebrew formula in `racetozero/homebrew-tap`

The agent skills in `.agents/skills` are served from `main`; `npx skills add racetozero/atlassian-toolkit` reads them there.

To check PGO locally, run `just pgo-bench`. It builds an ordinary and a PGO binary and compares them on held-out workloads.

## One-time setup

- The secret `CRATES_IO_TOKEN`.
- The secret `HOMEBREW_TAP_DEPLOY_KEY`: a deploy key with write access to `racetozero/homebrew-tap`.
- npm trusted publishing for `atlassian-toolkit` and `atlassian-toolkit-wasm` from `release.yml`.
- A PyPI trusted publisher for `atlassian-toolkit` from `release.yml` (owner `racetozero`, repository `atlassian-toolkit`, no environment).

## Each release

1. Set the version in `Cargo.toml` (`[workspace.package]`), `crates/atl-py/Cargo.toml`, `crates/atl-wasm/Cargo.toml`, `crates/atl-node/package.json` and `crates/atl-wasm/package.json`. In `CHANGELOG.md`, rename `## Unreleased` to `## <version>`.
2. Run `just release-check <version>`. It builds the archive for this machine and checks it.
3. Run `scripts/release.sh release <version>`. It runs the checks again, then pushes `main` and the tag.
