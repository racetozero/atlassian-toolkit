//! Node-API bindings for `atlassian-toolkit`, loaded by Node.js, Bun and Deno.
//!
//! JSON strings cross the boundary; `ts/index.ts` adds types and turns the
//! SDK's JSON error objects into `AtlError`. Async methods run on napi's
//! tokio runtime and return Promises.

use atl_sdk::{CallArgs, HttpMethod, markup, tasks};
use napi::bindgen_prelude::*;
use napi_derive::napi;
use serde_json::{Map, Value};

fn raise(err: atl_sdk::Error) -> Error {
    Error::from_reason(err.to_json().to_string())
}

fn invalid(message: String) -> Error {
    raise(atl_sdk::Error::Invalid(message))
}

fn parse(text: &str, what: &str) -> Result<Value> {
    serde_json::from_str(text).map_err(|err| invalid(format!("{what} is not valid JSON: {err}")))
}

#[napi]
pub struct NativeClient {
    inner: atl_sdk::Client,
}

#[napi]
impl NativeClient {
    /// Credentials from `atl auth login` and the ATL_* environment variables.
    #[napi(factory)]
    pub fn from_env(profile: Option<String>) -> Result<Self> {
        Ok(Self { inner: atl_sdk::Client::from_env(profile.as_deref()).map_err(raise)? })
    }

    /// Explicit credentials as JSON: {"jira": {"deployment", "base_url", "auth"}, ...}.
    #[napi(factory)]
    pub fn from_connections(connections: String) -> Result<Self> {
        Ok(Self { inner: atl_sdk::Client::from_connections_json(parse(&connections, "connections")?).map_err(raise)? })
    }

    #[napi]
    pub fn configured(&self, read_only: Option<bool>, max_retries: Option<u32>) -> NativeClient {
        let mut inner = self.inner.clone();
        if let Some(read_only) = read_only {
            inner.set_read_only(read_only);
        }
        if let Some(retries) = max_retries {
            inner.set_max_retries(retries);
        }
        NativeClient { inner }
    }

    #[napi]
    pub async fn task(&self, name: String, args: String) -> Result<String> {
        let args = parse(&args, "task arguments")?;
        tasks::run(&self.inner, &name, args).await.map(|v| v.to_string()).map_err(raise)
    }

    /// Runs a generated operation: args is {"params", "body", "files"}.
    #[napi]
    pub async fn call(&self, service: String, operation: String, args: String) -> Result<String> {
        let call: CallArgs = serde_json::from_str(&args).map_err(|err| invalid(format!("call arguments must be an object with params, body and files: {err}")))?;
        Ok(self.inner.call(&service, &operation, &call).await.map_err(raise)?.value().to_string())
    }

    #[napi]
    pub async fn request(&self, service: String, method: String, path: String, query: String, body: Option<String>) -> Result<String> {
        let method: HttpMethod = method.parse().map_err(raise)?;
        let query: Map<String, Value> = serde_json::from_str(&query).map_err(|err| invalid(format!("query must be a JSON object of parameter names to values: {err}")))?;
        let body = body.as_deref().map(|b| parse(b, "body")).transpose()?;
        Ok(self.inner.request(&service, method, &path, &query, body).await.map_err(raise)?.value().to_string())
    }
}

#[napi]
pub fn task_list() -> String {
    tasks::list_json().to_string()
}

#[napi]
pub fn markdown_to_adf(markdown: String) -> String {
    markup::markdown_to_adf(&markdown).to_string()
}

#[napi]
pub fn adf_to_markdown(adf: String) -> Result<String> {
    Ok(markup::adf_to_markdown(&parse(&adf, "ADF")?))
}

#[napi]
pub fn markdown_to_storage(markdown: String) -> String {
    markup::markdown_to_storage(&markdown)
}

#[napi]
pub fn storage_to_markdown(storage: String) -> String {
    markup::storage_to_markdown(&storage)
}

#[napi]
pub fn markdown_to_wiki(markdown: String) -> String {
    markup::markdown_to_wiki(&markdown)
}

#[napi]
pub fn wiki_to_markdown(wiki: String) -> String {
    markup::wiki_to_markdown(&wiki)
}
