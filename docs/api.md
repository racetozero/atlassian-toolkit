# The `atl api` commands, output, errors and retries

`atl api` has one command for each public REST operation: 3,085 in total, generated from Atlassian's OpenAPI specs.

```sh
atl api jira --help                          # resources
atl api jira issues --help                   # methods
atl api jira issues get-issue --help         # parameters as flags, with the full description
atl api jira issues get-issue --issue-id-or-key ABC-12 --fields summary
atl schema jira.getIssue                     # parameters and request and response schemas
```

| Service         | Covers                                                            | Cloud | Data Center |
| --------------- | ----------------------------------------------------------------- | ----: | ----------: |
| `jira`          | issues, projects, workflows, users, fields, search                |   630 |         391 |
| `jira-software` | boards, sprints, backlog, epics                                   |   105 |          46 |
| `jsm`           | service desks, requests, customers, queues                        |    75 |           — |
| `confluence`    | Cloud REST v2; Data Center REST API                               |   252 |         183 |
| `confluence-v1` | Cloud REST v1: CQL search, content, macros, templates             |   130 |           — |
| `bitbucket`     | repositories, pull requests, pipelines, projects                  |   300 |         574 |
| `trello`        | boards, lists, cards, members                                     |   261 |           — |
| `admin`         | organizations, users, groups, policies, API access, data security |   114 |           — |
| `admin-scim`    | user provisioning (SCIM 2.0)                                      |    24 |           — |

The deployment of your login picks the set of commands. For example, a Data Center Jira login gets the `/rest/api/2` commands, and a Cloud login gets `/rest/api/3`. To see the other set without a login, for `--help` or `--dry-run`, add `--deployment cloud|data-center`.

## Parameters and bodies

- Each path, query and header parameter is a flag (`--issue-id-or-key`). Repeat the flag for arrays. You can also pass `--params '{"issueIdOrKey":"ABC-1"}'` with the API's own names. When both give a value, the flag wins.
- For the body, use `--json '{...}'`, `--json @body.json`, or `--json -` for stdin.
- Multipart operations, such as attachments, take `--upload FILE` (repeatable). `atl` adds the `X-Atlassian-Token: no-check` header.
- `--dry-run` prints the method, URL, headers (with secrets masked) and body, and sends nothing. Product commands still send the reads they need, for example the transitions for `issue move`, and print the first write instead of sending it. Without a login, `atl` cannot send those reads, so it prints the first request against a placeholder URL (`your-domain.atlassian.net`, or `jira.example.com` with `--deployment data-center`).

## Raw requests

To call any path with the service's login:

```sh
atl api jira get /rest/api/3/search/jql -f jql="project = ABC" -F maxResults=5 --jq '.issues[].key'
atl api jira post /rest/api/3/issue -F fields[project][key]=ABC -f fields[summary]="New" -F fields[issuetype][name]=Task
```

The methods are `get`, `post`, `put`, `patch`, `delete`, `head` and `options`. The field flags work like `gh api`:

- `-f key=value` sends a string.
- `-F key=value` converts `true`, `false`, `null`, numbers and `@file`.
- Keys nest: `a[b]=v` and `list[]=v`.

For GET, HEAD, DELETE and OPTIONS, the fields become query parameters. For the other methods, they become a JSON body.

Other flags:

- `--input FILE|-` reads the body.
- `-H "Name: value"` adds a header.
- `-i` prints the status line and the headers.
- `--silent` prints nothing on success.

## Output

`atl api` prints the response JSON. An empty response prints `{"status": 204}`. A body that is not JSON is written as it is, and `-o FILE` saves any body to a file.

`--page-all` follows the API's pagination and prints one JSON line for each page. `--page-limit` (default 10) sets the maximum number of pages, and `--page-delay` (default 100 ms) sets the wait between them.

`--jq EXPR` filters any command's output with jq. Each result prints on its own line, and strings print without quotes. `--select a,b.c` keeps only the fields you name.

## Errors and exit codes

In a terminal, an error is one line on stderr. If there are valid values, `atl` lists them below it. When stderr is not a terminal, the error is JSON:

```json
{
  "error": {
    "code": "api",
    "status": 404,
    "message": "GET https://acme.atlassian.net/rest/api/3/issue/NOPE-1 returned HTTP 404: Issue does not exist or you do not have permission to see it.",
    "details": { "errorMessages": ["…"] }
  }
}
```

| Exit | Meaning                                                     |
| ---: | ----------------------------------------------------------- |
|    0 | success                                                     |
|    1 | the server returned an error                                |
|    2 | no credential, or the server rejected it (401 or 403)       |
|    3 | invalid input: a flag, JSON, a missing parameter, or a body |
|    4 | unknown service, operation or schema                        |
|    5 | network, file system or other failure                       |

## Retries

`atl` retries HTTP 429 for every request. It retries 502, 503, 504 and connection failures only for requests that are safe to send again (GET, HEAD, OPTIONS, PUT and DELETE). A POST that fails is not sent again, so an issue is never created twice.

`atl` uses `Retry-After` when the server sends it, up to 60 seconds. Otherwise it waits 0.5 s, 1 s, then 2 s. `ATL_MAX_RETRIES` changes the number of retries (default 3).

## Debugging

`--debug` (or `ATL_DEBUG=1`) prints each request, its status, its time and each retry on stderr. stdout does not change.

```text
$ atl --debug jira issue list -p ABC
atl: > POST https://acme.atlassian.net/rest/api/3/search/jql
atl: < 200 https://acme.atlassian.net/rest/api/3/search/jql (184 ms)
```

`--debug=full` also prints headers and bodies. It masks `Authorization`, `Cookie` and `Set-Cookie`, and cuts each body at 4,000 characters. The SDKs and the MCP server read `ATL_DEBUG` too.
