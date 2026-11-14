# Agents and the MCP server

## The CLI with `--agent`

There is a core skill, `atl`, and one skill for each product: `atl-jira`, `atl-confluence`, `atl-bitbucket`, `atl-jsm`, `atl-trello` and `atl-admin`. They are in [`.agents/skills`](../.agents/skills).

```sh
npx skills add racetozero/atlassian-toolkit     # or bunx; pick the skills and agents
atl skill --install                             # all skills to .claude/skills/
atl skill --install .agents/skills              # or another directory
atl skill --list                                # names and descriptions
atl skill atl-jira                              # print one skill
export ATL_AGENT=1                              # or add --agent to each command
```

With `--agent`:

- stdout is one JSON line, both on success and on failure:

  ```json
  {"ok":true,"command":"jira issue list","data":[...],"meta":{"count":2,"next_page_token":"…","next":"rerun with --page-token …"}}
  {"ok":false,"command":"jira issue view","error":{"code":"api","message":"…","status":404}}
  ```

  The error codes are `auth`, `api`, `invalid`, `unknown`, `read_only`, `confirmation_required`, `input_required` and `internal`.

- A missing or ambiguous value is `input_required` (exit 3), and the error lists the valid values, so the agent can retry once with the correct one:

  ```json
  "input":{"arg":"status","choices":[{"value":"Done","label":"Done (Close)"},…]}
  ```

- Nothing prompts, and the full-screen browser does not open. Destructive commands fail with `confirmation_required` until the agent adds `--yes`.
- `atl` never reads stdin unless the agent asks for it with `--body-file -`, so a command cannot hang on an open pipe.
- Product commands return small flat records. For example, an issue has its key, summary, status, type, assignee, priority, last update and URL. Rich text is Markdown.
- The `atl api` commands remove noise from the JSON: `self` links, `expand`, avatar URLs, `_expandable`, most `_links`, and null fields. ADF becomes Markdown. `--raw` keeps the response as the server sent it.

`atl task run` takes one JSON object, inline, as `@file`, or as `-` for stdin. This means Markdown and field values need no shell quoting:

```sh
atl task list
atl task describe jira_issue_move
atl task run jira_issue_move '{"key":"ABC-1","status":"Done"}'
```

Give agents a read-only login (`atl auth login --read-only`) unless they must write.

## MCP server

`atl mcp` runs an MCP server on stdio with your `atl auth login` credentials:

```sh
claude mcp add atlassian -- atl mcp --read-only
```

```json
{
  "mcpServers": {
    "atlassian": { "command": "atl", "args": ["mcp", "--services", "jira,confluence"] }
  }
}
```

By default the server has two kinds of tools:

- **Task tools:** 136 tools such as `atl_jira_issue_list`, `atl_jira_issue_move` and `atl_confluence_page_get`. They run the same code as the CLI commands. To hide them, add `--no-tasks`.
- **Three operation tools** that reach every API operation: `atlassian_search_operations`, `atlassian_describe_operation` and `atlassian_call_operation`.

`--mode full` gives one tool for each operation instead, which is about 1,900 tools for all services. Use `--services` and `--resources` to select fewer: `--services jira --resources issues,issue-search` gives 31 tools.

Operation results get the same cleanup as `atl api --agent`: no `self` links, avatars, `_expandable` or null fields, and ADF as Markdown.

`--read-only` hides and refuses everything that is not a read. If a result is longer than 100,000 characters, the server cuts it and adds a note that asks for a narrower request.
