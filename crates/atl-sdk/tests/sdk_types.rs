//! Generates the typed task helpers of the TypeScript and Python SDKs from the
//! task registry, and fails when the checked-in files are out of date.
//! `ATL_UPDATE_SDK_TYPES=1 cargo test -p atl-sdk --test sdk_types` rewrites them.

use serde_json::Value;
use std::path::PathBuf;

const HEADER: &str = "Generated from the task registry (crates/atl-sdk/src/tasks.rs) by\ncrates/atl-sdk/tests/sdk_types.rs. Do not edit; run\n`ATL_UPDATE_SDK_TYPES=1 cargo test -p atl-sdk --test sdk_types`.";

fn pascal(name: &str) -> String {
    name.split('_')
        .map(|part| {
            let mut chars = part.chars();
            chars.next().map_or(String::new(), |first| {
                first.to_ascii_uppercase().to_string() + chars.as_str()
            })
        })
        .collect()
}

fn camel(name: &str) -> String {
    let pascal = pascal(name);
    let mut chars = pascal.chars();
    chars.next().map_or(String::new(), |first| {
        first.to_ascii_lowercase().to_string() + chars.as_str()
    })
}

/// (type, nullable) of a property schema.
fn base_types(schema: &Value) -> (Vec<String>, bool) {
    match &schema["type"] {
        Value::String(kind) => (vec![kind.clone()], false),
        Value::Array(kinds) => {
            let kinds: Vec<String> = kinds
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect();
            let nullable = kinds.iter().any(|k| k == "null");
            (
                kinds.into_iter().filter(|k| k != "null").collect(),
                nullable,
            )
        }
        _ => (Vec::new(), false),
    }
}

/// Maps one schema to a type with `lang`'s names; `at` names the place for
/// the panic message when a shape is not supported.
fn type_of(schema: &Value, at: &str, ts: bool) -> String {
    if let Some(values) = schema["enum"].as_array() {
        let literals: Vec<String> = values
            .iter()
            .map(|v| match v.as_str() {
                Some(text) => format!("\"{text}\""),
                None => panic!("{at}: only string enums are supported, got {v}"),
            })
            .collect();
        return match ts {
            true => literals.join(" | "),
            false => format!("Literal[{}]", literals.join(", ")),
        };
    }
    let (kinds, nullable) = base_types(schema);
    let [kind] = kinds.as_slice() else {
        panic!(
            "{at}: unsupported schema {schema}; teach crates/atl-sdk/tests/sdk_types.rs this shape"
        );
    };
    let inner = match (kind.as_str(), ts) {
        ("string", true) => "string".to_owned(),
        ("string", false) => "str".to_owned(),
        ("integer" | "number", true) => "number".to_owned(),
        ("integer", false) => "int".to_owned(),
        ("number", false) => "float".to_owned(),
        ("boolean", true) => "boolean".to_owned(),
        ("boolean", false) => "bool".to_owned(),
        ("array", _) => {
            let item = type_of(&schema["items"], &format!("{at}[]"), ts);
            match ts {
                true if item.contains(' ') => format!("({item})[]"),
                true => format!("{item}[]"),
                false => format!("list[{item}]"),
            }
        }
        ("object", true) if schema["additionalProperties"] != Value::Bool(false) => {
            "Record<string, unknown>".to_owned()
        }
        ("object", false) if schema["additionalProperties"] != Value::Bool(false) => {
            "dict[str, Any]".to_owned()
        }
        _ => panic!(
            "{at}: unsupported schema {schema}; teach crates/atl-sdk/tests/sdk_types.rs this shape"
        ),
    };
    match (nullable, ts) {
        (false, _) => inner,
        (true, true) => format!("{inner} | null"),
        (true, false) => format!("{inner} | None"),
    }
}

struct Prop {
    name: String,
    description: Option<String>,
    required: bool,
    schema: Value,
}

fn props(schema: &Value) -> Vec<Prop> {
    let required: Vec<&str> = schema["required"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .collect();
    schema["properties"]
        .as_object()
        .into_iter()
        .flatten()
        .map(|(name, prop)| Prop {
            name: name.clone(),
            description: prop["description"].as_str().map(str::to_owned),
            required: required.contains(&name.as_str()),
            schema: prop.clone(),
        })
        .collect()
}

fn jsdoc(text: &str, indent: &str) -> String {
    let text = text.replace("*/", "*\\/");
    let lines: Vec<&str> = text.lines().collect();
    match lines.as_slice() {
        [one] => format!("{indent}/** {one} */\n"),
        many => {
            let mut out = format!("{indent}/**\n");
            for line in many {
                out.push_str(&format!("{indent} * {line}\n").replace(" \n", "\n"));
            }
            out.push_str(&format!("{indent} */\n"));
            out
        }
    }
}

fn typescript(tasks: &[(String, String, Value)]) -> String {
    let mut out = String::new();
    for line in HEADER.lines() {
        out.push_str(&format!("// {line}\n"));
    }
    out.push_str("import type { TaskResults } from \"./common.ts\"\n\n");
    out.push_str("/** The result type of a task: from `TaskResults` when known, else `unknown`. */\nexport type TaskResult<N extends string> = N extends keyof TaskResults ? TaskResults[N] : unknown\n\n");
    for (name, description, schema) in tasks {
        out.push_str(&jsdoc(&format!("Arguments of `{name}`: {description}"), ""));
        out.push_str(&format!("export interface {}Args {{\n", pascal(name)));
        for prop in props(schema) {
            if let Some(description) = &prop.description {
                out.push_str(&jsdoc(description, "  "));
            }
            let key = match prop
                .name
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_')
            {
                true => prop.name.clone(),
                false => format!("\"{}\"", prop.name),
            };
            let optional = if prop.required { "" } else { "?" };
            let kind = type_of(&prop.schema, &format!("{name}.{}", prop.name), true);
            out.push_str(&format!("  {key}{optional}: {kind}\n"));
        }
        out.push_str("}\n\n");
    }
    out.push_str(
        "/** The arguments of every task, by task name. */\nexport interface TaskArgs {\n",
    );
    for (name, _, _) in tasks {
        out.push_str(&format!("  {name}: {}Args\n", pascal(name)));
    }
    out.push_str("}\nexport type TaskName = keyof TaskArgs\n\n");
    let codes: Vec<String> = atl_sdk::error::CODES
        .iter()
        .map(|c| format!("\"{c}\""))
        .collect();
    out.push_str(&format!(
        "/** Every value of `AtlError.code`. */\nexport type ErrorCode = {}\n\n",
        codes.join(" | ")
    ));
    out.push_str("/** One typed method per task; each SDK supplies `task()` over its own transport. */\nexport abstract class GeneratedTasks {\n");
    out.push_str("  /** Runs any task from `tasks()`, for example `task(\"jira_issue_view\", { key: \"A-1\" })`. */\n");
    out.push_str(
        "  abstract task<T = unknown>(name: string, args?: Record<string, unknown>): Promise<T>\n",
    );
    for (name, description, schema) in tasks {
        out.push('\n');
        out.push_str(&jsdoc(description, "  "));
        let default = match props(schema).iter().any(|p| p.required) {
            true => "",
            false => " = {}",
        };
        out.push_str(&format!(
            "  {}(args: {}Args{default}): Promise<TaskResult<\"{name}\">> {{\n    return this.task(\"{name}\", {{ ...args }})\n  }}\n",
            camel(name),
            pascal(name),
        ));
    }
    out.push_str("}\n");
    out
}

fn python(tasks: &[(String, String, Value)]) -> String {
    let doc = |text: &str| text.replace('\\', "\\\\").replace("\"\"\"", "\\\"\\\"\\\"");
    // `__all__` sorted, as ruff's RUF022 expects.
    let mut exported: Vec<String> = [
        "ErrorCode".to_owned(),
        "GeneratedTasks".to_owned(),
        "TaskName".to_owned(),
    ]
    .into_iter()
    .chain(
        tasks
            .iter()
            .map(|(name, _, _)| format!("{}Args", pascal(name))),
    )
    .collect();
    exported.sort();
    let exported: Vec<String> = exported.iter().map(|name| format!("\"{name}\"")).collect();
    let mut out = format!(
        "__all__ = [{}]\n\n\nclass ErrorCode(StrEnum):\n    \"\"\"Every value of ``AtlError.code``.\"\"\"\n\n{}\n\nclass TaskName(StrEnum):\n    \"\"\"Every task name.\"\"\"\n\n",
        exported.join(", "),
        atl_sdk::error::CODES
            .iter()
            .map(|c| format!("    {} = \"{c}\"", c.to_ascii_uppercase()))
            .collect::<Vec<_>>()
            .join("\n")
    );
    for (name, _, _) in tasks {
        out.push_str(&format!("    {} = \"{name}\"\n", name.to_ascii_uppercase()));
    }
    for (name, description, schema) in tasks {
        out.push_str(&format!(
            "\n\n{}Args = TypedDict(\n    \"{}Args\",\n    {{\n",
            pascal(name),
            pascal(name)
        ));
        for prop in props(schema) {
            let kind = type_of(&prop.schema, &format!("{name}.{}", prop.name), false);
            let wrapper = if prop.required {
                "Required"
            } else {
                "NotRequired"
            };
            out.push_str(&format!("        \"{}\": {wrapper}[{kind}],\n", prop.name));
        }
        out.push_str("    },\n)\n");
        out.push_str(&format!(
            "{}Args.__doc__ = \"\"\"Arguments of ``{name}``: {}\"\"\"\n",
            pascal(name),
            doc(description)
        ));
    }
    out.push_str("\n\nclass GeneratedTasks:\n    \"\"\"One typed method per task; the client supplies ``task()``.\"\"\"\n\n");
    out.push_str("    def task(self, name: str, args: Mapping[str, Any] | None = None, **kwargs: Any) -> Any:\n        raise NotImplementedError\n");
    for (name, description, _) in tasks {
        out.push_str(&format!(
            "\n    def {name}(self, **kwargs: Unpack[{}Args]) -> Any:\n        \"\"\"{}\"\"\"\n        return self.task(TaskName.{}, kwargs)\n",
            pascal(name),
            doc(description),
            name.to_ascii_uppercase()
        ));
    }
    // Import only what the body uses, so ruff's F401 stays quiet.
    let typing: Vec<&str> = [
        "Any",
        "Literal",
        "NotRequired",
        "Required",
        "TypedDict",
        "Unpack",
    ]
    .into_iter()
    .filter(|name| {
        out.contains(&format!("{name}["))
            || out.contains(&format!("{name}("))
            || out.contains(&format!(": {name}"))
    })
    .collect();
    format!(
        "\"\"\"{HEADER}\n\"\"\"\n\nfrom __future__ import annotations\n\nfrom collections.abc import Mapping\nfrom enum import StrEnum\nfrom typing import {}\n\n{out}",
        typing.join(", ")
    )
}

#[test]
fn generated_sdk_types_match_the_task_registry() {
    let tasks: Vec<(String, String, Value)> = atl_sdk::tasks::list()
        .iter()
        .map(|t| (t.name.to_owned(), t.description.to_owned(), (t.schema)()))
        .collect();
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    let files = [
        (root.join("atl-node/ts/tasks.ts"), typescript(&tasks)),
        (root.join("atl-py/python/atl_sdk/_tasks.py"), python(&tasks)),
    ];
    let update = std::env::var("ATL_UPDATE_SDK_TYPES").is_ok_and(|v| v == "1");
    for (path, text) in files {
        if update {
            std::fs::write(&path, &text).expect("write generated SDK types");
            continue;
        }
        let current = std::fs::read_to_string(&path).unwrap_or_default();
        assert!(
            current == text,
            "{} is out of date with the task registry; run `ATL_UPDATE_SDK_TYPES=1 cargo test -p atl-sdk --test sdk_types` and commit crates/atl-node/ts/tasks.ts and crates/atl-py/python/atl_sdk/_tasks.py",
            path.display()
        );
    }
}
