//! Generates the typed clients of the TypeScript and Python SDKs from the task
//! registry, and fails when the checked-in files are out of date.
//! `ATL_UPDATE_SDK_TYPES=1 cargo test -p atlassian-toolkit --test sdk_types` rewrites them.
//!
//! Every task becomes a method on a product namespace by the same rule as the
//! CLI path: `jira_issue_list` is `atl jira issue list` and
//! `client.jira.issue.list(...)`; a two-part name such as `jira_me` is a method
//! on the product. Arguments become `TypedDict`s (Python) and interfaces
//! (TypeScript); results become pydantic models and interfaces, named after
//! the Rust types in each task's output schema.

use serde_json::{Map, Value};
use std::collections::BTreeMap;
use std::path::PathBuf;

const HEADER: &str = "Generated from the task registry (crates/atl-sdk/src/tasks/) by\ncrates/atl-sdk/tests/sdk_types.rs. Do not edit; run\n`ATL_UPDATE_SDK_TYPES=1 cargo test -p atlassian-toolkit --test sdk_types`.";

/// The product namespaces in the order the clients list them, with the text
/// that documents each one.
const PRODUCTS: [(&str, &str, &str); 6] = [
    (
        "jira",
        "Jira",
        "Jira and Jira Software: issues, comments, sprints, boards, filters and JQL.",
    ),
    (
        "confluence",
        "Confluence",
        "Confluence pages, blogs, spaces, comments, labels and attachments.",
    ),
    (
        "bitbucket",
        "Bitbucket",
        "Bitbucket repositories, branches, pull requests and pipelines.",
    ),
    (
        "jsm",
        "Jsm",
        "Jira Service Management desks, queues, requests and approvals.",
    ),
    (
        "trello",
        "Trello",
        "Trello boards, lists, cards and checklists.",
    ),
    (
        "admin",
        "Admin",
        "Atlassian Admin organizations, users, groups and audit events.",
    ),
];

/// Names a Python keyword or a pydantic `BaseModel` attribute: a field with
/// one of these names gets a trailing `_` and an alias.
const PY_RESERVED: [&str; 47] = [
    "False",
    "None",
    "True",
    "and",
    "as",
    "assert",
    "async",
    "await",
    "break",
    "class",
    "continue",
    "def",
    "del",
    "elif",
    "else",
    "except",
    "finally",
    "for",
    "from",
    "global",
    "if",
    "import",
    "in",
    "is",
    "lambda",
    "nonlocal",
    "not",
    "or",
    "pass",
    "raise",
    "return",
    "try",
    "while",
    "with",
    "yield",
    "copy",
    "dict",
    "json",
    "schema",
    "schema_json",
    "construct",
    "validate",
    "parse_obj",
    "parse_raw",
    "parse_file",
    "from_orm",
    "update_forward_refs",
];

struct Task {
    name: String,
    description: String,
    args: Value,
    output: Value,
}

/// `(product, resource, method)` of a task name.
fn path(name: &str) -> (&str, Option<&str>, String) {
    let parts: Vec<&str> = name.split('_').collect();
    match parts.as_slice() {
        [product, method] => (product, None, (*method).to_owned()),
        [product, resource, rest @ ..] if !rest.is_empty() => {
            (product, Some(resource), rest.join("_"))
        }
        _ => panic!("task `{name}`: a name needs a product and a method, for example jira_me"),
    }
}

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

#[derive(Clone, Copy, PartialEq)]
enum Lang {
    Ts,
    Py,
}

/// Maps one schema to a type; `at` names the place for the panic message
/// when a shape is not supported.
fn type_of(schema: &Value, at: &str, lang: Lang) -> String {
    let ts = lang == Lang::Ts;
    let any = if ts { "unknown" } else { "Any" };
    if schema == &Value::Bool(true) || schema.as_object().is_some_and(Map::is_empty) {
        return any.to_owned();
    }
    if let Some(target) = schema["$ref"].as_str() {
        return target
            .strip_prefix("#/$defs/")
            .unwrap_or_else(|| panic!("{at}: unsupported $ref {target}"))
            .to_owned();
    }
    if let Some(options) = schema["anyOf"].as_array() {
        let (nulls, others): (Vec<&Value>, Vec<&Value>) =
            options.iter().partition(|o| o["type"] == "null");
        let [one] = others.as_slice() else {
            panic!("{at}: anyOf with more than one non-null type: {schema}");
        };
        let inner = type_of(one, at, lang);
        return match (nulls.is_empty(), ts) {
            (true, _) => inner,
            (false, true) => format!("{inner} | null"),
            (false, false) => format!("{inner} | None"),
        };
    }
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
    let (kinds, nullable) = match &schema["type"] {
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
    };
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
            let item = type_of(&schema["items"], &format!("{at}[]"), lang);
            match ts {
                true if item.contains(' ') => format!("({item})[]"),
                true => format!("{item}[]"),
                false => format!("list[{item}]"),
            }
        }
        ("object", _) if schema.get("properties").is_none() => {
            let value = match &schema["additionalProperties"] {
                Value::Object(inner) if !inner.is_empty() => {
                    type_of(&schema["additionalProperties"], &format!("{at}{{}}"), lang)
                }
                _ => any.to_owned(),
            };
            match ts {
                true => format!("Record<string, {value}>"),
                false => format!("dict[str, {value}]"),
            }
        }
        _ => panic!("{at}: unsupported schema {schema}; give the nested object a named Rust type"),
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

/// Every named result type (from `$defs` and from object outputs' titles),
/// sorted by name. The registry test guarantees one shape per name.
fn models(tasks: &[Task]) -> BTreeMap<String, Value> {
    let mut models = BTreeMap::new();
    for task in tasks {
        for (name, schema) in task.output["$defs"].as_object().into_iter().flatten() {
            models.insert(name.clone(), schema.clone());
        }
        if task.output["type"] == "object" {
            let title = task.output["title"]
                .as_str()
                .unwrap_or_else(|| panic!("{}: an object output needs a title", task.name));
            let mut schema = task.output.clone();
            if let Value::Object(map) = &mut schema {
                map.remove("$defs");
                map.remove("title");
            }
            models.insert(title.to_owned(), schema);
        }
    }
    models
}

/// The result type of a task.
fn output_type(task: &Task, lang: Lang) -> String {
    match task.output["title"].as_str() {
        Some(title) if task.output["type"] == "object" => title.to_owned(),
        _ => type_of(&task.output, &format!("{} output", task.name), lang),
    }
}

/// `(product, class name, doc, tasks by resource)` for each product.
/// A product's tasks by resource; `None` holds the product's own methods.
type Groups<'a> = BTreeMap<Option<String>, Vec<&'a Task>>;

fn tree(tasks: &[Task]) -> Vec<(&'static str, &'static str, &'static str, Groups<'_>)> {
    PRODUCTS
        .iter()
        .map(|&(product, class, doc)| {
            let mut groups = Groups::new();
            for task in tasks.iter().filter(|t| path(&t.name).0 == product) {
                groups
                    .entry(path(&task.name).1.map(str::to_owned))
                    .or_default()
                    .push(task);
            }
            let methods: Vec<String> = groups
                .get(&None)
                .into_iter()
                .flatten()
                .map(|t| path(&t.name).2)
                .collect();
            for resource in groups.keys().flatten() {
                assert!(
                    !methods.contains(resource),
                    "{product}: `{resource}` is both a resource and a method"
                );
            }
            (product, class, doc, groups)
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

fn ts_key(name: &str) -> String {
    match name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        true => name.to_owned(),
        false => format!("\"{name}\""),
    }
}

fn ts_interface(out: &mut String, name: &str, schema: &Value, description: Option<&str>) {
    if let Some(description) = description {
        out.push_str(&jsdoc(description, ""));
    }
    out.push_str(&format!("export interface {name} {{\n"));
    for prop in props(schema) {
        if let Some(description) = &prop.description {
            out.push_str(&jsdoc(description, "  "));
        }
        let optional = if prop.required { "" } else { "?" };
        let kind = type_of(&prop.schema, &format!("{name}.{}", prop.name), Lang::Ts);
        out.push_str(&format!("  {}{optional}: {kind}\n", ts_key(&prop.name)));
    }
    if schema["additionalProperties"].is_object() || schema["additionalProperties"] == true {
        out.push_str("  [field: string]: unknown\n");
    }
    out.push_str("}\n\n");
}

fn typescript(tasks: &[Task]) -> String {
    let mut out = String::new();
    for line in HEADER.lines() {
        out.push_str(&format!("// {line}\n"));
    }
    out.push('\n');
    for task in tasks {
        let description = format!("Arguments of `{}`: {}", task.name, task.description);
        ts_interface(
            &mut out,
            &format!("{}Args", pascal(&task.name)),
            &task.args,
            Some(&description),
        );
    }
    for (name, schema) in models(tasks) {
        ts_interface(&mut out, &name, &schema, schema["description"].as_str());
    }
    out.push_str(
        "/** The arguments of every task, by task name. */\nexport interface TaskArgs {\n",
    );
    for task in tasks {
        out.push_str(&format!("  {}: {}Args\n", task.name, pascal(&task.name)));
    }
    out.push_str(
        "}\n\n/** The result of every task, by task name. */\nexport interface TaskResults {\n",
    );
    for task in tasks {
        out.push_str(&format!(
            "  {}: {}\n",
            task.name,
            output_type(task, Lang::Ts)
        ));
    }
    out.push_str("}\n\nexport type TaskName = keyof TaskArgs\n\n");
    let codes: Vec<String> = atl_sdk::error::CODES
        .iter()
        .map(|c| format!("\"{c}\""))
        .collect();
    out.push_str(&format!(
        "/** Every value of `AtlError.code`. */\nexport type ErrorCode = {}\n\n",
        codes.join(" | ")
    ));
    out.push_str("/** The product namespaces; each SDK supplies `task()` over its own transport. */\nexport abstract class GeneratedTasks {\n");
    out.push_str("  /** Runs any task by name, for example `task(\"jira_issue_view\", { key: \"A-1\" })`. */\n");
    out.push_str("  abstract task<N extends TaskName>(name: N, args?: TaskArgs[N]): Promise<TaskResults[N]>\n");
    let method = |task: &Task, indent: &str| {
        let has_required = props(&task.args).iter().any(|p| p.required);
        let args = match has_required {
            true => format!("args: {}Args", pascal(&task.name)),
            false => format!("args: {}Args = {{}}", pascal(&task.name)),
        };
        format!(
            "{}{indent}{}: ({args}): Promise<{}> => this.task(\"{}\", args),\n",
            jsdoc(&task.description, indent),
            camel(&path(&task.name).2),
            output_type(task, Lang::Ts),
            task.name,
        )
    };
    for (product, _, doc, groups) in tree(tasks) {
        out.push('\n');
        out.push_str(&jsdoc(doc, "  "));
        out.push_str(&format!("  readonly {product} = {{\n"));
        for (resource, members) in &groups {
            match resource {
                None => {
                    for task in members {
                        out.push_str(&method(task, "    "));
                    }
                }
                Some(resource) => {
                    out.push_str(&jsdoc(&format!("`atl {product} {resource}`"), "    "));
                    out.push_str(&format!("    {resource}: {{\n"));
                    for task in members {
                        out.push_str(&method(task, "      "));
                    }
                    out.push_str("    },\n");
                }
            }
        }
        out.push_str("  }\n");
    }
    out.push_str("}\n");
    out
}

fn py_doc(text: &str) -> String {
    text.replace('\\', "\\\\").replace("\"\"\"", "\\\"\\\"\\\"")
}

fn py_model(out: &mut String, name: &str, schema: &Value) {
    out.push_str(&format!("\n\nclass {name}(_Model):\n"));
    if let Some(description) = schema["description"].as_str() {
        out.push_str(&format!("    \"\"\"{}\"\"\"\n\n", py_doc(description)));
    }
    let fields = props(schema);
    if fields.is_empty() {
        out.push_str("    pass\n");
    }
    for prop in fields {
        // A field may be named `list` (a Trello card's list), so use the alias.
        let mut kind = type_of(&prop.schema, &format!("{name}.{}", prop.name), Lang::Py)
            .replace("list[", "_list[");
        if !prop.required && !kind.ends_with("| None") && kind != "Any" {
            kind.push_str(" | None");
        }
        let reserved = PY_RESERVED.contains(&prop.name.as_str());
        let field = if reserved {
            format!("{}_", prop.name)
        } else {
            prop.name.clone()
        };
        let default = match (prop.required, reserved) {
            (true, false) => String::new(),
            (false, false) => " = None".to_owned(),
            (true, true) => format!(" = pydantic.Field(alias=\"{}\")", prop.name),
            (false, true) => format!(" = pydantic.Field(None, alias=\"{}\")", prop.name),
        };
        out.push_str(&format!("    {field}: {kind}{default}\n"));
        if let Some(description) = &prop.description {
            out.push_str(&format!("    \"\"\"{}\"\"\"\n", py_doc(description)));
        }
    }
}

/// The Python statement that converts `data` (the task's JSON) to its result type.
fn py_convert(task: &Task, data: &str) -> String {
    let output = &task.output;
    let item_model = output["items"]["$ref"]
        .as_str()
        .and_then(|r| r.strip_prefix("#/$defs/"));
    match (output["type"].as_str(), item_model) {
        (Some("object"), _) => format!("{}.model_validate({data})", output_type(task, Lang::Py)),
        (Some("array"), Some(model)) => {
            format!("[{model}.model_validate(item) for item in {data}]")
        }
        _ => data.to_owned(),
    }
}

fn python(tasks: &[Task]) -> String {
    let models = models(tasks);
    let tree = tree(tasks);
    let mut exported: Vec<String> = ["ErrorCode", "TaskName"].map(str::to_owned).into();
    exported.extend(tasks.iter().map(|t| format!("{}Args", pascal(&t.name))));
    exported.extend(models.keys().cloned());
    for (product, class, _, groups) in &tree {
        exported.extend([format!("{class}Api"), format!("Async{class}Api")]);
        for resource in groups.keys().flatten() {
            let name = format!("{class}{}Api", pascal(resource));
            assert!(
                !models.contains_key(&name),
                "{product}: namespace `{name}` clashes with a result model"
            );
            exported.extend([name.clone(), format!("Async{name}")]);
        }
    }
    exported.sort();
    let mut out = format!(
        "__all__ = [{}]\n\n# Methods and fields can be named `list`, so annotations use this alias.\n_list = list\n\n\nclass ErrorCode(StrEnum):\n    \"\"\"Every value of ``AtlError.code``.\"\"\"\n\n{}\n\n\nclass TaskName(StrEnum):\n    \"\"\"Every task name.\"\"\"\n\n",
        exported
            .iter()
            .map(|n| format!("\"{n}\""))
            .collect::<Vec<_>>()
            .join(", "),
        atl_sdk::error::CODES
            .iter()
            .map(|c| format!("    {} = \"{c}\"", c.to_ascii_uppercase()))
            .collect::<Vec<_>>()
            .join("\n")
    );
    for task in tasks {
        out.push_str(&format!(
            "    {} = \"{}\"\n",
            task.name.to_ascii_uppercase(),
            task.name
        ));
    }
    for task in tasks {
        let name = format!("{}Args", pascal(&task.name));
        out.push_str(&format!(
            "\n\n{name} = TypedDict(\n    \"{name}\",\n    {{\n"
        ));
        for prop in props(&task.args) {
            let kind = type_of(
                &prop.schema,
                &format!("{}.{}", task.name, prop.name),
                Lang::Py,
            );
            let wrapper = if prop.required {
                "Required"
            } else {
                "NotRequired"
            };
            out.push_str(&format!("        \"{}\": {wrapper}[{kind}],\n", prop.name));
        }
        out.push_str("    },\n)\n");
        out.push_str(&format!(
            "{name}.__doc__ = \"\"\"Arguments of ``{}``: {}\"\"\"\n",
            task.name,
            py_doc(&task.description)
        ));
    }
    out.push_str("\n\nclass _Model(pydantic.BaseModel):\n    \"\"\"A task result. Unknown fields are kept, so a newer server never breaks parsing.\"\"\"\n\n    model_config = pydantic.ConfigDict(\n        extra=\"allow\", defer_build=True, validate_by_name=True, serialize_by_alias=True\n    )\n");
    for (name, schema) in &models {
        py_model(&mut out, name, schema);
    }
    out.push_str("\n\nclass _Tasks(Protocol):\n    def task(self, name: str, args: Mapping[str, Any] | None = None) -> Any: ...\n\n\nclass _AsyncTasks(Protocol):\n    async def task(self, name: str, args: Mapping[str, Any] | None = None) -> Any: ...\n");
    for asynchronous in [false, true] {
        let prefix = if asynchronous { "Async" } else { "" };
        let runner = if asynchronous {
            "_AsyncTasks"
        } else {
            "_Tasks"
        };
        let method = |task: &Task| {
            let result = output_type(task, Lang::Py).replacen("list[", "_list[", 1);
            let call = format!(
                "self._client.task(TaskName.{}, kwargs)",
                task.name.to_ascii_uppercase()
            );
            let (def, call) = match asynchronous {
                true => ("async def", format!("await {call}")),
                false => ("def", call),
            };
            format!(
                "\n    {def} {}(self, **kwargs: Unpack[{}Args]) -> {result}:\n        \"\"\"{}\"\"\"\n        return {}\n",
                path(&task.name).2,
                pascal(&task.name),
                py_doc(&task.description),
                py_convert(task, &call),
            )
        };
        for (product, class, doc, groups) in &tree {
            for (resource, members) in groups {
                let Some(resource) = resource else { continue };
                out.push_str(&format!(
                    "\n\nclass {prefix}{class}{}Api:\n    \"\"\"``atl {product} {resource}``\"\"\"\n\n    def __init__(self, client: {runner}) -> None:\n        self._client = client\n",
                    pascal(resource)
                ));
                for task in members {
                    out.push_str(&method(task));
                }
            }
            out.push_str(&format!(
                "\n\nclass {prefix}{class}Api:\n    \"\"\"{}\"\"\"\n\n    def __init__(self, client: {runner}) -> None:\n        self._client = client\n",
                py_doc(doc)
            ));
            for resource in groups.keys().flatten() {
                out.push_str(&format!(
                    "        self.{resource} = {prefix}{class}{}Api(client)\n        \"\"\"``atl {product} {resource}``\"\"\"\n",
                    pascal(resource)
                ));
            }
            for task in groups.get(&None).into_iter().flatten() {
                out.push_str(&method(task));
            }
        }
        let def = if asynchronous { "async def" } else { "def" };
        out.push_str(&format!(
            "\n\nclass Generated{prefix}Client:\n    \"\"\"The product namespaces of ``{prefix}Client``; the client supplies ``task()``.\"\"\"\n\n    {def} task(self, name: str, args: Mapping[str, Any] | None = None) -> Any:\n        raise NotImplementedError\n\n    def __init__(self) -> None:\n"
        ));
        for (product, class, doc, _) in &tree {
            out.push_str(&format!(
                "        self.{product} = {prefix}{class}Api(self)\n        \"\"\"{}\"\"\"\n",
                py_doc(doc)
            ));
        }
    }
    let typing: Vec<&str> = [
        "Any",
        "Literal",
        "NotRequired",
        "Protocol",
        "Required",
        "TypedDict",
        "Unpack",
    ]
    .into_iter()
    .filter(|name| {
        out.contains(&format!("{name}["))
            || out.contains(&format!("{name}("))
            || out.contains(&format!(": {name}"))
            || out.contains(&format!("-> {name}"))
            || out.contains(&format!("({name})"))
    })
    .collect();
    format!(
        "\"\"\"{HEADER}\n\"\"\"\n\nfrom __future__ import annotations\n\nfrom collections.abc import Mapping\nfrom enum import StrEnum\nfrom typing import {}\n\nimport pydantic\n\n{out}",
        typing.join(", ")
    )
}

#[test]
fn generated_sdk_types_match_the_task_registry() {
    let tasks: Vec<Task> = atl_sdk::tasks::list()
        .iter()
        .map(|t| Task {
            name: t.name.to_owned(),
            description: t.description.to_owned(),
            args: (t.schema)(),
            output: (t.output)(),
        })
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
            "{} is out of date with the task registry; run `ATL_UPDATE_SDK_TYPES=1 cargo test -p atlassian-toolkit --test sdk_types` and commit crates/atl-node/ts/tasks.ts and crates/atl-py/python/atl_sdk/_tasks.py",
            path.display()
        );
    }
}
