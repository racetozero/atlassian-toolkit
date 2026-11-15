//! Python bindings for `atlassian-toolkit`.
//!
//! The native module `atl_sdk._atl` is a thin adapter: JSON strings in, JSON
//! strings out, and errors as `AtlError` carrying the SDK's JSON error object.
//! `python/atl_sdk/__init__.py` turns that into an idiomatic Python API. Sync
//! calls release the GIL; the `*_async` methods return asyncio awaitables.
//! Both run on one shared tokio runtime.

use atl_sdk::{CallArgs, HttpMethod, markup, tasks};
use pyo3::exceptions::PyException;
use pyo3::prelude::*;
use pyo3_async_runtimes::tokio::future_into_py;
use serde_json::{Map, Value};
use std::sync::LazyLock;

pyo3::create_exception!(_atl, NativeError, PyException);

static RUNTIME: LazyLock<tokio::runtime::Runtime> = LazyLock::new(|| {
    tokio::runtime::Builder::new_multi_thread().enable_all().build().expect("the tokio runtime starts")
});

fn raise(err: atl_sdk::Error) -> PyErr {
    NativeError::new_err(err.to_json().to_string())
}

fn invalid(message: String) -> PyErr {
    raise(atl_sdk::Error::Invalid(message))
}

fn parse(text: &str, what: &str) -> PyResult<Value> {
    serde_json::from_str(text).map_err(|err| invalid(format!("{what} is not valid JSON: {err}")))
}

#[pyclass(module = "atl_sdk._atl", frozen)]
struct Client {
    inner: atl_sdk::Client,
}

#[pymethods]
impl Client {
    /// Credentials from `atl auth login` and the ATL_* environment variables.
    #[new]
    #[pyo3(signature = (profile = None))]
    fn new(profile: Option<&str>) -> PyResult<Self> {
        Ok(Self { inner: atl_sdk::Client::from_env(profile).map_err(raise)? })
    }

    /// Explicit credentials as JSON: {"jira": {"deployment", "base_url", "auth"}, ...}.
    #[staticmethod]
    fn from_connections(connections: &str) -> PyResult<Self> {
        let value = parse(connections, "connections")?;
        Ok(Self { inner: atl_sdk::Client::from_connections_json(value).map_err(raise)? })
    }

    /// A copy that refuses writes, or retries a different number of times.
    #[pyo3(signature = (read_only = None, max_retries = None))]
    fn configured(&self, read_only: Option<bool>, max_retries: Option<u32>) -> Self {
        let mut inner = self.inner.clone();
        if let Some(read_only) = read_only {
            inner.set_read_only(read_only);
        }
        if let Some(retries) = max_retries {
            inner.set_max_retries(retries);
        }
        Self { inner }
    }

    fn task(&self, py: Python<'_>, name: &str, args: &str) -> PyResult<String> {
        let args = parse(args, "task arguments")?;
        let client = &self.inner;
        py.detach(|| RUNTIME.block_on(tasks::run(client, name, args))).map(|v| v.to_string()).map_err(raise)
    }

    /// `task` as an asyncio awaitable that resolves to the JSON result.
    fn task_async<'py>(&self, py: Python<'py>, name: String, args: &str) -> PyResult<Bound<'py, PyAny>> {
        let args = parse(args, "task arguments")?;
        let client = self.inner.clone();
        future_into_py(py, async move { tasks::run(&client, &name, args).await.map(|v| v.to_string()).map_err(raise) })
    }

    /// Runs a generated operation: args is {"params": {...}, "body": ..., "files": [...]}.
    fn call(&self, py: Python<'_>, service: &str, operation: &str, args: &str) -> PyResult<String> {
        let call = call_args(args)?;
        let client = &self.inner;
        py.detach(|| RUNTIME.block_on(client.call(service, operation, &call))).map(|r| r.value().to_string()).map_err(raise)
    }

    /// `call` as an asyncio awaitable.
    fn call_async<'py>(&self, py: Python<'py>, service: String, operation: String, args: &str) -> PyResult<Bound<'py, PyAny>> {
        let call = call_args(args)?;
        let client = self.inner.clone();
        future_into_py(py, async move { client.call(&service, &operation, &call).await.map(|r| r.value().to_string()).map_err(raise) })
    }

    /// Sends any request to a service's server: query is a JSON object, body optional JSON.
    #[pyo3(signature = (service, method, path, query, body = None))]
    fn request(&self, py: Python<'_>, service: &str, method: &str, path: &str, query: &str, body: Option<&str>) -> PyResult<String> {
        let (method, query, body) = request_args(method, query, body)?;
        let client = &self.inner;
        py.detach(|| RUNTIME.block_on(client.request(service, method, path, &query, body))).map(|r| r.value().to_string()).map_err(raise)
    }

    /// `request` as an asyncio awaitable.
    #[pyo3(signature = (service, method, path, query, body = None))]
    fn request_async<'py>(&self, py: Python<'py>, service: String, method: &str, path: String, query: &str, body: Option<&str>) -> PyResult<Bound<'py, PyAny>> {
        let (method, query, body) = request_args(method, query, body)?;
        let client = self.inner.clone();
        future_into_py(py, async move { client.request(&service, method, &path, &query, body).await.map(|r| r.value().to_string()).map_err(raise) })
    }
}

fn call_args(args: &str) -> PyResult<CallArgs> {
    serde_json::from_str(args).map_err(|err| invalid(format!("call arguments must be an object with params, body and files: {err}")))
}

fn request_args(method: &str, query: &str, body: Option<&str>) -> PyResult<(HttpMethod, Map<String, Value>, Option<Value>)> {
    let method: HttpMethod = method.parse().map_err(raise)?;
    let query = serde_json::from_str(query).map_err(|err| invalid(format!("query must be a JSON object of parameter names to values: {err}")))?;
    let body = body.map(|b| parse(b, "body")).transpose()?;
    Ok((method, query, body))
}

/// The task registry as JSON: [{"name", "description", "read_only", "schema"}].
#[pyfunction]
fn task_list() -> String {
    tasks::list_json().to_string()
}

#[pyfunction]
fn markdown_to_adf(markdown: &str) -> String {
    markup::markdown_to_adf(markdown).to_string()
}

#[pyfunction]
fn adf_to_markdown(adf: &str) -> PyResult<String> {
    Ok(markup::adf_to_markdown(&parse(adf, "ADF")?))
}

#[pyfunction]
fn markdown_to_storage(markdown: &str) -> String {
    markup::markdown_to_storage(markdown)
}

#[pyfunction]
fn storage_to_markdown(storage: &str) -> String {
    markup::storage_to_markdown(storage)
}

#[pyfunction]
fn markdown_to_wiki(markdown: &str) -> String {
    markup::markdown_to_wiki(markdown)
}

#[pyfunction]
fn wiki_to_markdown(wiki: &str) -> String {
    markup::wiki_to_markdown(wiki)
}

#[pymodule]
fn _atl(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // The async methods run on the same runtime as the sync ones. `set` fails
    // only when the module is initialised twice, and then it is already set.
    let _ = pyo3_async_runtimes::tokio::init_with_runtime(&RUNTIME);
    m.add_class::<Client>()?;
    m.add("NativeError", m.py().get_type::<NativeError>())?;
    for function in [
        wrap_pyfunction!(task_list, m)?,
        wrap_pyfunction!(markdown_to_adf, m)?,
        wrap_pyfunction!(adf_to_markdown, m)?,
        wrap_pyfunction!(markdown_to_storage, m)?,
        wrap_pyfunction!(storage_to_markdown, m)?,
        wrap_pyfunction!(markdown_to_wiki, m)?,
        wrap_pyfunction!(wiki_to_markdown, m)?,
    ] {
        m.add_function(function)?;
    }
    Ok(())
}
