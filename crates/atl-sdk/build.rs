//! Normalizes the vendored Atlassian OpenAPI documents into one compact,
//! uniform catalog file per service variant (`$OUT_DIR/<catalog>.json`), plus
//! its component schemas (`$OUT_DIR/<catalog>.schemas.json`).
//!
//! The runtime (`src/catalog.rs`) only understands the normalized format, so
//! every quirk of the upstream documents is handled here: missing or duplicate
//! `operationId`s, missing tags, `$ref` parameters and request bodies, schema
//! name collisions between merged documents, and CLI flag collisions.

use serde_json::{Map, Value, json};
use std::collections::{HashMap, HashSet};
use std::path::Path;

struct Source {
    file: &'static str,
    /// Prepended to every path so paths are relative to the connection base URL.
    prefix: &'static str,
    /// Keep only paths that start with this (before the prefix is added).
    only: Option<&'static str>,
    /// Drop paths that start with this.
    skip: Option<&'static str>,
}

const fn src(file: &'static str, prefix: &'static str) -> Source {
    Source {
        file,
        prefix,
        only: None,
        skip: None,
    }
}

/// Catalog key -> merged source documents. Keep in sync with `SERVICES` in
/// `src/catalog.rs`.
const CATALOGS: &[(&str, &[Source])] = &[
    ("jira-cloud", &[src("jira-cloud.json", "")]),
    // The Jira Data Center document holds both the platform and the Agile
    // (Jira Software) API, with clashing tags such as `issue`; split them so
    // `atl jira` and `atl jira-software` match their Cloud counterparts.
    (
        "jira-dc",
        &[Source {
            file: "jira-dc.json",
            prefix: "/rest",
            only: None,
            skip: Some("/agile/"),
        }],
    ),
    (
        "jira-software-dc",
        &[Source {
            file: "jira-dc.json",
            prefix: "/rest",
            only: Some("/agile/"),
            skip: None,
        }],
    ),
    (
        "jira-software-cloud",
        &[src("jira-software-cloud.json", "")],
    ),
    ("jsm-cloud", &[src("jsm-cloud.json", "")]),
    (
        "confluence-cloud",
        &[src("confluence-v2-cloud.json", "/wiki/api/v2")],
    ),
    ("confluence-dc", &[src("confluence-dc.json", "")]),
    (
        "confluence-v1-cloud",
        &[src("confluence-v1-cloud.json", "")],
    ),
    ("bitbucket-cloud", &[src("bitbucket-cloud.json", "/2.0")]),
    ("bitbucket-dc", &[src("bitbucket-dc.json", "/rest")]),
    ("trello-cloud", &[src("trello-cloud.json", "/1")]),
    (
        "admin-cloud",
        &[
            src("admin-organization-cloud.json", "/admin"),
            src("admin-user-management-cloud.json", ""),
            src("admin-api-access-cloud.json", "/admin/api-access/v1"),
            src("admin-control-cloud.json", ""),
            src("admin-dlp-cloud.json", "/admin/dlp/v1"),
        ],
    ),
    (
        "admin-scim-cloud",
        &[src("admin-user-provisioning-cloud.json", "")],
    ),
];

/// Flags the CLI defines on every operation command. A parameter whose kebab
/// name collides with one of these is exposed as `--param-<name>`.
const RESERVED_FLAGS: &[&str] = &[
    "help",
    "params",
    "json",
    "upload",
    "dry-run",
    "page-all",
    "page-limit",
    "page-delay",
    "output",
    "profile",
    "deployment",
    "version",
    "agent",
    "format",
    "select",
    "raw",
    "yes",
];

const METHODS: &[&str] = &["get", "put", "post", "delete", "patch", "head", "options"];

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=specs");
    let out_dir = std::env::var("OUT_DIR").expect("cargo sets OUT_DIR");
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("cargo sets CARGO_MANIFEST_DIR");
    // Operation counts, so listing services does not parse every catalog.
    let mut counts = String::from("const OPERATION_COUNTS: &[(&str, usize)] = &[\n");
    for (name, sources) in CATALOGS {
        let (catalog, schemas) = build_catalog(Path::new(&manifest_dir), sources);
        let ops = catalog["ops"].as_array().map_or(0, Vec::len);
        counts.push_str(&format!("    ({name:?}, {ops}),\n"));
        // Component schemas go in their own file: they are about half of a
        // catalog and only `$ref` lookups read them, so the runtime parses
        // them on first use instead of on every catalog load.
        for (file, value) in [
            (format!("{name}.json"), catalog),
            (format!("{name}.schemas.json"), schemas),
        ] {
            let text = serde_json::to_string(&value).expect("catalog serializes");
            std::fs::write(Path::new(&out_dir).join(&file), text)
                .unwrap_or_else(|err| panic!("cannot write catalog file {file}: {err}"));
        }
    }
    counts.push_str("];\n");
    std::fs::write(Path::new(&out_dir).join("operation_counts.rs"), counts)
        .unwrap_or_else(|err| panic!("cannot write operation counts: {err}"));
}

#[derive(Default)]
struct Builder {
    tags: Vec<Value>,
    tag_index: HashMap<String, usize>,
    ops: Vec<Value>,
    params: Vec<Value>,
    schemas: Map<String, Value>,
    ids: HashSet<String>,
    /// (tag index, name) pairs already used.
    names: HashSet<(usize, String)>,
}

/// The catalog (tags, operations, parameters) and its component schemas.
fn build_catalog(root: &Path, sources: &[Source]) -> (Value, Value) {
    let mut builder = Builder::default();
    for source in sources {
        let path = root.join("specs").join(source.file);
        println!("cargo:rerun-if-changed={}", path.display());
        let text = std::fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("cannot read spec {}: {err}", path.display()));
        let mut spec: Value = serde_json::from_str(&text)
            .unwrap_or_else(|err| panic!("spec {} is not valid JSON: {err}", path.display()));
        let stem = source.file.trim_end_matches(".json");
        rename_colliding_schemas(&mut spec, &builder.schemas, stem);
        builder.add_spec(&spec, source);
    }
    let catalog = json!({
        "tags": builder.tags,
        "ops": builder.ops,
        "params": builder.params,
    });
    (catalog, Value::Object(builder.schemas))
}

/// When documents are merged, two of them may define a schema with the same
/// name and different content. Rename the newcomer's schema and rewrite every
/// `$ref` to it inside that document.
fn rename_colliding_schemas(spec: &mut Value, existing: &Map<String, Value>, stem: &str) {
    let Some(schemas) = spec
        .pointer("/components/schemas")
        .and_then(Value::as_object)
    else {
        return;
    };
    let prefix = pascal(stem);
    let renames: HashMap<String, String> = schemas
        .iter()
        .filter(|(name, schema)| {
            existing
                .get(name.as_str())
                .is_some_and(|other| cleaned(schema) != *other)
        })
        .map(|(name, _)| {
            (
                format!("#/components/schemas/{name}"),
                format!("{prefix}{name}"),
            )
        })
        .collect();
    if renames.is_empty() {
        return;
    }
    rewrite_refs(spec, &renames);
    let schemas = spec
        .pointer_mut("/components/schemas")
        .and_then(Value::as_object_mut)
        .expect("checked above");
    let old = std::mem::take(schemas);
    for (name, schema) in old {
        let key = format!("#/components/schemas/{name}");
        let new_name = renames.get(&key).cloned().unwrap_or(name);
        schemas.insert(new_name, schema);
    }
}

fn rewrite_refs(value: &mut Value, renames: &HashMap<String, String>) {
    match value {
        Value::Object(map) => {
            if let Some(Value::String(target)) = map.get_mut("$ref")
                && let Some(new_name) = renames.get(target.as_str())
            {
                *target = format!("#/components/schemas/{new_name}");
            }
            for child in map.values_mut() {
                rewrite_refs(child, renames);
            }
        }
        Value::Array(items) => items
            .iter_mut()
            .for_each(|item| rewrite_refs(item, renames)),
        _ => {}
    }
}

impl Builder {
    fn add_spec(&mut self, spec: &Value, source: &Source) {
        let tag_docs: HashMap<&str, &str> = spec
            .get("tags")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(|tag| {
                Some((
                    tag.get("name")?.as_str()?,
                    tag.get("description").and_then(Value::as_str).unwrap_or(""),
                ))
            })
            .collect();

        if let Some(schemas) = spec
            .pointer("/components/schemas")
            .and_then(Value::as_object)
        {
            for (name, schema) in schemas {
                self.schemas
                    .entry(name.clone())
                    .or_insert_with(|| cleaned(schema));
            }
        }

        let Some(paths) = spec.get("paths").and_then(Value::as_object) else {
            return;
        };
        for (raw_path, item) in paths {
            if source.only.is_some_and(|only| !raw_path.starts_with(only))
                || source.skip.is_some_and(|skip| raw_path.starts_with(skip))
            {
                continue;
            }
            let path_params = item
                .get("parameters")
                .and_then(Value::as_array)
                .map(Vec::as_slice)
                .unwrap_or_default();
            for method in METHODS {
                let Some(op) = item.get(*method) else {
                    continue;
                };
                self.add_operation(
                    spec,
                    &tag_docs,
                    source.prefix,
                    raw_path,
                    method,
                    op,
                    path_params,
                );
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn add_operation(
        &mut self,
        spec: &Value,
        tag_docs: &HashMap<&str, &str>,
        prefix: &str,
        raw_path: &str,
        method: &str,
        op: &Value,
        path_params: &[Value],
    ) {
        let summary = op.get("summary").and_then(Value::as_str).map(str::trim);
        let tag_title = op
            .get("tags")
            .and_then(Value::as_array)
            .and_then(|tags| tags.first())
            .and_then(Value::as_str)
            .map(str::to_owned)
            .unwrap_or_else(|| {
                raw_path
                    .split('/')
                    .find(|seg| !seg.is_empty() && !seg.starts_with('{'))
                    .unwrap_or("default")
                    .to_owned()
            });
        let tag = self.tag(&tag_title, tag_docs.get(tag_title.as_str()).copied());

        let base_id = match op.get("operationId").and_then(Value::as_str) {
            Some(id) if !id.trim().is_empty() => id.trim().to_owned(),
            _ => match summary.filter(|s| !s.is_empty()) {
                Some(summary) => kebab(summary),
                None => kebab(&format!("{method} {}", raw_path.replace(['{', '}'], ""))),
            },
        };
        let id = unique(&base_id, |candidate| self.ids.contains(candidate));
        self.ids.insert(id.clone());
        let name = unique(&kebab(&id), |candidate| {
            self.names.contains(&(tag, candidate.to_owned()))
        });
        self.names.insert((tag, name.clone()));

        let start = self.params.len();
        self.add_params(spec, path_params, op.get("parameters"));
        let end = self.params.len();

        let mut out = Map::new();
        out.insert("id".into(), id.into());
        out.insert("name".into(), name.into());
        out.insert("tag".into(), tag.into());
        out.insert("method".into(), method.to_ascii_uppercase().into());
        out.insert(
            "path".into(),
            format!("{}{raw_path}", prefix.trim_end_matches('/')).into(),
        );
        if let Some(summary) = summary.filter(|s| !s.is_empty()) {
            out.insert("summary".into(), summary.into());
        }
        if let Some(description) = op.get("description").and_then(Value::as_str) {
            out.insert("description".into(), description.trim().into());
        }
        if op.get("deprecated").and_then(Value::as_bool) == Some(true) {
            out.insert("deprecated".into(), true.into());
        }
        out.insert("params".into(), json!([start, end]));
        if let Some(body) = op.get("requestBody").map(|body| resolve(spec, body))
            && let Some(body) = request_body(body)
        {
            out.insert("body".into(), body);
        }
        if let Some(response) = response_schema(spec, op) {
            out.insert("response".into(), response);
        }
        self.ops.push(Value::Object(out));
    }

    fn tag(&mut self, title: &str, description: Option<&str>) -> usize {
        let name = match kebab(title) {
            name if name.is_empty() => "default".to_owned(),
            name => name,
        };
        if let Some(&index) = self.tag_index.get(&name) {
            return index;
        }
        let index = self.tags.len();
        let mut tag = Map::new();
        tag.insert("name".into(), name.clone().into());
        tag.insert("title".into(), title.into());
        if let Some(description) = description.map(str::trim).filter(|d| !d.is_empty()) {
            tag.insert("description".into(), description.into());
        }
        self.tags.push(Value::Object(tag));
        self.tag_index.insert(name, index);
        index
    }

    fn add_params(&mut self, spec: &Value, path_level: &[Value], op_level: Option<&Value>) {
        // Operation-level parameters override path-level ones with the same
        // name and location.
        let mut merged: Vec<&Value> = Vec::new();
        let op_level = op_level
            .and_then(Value::as_array)
            .map(Vec::as_slice)
            .unwrap_or_default();
        for param in path_level.iter().chain(op_level) {
            let param = resolve(spec, param);
            let key = |p: &Value| {
                (
                    p.get("name").and_then(Value::as_str).map(str::to_owned),
                    p.get("in").and_then(Value::as_str).map(str::to_owned),
                )
            };
            let this = key(param);
            merged.retain(|existing| key(existing) != this);
            merged.push(param);
        }

        let mut flags: HashSet<String> = HashSet::new();
        for param in merged {
            let (Some(name), Some(location)) = (
                param.get("name").and_then(Value::as_str),
                param.get("in").and_then(Value::as_str),
            ) else {
                continue;
            };
            if !matches!(location, "path" | "query" | "header")
                || (location == "header" && name.eq_ignore_ascii_case("authorization"))
            {
                continue;
            }
            let mut flag = kebab(name);
            if flag.is_empty() {
                flag = "value".to_owned();
            }
            if RESERVED_FLAGS.contains(&flag.as_str()) {
                flag = format!("param-{flag}");
            }
            if flags.contains(&flag) {
                flag = format!("{flag}-{location}");
            }
            let flag = unique(&flag, |candidate| flags.contains(candidate));
            flags.insert(flag.clone());

            let mut out = Map::new();
            out.insert("name".into(), name.into());
            out.insert("flag".into(), flag.into());
            out.insert("in".into(), location.into());
            let required =
                location == "path" || param.get("required").and_then(Value::as_bool) == Some(true);
            out.insert("required".into(), required.into());
            if let Some(description) = param
                .get("description")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|d| !d.is_empty())
            {
                out.insert("description".into(), description.into());
            }
            if let Some(schema) = param.get("schema") {
                out.insert("schema".into(), cleaned(schema));
            }
            self.params.push(Value::Object(out));
        }
    }
}

/// Follows a local `$ref` (parameters, requestBodies, responses) once.
fn resolve<'a>(spec: &'a Value, value: &'a Value) -> &'a Value {
    match value.get("$ref").and_then(Value::as_str) {
        Some(target) if target.starts_with("#/") => spec.pointer(&target[1..]).unwrap_or(value),
        _ => value,
    }
}

fn request_body(body: &Value) -> Option<Value> {
    let content = body.get("content")?.as_object()?;
    let rank = |content_type: &str| match content_type {
        "application/json" => 0,
        ct if ct.ends_with("+json") => 1,
        "multipart/form-data" => 2,
        "application/x-www-form-urlencoded" => 3,
        "text/plain" => 4,
        _ => 5,
    };
    let (content_type, media) = content.iter().min_by_key(|(ct, _)| rank(ct))?;
    let content_type = if content_type == "*/*" {
        "application/json"
    } else {
        content_type
    };
    Some(json!({
        "content_type": content_type,
        "required": body.get("required").and_then(Value::as_bool).unwrap_or(false),
        "schema": media.get("schema").map(cleaned).unwrap_or_else(|| json!({})),
    }))
}

fn response_schema(spec: &Value, op: &Value) -> Option<Value> {
    let responses = op.get("responses")?.as_object()?;
    ["200", "201", "202", "default"]
        .iter()
        .filter_map(|code| responses.get(*code))
        .map(|response| resolve(spec, response))
        .find_map(|response| {
            let content = response.get("content")?.as_object()?;
            let media = content
                .get("application/json")
                .or_else(|| content.values().next())?;
            media.get("schema").map(cleaned)
        })
}

/// Copies a schema without `example`, `examples`, and `x-` extension keywords.
/// Property names under `properties` are kept even when they look like those
/// keywords, and literal values (`enum`, `default`, `const`) are not touched.
fn cleaned(schema: &Value) -> Value {
    match schema {
        Value::Object(map) => Value::Object(
            map.iter()
                .filter(|(key, _)| {
                    !(key.as_str() == "example"
                        || key.as_str() == "examples"
                        || key.starts_with("x-"))
                })
                .map(|(key, value)| {
                    let value = match key.as_str() {
                        "properties" | "patternProperties" => match value {
                            Value::Object(props) => Value::Object(
                                props.iter().map(|(k, v)| (k.clone(), cleaned(v))).collect(),
                            ),
                            other => other.clone(),
                        },
                        "enum" | "default" | "const" => value.clone(),
                        _ => cleaned(value),
                    };
                    (key.clone(), value)
                })
                .collect(),
        ),
        Value::Array(items) => Value::Array(items.iter().map(cleaned).collect()),
        other => other.clone(),
    }
}

/// `base`, then `base-2`, `base-3`, ... until `taken` returns false.
fn unique(base: &str, taken: impl Fn(&str) -> bool) -> String {
    if !taken(base) {
        return base.to_owned();
    }
    (2..)
        .map(|n| format!("{base}-{n}"))
        .find(|candidate| !taken(candidate))
        .expect("an unused suffix exists")
}

/// `getJQLQuery` -> `get-jql-query`, `Issue search (apps)` -> `issue-search-apps`,
/// `Admin APIs` -> `admin-apis`, `OAuth Client` -> `oauth-client`.
fn kebab(text: &str) -> String {
    let chars: Vec<char> = text.replace("OAuth", "Oauth").chars().collect();
    let mut out = String::with_capacity(text.len() + 4);
    for (i, &c) in chars.iter().enumerate() {
        if c == '\'' || c == '\u{2019}' {
            // "project's" -> "projects", not "project-s".
            continue;
        }
        if !c.is_ascii_alphanumeric() {
            if !out.is_empty() && !out.ends_with('-') {
                out.push('-');
            }
            continue;
        }
        if c.is_ascii_uppercase() && i > 0 && !out.is_empty() && !out.ends_with('-') {
            let prev = chars[i - 1];
            // A plural acronym ("APIs", "IDs") stays one word.
            let plural = chars.get(i + 1) == Some(&'s')
                && !chars.get(i + 2).is_some_and(char::is_ascii_lowercase);
            let next_lower = !plural && chars.get(i + 1).is_some_and(char::is_ascii_lowercase);
            if prev.is_ascii_lowercase()
                || prev.is_ascii_digit()
                || (prev.is_ascii_uppercase() && next_lower)
            {
                out.push('-');
            }
        }
        out.push(c.to_ascii_lowercase());
    }
    while out.ends_with('-') {
        out.pop();
    }
    out
}

fn pascal(text: &str) -> String {
    kebab(text)
        .split('-')
        .map(|word| {
            let mut chars = word.chars();
            chars
                .next()
                .map(|first| first.to_ascii_uppercase().to_string() + chars.as_str())
                .unwrap_or_default()
        })
        .collect()
}
