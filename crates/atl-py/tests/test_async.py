"""The asyncio client against a local mock server that answers after a delay."""

import asyncio
import json
import threading
import time
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from urllib.parse import parse_qs, urlparse

import atl_sdk
import pytest

DELAY = 0.2


class Slow(BaseHTTPRequestHandler):
    protocol_version = 'HTTP/1.1'

    def log_message(self, *args):
        pass

    def reply(self, status, body):
        data = json.dumps(body).encode()
        self.send_response(status)
        self.send_header('content-type', 'application/json')
        self.send_header('content-length', str(len(data)))
        self.end_headers()
        self.wfile.write(data)

    def do_GET(self):
        url = urlparse(self.path)
        time.sleep(float(parse_qs(url.query).get('wait', [DELAY])[0]))
        if url.path.startswith('/rest/api/3/issue/'):
            key = url.path.rsplit('/', 1)[1]
            self.reply(200, {'key': key, 'fields': {'summary': f'Summary of {key}'}})
        else:
            self.reply(404, {'errorMessages': ['Issue does not exist']})

    def do_POST(self):
        self.rfile.read(int(self.headers['content-length']))
        issue = {
            'key': 'A-1',
            'fields': {
                'summary': 'Fix login',
                'status': {'name': 'To Do', 'statusCategory': {'key': 'new'}},
                'issuetype': {'name': 'Bug'},
            },
        }
        self.reply(200, {'issues': [issue]})


@pytest.fixture(scope='module')
def connections():
    ThreadingHTTPServer.request_queue_size = 128
    server = ThreadingHTTPServer(('127.0.0.1', 0), Slow)
    threading.Thread(target=server.serve_forever, daemon=True).start()
    yield {
        'jira': {
            'deployment': 'cloud',
            'base_url': f'http://127.0.0.1:{server.server_port}',
            'auth': {'kind': 'bearer', 'token': 't'},
        }
    }
    server.shutdown()


def test_gather_runs_calls_at_the_same_time(connections):
    client = atl_sdk.AsyncClient(connections=connections, max_retries=0)
    keys = [f'A-{n}' for n in range(1, 9)]

    async def main():
        started = time.perf_counter()
        issues = await asyncio.gather(
            *(client.call('jira', 'getIssue', {'issueIdOrKey': key}) for key in keys)
        )
        return issues, time.perf_counter() - started

    issues, elapsed = asyncio.run(main())
    assert [issue['key'] for issue in issues] == keys
    # One after another would take 8 x 0.2 s = 1.6 s.
    assert elapsed < 4 * DELAY, f'{elapsed:.2f} s'


def test_a_call_does_not_block_the_event_loop(connections):
    client = atl_sdk.AsyncClient(connections=connections, max_retries=0)

    async def main():
        ticks = 0

        async def tick():
            nonlocal ticks
            while True:
                await asyncio.sleep(0.01)
                ticks += 1

        ticker = asyncio.create_task(tick())
        await client.request('jira', 'GET', '/rest/api/3/issue/A-1')
        ticker.cancel()
        return ticks

    # The loop ran other work while the 0.2 s request waited.
    assert asyncio.run(main()) >= 5


def test_tasks_return_the_same_result_as_the_sync_client(connections):
    sync = atl_sdk.Client(connections=connections, max_retries=0)
    client = atl_sdk.AsyncClient(connections=connections, max_retries=0)
    expected = sync.task('jira_issue_list', project='A')
    assert asyncio.run(client.task('jira_issue_list', project='A')) == expected
    assert expected['issues'][0]['key'] == 'A-1'
    page = asyncio.run(client.jira.issue.list(project='A'))
    assert isinstance(page, atl_sdk.IssueList)
    assert page == sync.jira.issue.list(project='A')


def test_errors_are_atl_errors(connections):
    client = atl_sdk.AsyncClient(connections=connections, max_retries=0)
    with pytest.raises(atl_sdk.AtlError) as api:
        asyncio.run(client.request('jira', 'GET', '/missing', {'wait': 0}))
    assert (api.value.code, api.value.status) == ('api', 404)
    with pytest.raises(atl_sdk.AtlError) as invalid:
        asyncio.run(client.task('jira_issue_view', keys='A-1'))
    assert invalid.value.code == 'invalid'
    assert 'unknown field `keys`' in str(invalid.value)


def test_read_only_refuses_writes(connections):
    client = atl_sdk.AsyncClient(connections=connections, read_only=True)
    with pytest.raises(atl_sdk.AtlError) as refused:
        asyncio.run(client.request('jira', 'POST', '/rest/api/3/issue', body={}))
    assert refused.value.code == 'read_only'


def test_cancel_stops_a_slow_call_without_stray_errors(connections):
    client = atl_sdk.AsyncClient(connections=connections, max_retries=0)
    unhandled = []

    async def main():
        asyncio.get_running_loop().set_exception_handler(
            lambda _loop, context: unhandled.append(context)
        )
        started = time.perf_counter()
        with pytest.raises(TimeoutError):
            await asyncio.wait_for(
                client.request('jira', 'GET', '/rest/api/3/issue/A-1', {'wait': 5}),
                0.1,
            )
        elapsed = time.perf_counter() - started
        # Give any late completion callback time to run.
        await asyncio.sleep(0.3)
        return elapsed

    assert asyncio.run(main()) < 1
    assert unhandled == []
