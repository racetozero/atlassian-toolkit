# Changelog

## 0.0.1

The first public release.

### Added

- `atl`, one CLI for Jira, Jira Software, Jira Service Management,
  Confluence, Bitbucket, Trello and the Atlassian Admin APIs, on Atlassian
  Cloud and on Data Center. Product commands read `atl <product> <resource>
<verb>`; `atl api` reaches every one of the 3,085 public REST operations,
  generated from Atlassian's OpenAPI documents.
- Jira: issues (list, view, create, edit, delete, move, assign, comment,
  watch, link, clone, worklog, attachments), custom fields by name with
  `--field NAME=VALUE`, JQL check and suggestions, saved filters, boards,
  sprints and epics.
- Confluence: pages as Markdown (get, create, update, edit, delete, move,
  children), footer comments, labels, version history, attachments, spaces
  and search.
- Bitbucket: repositories, branches, pull requests (create, edit, review,
  request changes, decline, merge, reviewers, checkout, diff) and Pipelines
  (list, view, logs, run).
- Jira Service Management desks, queues and customer requests; Trello boards,
  lists and cards; Atlassian Admin organizations, users, groups, domains and
  the audit log.
- A full-screen terminal browser (`atl browse`, or `--browse` on issue
  lists), an agent mode with one JSON envelope per
  command, and an MCP server with search, describe, call and 136 task tools.
- Sign-in with API tokens, scoped tokens, personal access tokens, or OAuth 2.0
  in the browser. Secrets are kept in the config file (mode 0600).
- SDKs for Rust, Python and TypeScript (Node.js, Bun and Deno) on the same
  core.
- Installers for macOS, Linux and Windows, a Homebrew formula, and
  `atl update` with a once-a-day update notice.
- Prompts for missing values at a terminal (project, issue type, summary,
  status, assignee, page space and title, comment text in `$EDITOR`), and
  `--no-input` / `ATL_NO_INPUT` to turn them off. Agents and `--no-input`
  get an `input_required` error that lists the valid values instead.
- A user name that matches several Jira users is no longer resolved to the
  first match; `atl` asks, or lists them. `id:<account>` names a user exactly.
- Piped text is the body of comments and new issues and pages.
- `atl task list|describe|run` runs any task with one JSON argument object.
- Jira: list filters `-w/--watching`, `--history`, `-R/--resolution`,
  `-P/--parent`, `--sprint`, `--created-before`, `--updated-before`;
  `sprint issues [--current|--prev|--next]` and `epic issues` with every
  list filter; create and edit with components, fix and affected versions,
  estimate, reporter, assignee and parent (`-` removes a value on edit),
  `--skip-notify` and `--web`; `move` with `--resolution` and `--assignee`;
  `clone` with priority, assignee, labels and `--replace FIND:REPLACE`;
  `unlink`, `remote-link`, internal comments, and `release list`.
- `--format tsv`, `--no-headers`, `--no-truncate`, and `--columns` (an alias
  of `--select`, which also picks table columns).
- Confluence: `page export` to Markdown folders with front matter and
  attachments, `page push` to publish Markdown files back (with a version
  check), `page get --front-matter`, `page children --recursive` (with Cloud
  folders), `page find`, `page copy-tree`, attachment `--pattern`,
  `--download` and `--replace`, `attachment delete`, comment replies, inline
  comments and `comment delete`. Images now round-trip as Markdown images.
- `atl convert` between Markdown, ADF, Confluence storage and Jira wiki
  markup, offline.
- `atl api` takes `gh api` style `-f`/`-F` fields, `-H` headers, `--input`,
  `-i/--include` and `--silent`; a global `--jq` filters any command's
  output; long views open in a pager with styled Markdown (`--no-pager`).
- Data Center behind gateways: client certificates and private CAs
  (`--client-cert`, `--client-key`, `--ca-cert`, or `ATL_<FAMILY>_CLIENT_CERT`,
  `_CLIENT_KEY`, `_CA_CERT`), session cookie logins (`--method cookie`) and
  `ATL_<FAMILY>_COOKIE`, `.netrc` (`--method netrc`, and a URL without a token),
  and anonymous access (`--method none`).
- `--debug` / `ATL_DEBUG=1` prints each HTTP request, status, duration and
  retry to stderr; `--debug=full` adds headers and bodies with secrets
  masked. `atl auth token [PRODUCT]` prints a stored token for scripts.
- The TypeScript and Python SDKs have a typed method for every task,
  generated from the task schemas (`ATL_UPDATE_SDK_TYPES=1 cargo test -p
atl-sdk --test sdk_types`).
- Bitbucket: `pr comments` (general and inline), `pr checks` (exit 1 when a
  build failed), `pr unapprove`, `pr create --draft`, `pr ready`,
  `branch create|delete`, `commit list`, `pipeline stop`.
- Jira: `comment list|edit|delete`, `issue worklogs`, `issue history`, links
  in `issue view`, several keys in `issue move` and `issue assign`,
  `component list`, `sprint create|start|close`, and `--web` on issue lists.
- Jira Service Management: request approvals (`approve`, `decline`),
  participants, SLAs and attachments. Trello: `card edit` (name,
  description, due date, done, labels, members) and checklists. Atlassian
  Admin: `group add-member|remove-member`. Confluence: blog posts
  (`blog list|get|create`) and `space create`.
- `atl alias set|list|delete` and `atl man --dir DIR`.
- Agent skills in `.agents/skills`: a core `atl` skill and one per product
  (`atl-jira`, `atl-confluence`, `atl-bitbucket`, `atl-jsm`, `atl-trello`,
  `atl-admin`). Install them with `npx skills add racetozero/atlassian-toolkit`
  or `atl skill --install`; `atl skill NAME` prints one and `--list` lists them.
- Help, man pages and completion list commands in alphabetical order.
- `atl mcp` is built on the official MCP Rust SDK (`rmcp`).
- Release binaries are built with profile-guided optimization, the Python
  SDK has a wheel for each CPython version from 3.11 to 3.15 (including the
  free-threaded 3.14t and 3.15t), and the WebAssembly SDK is published to npm
  as `atl-sdk-wasm`.
- Issue lists (`jira issue list`, `sprint issues`, `epic issues`, `filter
run`) print a table in a terminal; `--browse` (`-i`) opens the full-screen
  browser.
