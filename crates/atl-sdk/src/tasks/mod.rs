//! The task registry: every task-level Jira, Confluence, Bitbucket and Jira
//! Service Management operation as a named JSON-in, JSON-out function with a
//! JSON Schema for its arguments. Names start with the product (`jira_`,
//! `confluence_`, `bitbucket_`, `jsm_`, `trello_`, `admin_`), the same as
//! `atl <product> ...`.
//!
//! The MCP server lists these as tools, and the Python, Node and WebAssembly
//! SDKs call them through `run`, so all of them share one implementation.

use crate::client::Client;
use crate::error::{Error, Result};
use schemars::JsonSchema;
use serde::Deserialize;
use serde::de::DeserializeOwned;
use serde_json::{Value, json};

#[derive(Clone, Copy)]
pub struct TaskDef {
    pub name: &'static str,
    pub description: &'static str,
    /// True when the task never changes server state.
    pub read_only: bool,
    /// Deletes, merges, declines, suspends or removes access: the CLI asks
    /// first, and the MCP server marks the tool destructive.
    pub destructive: bool,
    /// JSON Schema of the arguments object.
    pub schema: fn() -> Value,
}

impl TaskDef {
    /// `{name, description, read_only, schema}`, as `atl task` and the SDKs list it.
    pub fn to_json(&self) -> Value {
        json!({ "name": self.name, "description": self.description, "read_only": self.read_only, "schema": (self.schema)() })
    }
}

fn schema<T: JsonSchema>() -> Value {
    let mut value = serde_json::to_value(schemars::schema_for!(T)).unwrap_or_default();
    if let Value::Object(map) = &mut value {
        map.remove("$schema");
        map.remove("title");
    }
    value
}

fn default_limit() -> usize {
    50
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct KeyArgs {
    /// Issue key, for example ABC-123.
    key: String,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct Empty {}

fn default_comments() -> usize {
    5
}

fn default_true() -> bool {
    true
}

macro_rules! task {
    ($name:literal, $read_only:expr, $args:ty, $description:literal) => {
        task!(@ $name, $read_only, false, $args, $description)
    };
    ($name:literal, $read_only:expr, $args:ty, $description:literal, destructive) => {
        task!(@ $name, $read_only, true, $args, $description)
    };
    (@ $name:literal, $read_only:expr, $destructive:expr, $args:ty, $description:literal) => {
        TaskDef {
            name: $name,
            description: $description,
            read_only: $read_only,
            destructive: $destructive,
            schema: $crate::tasks::schema::<$args>,
        }
    };
}

mod admin;
mod bitbucket;
mod confluence;
mod jira;
mod jsm;
mod trello;

/// Every task, product by product.
pub fn list() -> &'static [TaskDef] {
    static ALL: std::sync::OnceLock<Vec<TaskDef>> = std::sync::OnceLock::new();
    ALL.get_or_init(|| {
        [
            jira::TASKS,
            confluence::TASKS,
            bitbucket::TASKS,
            jsm::TASKS,
            trello::TASKS,
            admin::TASKS,
        ]
        .concat()
    })
}

/// The registry as JSON for the language SDKs: `[{"name", "description", "read_only", "schema"}]`.
pub fn list_json() -> Value {
    list().iter().map(TaskDef::to_json).collect()
}

pub fn find(name: &str) -> Option<&'static TaskDef> {
    list().iter().find(|task| task.name == name)
}

fn parse<T: DeserializeOwned>(name: &str, args: Value) -> Result<T> {
    let args = if args.is_null() { json!({}) } else { args };
    serde_json::from_value(args).map_err(|err| {
        Error::Invalid(format!(
            "arguments for task `{name}` are invalid: {err}; check the task's input schema"
        ))
    })
}

fn to_json(value: impl serde::Serialize) -> Result<Value> {
    serde_json::to_value(value)
        .map_err(|err| Error::Invalid(format!("cannot encode the result: {err}")))
}

/// Runs one task. `args` is a JSON object matching the task's schema.
pub async fn run(client: &Client, name: &str, args: Value) -> Result<Value> {
    match name.split('_').next() {
        Some("jira") => jira::run(client, name, args).await,
        Some("confluence") => confluence::run(client, name, args).await,
        Some("bitbucket") => bitbucket::run(client, name, args).await,
        Some("jsm") => jsm::run(client, name, args).await,
        Some("trello") => trello::run(client, name, args).await,
        Some("admin") => admin::run(client, name, args).await,
        _ => Err(unknown(name)),
    }
}

fn unknown(name: &str) -> Error {
    Error::Unknown(format!(
        "unknown task `{name}`; available: {}",
        list().iter().map(|t| t.name).collect::<Vec<_>>().join(", ")
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_task_schema_is_a_closed_object_and_names_are_unique() {
        for task in list() {
            let schema = (task.schema)();
            assert_eq!(schema["type"], "object", "{}", task.name);
            assert_eq!(
                schema["additionalProperties"], false,
                "{} rejects unknown arguments",
                task.name
            );
        }
        let mut names: Vec<&str> = list().iter().map(|t| t.name).collect();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), list().len(), "task names are unique");
    }

    #[tokio::test]
    async fn bad_arguments_name_the_task_and_the_field() {
        let client = Client::with_connections([]);
        let err = run(&client, "jira_issue_view", json!({ "keys": "A-1" }))
            .await
            .expect_err("unknown field");
        assert!(
            err.to_string()
                .contains("arguments for task `jira_issue_view` are invalid: unknown field `keys`"),
            "{err}"
        );
        let err = run(&client, "nope", Value::Null)
            .await
            .expect_err("unknown task");
        assert!(matches!(err, Error::Unknown(_)));
        // Every registered name reaches a handler (none is `unknown`): with
        // no credentials each fails on its arguments or on the login.
        for task in list() {
            let err = run(&client, task.name, json!({}))
                .await
                .expect_err("no credentials");
            assert!(
                matches!(err, Error::Invalid(_) | Error::MissingCredentials(_)),
                "{}: {err:?}",
                task.name
            );
        }
    }
}
