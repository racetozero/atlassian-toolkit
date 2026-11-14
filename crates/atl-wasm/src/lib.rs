//! WebAssembly bindings for `atlassian-toolkit`: the same tasks, generated operations
//! and Markdown converters as the native SDKs, running on the platform's
//! `fetch`. Values cross as plain JavaScript objects; failures reject with an
//! `Error` whose message is the SDK's JSON error object, which `js/index.ts`
//! turns into `AtlError`.

use atl_sdk::{CallArgs, HttpMethod, markup, tasks};
use js_sys::Promise;
use serde_json::{Map, Value};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::future_to_promise;

fn to_js(value: &Value) -> JsValue {
    value.serialize(&serde_wasm_bindgen::Serializer::json_compatible()).unwrap_or(JsValue::NULL)
}

use serde::Serialize as _;

fn from_js(value: JsValue, what: &str) -> Result<Value, JsValue> {
    if value.is_undefined() || value.is_null() {
        return Ok(Value::Null);
    }
    serde_wasm_bindgen::from_value(value).map_err(|err| reject(atl_sdk::Error::Invalid(format!("{what} cannot be read as JSON: {err}"))))
}

fn reject(err: atl_sdk::Error) -> JsValue {
    js_sys::Error::new(&err.to_json().to_string()).into()
}

#[wasm_bindgen]
pub struct Client {
    inner: atl_sdk::Client,
}

#[wasm_bindgen]
impl Client {
    /// Explicit credentials: {jira: {deployment, base_url, auth: {kind: "basic", username, secret}}, ...}.
    #[wasm_bindgen(js_name = withConnections)]
    pub fn with_connections(connections: JsValue) -> Result<Client, JsValue> {
        let value = from_js(connections, "connections")?;
        Ok(Client { inner: atl_sdk::Client::from_connections_json(value).map_err(reject)? })
    }

    pub fn configured(&self, read_only: Option<bool>, max_retries: Option<u32>) -> Client {
        let mut inner = self.inner.clone();
        if let Some(read_only) = read_only {
            inner.set_read_only(read_only);
        }
        if let Some(retries) = max_retries {
            inner.set_max_retries(retries);
        }
        Client { inner }
    }

    pub fn task(&self, name: String, args: JsValue) -> Result<Promise, JsValue> {
        let args = from_js(args, "task arguments")?;
        let client = self.inner.clone();
        Ok(future_to_promise(async move { tasks::run(&client, &name, args).await.map(|v| to_js(&v)).map_err(reject) }))
    }

    /// Runs a generated operation: args is {params, body}.
    pub fn call(&self, service: String, operation: String, args: JsValue) -> Result<Promise, JsValue> {
        let call: CallArgs = match from_js(args, "call arguments")? {
            Value::Null => CallArgs::default(),
            value => serde_json::from_value(value).map_err(|err| reject(atl_sdk::Error::Invalid(format!("call arguments must be an object with params and body: {err}"))))?,
        };
        let client = self.inner.clone();
        Ok(future_to_promise(async move { Ok(to_js(&client.call(&service, &operation, &call).await.map_err(reject)?.value())) }))
    }

    pub fn request(&self, service: String, method: String, path: String, query: JsValue, body: JsValue) -> Result<Promise, JsValue> {
        let method: HttpMethod = method.parse().map_err(reject)?;
        let query: Map<String, Value> = match from_js(query, "query")? {
            Value::Null => Map::new(),
            value => serde_json::from_value(value).map_err(|err| reject(atl_sdk::Error::Invalid(format!("query must be an object of parameter names to values: {err}"))))?,
        };
        let body = Some(from_js(body, "body")?).filter(|b| !b.is_null());
        let client = self.inner.clone();
        Ok(future_to_promise(async move { Ok(to_js(&client.request(&service, method, &path, &query, body).await.map_err(reject)?.value())) }))
    }
}

#[wasm_bindgen(js_name = taskList)]
pub fn task_list() -> JsValue {
    to_js(&tasks::list_json())
}

#[wasm_bindgen(js_name = markdownToAdf)]
pub fn markdown_to_adf(markdown: &str) -> JsValue {
    to_js(&markup::markdown_to_adf(markdown))
}

#[wasm_bindgen(js_name = adfToMarkdown)]
pub fn adf_to_markdown(adf: JsValue) -> Result<String, JsValue> {
    Ok(markup::adf_to_markdown(&from_js(adf, "ADF")?))
}

#[wasm_bindgen(js_name = markdownToStorage)]
pub fn markdown_to_storage(markdown: &str) -> String {
    markup::markdown_to_storage(markdown)
}

#[wasm_bindgen(js_name = storageToMarkdown)]
pub fn storage_to_markdown(storage: &str) -> String {
    markup::storage_to_markdown(storage)
}

#[wasm_bindgen(js_name = markdownToWiki)]
pub fn markdown_to_wiki(markdown: &str) -> String {
    markup::markdown_to_wiki(markdown)
}

#[wasm_bindgen(js_name = wikiToMarkdown)]
pub fn wiki_to_markdown(wiki: &str) -> String {
    markup::wiki_to_markdown(wiki)
}
