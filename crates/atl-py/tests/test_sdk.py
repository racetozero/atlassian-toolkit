"""End-to-end tests of the Python SDK against a local mock Jira and Confluence."""

import json
import threading
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

import atl_sdk
import pytest

REQUESTS = []


class Mock(BaseHTTPRequestHandler):
    def log_message(self, *args):
        pass

    def reply(self, status, body):
        data = json.dumps(body).encode()
        self.send_response(status)
        self.send_header('content-type', 'application/json')
        self.send_header('content-length', str(len(data)))
        self.end_headers()
        self.wfile.write(data)

    def do_POST(self):
        body = json.loads(self.rfile.read(int(self.headers['content-length'])) or b'{}')
        REQUESTS.append(('POST', self.path, body, self.headers.get('authorization')))
        if self.path == '/rest/api/3/search/jql':
            issue = {
                'key': 'A-1',
                'fields': {
                    'summary': 'Fix login',
                    'status': {'name': 'To Do', 'statusCategory': {'key': 'new'}},
                    'issuetype': {'name': 'Bug'},
                },
            }
            self.reply(200, {'issues': [issue], 'nextPageToken': 'p2'})
        else:
            self.reply(404, {'errorMessages': ['no route']})

    def do_GET(self):
        REQUESTS.append(('GET', self.path, None, self.headers.get('authorization')))
        if self.path.startswith('/rest/api/3/issue/A-1'):
            self.reply(200, {'key': 'A-1', 'fields': {'summary': 'Fix login'}})
        elif self.path.startswith('/rest/api/3/serverInfo'):
            self.reply(200, {'version': '1001.0.0'})
        else:
            self.reply(404, {'errorMessages': ['Issue does not exist']})


@pytest.fixture(scope='module')
def client():
    # Threaded with a deep backlog: the thread test sends many requests at once.
    ThreadingHTTPServer.request_queue_size = 128
    server = ThreadingHTTPServer(('127.0.0.1', 0), Mock)
    threading.Thread(target=server.serve_forever, daemon=True).start()
    connection = {
        'deployment': 'cloud',
        'base_url': f'http://127.0.0.1:{server.server_port}',
        'auth': {'kind': 'bearer', 'token': 't'},
    }
    yield atl_sdk.Client(connections={'jira': connection}, max_retries=0)
    server.shutdown()


def test_tasks_return_small_records_and_send_credentials(client):
    page = client.jira.issue.list(project='A', status=['~Done'])
    assert isinstance(page, atl_sdk.IssueList)
    assert page.issues[0].key == 'A-1'
    assert page.issues[0].status_category == 'new'
    assert page.next_page_token == 'p2'
    assert page.jql == 'project = "A" AND status NOT IN ("Done") ORDER BY updated DESC'
    method, path, body, auth = REQUESTS[-1]
    assert (method, path, auth) == ('POST', '/rest/api/3/search/jql', 'Bearer t')
    assert body['jql'] == page.jql


def test_generated_operations_and_raw_requests(client):
    assert (
        client.call('jira', 'getIssue', {'issueIdOrKey': 'A-1'})['fields']['summary']
        == 'Fix login'
    )
    assert (
        client.request('jira', 'GET', '/rest/api/3/serverInfo')['version'] == '1001.0.0'
    )


def test_errors_carry_code_status_and_details(client):
    with pytest.raises(atl_sdk.AtlError) as error:
        client.jira.issue.view(key='NOPE-1')
    assert (error.value.code, error.value.status) == ('api', 404)
    assert error.value.details == {'errorMessages': ['Issue does not exist']}
    with pytest.raises(atl_sdk.AtlError) as error:
        client.jira.issue.view(keys='A-1')  # type: ignore[call-arg]
    assert error.value.code == 'invalid' and 'unknown field `keys`' in str(error.value)
    read_only = atl_sdk.Client(
        connections={
            'jira': {
                'deployment': 'cloud',
                'base_url': 'http://127.0.0.1:9',
                'auth': {'kind': 'bearer', 'token': 't'},
            }
        },
        read_only=True,
    )
    with pytest.raises(atl_sdk.AtlError) as error:
        read_only.jira.issue.comment(key='A-1', body='hi')
    assert error.value.code == 'read_only'


def test_markup_and_task_catalog():
    adf = atl_sdk.markdown_to_adf('**hi**')
    assert atl_sdk.adf_to_markdown(adf) == '**hi**'
    assert atl_sdk.markdown_to_wiki('**hi**') == '*hi*'
    assert atl_sdk.storage_to_markdown(atl_sdk.markdown_to_storage('# T')) == '# T'
    names = {task['name'] for task in atl_sdk.tasks()}
    assert {'jira_issue_list', 'confluence_page_get', 'jira_jql_check'} <= names


def test_every_task_is_a_method_on_its_cli_path(client):
    sync = client
    asynchronous = atl_sdk.AsyncClient(connections={})
    for task in atl_sdk.tasks():
        product, *rest = task['name'].split('_')
        # `jira_me` is `client.jira.me`; `jira_issue_link_types` is `client.jira.issue.link_types`.
        path = rest if len(rest) == 1 else [rest[0], '_'.join(rest[1:])]
        for root in (sync, asynchronous):
            target = getattr(root, product)
            for part in path:
                target = getattr(target, part)
            assert callable(target), task['name']
        assert atl_sdk.TaskName(task['name']) == task['name']
        assert task['description']
        assert task['schema']['type'] == 'object'
        assert task['output_schema']['type'] in ('object', 'array')


def test_namespaced_results_match_the_task_by_name(client):
    typed = client.jira.issue.list(project='A')
    by_name = client.task(atl_sdk.TaskName.JIRA_ISSUE_LIST, {'project': 'A'})
    assert typed.model_dump(mode='json') == by_name


def test_models_keep_fields_a_newer_server_adds():
    issue = atl_sdk.IssueSummary.model_validate(
        {
            'key': 'A-1',
            'summary': 's',
            'status': 'Done',
            'status_category': 'done',
            'type': 'Bug',
            'url': 'u',
            'rank': 7,
        }
    )
    assert issue.model_extra == {'rank': 7}
    assert issue.assignee is None


def test_python_keyword_fields_use_an_alias():
    link = atl_sdk.Linked.model_validate({'from': 'A-1', 'to': 'A-2', 'type': 'Blocks'})
    assert link.from_ == 'A-1'
    assert link.model_dump() == {'from': 'A-1', 'to': 'A-2', 'type': 'Blocks'}


def test_calls_from_many_threads_share_one_client(client):
    """The client is safe to share; on free-threaded CPython the calls run in parallel."""
    from concurrent.futures import ThreadPoolExecutor

    with ThreadPoolExecutor(max_workers=16) as pool:
        pages = list(pool.map(lambda _: client.jira.issue.list(project='A'), range(64)))
    assert {page.issues[0].key for page in pages} == {'A-1'}


@pytest.mark.parametrize(
    'markdown',
    [
        '# Title\n\nA paragraph with **bold**, *italic* and `code`.',
        '- one\n- two\n  - nested',
        '1. first\n2. second',
        '[a link](https://example.com)',
        '```python\nprint("hi")\n```',
        '> a quote',
    ],
)
def test_markdown_survives_adf_and_storage(markdown):
    assert (
        atl_sdk.adf_to_markdown(atl_sdk.markdown_to_adf(markdown)).strip() == markdown
    )
    storage = atl_sdk.markdown_to_storage(markdown)
    assert atl_sdk.storage_to_markdown(storage).strip() == markdown


def test_unknown_task_names_list_the_choices(client):
    with pytest.raises(atl_sdk.AtlError) as error:
        client.task('jira_issue_nope', {})
    assert error.value.code in ('unknown', 'invalid')
    assert 'jira_issue_list' in str(error.value)
