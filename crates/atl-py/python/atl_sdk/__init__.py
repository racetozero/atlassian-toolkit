"""Python SDK for the Atlassian suite, built on the ``atl`` Rust core.

Jira (platform, Software, Service Management), Confluence, Bitbucket, Trello
and the Atlassian Admin APIs, on Atlassian Cloud and Data Center.

>>> import atl_sdk
>>> client = atl_sdk.Client()                       # credentials from `atl auth login`
>>> client.jira.issue.list(project="ABC", status=["In Progress"]).issues[0].key
>>> client.confluence.page.get(page="DOCS:Handbook").markdown
>>> client.call("jira", "getIssue", params={"issueIdOrKey": "ABC-1"})

Every task in :func:`tasks` is a method on its product, on the same path as
the CLI: ``atl jira issue view`` is ``client.jira.issue.view(key="A-1")``. The
arguments are ``TypedDict``s and the results pydantic models, generated in
``_tasks.py``. :class:`AsyncClient` has the same methods as coroutines.
"""

from __future__ import annotations

import json
from collections.abc import Mapping
from typing import Any, TypedDict

from . import _atl, _tasks
from ._tasks import *
from ._tasks import GeneratedAsyncClient, GeneratedClient

__all__ = [
    'AsyncClient',
    'AtlError',
    'Client',
    'adf_to_markdown',
    'markdown_to_adf',
    'markdown_to_storage',
    'markdown_to_wiki',
    'storage_to_markdown',
    'tasks',
    'wiki_to_markdown',
]
__all__ += _tasks.__all__


class Choice(TypedDict):
    """One valid value of an ``input_required`` error."""

    value: str
    label: str


class InputRequired(TypedDict):
    """The argument an ``input_required`` error asks for, and its choices."""

    arg: str
    choices: list[Choice]


class AtlError(Exception):
    """An error from the SDK.

    ``code`` is an :class:`ErrorCode` value. For ``api`` errors
    ``status`` is the HTTP status and ``details`` the server's JSON error
    body. For ``input_required`` errors ``input`` names the argument and its
    valid values: retry with one of ``input["choices"][i]["value"]``.
    """

    def __init__(
        self,
        code: str,
        message: str,
        status: int | None = None,
        details: Any = None,
        input_required: InputRequired | None = None,
    ):
        super().__init__(message)
        self.code: str = code
        self.status: int | None = status
        self.details: Any = details
        self.input: InputRequired | None = input_required


def _error(err: _atl.NativeError) -> AtlError:
    data = json.loads(str(err))
    return AtlError(
        data['code'],
        data['message'],
        data.get('status'),
        data.get('details'),
        data.get('input'),
    )


def _native(fn, *args):
    try:
        return fn(*args)
    except _atl.NativeError as err:
        raise _error(err) from None


async def _native_async(fn, *args):
    try:
        return await fn(*args)
    except _atl.NativeError as err:
        raise _error(err) from None


def _connect(
    profile: str | None,
    connections: Mapping[str, Any] | None,
    read_only: bool | None,
    max_retries: int | None,
):
    if connections is not None:
        native = _native(_atl.Client.from_connections, json.dumps(connections))
    else:
        native = _native(_atl.Client, profile)
    return native.configured(read_only, max_retries)


def tasks() -> list[dict[str, Any]]:
    """Every task: name, description, read_only, and the JSON Schema of its arguments."""
    return json.loads(_atl.task_list())


class Client(GeneratedClient):
    """A client for every Atlassian product the profile (or ``connections``) covers.

    ``connections`` maps a product family (``jira``, ``confluence``,
    ``bitbucket``, ``trello``, ``admin``, ``admin-scim``) to
    ``{"deployment": "cloud" | "data-center", "base_url": ..., "auth": {...}}``
    where auth is ``{"kind": "basic", "username", "secret"}``,
    ``{"kind": "bearer", "token"}``, ``{"kind": "trello", "key", "token"}``,
    ``{"kind": "cookie", "cookie"}``, ``{"kind": "netrc"}`` or ``{"kind": "none"}``.
    A connection can also name PEM files: ``client_cert``, ``client_key`` and
    ``ca_cert`` (mutual TLS and private CAs).
    """

    def __init__(
        self,
        profile: str | None = None,
        *,
        connections: Mapping[str, Any] | None = None,
        read_only: bool | None = None,
        max_retries: int | None = None,
    ):
        self._native = _connect(profile, connections, read_only, max_retries)
        super().__init__()

    def task(
        self, name: str, args: Mapping[str, Any] | None = None, **kwargs: Any
    ) -> Any:
        """Runs a task such as ``jira_issue_list`` with keyword or dict arguments."""
        merged = {**(args or {}), **kwargs}
        return json.loads(_native(self._native.task, name, json.dumps(merged)))

    def call(
        self,
        service: str,
        operation: str,
        params: Mapping[str, Any] | None = None,
        body: Any = None,
        files: list[str] | None = None,
    ) -> Any:
        """Runs any generated operation, for example ``call("jira", "getIssue", {"issueIdOrKey": "A-1"})``."""
        args = {'params': dict(params or {}), 'body': body, 'files': list(files or [])}
        return json.loads(
            _native(self._native.call, service, operation, json.dumps(args))
        )

    def request(
        self,
        service: str,
        method: str,
        path: str,
        query: Mapping[str, Any] | None = None,
        body: Any = None,
    ) -> Any:
        """Sends a request to any path on a service's server."""
        encoded = None if body is None else json.dumps(body)
        return json.loads(
            _native(
                self._native.request,
                service,
                method,
                path,
                json.dumps(dict(query or {})),
                encoded,
            )
        )


class AsyncClient(GeneratedAsyncClient):
    """The asyncio version of :class:`Client`: the same arguments, and every
    method is a coroutine. Calls run on the SDK's own thread pool, so they do
    not block the event loop, and ``asyncio.gather`` runs them at the same time.

    >>> client = atl_sdk.AsyncClient()
    >>> page = await client.jira.issue.list(project="ABC")
    """

    def __init__(
        self,
        profile: str | None = None,
        *,
        connections: Mapping[str, Any] | None = None,
        read_only: bool | None = None,
        max_retries: int | None = None,
    ):
        self._native = _connect(profile, connections, read_only, max_retries)
        super().__init__()

    async def task(
        self, name: str, args: Mapping[str, Any] | None = None, **kwargs: Any
    ) -> Any:
        """Runs a task such as ``jira_issue_list`` with keyword or dict arguments."""
        merged = {**(args or {}), **kwargs}
        return json.loads(
            await _native_async(self._native.task_async, name, json.dumps(merged))
        )

    async def call(
        self,
        service: str,
        operation: str,
        params: Mapping[str, Any] | None = None,
        body: Any = None,
        files: list[str] | None = None,
    ) -> Any:
        """Runs any generated operation, for example ``await call("jira", "getIssue", {"issueIdOrKey": "A-1"})``."""
        args = {'params': dict(params or {}), 'body': body, 'files': list(files or [])}
        return json.loads(
            await _native_async(
                self._native.call_async, service, operation, json.dumps(args)
            )
        )

    async def request(
        self,
        service: str,
        method: str,
        path: str,
        query: Mapping[str, Any] | None = None,
        body: Any = None,
    ) -> Any:
        """Sends a request to any path on a service's server."""
        encoded = None if body is None else json.dumps(body)
        return json.loads(
            await _native_async(
                self._native.request_async,
                service,
                method,
                path,
                json.dumps(dict(query or {})),
                encoded,
            )
        )


def markdown_to_adf(markdown: str) -> dict[str, Any]:
    """Markdown to Atlassian Document Format (Jira Cloud rich text)."""
    return json.loads(_atl.markdown_to_adf(markdown))


def adf_to_markdown(adf: Mapping[str, Any]) -> str:
    return _native(_atl.adf_to_markdown, json.dumps(adf))


def markdown_to_storage(markdown: str) -> str:
    """Markdown to Confluence storage format."""
    return _atl.markdown_to_storage(markdown)


def storage_to_markdown(storage: str) -> str:
    return _atl.storage_to_markdown(storage)


def markdown_to_wiki(markdown: str) -> str:
    """Markdown to Jira wiki markup (Jira Data Center rich text)."""
    return _atl.markdown_to_wiki(markdown)


def wiki_to_markdown(wiki: str) -> str:
    return _atl.wiki_to_markdown(wiki)
