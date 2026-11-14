# atl

[![CI](https://github.com/racetozero/atlassian-toolkit/actions/workflows/ci.yml/badge.svg)](https://github.com/racetozero/atlassian-toolkit/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/racetozero/atlassian-toolkit?sort=semver)](https://github.com/racetozero/atlassian-toolkit/releases)
[![Rust](https://img.shields.io/badge/Rust-stable-orange?logo=rust)](https://github.com/racetozero/atlassian-toolkit/blob/main/rust-toolchain.toml)
[![Downloads](https://img.shields.io/github/downloads/racetozero/atlassian-toolkit/total)](https://github.com/racetozero/atlassian-toolkit/releases)
[![crates.io](https://img.shields.io/crates/v/atl?label=crates.io)](https://crates.io/crates/atl)
[![npm SDK](https://img.shields.io/npm/v/atl-sdk?label=npm%20SDK)](https://www.npmjs.com/package/atl-sdk)
[![npm WASM](https://img.shields.io/npm/v/atl-sdk-wasm?label=npm%20WASM)](https://www.npmjs.com/package/atl-sdk-wasm)
[![PyPI](https://img.shields.io/pypi/v/atlassian-toolkit?label=PyPI)](https://pypi.org/project/atlassian-toolkit/)
[![Python](https://img.shields.io/pypi/pyversions/atlassian-toolkit)](https://pypi.org/project/atlassian-toolkit/)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue)](LICENSE)

**Use Jira, Confluence and Bitbucket from your terminal, or let your AI agent use them for you.**

`atl` is one command-line tool for Atlassian, and it is agent ready from the first command. Claude Code, Codex, Cursor and other agents can find issues, update pages and review pull requests with the same commands you type, through a JSON mode that never stops to ask a question, a built-in MCP server, and installable skills.

It works with Jira, Confluence, Bitbucket, Jira Service Management, Trello and Atlassian Admin, on Atlassian Cloud and on self-hosted Data Center. The commands are the same on both.

```sh
atl jira issue list -a me -s ~Done
```

That command lists your open issues. Most of what you do in `atl` reads like that.

## Why atl

- **Agent ready.** Add `--agent` and every command prints one JSON line. Nothing prompts. When a value is missing, the error lists the valid choices, so the agent gets it right on the next try. Destructive commands wait for `--yes`.
- **MCP built in.** `atl mcp` serves 136 Jira, Confluence, Bitbucket, JSM, Trello and Admin tools to any MCP client, with no extra server to install.
- **Skills for your agent.** One core skill and one per product teach the commands. Install them with `npx skills add racetozero/atlassian-toolkit` or `atl skill --install`.
- **Safe by default for automation.** A read-only login blocks every write in the CLI, the MCP server and the SDKs alike.
- **Cloud and Data Center.** One tool and one set of commands for both, so a migration does not change your scripts.
- **Every API, not only the common ones.** `atl api` reaches all 3,085 public Atlassian REST operations with your normal login.
- **Confluence in Markdown.** Pull pages to Markdown, edit them in your editor, keep them in git, and push them back.
- **Knows your branch.** On `feature/ABC-123-login-fix`, `atl jira issue view` opens `ABC-123` without the key.
- **Output for the reader.** Tables in a terminal, JSON in a pipe, plus CSV, `--select` and `--jq` when you need them.
- **A full-screen browser.** `atl browse` lets you move through issues, filters and spaces with the arrow keys.
- **Every sign-in your company uses.** API tokens, scoped tokens, OAuth, personal access tokens, SSO cookies and client certificates.
- **Build on it.** The same tasks are in SDKs for Python 3.11+, TypeScript on Node, Bun and Deno, browser WebAssembly, and Rust.
- **One fast file.** A single native binary with nothing else to install. `atl update` keeps it current.

## Who it's for

- **Developers.** You can update a ticket, open a pull request and check a build without leaving your editor. If your branch is named `feature/ABC-123-login-fix`, `atl` knows you're working on `ABC-123`, so you don't type the key.
- **People who write docs.** Write Confluence pages in Markdown, in your own editor and in git. `atl` converts them to Confluence format and back. You can export a whole page tree to a folder and publish it again later.
- **Support and service desk teams.** Read the queue, reply to customers, approve requests and see how much SLA time is left.
- **Admins and automation.** Every command prints JSON when its output goes to a script. There are SDKs for Python, TypeScript and Rust, and a mode made for AI coding agents.

## A few examples

**Start your day.** See what's assigned to you, then open the issue for the branch you're on:

```sh
atl jira issue list -a me -s ~Done
atl jira issue view
```

**Move work along.** Change the status, leave a comment, or move several issues at once:

```sh
atl jira issue move ABC-12 Done --comment "Fixed in v2.3"
atl jira issue move ABC-12 ABC-13 ABC-14 "In Review"
```

If you leave out the status, `atl` lists the ones you can choose. When `atl` needs a value you didn't give, it shows the options and lets you pick one.

**Report a bug from a file you already wrote:**

```sh
atl jira issue create -p ABC -t Bug -s "Login fails on Safari" --body-file bug.md
```

**Keep docs in Markdown.** Pull a page down, edit it, and push it back. `atl` updates the version number for you.

```sh
atl confluence page get DOCS:Handbook > handbook.md
atl confluence page update DOCS:Handbook --file handbook.md
```

To keep a whole section of your wiki in a git repository:

```sh
atl confluence page export DOCS:Handbook -r --dest docs/
atl confluence page push docs/
```

**Ship code.** Open a pull request from your current branch, check its builds, and merge:

```sh
atl bitbucket pr create -t "Fix login on Safari"
atl bitbucket pr checks 42
atl bitbucket pr merge 42 --strategy squash
```

**Handle the service desk:**

```sh
atl jsm request list --desk HELP
atl jsm request approve HELP-7
```

**Look around.** `atl browse` opens a full-screen view of your issues, filters and Confluence spaces in the terminal. You can move through issues with the arrow keys, preview them, and change their status.

To see every command, read [the command guide](docs/commands.md) or add `--help` to any command.

## When there's no command for it

The everyday commands cover common work. For everything else, `atl api` gives you every public Atlassian REST API: 3,085 operations, built from Atlassian's own API specs. You use your normal login, and you don't write any HTTP code.

```sh
atl api jira issues get-issue --issue-id-or-key ABC-12
atl api jira get /rest/api/3/serverInfo
```

[More about `atl api`](docs/api.md).

## Install

On macOS or Linux:

```sh
curl -LsSf https://raw.githubusercontent.com/racetozero/atlassian-toolkit/main/install.sh | sh
```

On Windows:

```powershell
powershell -ExecutionPolicy ByPass -c "irm https://raw.githubusercontent.com/racetozero/atlassian-toolkit/main/install.ps1 | iex"
```

Or use Homebrew (`brew install racetozero/tap/atl`) or Cargo (`cargo install atl`).

`atl` is a single file with nothing else to install. When a new version is out, `atl` prints a short notice after a command, and `atl update` installs it.

## Sign in

```sh
atl auth login
```

A short form asks which product you use, its address, and your token. `atl` checks the token with the server before it saves it, then keeps it in a config file in `~/.config/atl` that only you can read. There is no keychain dialog, so agents and scripts never stop to wait for one.

`atl` supports the ways companies usually sign in to Atlassian:

- **Cloud:** API tokens, scoped tokens and single sign-on through OAuth.
- **Data Center:** personal access tokens, passwords, SSO session cookies and client certificates.

In CI, you can set environment variables instead of signing in. [Sign-in details](docs/auth.md).

## For scripts and AI agents

`atl` changes its output for the reader. In a terminal you get tables and readable text. In a pipe you get JSON. With `--agent`, every command prints one short JSON line, and `atl` never stops to ask a question. This makes it a good tool for AI coding agents such as Claude Code and Codex.

If an agent leaves out a value, the error lists the values it can use, so the agent can retry once and succeed.

To keep an agent from changing anything, give it a read-only login (`atl auth login --read-only`). The check is in `atl`'s core, so the same rule covers the CLI, the MCP server and the SDKs.

Agent skills teach your agent the commands. There is one core skill and one for each product. Install them with the [skills](https://github.com/vercel-labs/skills) CLI, which works with Claude Code, Codex, Cursor and other agents, or with `atl` itself:

```sh
npx skills add racetozero/atlassian-toolkit     # or: bunx skills add racetozero/atlassian-toolkit
atl skill --install                             # writes them to .claude/skills/
```

`atl mcp` also runs an MCP server for tools that use the Model Context Protocol. [More for agents](docs/agent-mode.md).

## SDKs

The CLI is built on a library, and you can use the same library in your own code. It is available for Python, TypeScript (Node.js, Bun, Deno and browsers) and Rust. A task such as "list issues" or "get a page" works the same way in every language and in the CLI.

```python
import atl_sdk

issues = atl_sdk.Client().jira_issue_list(project='ABC')['issues']
```

```ts
import { Client } from "atl-sdk"

const { issues } = await Client.fromEnv().jiraIssueList({ project: "ABC" })
```

[SDK guide](docs/sdk.md).

## Learn more

- [Commands](docs/commands.md): every command, with examples
- [Signing in](docs/auth.md): tokens, OAuth, SSO and the config file
- [The `atl api` commands](docs/api.md): output formats, errors, retries and debugging
- [Agents and MCP](docs/agent-mode.md)
- [SDKs](docs/sdk.md)
- [Contributing](CONTRIBUTING.md)

## License

MIT
