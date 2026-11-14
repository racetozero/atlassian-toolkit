---
name: atl
description: Use the `atl` CLI to read and change Atlassian data (Jira, Jira Service Management, Confluence, Bitbucket, Trello, Atlassian Admin) on Cloud and Data Center. Use for any Atlassian task, for raw Atlassian REST calls, and to learn the agent mode, errors and auth that the product skills (atl-jira, atl-confluence, atl-bitbucket, atl-jsm, atl-trello, atl-admin) build on.
---

# atl

Commands read `atl <product> <resource> <verb>`. Always add `--agent` (or set `ATL_AGENT=1`).

## Output

With `--agent`, every command prints one JSON line on stdout and never prompts:

```json
{"ok":true,"command":"jira issue list","data":[...],"meta":{"count":2,"next_page_token":"..."}}
{"ok":false,"command":"jira issue view","error":{"code":"api","message":"...","status":404}}
```

- `error.code` is one of `auth`, `api`, `invalid`, `unknown`, `read_only`, `confirmation_required`, `input_required`, `internal`.
- Exit codes: 0 ok, 1 API error, 2 auth, 3 invalid input, 4 unknown command, 5 other.
- `meta.next_page_token`: run the command again with `--page-token TOKEN` for the next page.
- `--select key,summary` keeps only those fields. `--jq '.[].key'` filters the data. Use both to save tokens.

## Errors

- `input_required`: a value is missing or ambiguous. Run the command again with one of `error.input.choices[].value` as the argument named in `error.input.arg`. For example, `atl jira issue move KEY --agent` without a status lists the statuses.
- `confirmation_required`: the command deletes, merges or suspends something. Run it again with `--yes` only if the user asked for that change.
- `auth`: ask the user to run `atl auth login`, or to set `ATL_<PRODUCT>_URL`, `ATL_<PRODUCT>_USER` and `ATL_<PRODUCT>_TOKEN`.
- `read_only`: the login refuses writes. Do not retry.
- To see the HTTP calls behind a failure, add `--debug` (stderr; `--debug=full` adds headers and bodies with secrets masked).

## Input

- Long or multi-line text: `--body-file FILE`, or `--body-file -` for stdin. Rich text is Markdown.
- `--dry-run` prints the first write as JSON instead of sending it. Reads still run.
- Any task with one JSON object, with no shell quoting: `atl task run jira_issue_create '{"project":"ABC","type":"Task","summary":"S","description":"..."}' --agent`. `atl task list --agent` lists the tasks and `atl task describe NAME --agent` prints the schema.
- Commands that take an issue key use the key in the current git branch name when you leave it out.

## Products

Load the product skill for the commands: `atl-jira`, `atl-confluence`, `atl-bitbucket`, `atl-jsm`, `atl-trello`, `atl-admin`. `atl <product> --help` also lists them.

## Any REST operation

When no product command fits, `atl api` has every public Atlassian REST operation:

- `atl services --agent` lists the services: jira, jira-software, jsm, confluence, confluence-v1, bitbucket, trello, admin, admin-scim.
- `atl api SERVICE --help` lists resources; `atl api SERVICE RESOURCE --help` lists methods.
- `atl schema SERVICE.OPERATION_ID --agent` shows the parameters (with their flags), body and response.
- `atl api SERVICE RESOURCE METHOD --agent --PARAM-FLAG VALUE [--params '{"name":"v"}'] [--json '{...}' | --json @file | --json -]`
- Any path: `atl api jira get /rest/api/3/serverInfo --agent`. `post`, `put`, `patch` and `delete` take `--json`, or gh-style fields: `-f key=string`, `-F key=typed-or-@file`, `fields[project][key]=ABC`.
- `--page-all` follows pagination. `--raw` keeps the response exactly as the server sent it.
