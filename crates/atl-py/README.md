# atlassian-toolkit (Python SDK)

Python SDK for Jira, Confluence, Bitbucket, Trello and Atlassian Admin, on Cloud and Data Center, built on the `atl` Rust core. Install it with `uv add atlassian-toolkit`, then `import atl_sdk`. Wheels cover CPython 3.11 to 3.15, including the free-threaded 3.14t and 3.15t builds.

```python
import atl_sdk

client = atl_sdk.Client()  # uses `atl auth login`
page = client.jira.issue.list(project='ABC', status=['In Progress'], assignee='me')
for issue in page.issues:  # IssueList and IssueSummary are pydantic models
    print(issue.key, issue.summary)

client.jira.issue.move(key='ABC-12', status='Done', comment='Shipped in **v2**')
print(client.confluence.page.get(page='DOCS:Handbook').markdown)
client.call('jira', 'getIssue', {'issueIdOrKey': 'ABC-12'})  # any generated operation
client.request('jira', 'GET', '/rest/api/3/serverInfo')  # any path
```

Explicit credentials: `atl_sdk.Client(connections={"jira": {"deployment": "cloud", "base_url": "https://acme.atlassian.net", "auth": {"kind": "basic", "username": "me@acme.com", "secret": TOKEN}}})`. `read_only=True` refuses every write.

Every task is a method on its CLI path (`atl jira issue list` is `client.jira.issue.list`), with `TypedDict` arguments and a pydantic result. `atl_sdk.AsyncClient` has the same methods as coroutines, for asyncio:

```python
client = atl_sdk.AsyncClient()
issues = await asyncio.gather(
    *(client.jira.issue.view(key=k) for k in ['ABC-1', 'ABC-2'])
)
```

`atl_sdk.tasks()` lists every task with the JSON Schemas of its arguments and result. Errors raise `atl_sdk.AtlError` with `code`, `status` and `details`. Markup helpers: `markdown_to_adf`, `adf_to_markdown`, `markdown_to_storage`, `storage_to_markdown`, `markdown_to_wiki`, `wiki_to_markdown`.

Develop: `uv sync && uv run pytest -q` (uv builds the extension through maturin). Wheels: `uv build`.
