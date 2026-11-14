"""Shared deterministic workloads for atl PGO training and measurement."""

from __future__ import annotations

import gzip
import json
import os
import platform
import re
import statistics
import subprocess
import threading
from collections.abc import Iterable
from dataclasses import dataclass
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from typing import Self

REPOSITORY_ROOT = Path(__file__).resolve().parent.parent
SENSITIVE_ENV_MARKERS = ('API_KEY', 'CREDENTIAL', 'PASSWORD', 'SECRET', 'TOKEN')
OK = '"ok":true'

MARKDOWN = """# Release notes

Some **bold** text, a [link](https://example.com), `inline code` and a list:

- one
- two with *emphasis*
  - nested

| Column | Value |
|---|---|
| a | 1 |

```rust
fn main() { println!("hello"); }
```

> A quote.
"""

STORAGE = (
    '<h1>Handbook</h1><p>Welcome to the <strong>handbook</strong>. See '
    '<a href="https://example.com">the site</a>.</p><ul><li>First</li><li>Second</li></ul>'
    '<ac:structured-macro ac:name="code"><ac:parameter ac:name="language">python</ac:parameter>'
    '<ac:plain-text-body><![CDATA[print("hi")]]></ac:plain-text-body></ac:structured-macro>'
    '<table><tbody><tr><th>Key</th><th>Value</th></tr><tr><td>a</td><td>1</td></tr></tbody></table>'
)


def adf(text: str) -> dict[str, object]:
    return {
        'type': 'doc',
        'version': 1,
        'content': [
            {
                'type': 'paragraph',
                'content': [
                    {'type': 'text', 'text': text},
                    {'type': 'text', 'text': ' bold', 'marks': [{'type': 'strong'}]},
                ],
            },
            {
                'type': 'bulletList',
                'content': [
                    {
                        'type': 'listItem',
                        'content': [
                            {
                                'type': 'paragraph',
                                'content': [{'type': 'text', 'text': item}],
                            }
                        ],
                    }
                    for item in ('alpha', 'beta', 'gamma')
                ],
            },
        ],
    }


def issue(number: int) -> dict[str, object]:
    key = f'ABC-{number}'
    return {
        'id': str(10_000 + number),
        'key': key,
        'self': f'https://example.invalid/rest/api/3/issue/{key}',
        'fields': {
            'summary': f'Fixture issue {number}',
            'status': {
                'name': 'In Progress',
                'statusCategory': {'key': 'indeterminate'},
            },
            'issuetype': {'name': 'Bug', 'iconUrl': 'https://example.invalid/icon.png'},
            'assignee': {'displayName': 'Ada Lovelace', 'accountId': 'a1'},
            'reporter': {'displayName': 'Grace Hopper', 'accountId': 'g1'},
            'priority': {'name': 'High'},
            'labels': ['backend', 'pgo'],
            'components': [{'name': 'api'}],
            'created': '2026-01-01T10:00:00.000+0000',
            'updated': '2026-01-02T10:00:00.000+0000',
            'description': adf(f'Description of issue {number}.'),
            'comment': {
                'comments': [
                    {
                        'id': '1',
                        'author': {'displayName': 'Ada'},
                        'created': '2026-01-03',
                        'body': adf('A comment'),
                    }
                ]
            },
            'issuelinks': [],
            'subtasks': [],
        },
    }


class _MockAtlassianHandler(BaseHTTPRequestHandler):
    protocol_version = 'HTTP/1.1'

    def _reply(self) -> None:
        length = int(self.headers.get('content-length', '0'))
        if length:
            self.rfile.read(length)
        path = self.path.split('?', 1)[0]
        body: object = {}
        if path.endswith('/search/jql'):
            body = {'issues': [issue(n) for n in range(1, 26)], 'isLast': True}
        elif match := re.fullmatch(r'/rest/api/3/issue/ABC-(\d+)', path):
            body = issue(int(match.group(1)))
        elif match := re.fullmatch(r'/wiki/api/v2/pages/(\d+)', path):
            body = {
                'id': match.group(1),
                'title': 'Handbook',
                'spaceId': '1',
                'version': {'number': 3},
                'body': {'storage': {'value': STORAGE}},
                '_links': {'webui': '/spaces/DOCS/pages/1'},
            }
        elif path.startswith('/wiki/api/v2/spaces'):
            body = {
                'id': '1',
                'key': 'DOCS',
                'name': 'Docs',
                'results': [{'id': '1', 'key': 'DOCS', 'name': 'Docs'}],
            }
        elif path.startswith('/rest/api/3/myself'):
            body = {'accountId': 'a1', 'displayName': 'Ada Lovelace'}
        encoded = json.dumps(body, separators=(',', ':')).encode()
        self.send_response(200)
        self.send_header('content-type', 'application/json')
        self.send_header('content-length', str(len(encoded)))
        self.end_headers()
        self.wfile.write(encoded)

    do_GET = _reply
    do_POST = _reply

    def log_message(self, format: str, *args: object) -> None:
        del format, args


class MockAtlassian:
    """A loopback-only server that answers like Jira and Confluence Cloud."""

    def __init__(self) -> None:
        self._server = ThreadingHTTPServer(('127.0.0.1', 0), _MockAtlassianHandler)
        self._thread = threading.Thread(target=self._server.serve_forever, daemon=True)

    @property
    def base_url(self) -> str:
        host, port = self._server.server_address[:2]
        return f'http://{host}:{port}'

    def __enter__(self) -> Self:
        self._thread.start()
        return self

    def __exit__(self, *exc_info: object) -> None:
        self._server.shutdown()
        self._server.server_close()
        self._thread.join(timeout=5)


@dataclass(frozen=True, slots=True)
class Workload:
    """One process invocation and its expected output."""

    name: str
    arguments: tuple[str, ...]
    expected_output: str
    repetitions: int = 1
    stdin: str | None = None
    # Talks to the mock server; the benchmark takes fewer samples of these.
    network: bool = False


def prepare_fixture(root: Path, base_url: str) -> tuple[Path, dict[str, str]]:
    """Create an isolated config directory and point Jira and Confluence at the mock."""

    home = root / 'home'
    project = root / 'project'
    home.mkdir(parents=True, exist_ok=True)
    project.mkdir(parents=True, exist_ok=True)
    environment = {
        key: value
        for key, value in os.environ.items()
        if not key.startswith('ATL_')
        and not any(marker in key.upper() for marker in SENSITIVE_ENV_MARKERS)
    }
    environment.update(
        {
            'HOME': str(home),
            'USERPROFILE': str(home),
            'XDG_CONFIG_HOME': str(home / '.config'),
            'APPDATA': str(home / 'AppData' / 'Roaming'),
            'ATL_CONFIG_DIR': str(home / 'atl'),
            'ATL_JIRA_URL': base_url,
            'ATL_JIRA_USER': 'ada@example.com',
            'ATL_JIRA_TOKEN': 'pgo-token',
            'ATL_JIRA_DEPLOYMENT': 'cloud',
            'ATL_CONFLUENCE_URL': base_url,
            'ATL_CONFLUENCE_USER': 'ada@example.com',
            'ATL_CONFLUENCE_TOKEN': 'pgo-token',
            'ATL_CONFLUENCE_DEPLOYMENT': 'cloud',
            'ATL_NO_UPDATE_NOTIFIER': '1',
            'ATL_MAX_RETRIES': '0',
            'NO_COLOR': '1',
            'NO_PROXY': '127.0.0.1,localhost',
            'no_proxy': '127.0.0.1,localhost',
        }
    )
    return project, environment


def training_workloads() -> tuple[Workload, ...]:
    return (
        Workload('version', ('--version',), 'atl', repetitions=6),
        Workload('help', ('--help',), 'Usage:', repetitions=4),
        Workload(
            'issue_list_help',
            ('jira', 'issue', 'list', '--help'),
            'Usage:',
            repetitions=3,
        ),
        Workload(
            'api_help', ('api', 'jira', 'issues', '--help'), 'Usage:', repetitions=3
        ),
        Workload('services', ('services', '--agent'), OK, repetitions=3),
        Workload('schema', ('schema', 'jira.getIssue', '--agent'), OK, repetitions=3),
        Workload('task_list', ('task', 'list', '--agent'), OK, repetitions=2),
        Workload('completions', ('completions', 'zsh'), 'atl', repetitions=2),
        Workload(
            'to_adf', ('convert', '--to', 'adf'), '"doc"', repetitions=4, stdin=MARKDOWN
        ),
        Workload(
            'to_storage',
            ('convert', '--to', 'storage'),
            '<h1>',
            repetitions=3,
            stdin=MARKDOWN,
        ),
        Workload(
            'storage_to_markdown',
            ('convert', '--from', 'storage', '--to', 'markdown'),
            '# Handbook',
            repetitions=3,
            stdin=STORAGE,
        ),
        Workload(
            'issue_list',
            ('jira', 'issue', 'list', '-p', 'ABC', '--agent'),
            OK,
            repetitions=4,
            network=True,
        ),
        Workload(
            'issue_table',
            ('jira', 'issue', 'list', '-p', 'ABC', '--format', 'table'),
            'ABC-1',
            repetitions=3,
            network=True,
        ),
        Workload(
            'issue_view',
            ('jira', 'issue', 'view', 'ABC-1', '--agent'),
            OK,
            repetitions=4,
            network=True,
        ),
        Workload(
            'api_get',
            (
                'api',
                'jira',
                'issues',
                'get-issue',
                '--issue-id-or-key',
                'ABC-2',
                '--agent',
            ),
            OK,
            repetitions=3,
            network=True,
        ),
        Workload(
            'page_get',
            ('confluence', 'page', 'get', '123', '--agent'),
            OK,
            repetitions=3,
            network=True,
        ),
    )


def held_out_workloads() -> tuple[Workload, ...]:
    return (
        Workload('startup_help', ('confluence', 'page', '--help'), 'Usage:'),
        Workload('catalog_schema', ('schema', 'confluence.getPageById', '--agent'), OK),
        Workload(
            'markdown_to_adf',
            ('convert', '--to', 'adf'),
            '"doc"',
            stdin=MARKDOWN.replace('Release', 'Held-out'),
        ),
        Workload(
            'issue_search',
            ('jira', 'issue', 'list', '-p', 'ABC', '-s', 'Done', '--agent'),
            OK,
            network=True,
        ),
        Workload(
            'issue_detail',
            ('jira', 'issue', 'view', 'ABC-7', '--agent'),
            OK,
            network=True,
        ),
    )


def run_workload(
    binary: Path,
    workload: Workload,
    *,
    cwd: Path,
    environment: dict[str, str],
    profile_directory: Path | None = None,
) -> subprocess.CompletedProcess[str]:
    child_environment = environment.copy()
    if profile_directory is not None:
        child_environment['LLVM_PROFILE_FILE'] = str(
            profile_directory / f'atl-{workload.name}-%m-%p.profraw'
        )
    completed = subprocess.run(
        [str(binary), *workload.arguments],
        cwd=cwd,
        env=child_environment,
        input=workload.stdin if workload.stdin is not None else '',
        text=True,
        capture_output=True,
        check=False,
        timeout=30,
    )
    output = completed.stdout + completed.stderr
    if completed.returncode != 0:
        raise RuntimeError(
            f'{workload.name} returned {completed.returncode}: {output.strip()[:2000]}'
        )
    if workload.expected_output not in output:
        raise RuntimeError(
            f'{workload.name} did not contain {workload.expected_output!r}: '
            f'{output.strip()[:2000]}'
        )
    return completed


def executable_size(path: Path) -> tuple[int, int]:
    data = path.read_bytes()
    return len(data), len(gzip.compress(data, compresslevel=9, mtime=0))


def host_description() -> str:
    return f'{platform.system()} {platform.machine()}'


def geometric_mean(values: Iterable[float]) -> float:
    values = tuple(values)
    if not values or any(value <= 0 for value in values):
        raise ValueError('geometric mean needs positive values')
    return statistics.geometric_mean(values)
