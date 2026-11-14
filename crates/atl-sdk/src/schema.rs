//! JSON Schema helpers: inline `$ref`s to a bounded depth and describe an
//! operation's inputs as one JSON Schema object.

use crate::catalog::{Catalog, Operation, ServiceDef};
use serde_json::value::RawValue;
use serde_json::{Map, Value, json};

const REF_PREFIX: &str = "#/components/schemas/";

/// Parses `schema` and replaces `$ref`s with the referenced schema, `depth`
/// levels deep. Deeper or recursive references stay as `{"$ref": ...}` so the
/// caller can look them up with [`named`].
pub fn expand(catalog: &Catalog, schema: &RawValue, depth: usize) -> Value {
    let value: Value = serde_json::from_str(schema.get()).unwrap_or(Value::Null);
    let mut stack = Vec::new();
    expand_value(catalog, value, depth, &mut stack)
}

/// Returns a named component schema, expanded `depth` levels.
pub fn named(catalog: &Catalog, name: &str, depth: usize) -> Option<Value> {
    let raw = catalog.schemas.get(name.trim_start_matches(REF_PREFIX))?;
    Some(expand(catalog, raw, depth))
}

fn expand_value(catalog: &Catalog, value: Value, depth: usize, stack: &mut Vec<String>) -> Value {
    match value {
        Value::Object(map) => {
            if let Some(Value::String(target)) = map.get("$ref") {
                let name = target.trim_start_matches(REF_PREFIX).to_owned();
                if depth == 0 || stack.contains(&name) {
                    return Value::Object(map);
                }
                let Some(raw) = catalog.schemas.get(name.as_str()) else {
                    return Value::Object(map);
                };
                let inner: Value = serde_json::from_str(raw.get()).unwrap_or(Value::Null);
                stack.push(name);
                let expanded = expand_value(catalog, inner, depth - 1, stack);
                stack.pop();
                return expanded;
            }
            Value::Object(
                map.into_iter()
                    .map(|(key, child)| {
                        let child = match key.as_str() {
                            "enum" | "default" | "const" => child,
                            _ => expand_value(catalog, child, depth, stack),
                        };
                        (key, child)
                    })
                    .collect(),
            )
        }
        Value::Array(items) => Value::Array(
            items
                .into_iter()
                .map(|item| expand_value(catalog, item, depth, stack))
                .collect(),
        ),
        other => other,
    }
}

/// One JSON Schema object describing everything a caller passes to the
/// operation: each parameter by its API name, plus `body` when the operation
/// takes a request body. Used by `atl schema` and by the MCP server.
pub fn operation_input(catalog: &Catalog, op: &Operation, depth: usize) -> Value {
    let mut properties = Map::new();
    let mut required = Vec::new();
    for param in catalog.params_of(op) {
        let mut schema = match param.schema {
            Some(raw) => expand(catalog, raw, depth),
            None => json!({ "type": "string" }),
        };
        if let (Value::Object(map), Some(description)) = (&mut schema, &param.description) {
            map.entry("description")
                .or_insert_with(|| Value::String(description.to_string()));
        }
        if let Value::Object(map) = &mut schema {
            map.insert("x-in".into(), param.location.as_str().into());
        }
        if param.required {
            required.push(Value::String(param.name.to_string()));
        }
        properties.insert(param.name.to_string(), schema);
    }
    if let Some(body) = &op.body {
        let mut schema = expand(catalog, body.schema, depth);
        if let Value::Object(map) = &mut schema {
            map.insert(
                "x-content-type".into(),
                body.content_type.to_string().into(),
            );
            if body.is_multipart() {
                map.insert(
                    "description".into(),
                    "Multipart upload. Pass file paths with `--upload`; other form fields as a JSON object.".into(),
                );
            }
        }
        if body.required {
            required.push("body".into());
        }
        properties.insert("body".into(), schema);
    }
    json!({ "type": "object", "properties": properties, "required": required })
}

/// Everything a caller needs to invoke an operation, as JSON: the CLI
/// command, HTTP method and path, parameters, and expanded body and response
/// schemas. Used by `atl schema` and the MCP describe tool.
pub fn describe_operation(
    service: &ServiceDef,
    catalog: &Catalog,
    op: &Operation,
    depth: usize,
) -> Value {
    let params: Vec<Value> = catalog
        .params_of(op)
        .iter()
        .map(|param| {
            json!({
                "name": param.name,
                "flag": format!("--{}", param.flag),
                "in": param.location.as_str(),
                "required": param.required,
                "description": param.description,
                "schema": param.schema.map(|raw| expand(catalog, raw, depth)),
            })
        })
        .collect();
    json!({
        "service": service.name,
        "operation": op.id,
        "cli": format!("atl {} {} {}", service.name, catalog.tag(op.tag).name, op.name),
        "method": op.method.as_str(),
        "path": op.path,
        "summary": op.summary,
        "description": op.description,
        "deprecated": op.deprecated,
        "parameters": params,
        "body": op.body.as_ref().map(|body| json!({
            "content_type": body.content_type,
            "required": body.required,
            "schema": expand(catalog, body.schema, depth),
        })),
        "response": op.response.map(|raw| expand(catalog, raw, depth)),
    })
}
