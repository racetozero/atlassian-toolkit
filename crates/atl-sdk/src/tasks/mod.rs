//! The task registry: every task-level Jira, Confluence, Bitbucket and Jira
//! Service Management operation as a named JSON-in, JSON-out function with a
//! JSON Schema for its arguments and one for its result. Names start with the product (`jira_`,
//! `confluence_`, `bitbucket_`, `jsm_`, `trello_`, `admin_`), the same as
//! `atl <product> ...`.
//!
//! The MCP server lists these as tools, and the Python, Node and WebAssembly
//! SDKs call them through `run`, so all of them share one implementation.

use crate::client::Client;
use crate::error::{Error, Result};
use schemars::JsonSchema;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
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
    /// JSON Schema of the result.
    pub output: fn() -> Value,
}

impl TaskDef {
    /// `{name, description, read_only, schema, output_schema}`, as `atl task` and the SDKs list it.
    pub fn to_json(&self) -> Value {
        json!({
            "name": self.name,
            "description": self.description,
            "read_only": self.read_only,
            "schema": (self.schema)(),
            "output_schema": (self.output)(),
        })
    }
}

fn schema<T: JsonSchema>() -> Value {
    let mut value = output_schema::<T>();
    if let Value::Object(map) = &mut value {
        map.remove("title");
    }
    value
}

/// Like [`schema`], but keeps the `title` (the Rust type name), which the SDK
/// generator uses to name result models.
fn output_schema<T: JsonSchema>() -> Value {
    let mut value = serde_json::to_value(schemars::schema_for!(T)).unwrap_or_default();
    if let Value::Object(map) = &mut value {
        map.remove("$schema");
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

/// Registers a task: `task!("jira_issue_view", read_only, Args => Output, "description")`,
/// with `destructive` after the description for tasks the CLI confirms first.
/// The handler returns through [`output`], which checks the `Output` type in
/// debug builds.
macro_rules! task {
    ($name:literal, $read_only:expr, $args:ty => $out:ty, $description:literal) => {
        task!(@ $name, $read_only, false, $args, $out, $description)
    };
    ($name:literal, $read_only:expr, $args:ty => $out:ty, $description:literal, destructive) => {
        task!(@ $name, $read_only, true, $args, $out, $description)
    };
    (@ $name:literal, $read_only:expr, $destructive:expr, $args:ty, $out:ty, $description:literal) => {
        TaskDef {
            name: $name,
            description: $description,
            read_only: $read_only,
            destructive: $destructive,
            schema: $crate::tasks::schema::<$args>,
            output: $crate::tasks::output_schema::<$out>,
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

/// Encodes the result of task `name`. Debug builds (every test) check that
/// `T` has the output schema the task registered, so the published schema
/// cannot drift from what the handler returns.
fn output<T: Serialize + JsonSchema>(name: &str, value: T) -> Result<Value> {
    debug_assert_eq!(
        Some(output_schema::<T>()),
        find(name).map(|task| (task.output)()),
        "task `{name}` returns a type whose schema differs from the output type in its task! registration"
    );
    serde_json::to_value(value)
        .map_err(|err| Error::Invalid(format!("cannot encode the result of `{name}`: {err}")))
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

    #[test]
    fn output_schemas_are_objects_or_arrays_and_type_names_are_unique() {
        let mut seen: std::collections::HashMap<String, (Value, &str)> = Default::default();
        for task in list() {
            let schema = (task.output)();
            assert!(
                matches!(schema["type"].as_str(), Some("object" | "array")),
                "{}: the output must be an object or an array, got {schema}",
                task.name
            );
            let named = schema["$defs"].as_object().into_iter().flatten();
            let top = schema["title"]
                .as_str()
                .filter(|_| schema["type"] == "object")
                .map(|title| (title.to_owned(), schema.clone()));
            for (name, definition) in named.map(|(n, d)| (n.clone(), d.clone())).chain(top) {
                let mut definition = definition;
                if let Value::Object(map) = &mut definition {
                    map.remove("$defs");
                    map.remove("title");
                }
                if let Some((other, first)) = seen.get(&name) {
                    assert_eq!(
                        &definition, other,
                        "type `{name}` in `{}` and `{first}` has two shapes; give one a \
                         #[schemars(rename = \"...\")] so the SDK models get distinct names",
                        task.name
                    );
                } else {
                    seen.insert(name, (definition, task.name));
                }
            }
        }
    }

    #[test]
    #[cfg(debug_assertions)]
    #[should_panic(expected = "returns a type whose schema differs")]
    fn output_rejects_a_type_other_than_the_registered_one() {
        let _ = output("jira_issue_count", Vec::<String>::new());
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
