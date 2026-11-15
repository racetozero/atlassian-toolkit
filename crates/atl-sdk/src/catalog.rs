//! The service registry and the runtime view of the normalized OpenAPI
//! catalogs that `build.rs` writes to `$OUT_DIR`.
//!
//! A catalog is an arena: tags, operations and parameters each live in one
//! `Vec`, operations refer to their tag by index and to their parameters by an
//! index range. Text and JSON Schemas borrow from the embedded catalog string,
//! so loading a catalog allocates only the vectors and the escaped strings.

use crate::config::{Deployment, Family};
use serde::Deserialize;
use serde_json::value::RawValue;
use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt;
use std::sync::OnceLock;

/// One command-line service such as `jira` or `confluence`.
#[derive(Debug)]
pub struct ServiceDef {
    /// What the user types after `atl`.
    pub name: &'static str,
    pub title: &'static str,
    /// Which stored credential the service uses.
    pub family: Family,
    /// Catalog key for Atlassian Cloud, if the service exists there.
    pub cloud: Option<&'static str>,
    /// Catalog key for Atlassian Data Center, if the service exists there.
    pub data_center: Option<&'static str>,
    /// Shown when the service has no variant for the requested deployment.
    pub data_center_note: Option<&'static str>,
}

impl ServiceDef {
    pub fn catalog_key(&self, deployment: Deployment) -> Option<&'static str> {
        match deployment {
            Deployment::Cloud => self.cloud,
            Deployment::DataCenter => self.data_center,
        }
    }
}

static SERVICES: &[ServiceDef] = &[
    ServiceDef {
        name: "jira",
        title: "Jira platform: issues, projects, workflows, users, fields, search",
        family: Family::Jira,
        cloud: Some("jira-cloud"),
        data_center: Some("jira-dc"),
        data_center_note: None,
    },
    ServiceDef {
        name: "jira-software",
        title: "Jira Software (Agile): boards, sprints, backlog, epics",
        family: Family::Jira,
        cloud: Some("jira-software-cloud"),
        data_center: Some("jira-software-dc"),
        data_center_note: None,
    },
    ServiceDef {
        name: "jsm",
        title: "Jira Service Management: service desks, requests, customers, queues",
        family: Family::Jira,
        cloud: Some("jsm-cloud"),
        data_center: None,
        data_center_note: Some(
            "Atlassian publishes no OpenAPI document for Jira Service Management Data Center; use `atl jsm` (the task commands work on Data Center) or `atl api jira get /rest/servicedeskapi/...`",
        ),
    },
    ServiceDef {
        name: "confluence",
        title: "Confluence (Cloud REST v2; Data Center REST API)",
        family: Family::Confluence,
        cloud: Some("confluence-cloud"),
        data_center: Some("confluence-dc"),
        data_center_note: None,
    },
    ServiceDef {
        name: "confluence-v1",
        title: "Confluence Cloud REST v1: CQL search, content, macros, templates",
        family: Family::Confluence,
        cloud: Some("confluence-v1-cloud"),
        data_center: None,
        data_center_note: Some("on Data Center the v1-style API is `atl api confluence`"),
    },
    ServiceDef {
        name: "bitbucket",
        title: "Bitbucket: repositories, pull requests, pipelines, projects",
        family: Family::Bitbucket,
        cloud: Some("bitbucket-cloud"),
        data_center: Some("bitbucket-dc"),
        data_center_note: None,
    },
    ServiceDef {
        name: "trello",
        title: "Trello: boards, lists, cards, members",
        family: Family::Trello,
        cloud: Some("trello-cloud"),
        data_center: None,
        data_center_note: Some("Trello exists only on Atlassian Cloud"),
    },
    ServiceDef {
        name: "admin",
        title: "Atlassian Admin: organizations, users, groups, policies, API access, data security",
        family: Family::Admin,
        cloud: Some("admin-cloud"),
        data_center: None,
        data_center_note: Some("the Atlassian Admin APIs exist only on Atlassian Cloud"),
    },
    ServiceDef {
        name: "admin-scim",
        title: "Atlassian Admin user provisioning (SCIM 2.0)",
        family: Family::AdminScim,
        cloud: Some("admin-scim-cloud"),
        data_center: None,
        data_center_note: Some("SCIM user provisioning exists only on Atlassian Cloud"),
    },
];

pub fn services() -> &'static [ServiceDef] {
    SERVICES
}

pub fn service(name: &str) -> Option<&'static ServiceDef> {
    SERVICES.iter().find(|service| service.name == name)
}

macro_rules! embedded {
    ($($key:literal),* $(,)?) => {
        /// The catalog and component schema documents for a key.
        fn embedded(key: &str) -> Option<(&'static str, &'static str)> {
            match key {
                $($key => Some((
                    include_str!(concat!(env!("OUT_DIR"), "/", $key, ".json")),
                    include_str!(concat!(env!("OUT_DIR"), "/", $key, ".schemas.json")),
                )),)*
                _ => None,
            }
        }
        /// Every catalog key, in registry order.
        pub(crate) const CATALOG_KEYS: &[&str] = &[$($key),*];
    };
}

embedded!(
    "jira-cloud",
    "jira-dc",
    "jira-software-cloud",
    "jira-software-dc",
    "jsm-cloud",
    "confluence-cloud",
    "confluence-dc",
    "confluence-v1-cloud",
    "bitbucket-cloud",
    "bitbucket-dc",
    "trello-cloud",
    "admin-cloud",
    "admin-scim-cloud",
);

include!(concat!(env!("OUT_DIR"), "/operation_counts.rs"));

/// The number of operations in a catalog, without parsing it.
pub fn operation_count(key: &str) -> Option<usize> {
    OPERATION_COUNTS
        .iter()
        .find(|(name, _)| *name == key)
        .map(|(_, count)| *count)
}

/// Returns the parsed catalog for a key such as `jira-cloud`. Each catalog is
/// parsed once per process.
///
/// # Panics
///
/// Panics when `key` is not in [`CATALOG_KEYS`] or the embedded catalog is
/// malformed; both are build defects, not user input.
pub fn catalog(key: &str) -> &'static Catalog {
    static CACHE: [OnceLock<Catalog>; CATALOG_KEYS.len()] =
        [const { OnceLock::new() }; CATALOG_KEYS.len()];
    let index = CATALOG_KEYS
        .iter()
        .position(|candidate| *candidate == key)
        .unwrap_or_else(|| panic!("unknown catalog key {key:?}"));
    CACHE[index].get_or_init(|| {
        let (text, schemas) = embedded(key).expect("every key has an embedded catalog");
        let mut catalog: Catalog = serde_json::from_str(text)
            .unwrap_or_else(|err| panic!("embedded catalog {key} is malformed: {err}"));
        catalog.key = CATALOG_KEYS[index];
        catalog.schema_text = schemas;
        catalog
    })
}

/// Index of a tag (CLI resource) inside its catalog.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
#[serde(transparent)]
pub struct TagId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum HttpMethod {
    Get,
    Put,
    Post,
    Delete,
    Patch,
    Head,
    Options,
}

impl HttpMethod {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Get => "GET",
            Self::Put => "PUT",
            Self::Post => "POST",
            Self::Delete => "DELETE",
            Self::Patch => "PATCH",
            Self::Head => "HEAD",
            Self::Options => "OPTIONS",
        }
    }

    /// True for methods that do not change server state.
    pub fn is_read_only(self) -> bool {
        matches!(self, Self::Get | Self::Head | Self::Options)
    }

    /// True when a request to `path` only reads: read-only methods, plus POST
    /// searches (Jira `/search`, `/search/jql`, `/jql/...`, `/expression/eval`).
    /// Read-only clients and dry runs let these through.
    pub fn is_safe_for(self, path: &str) -> bool {
        self.is_read_only()
            || (self == Self::Post
                && (path.ends_with("/search")
                    || path.contains("/search/")
                    || path.contains("/jql/")
                    || path.ends_with("/expression/eval")))
    }

    /// True when sending the request twice has the effect of sending it
    /// once, so a failed attempt can be retried.
    pub fn is_repeatable(self) -> bool {
        self.is_read_only() || matches!(self, Self::Put | Self::Delete)
    }
}

impl std::str::FromStr for HttpMethod {
    type Err = crate::Error;

    /// Case-insensitive: `get`, `GET`, ...
    fn from_str(text: &str) -> Result<Self, Self::Err> {
        serde_json::from_value(text.to_ascii_uppercase().into()).map_err(|_| {
            crate::Error::Invalid(format!(
                "unknown HTTP method `{text}`; use GET, POST, PUT, PATCH, DELETE, HEAD or OPTIONS"
            ))
        })
    }
}

impl fmt::Display for HttpMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ParamLocation {
    Path,
    Query,
    Header,
}

impl ParamLocation {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Path => "path",
            Self::Query => "query",
            Self::Header => "header",
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Tag {
    #[serde(borrow)]
    pub name: Cow<'static, str>,
    #[serde(borrow)]
    pub title: Cow<'static, str>,
    #[serde(borrow, default)]
    pub description: Option<Cow<'static, str>>,
}

#[derive(Debug, Deserialize)]
pub struct Param {
    /// The name the API uses (`issueIdOrKey`).
    #[serde(borrow)]
    pub name: Cow<'static, str>,
    /// The CLI flag without dashes (`issue-id-or-key`).
    #[serde(borrow)]
    pub flag: Cow<'static, str>,
    #[serde(rename = "in")]
    pub location: ParamLocation,
    pub required: bool,
    #[serde(borrow, default)]
    pub description: Option<Cow<'static, str>>,
    #[serde(borrow, default)]
    pub schema: Option<&'static RawValue>,
}

impl Param {
    /// The JSON Schema `type` of the parameter, when it is a plain string.
    pub fn schema_type(&self) -> Option<String> {
        #[derive(Deserialize)]
        struct TypeOnly {
            #[serde(rename = "type")]
            kind: Option<String>,
        }
        let schema = self.schema?;
        serde_json::from_str::<TypeOnly>(schema.get()).ok()?.kind
    }
}

#[derive(Debug, Deserialize)]
pub struct Body {
    #[serde(borrow)]
    pub content_type: Cow<'static, str>,
    pub required: bool,
    #[serde(borrow)]
    pub schema: &'static RawValue,
}

impl Body {
    pub fn is_multipart(&self) -> bool {
        self.content_type == "multipart/form-data"
    }
}

#[derive(Debug, Deserialize)]
pub struct Operation {
    /// The OpenAPI `operationId`, or a derived one (see build.rs).
    #[serde(borrow)]
    pub id: Cow<'static, str>,
    /// The CLI method name (`get-issue`).
    #[serde(borrow)]
    pub name: Cow<'static, str>,
    pub tag: TagId,
    pub method: HttpMethod,
    /// Path template relative to the connection base URL.
    #[serde(borrow)]
    pub path: Cow<'static, str>,
    #[serde(borrow, default)]
    pub summary: Option<Cow<'static, str>>,
    #[serde(borrow, default)]
    pub description: Option<Cow<'static, str>>,
    #[serde(default)]
    pub deprecated: bool,
    params: [u32; 2],
    #[serde(default)]
    pub body: Option<Body>,
    #[serde(borrow, default)]
    pub response: Option<&'static RawValue>,
}

impl Operation {
    /// True for reads and POST searches; read-only clients run only these.
    pub fn is_safe(&self) -> bool {
        self.method.is_safe_for(&self.path)
    }
}

#[derive(Debug, Deserialize)]
#[serde(bound(deserialize = "'de: 'static"))]
pub struct Catalog {
    pub tags: Vec<Tag>,
    pub ops: Vec<Operation>,
    pub params: Vec<Param>,
    #[serde(skip)]
    key: &'static str,
    /// The embedded component schemas document. Parsing it is most of the
    /// cost of loading a catalog, and only `$ref` lookups need it, so
    /// [`Catalog::schema`] parses it on first use.
    #[serde(skip)]
    schema_text: &'static str,
    #[serde(skip)]
    schemas: OnceLock<HashMap<&'static str, &'static RawValue>>,
}

impl Catalog {
    /// A component schema by name (the part after `#/components/schemas/`).
    ///
    /// # Panics
    ///
    /// Panics when the embedded schemas document is malformed, a build defect.
    pub fn schema(&self, name: &str) -> Option<&'static RawValue> {
        self.schemas
            .get_or_init(|| {
                serde_json::from_str(self.schema_text).unwrap_or_else(|err| {
                    panic!(
                        "embedded schemas of catalog {} are malformed: {err}",
                        self.key
                    )
                })
            })
            .get(name)
            .copied()
    }

    pub fn params_of(&self, op: &Operation) -> &[Param] {
        let [start, end] = op.params;
        &self.params[start as usize..end as usize]
    }

    pub fn tag(&self, id: TagId) -> &Tag {
        &self.tags[id.0 as usize]
    }

    /// Finds an operation by `operationId`, by CLI name, or by
    /// `<resource>.<method>`.
    pub fn find(&self, key: &str) -> Option<&Operation> {
        if let Some(op) = self.ops.iter().find(|op| op.id == key) {
            return Some(op);
        }
        if let Some((resource, method)) = key.split_once('.') {
            return self
                .ops
                .iter()
                .find(|op| op.name == method && self.tag(op.tag).name == resource);
        }
        let mut by_name = self.ops.iter().filter(|op| op.name == key);
        match (by_name.next(), by_name.next()) {
            (Some(op), None) => Some(op),
            _ => None,
        }
    }

    pub fn find_tag(&self, name: &str) -> Option<TagId> {
        self.tags
            .iter()
            .position(|tag| tag.name == name)
            .map(|index| TagId(index as u32))
    }

    pub fn ops_in(&self, tag: TagId) -> impl Iterator<Item = &Operation> {
        self.ops.iter().filter(move |op| op.tag == tag)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn operation_counts_match_the_catalogs() {
        for key in CATALOG_KEYS {
            assert_eq!(operation_count(key), Some(catalog(key).ops.len()), "{key}");
        }
        assert_eq!(operation_count("nope"), None);
    }

    #[test]
    fn every_registry_catalog_loads_with_operations() {
        for service in services() {
            for key in [service.cloud, service.data_center].into_iter().flatten() {
                let catalog = catalog(key);
                assert!(
                    catalog.ops.len() > 5,
                    "{key} has only {} operations",
                    catalog.ops.len()
                );
            }
        }
    }

    #[test]
    fn every_schema_reference_resolves() {
        fn refs(value: &serde_json::Value, out: &mut Vec<String>) {
            match value {
                serde_json::Value::Object(map) => {
                    if let Some(serde_json::Value::String(target)) = map.get("$ref") {
                        out.push(target.clone());
                    }
                    map.values().for_each(|child| refs(child, out));
                }
                serde_json::Value::Array(items) => items.iter().for_each(|child| refs(child, out)),
                _ => {}
            }
        }
        for key in CATALOG_KEYS {
            let catalog = catalog(key);
            let mut found = Vec::new();
            let raws = catalog
                .params
                .iter()
                .filter_map(|param| param.schema)
                .chain(
                    catalog
                        .ops
                        .iter()
                        .filter_map(|op| op.body.as_ref().map(|body| body.schema)),
                )
                .chain(catalog.ops.iter().filter_map(|op| op.response));
            for raw in raws {
                refs(&serde_json::from_str(raw.get()).unwrap(), &mut found);
            }
            assert!(!found.is_empty(), "{key} has no $ref");
            for target in found {
                let name = target.trim_start_matches("#/components/schemas/");
                assert!(
                    catalog.schema(name).is_some(),
                    "{key}: {target} does not resolve"
                );
            }
        }
    }

    #[test]
    fn names_are_unique_and_parameter_flags_do_not_collide() {
        for key in CATALOG_KEYS {
            let catalog = catalog(key);
            let mut ids = HashSet::new();
            let mut names = HashSet::new();
            for op in &catalog.ops {
                assert!(ids.insert(&op.id), "{key}: duplicate id {}", op.id);
                assert!(
                    names.insert((op.tag, &op.name)),
                    "{key}: duplicate name {} in {}",
                    op.name,
                    catalog.tag(op.tag).name
                );
                let mut flags = HashSet::new();
                for param in catalog.params_of(op) {
                    assert!(
                        flags.insert(&param.flag),
                        "{key} {}: duplicate flag {}",
                        op.id,
                        param.flag
                    );
                    assert!(!param.flag.is_empty());
                }
            }
        }
    }

    #[test]
    fn jira_get_issue_resolves_on_cloud_and_data_center() {
        let cloud = catalog("jira-cloud");
        let op = cloud.find("getIssue").expect("cloud getIssue");
        assert_eq!(op.path, "/rest/api/3/issue/{issueIdOrKey}");
        assert_eq!(cloud.tag(op.tag).name, "issues");
        assert_eq!(op.name, "get-issue");
        let key = cloud
            .params_of(op)
            .iter()
            .find(|p| p.name == "issueIdOrKey")
            .expect("param");
        assert_eq!(
            (key.flag.as_ref(), key.location, key.required),
            ("issue-id-or-key", ParamLocation::Path, true)
        );

        // The Data Center document mixes platform and Agile operations that
        // both define getIssue; the split keeps the platform one in `jira`.
        let dc = catalog("jira-dc");
        assert_eq!(
            dc.find("getIssue").expect("dc getIssue").path,
            "/rest/api/2/issue/{issueIdOrKey}"
        );
        assert!(dc.ops.iter().all(|op| !op.path.starts_with("/rest/agile/")));
        let agile = catalog("jira-software-dc");
        assert_eq!(
            agile.find("getIssue").expect("agile getIssue").path,
            "/rest/agile/1.0/issue/{issueIdOrKey}"
        );
        assert!(agile.find("board.get-all-boards").is_some());
    }

    #[test]
    fn derived_names_cover_documents_without_operation_ids_or_tags() {
        let bitbucket = catalog("bitbucket-cloud");
        let op = bitbucket
            .ops
            .iter()
            .find(|op| op.method == HttpMethod::Get && op.path == "/2.0/repositories/{workspace}")
            .expect("list repositories");
        assert_eq!(op.name, "list-repositories-in-a-workspace");

        let trello = catalog("trello-cloud");
        assert!(
            trello.find_tag("boards").is_some(),
            "trello groups by first path segment"
        );
    }

    #[test]
    fn prefixes_make_paths_relative_to_the_base_url() {
        assert!(
            catalog("confluence-cloud")
                .ops
                .iter()
                .all(|op| op.path.starts_with("/wiki/api/v2/"))
        );
        assert!(
            catalog("admin-cloud")
                .ops
                .iter()
                .all(|op| op.path.starts_with("/admin/") || op.path.starts_with("/users/"))
        );
        assert!(
            catalog("bitbucket-dc")
                .ops
                .iter()
                .all(|op| op.path.starts_with("/rest/"))
        );
    }
}
