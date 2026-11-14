//! Task-level Jira API that works the same on Cloud (`/rest/api/3`, rich
//! text as ADF, users by account ID) and Data Center (`/rest/api/2`, rich
//! text as wiki-markup strings, users by name).

use crate::catalog::HttpMethod;
use crate::client::Client;
use crate::config::{Deployment, Family};
use crate::error::{Choice, Error, Result};
use crate::json::{items, strings, text};
use crate::markup;
use crate::ql::quote;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use std::collections::BTreeMap;

const SUMMARY_FIELDS: &[&str] = &[
    "summary",
    "status",
    "issuetype",
    "assignee",
    "priority",
    "updated",
];

/// Issue search filters, turned into JQL by [`IssueQuery::to_jql`]. The
/// same struct is the `atl jira issue list` flags (feature `clap`) and the
/// `jira_issue_list` task arguments.
///
/// Values starting with `~` are negated (`~Done` means "not Done").
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(feature = "clap", derive(clap::Args))]
#[serde(default, deny_unknown_fields)]
pub struct IssueQuery {
    /// Project key (CLI default: [jira] project in .atl.toml).
    #[cfg_attr(feature = "clap", arg(short = 'p', long))]
    pub project: Option<String>,
    /// Statuses, for example `In Progress`; `~Done` excludes Done. Give several to match any of them.
    #[cfg_attr(feature = "clap", arg(short = 's', long))]
    pub status: Vec<String>,
    /// `me`, `x` (unassigned), or a user; ~ negates.
    #[cfg_attr(feature = "clap", arg(short = 'a', long))]
    pub assignee: Option<String>,
    /// `me` or a user; ~ negates.
    #[cfg_attr(feature = "clap", arg(short = 'r', long))]
    pub reporter: Option<String>,
    /// Issue types, for example Bug.
    #[serde(rename = "type")]
    #[cfg_attr(feature = "clap", arg(short = 't', long = "type"))]
    pub issue_type: Vec<String>,
    /// Labels; ~ negates.
    #[cfg_attr(feature = "clap", arg(short = 'l', long = "label"))]
    pub labels: Vec<String>,
    /// Priorities, for example High; ~ negates.
    #[cfg_attr(feature = "clap", arg(short = 'y', long))]
    pub priority: Vec<String>,
    /// Components; ~ negates.
    #[cfg_attr(feature = "clap", arg(short = 'C', long))]
    pub component: Vec<String>,
    /// Resolutions, for example Fixed or Unresolved; ~ negates.
    #[cfg_attr(feature = "clap", arg(short = 'R', long))]
    pub resolution: Vec<String>,
    /// Parent issue key (an epic on Cloud, or the parent of sub-tasks).
    #[cfg_attr(feature = "clap", arg(short = 'P', long))]
    pub parent: Option<String>,
    /// Only issues you watch.
    #[cfg_attr(feature = "clap", arg(short = 'w', long))]
    pub watching: bool,
    /// Issues you viewed recently, most recent first.
    #[cfg_attr(feature = "clap", arg(long))]
    pub history: bool,
    /// Sprint: current (open), next (future), closed, a sprint ID, or a sprint name.
    #[cfg_attr(feature = "clap", arg(long))]
    pub sprint: Option<String>,
    /// Created since: -7d, -1h, today, week, month, or a date (2026-01-31).
    #[cfg_attr(feature = "clap", arg(long, visible_alias = "created-after"))]
    pub created: Option<String>,
    /// Created before: -7d, today, week, month, or a date.
    #[cfg_attr(feature = "clap", arg(long))]
    pub created_before: Option<String>,
    /// Updated since: -7d, -1h, today, week, month, or a date.
    #[cfg_attr(feature = "clap", arg(long, visible_alias = "updated-after"))]
    pub updated: Option<String>,
    /// Updated before: -7d, today, week, month, or a date.
    #[cfg_attr(feature = "clap", arg(long))]
    pub updated_before: Option<String>,
    /// Full-text search over summary, description and comments.
    #[cfg_attr(feature = "clap", arg(long))]
    pub text: Option<String>,
    /// Extra JQL combined with AND; may end with ORDER BY.
    #[cfg_attr(feature = "clap", arg(short = 'q', long))]
    pub jql: Option<String>,
    /// Sort field (default updated), newest first unless `reverse`.
    #[cfg_attr(feature = "clap", arg(long))]
    pub order_by: Option<String>,
    /// Oldest first.
    #[cfg_attr(feature = "clap", arg(long))]
    pub reverse: bool,
    /// Issues per page, 1-100 (default 50).
    #[serde(default = "default_limit")]
    #[cfg_attr(feature = "clap", arg(long, default_value_t = 50))]
    pub limit: usize,
    /// `next_page_token` from the previous page.
    #[cfg_attr(feature = "clap", arg(long))]
    pub page_token: Option<String>,
}

pub(crate) fn default_limit() -> usize {
    50
}

fn user(value: &str) -> String {
    match value {
        "me" => "currentUser()".into(),
        other => quote(other),
    }
}

fn since(value: &str) -> String {
    match value {
        "today" => "startOfDay()".into(),
        "week" => "startOfWeek()".into(),
        "month" => "startOfMonth()".into(),
        "year" => "startOfYear()".into(),
        relative if relative.starts_with('-') => relative.into(),
        date => quote(date),
    }
}

impl IssueQuery {
    /// Builds JQL. With no filters at all it lists your unresolved issues,
    /// because Jira Cloud rejects unbounded searches.
    pub fn to_jql(&self) -> String {
        let mut clauses: Vec<String> = Vec::new();
        if let Some(project) = &self.project {
            clauses.push(format!("project = {}", quote(project)));
        }
        let mut list = |field: &str, values: &[String]| {
            let (negated, plain): (Vec<&String>, Vec<&String>) =
                values.iter().partition(|v| v.starts_with('~'));
            // `Unresolved` is a JQL keyword, not a resolution name.
            let value = |v: &str| match v.eq_ignore_ascii_case("unresolved") {
                true => "Unresolved".to_owned(),
                false => quote(v),
            };
            if !plain.is_empty() {
                let items: Vec<String> = plain.iter().map(|v| value(v)).collect();
                clauses.push(format!("{field} IN ({})", items.join(", ")));
            }
            if !negated.is_empty() {
                let items: Vec<String> = negated.iter().map(|v| value(&v[1..])).collect();
                clauses.push(format!("{field} NOT IN ({})", items.join(", ")));
            }
        };
        list("status", &self.status);
        list("issuetype", &self.issue_type);
        list("labels", &self.labels);
        list("priority", &self.priority);
        list("component", &self.component);
        list("resolution", &self.resolution);
        if let Some(parent) = &self.parent {
            clauses.push(format!("parent = {}", quote(parent)));
        }
        if self.watching {
            clauses.push("watcher = currentUser()".into());
        }
        if self.history {
            clauses.push("issuekey IN issueHistory()".into());
        }
        if let Some(sprint) = &self.sprint {
            clauses.push(match sprint.to_ascii_lowercase().as_str() {
                "current" | "open" | "active" => "sprint IN openSprints()".to_owned(),
                "next" | "future" => "sprint IN futureSprints()".to_owned(),
                "closed" => "sprint IN closedSprints()".to_owned(),
                id if !id.is_empty() && id.bytes().all(|b| b.is_ascii_digit()) => {
                    format!("sprint = {id}")
                }
                _ => format!("sprint = {}", quote(sprint)),
            });
        }
        for (field, value) in [("assignee", &self.assignee), ("reporter", &self.reporter)] {
            match value.as_deref() {
                None => {}
                Some("x") => clauses.push(format!("{field} IS EMPTY")),
                Some(negated) if negated.starts_with('~') => {
                    clauses.push(format!("{field} != {}", user(&negated[1..])))
                }
                Some(value) => clauses.push(format!("{field} = {}", user(value))),
            }
        }
        for (field, op, value) in [
            ("created", ">=", &self.created),
            ("created", "<", &self.created_before),
            ("updated", ">=", &self.updated),
            ("updated", "<", &self.updated_before),
        ] {
            if let Some(value) = value {
                clauses.push(format!("{field} {op} {}", since(value)));
            }
        }
        if let Some(text) = &self.text {
            clauses.push(format!("text ~ {}", quote(text)));
        }
        // A user ORDER BY must stay outside the parentheses to be valid JQL.
        let (filter, order) = match &self.jql {
            Some(jql) => match jql.to_ascii_lowercase().rfind("order by") {
                Some(at) => (jql[..at].trim(), Some(jql[at..].trim())),
                None => (jql.trim(), None),
            },
            None => ("", None),
        };
        if !filter.is_empty() {
            clauses.push(format!("({filter})"));
        }
        if clauses.is_empty() {
            clauses.push("assignee = currentUser() AND resolution = Unresolved".into());
        }
        let mut jql = clauses.join(" AND ");
        let default_order = if self.history {
            "lastViewed"
        } else {
            "updated"
        };
        match order {
            Some(order) => jql.push_str(&format!(" {order}")),
            None => jql.push_str(&format!(
                " ORDER BY {} {}",
                self.order_by.as_deref().unwrap_or(default_order),
                if self.reverse { "ASC" } else { "DESC" }
            )),
        }
        jql
    }
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct IssueSummary {
    pub key: String,
    pub summary: String,
    pub status: String,
    /// `new`, `indeterminate` (in progress), or `done`.
    pub status_category: String,
    #[serde(rename = "type")]
    pub issue_type: String,
    pub assignee: Option<String>,
    pub priority: Option<String>,
    pub updated: Option<String>,
    pub url: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct IssuePage {
    pub issues: Vec<IssueSummary>,
    /// Pass back as `page_token` for the next page.
    pub next_page_token: Option<String>,
    /// Total matches when the server reports it (Data Center).
    pub total: Option<u64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Comment {
    pub id: String,
    pub author: Option<String>,
    pub created: Option<String>,
    /// The comment as Markdown (Cloud) or wiki markup (Data Center).
    pub body: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct IssueDetail {
    #[serde(flatten)]
    pub summary: IssueSummary,
    pub reporter: Option<String>,
    pub created: Option<String>,
    pub labels: Vec<String>,
    pub components: Vec<String>,
    pub parent: Option<String>,
    pub subtasks: Vec<String>,
    /// The description as Markdown (Cloud) or wiki markup (Data Center).
    pub description: String,
    pub comments: Vec<Comment>,
    pub comment_count: usize,
    /// Links to other issues, as "relation key" (for example "blocks ABC-2").
    pub links: Vec<IssueLink>,
    /// Fields asked for by name (`issue view --field`), simplified: options
    /// become their value, users their name, rich text Markdown.
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub fields: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Default)]
pub struct NewIssue {
    pub project: String,
    pub issue_type: String,
    pub summary: String,
    /// Markdown; converted to ADF on Cloud.
    pub description: Option<String>,
    pub assignee: Option<String>,
    pub labels: Vec<String>,
    pub priority: Option<String>,
    pub parent: Option<String>,
    pub components: Vec<String>,
    pub fix_versions: Vec<String>,
    /// Versions the problem affects (Jira's `versions` field).
    pub affects_versions: Vec<String>,
    /// Original estimate, for example `2h` or `1d 4h`.
    pub estimate: Option<String>,
    /// `me` or a user (email or name).
    pub reporter: Option<String>,
    /// Other fields by name or ID. Text is converted by the field's type
    /// (see [`Jira::fields`]); other JSON is sent unchanged.
    pub fields: Vec<(String, Value)>,
}

#[derive(Debug, Clone, Default)]
pub struct IssueEdit {
    pub summary: Option<String>,
    pub description: Option<String>,
    pub priority: Option<String>,
    /// `me`, `x` (unassigned), or a user.
    pub assignee: Option<String>,
    pub parent: Option<String>,
    pub reporter: Option<String>,
    /// Original estimate, for example `2h`.
    pub estimate: Option<String>,
    pub add_labels: Vec<String>,
    pub remove_labels: Vec<String>,
    pub add_components: Vec<String>,
    pub remove_components: Vec<String>,
    pub add_fix_versions: Vec<String>,
    pub remove_fix_versions: Vec<String>,
    pub add_affects_versions: Vec<String>,
    pub remove_affects_versions: Vec<String>,
    /// Other fields by name or ID, as in [`NewIssue::fields`].
    pub fields: Vec<(String, Value)>,
    /// Do not email watchers about the change (Jira may need admin rights).
    pub skip_notify: bool,
}

impl IssueEdit {
    pub fn is_empty(&self) -> bool {
        self.summary.is_none()
            && self.description.is_none()
            && self.priority.is_none()
            && self.assignee.is_none()
            && self.parent.is_none()
            && self.reporter.is_none()
            && self.estimate.is_none()
            && self.add_labels.is_empty()
            && self.remove_labels.is_empty()
            && self.add_components.is_empty()
            && self.remove_components.is_empty()
            && self.add_fix_versions.is_empty()
            && self.remove_fix_versions.is_empty()
            && self.add_affects_versions.is_empty()
            && self.remove_affects_versions.is_empty()
            && self.fields.is_empty()
    }
}

/// Fields set together with a transition (they must be on its screen).
#[derive(Debug, Clone, Default)]
pub struct TransitionFields {
    pub resolution: Option<String>,
    /// `me`, `x` (unassigned), or a user.
    pub assignee: Option<String>,
}

/// What [`Jira::clone_issue`] changes in the copy.
#[derive(Debug, Clone, Default)]
pub struct CloneOptions {
    pub summary: Option<String>,
    pub priority: Option<String>,
    pub assignee: Option<String>,
    /// Replace the copied labels (when not empty).
    pub labels: Vec<String>,
    /// `(find, replace)` pairs applied to the summary and the description.
    pub replace: Vec<(String, String)>,
}

/// A project version ("release").
#[derive(Debug, Clone, Serialize)]
pub struct Release {
    pub id: String,
    pub name: String,
    pub released: bool,
    pub archived: bool,
    pub release_date: Option<String>,
    pub description: Option<String>,
}

/// `[{"name": X}, ...]`, the shape of components and versions.
fn named(values: &[String]) -> Value {
    values.iter().map(|v| json!({ "name": v })).collect()
}

/// Replaces text in every `text` string of an ADF document (or in a
/// wiki-markup string), keeping the formatting.
fn replace_text(value: &mut Value, find: &str, with: &str) {
    match value {
        Value::String(text) => *text = text.replace(find, with),
        Value::Array(items) => items.iter_mut().for_each(|v| replace_text(v, find, with)),
        Value::Object(map) => {
            for (key, child) in map.iter_mut() {
                match (key.as_str(), child) {
                    ("text", Value::String(text)) => *text = text.replace(find, with),
                    ("content", child) => replace_text(child, find, with),
                    _ => {}
                }
            }
        }
        _ => {}
    }
}

/// A system or custom field, from `GET /rest/api/{2|3}/field`.
#[derive(Debug, Clone, Serialize)]
pub struct Field {
    /// For example `customfield_10016` or `duedate`.
    pub id: String,
    pub name: String,
    pub custom: bool,
    /// JSON type: string, number, date, datetime, option, array, user, ...
    #[serde(rename = "type")]
    pub kind: Option<String>,
    /// Item type of an array field.
    pub items: Option<String>,
    /// The custom field type key, for example `...:textarea`.
    #[serde(skip)]
    pub custom_type: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Watchers {
    /// True when the signed-in user watches the issue.
    pub watching: bool,
    pub count: u64,
    pub watchers: Vec<User>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Created {
    pub key: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Transition {
    pub id: String,
    pub name: String,
    /// The status the transition leads to.
    pub to: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct User {
    /// Account ID (Cloud) or username (Data Center).
    pub id: String,
    pub name: String,
    pub email: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Project {
    pub key: String,
    pub name: String,
    pub id: String,
    #[serde(rename = "type")]
    pub project_type: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Board {
    pub id: u64,
    pub name: String,
    #[serde(rename = "type")]
    pub board_type: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Sprint {
    pub id: u64,
    pub name: String,
    pub state: String,
    pub start: Option<String>,
    pub end: Option<String>,
    pub goal: Option<String>,
}

pub struct Jira<'a> {
    client: &'a Client,
    cloud: bool,
}

impl<'a> Jira<'a> {
    pub fn new(client: &'a Client) -> Self {
        Self {
            client,
            cloud: client.deployment(Family::Jira) == Deployment::Cloud,
        }
    }

    fn api(&self) -> &'static str {
        if self.cloud {
            "/rest/api/3"
        } else {
            "/rest/api/2"
        }
    }

    /// The browser URL of an issue.
    pub fn browse_url(&self, key: &str) -> String {
        format!("{}/browse/{key}", self.client.site_url(Family::Jira))
    }

    async fn send(
        &self,
        method: HttpMethod,
        path: &str,
        query: &[(&str, String)],
        body: Option<Value>,
    ) -> Result<Value> {
        self.client
            .send_json(Family::Jira, method, path, query, body)
            .await
    }

    /// Rich text from the server as Markdown (Cloud ADF) or the raw string
    /// (Data Center wiki markup).
    fn rich_text(value: Option<&Value>) -> String {
        match value {
            Some(Value::String(text)) => markup::wiki_to_markdown(text),
            Some(adf @ Value::Object(_)) => markup::adf_to_markdown(adf),
            _ => String::new(),
        }
    }

    /// Markdown to the body format the server expects.
    fn body(&self, markdown: &str) -> Value {
        if self.cloud {
            markup::markdown_to_adf(markdown)
        } else {
            Value::String(markup::markdown_to_wiki(markdown))
        }
    }

    pub(crate) fn summary(&self, issue: &Value) -> IssueSummary {
        let key = text(issue, "/key").unwrap_or_default();
        IssueSummary {
            url: self.browse_url(&key),
            key,
            summary: text(issue, "/fields/summary").unwrap_or_default(),
            status: text(issue, "/fields/status/name").unwrap_or_default(),
            status_category: text(issue, "/fields/status/statusCategory/key").unwrap_or_default(),
            issue_type: text(issue, "/fields/issuetype/name").unwrap_or_default(),
            assignee: text(issue, "/fields/assignee/displayName"),
            priority: text(issue, "/fields/priority/name"),
            updated: text(issue, "/fields/updated"),
        }
    }

    /// One page of issues matching `jql`.
    pub async fn search(
        &self,
        jql: &str,
        limit: usize,
        page_token: Option<&str>,
    ) -> Result<IssuePage> {
        let limit = limit.clamp(1, 100);
        let response = if self.cloud {
            let mut body = json!({ "jql": jql, "fields": SUMMARY_FIELDS, "maxResults": limit });
            if let Some(token) = page_token {
                body["nextPageToken"] = token.into();
            }
            self.send(HttpMethod::Post, "/rest/api/3/search/jql", &[], Some(body))
                .await?
        } else {
            let start: u64 = match page_token {
                Some(token) => token.parse().map_err(|_| Error::Invalid(format!("page token `{token}` is not a number; pass the next_page_token of the previous page")))?,
                None => 0,
            };
            let body = json!({ "jql": jql, "fields": SUMMARY_FIELDS, "maxResults": limit, "startAt": start });
            self.send(HttpMethod::Post, "/rest/api/2/search", &[], Some(body))
                .await?
        };
        let issues: Vec<IssueSummary> = items(&response, "/issues")
            .map(|issue| self.summary(issue))
            .collect();
        let total = response.get("total").and_then(Value::as_u64);
        let next_page_token = if self.cloud {
            text(&response, "/nextPageToken")
        } else {
            let start = response.get("startAt").and_then(Value::as_u64).unwrap_or(0);
            let next = start + issues.len() as u64;
            (!issues.is_empty() && total.is_some_and(|total| next < total))
                .then(|| next.to_string())
        };
        Ok(IssuePage {
            issues,
            next_page_token,
            total,
        })
    }

    /// One issue with its description, the last `comments` comments, and
    /// the `extra` fields (names or IDs) in [`IssueDetail::fields`].
    pub async fn issue(&self, key: &str, comments: usize, extra: &[String]) -> Result<IssueDetail> {
        let mut fields = "summary,status,issuetype,assignee,priority,updated,created,reporter,labels,components,parent,subtasks,description,comment,issuelinks".to_owned();
        let mut wanted = Vec::new();
        if !extra.is_empty() {
            let all = self.fields(None).await?;
            for name in extra {
                let field = find_field(&all, name)?;
                fields.push(',');
                fields.push_str(&field.id);
                wanted.push((name.clone(), field.id.clone()));
            }
        }
        let issue = self
            .send(
                HttpMethod::Get,
                &format!("{}/issue/{key}", self.api()),
                &[("fields", fields)],
                None,
            )
            .await?;
        let all: Vec<Comment> = items(&issue, "/fields/comment/comments")
            .map(|c| Comment {
                id: text(c, "/id").unwrap_or_default(),
                author: text(c, "/author/displayName"),
                created: text(c, "/created"),
                body: Self::rich_text(c.get("body")),
            })
            .collect();
        let comment_count = issue
            .pointer("/fields/comment/total")
            .and_then(Value::as_u64)
            .map_or(all.len(), |t| t as usize);
        let skip = all.len().saturating_sub(comments);
        let names = |pointer: &str, field: &str| -> Vec<String> {
            items(&issue, pointer)
                .filter_map(|v| text(v, field))
                .collect()
        };
        Ok(IssueDetail {
            summary: self.summary(&issue),
            reporter: text(&issue, "/fields/reporter/displayName"),
            created: text(&issue, "/fields/created"),
            labels: strings(&issue, "/fields/labels"),
            components: names("/fields/components", "/name"),
            parent: text(&issue, "/fields/parent/key"),
            subtasks: names("/fields/subtasks", "/key"),
            description: Self::rich_text(issue.pointer("/fields/description")),
            comments: all.into_iter().skip(skip).collect(),
            comment_count,
            links: items(&issue, "/fields/issuelinks")
                .filter_map(|link| {
                    let (other, phrase) = match (link.get("outwardIssue"), link.get("inwardIssue"))
                    {
                        (Some(other), _) => (other, "/type/outward"),
                        (None, Some(other)) => (other, "/type/inward"),
                        (None, None) => return None,
                    };
                    Some(IssueLink {
                        relation: text(link, phrase)
                            .or_else(|| text(link, "/type/name"))
                            .unwrap_or_default(),
                        key: text(other, "/key").unwrap_or_default(),
                        summary: text(other, "/fields/summary"),
                        status: text(other, "/fields/status/name"),
                    })
                })
                .collect(),
            fields: wanted
                .into_iter()
                .map(|(name, id)| {
                    let value = issue
                        .pointer(&format!("/fields/{id}"))
                        .unwrap_or(&Value::Null);
                    (name, simplify(value))
                })
                .collect(),
        })
    }

    pub async fn create(&self, new: &NewIssue) -> Result<Created> {
        let mut fields = json!({
            "project": { "key": new.project },
            "issuetype": { "name": new.issue_type },
            "summary": new.summary,
        });
        if let Some(description) = &new.description {
            fields["description"] = self.body(description);
        }
        if !new.labels.is_empty() {
            fields["labels"] = json!(new.labels);
        }
        if let Some(priority) = &new.priority {
            fields["priority"] = json!({ "name": priority });
        }
        if let Some(parent) = &new.parent {
            fields["parent"] = json!({ "key": parent });
        }
        if let Some(assignee) = &new.assignee {
            fields["assignee"] = self
                .user_ref(assignee)
                .await
                .map_err(|e| e.for_arg("assignee"))?;
        }
        if let Some(reporter) = &new.reporter {
            fields["reporter"] = self
                .user_ref(reporter)
                .await
                .map_err(|e| e.for_arg("reporter"))?;
        }
        for (field, values) in [
            ("components", &new.components),
            ("fixVersions", &new.fix_versions),
            ("versions", &new.affects_versions),
        ] {
            if !values.is_empty() {
                fields[field] = named(values);
            }
        }
        if let Some(estimate) = &new.estimate {
            fields["timetracking"] = json!({ "originalEstimate": estimate });
        }
        if let Value::Object(map) = &mut fields {
            map.extend(self.field_values(&new.fields).await?);
        }
        let response = self
            .send(
                HttpMethod::Post,
                &format!("{}/issue", self.api()),
                &[],
                Some(json!({ "fields": fields })),
            )
            .await?;
        let key = text(&response, "/key").ok_or_else(|| {
            Error::Invalid(format!(
                "Jira created the issue but returned no key: {response}"
            ))
        })?;
        Ok(Created {
            url: self.browse_url(&key),
            key,
        })
    }

    pub async fn edit(&self, key: &str, edit: &IssueEdit) -> Result<()> {
        if edit.is_empty() {
            return Err(Error::Invalid(
                "nothing to change; pass --summary, --body, --priority, --assignee, --parent, --reporter, --estimate, --label, --component, --fix-version, --affects-version or --field NAME=VALUE"
                    .into(),
            ));
        }
        let mut fields = json!({});
        if let Some(summary) = &edit.summary {
            fields["summary"] = summary.clone().into();
        }
        if let Some(description) = &edit.description {
            fields["description"] = self.body(description);
        }
        if let Some(priority) = &edit.priority {
            fields["priority"] = json!({ "name": priority });
        }
        if let Some(assignee) = &edit.assignee {
            fields["assignee"] = self
                .user_ref(assignee)
                .await
                .map_err(|e| e.for_arg("assignee"))?;
        }
        if let Some(reporter) = &edit.reporter {
            fields["reporter"] = self
                .user_ref(reporter)
                .await
                .map_err(|e| e.for_arg("reporter"))?;
        }
        if let Some(parent) = &edit.parent {
            fields["parent"] = json!({ "key": parent });
        }
        if let Some(estimate) = &edit.estimate {
            fields["timetracking"] = json!({ "originalEstimate": estimate });
        }
        if let Value::Object(map) = &mut fields {
            map.extend(self.field_values(&edit.fields).await?);
        }
        let mut update = Map::new();
        let labels: Vec<Value> = edit
            .add_labels
            .iter()
            .map(|l| json!({ "add": l }))
            .chain(edit.remove_labels.iter().map(|l| json!({ "remove": l })))
            .collect();
        if !labels.is_empty() {
            update.insert("labels".into(), labels.into());
        }
        for (field, add, remove) in [
            ("components", &edit.add_components, &edit.remove_components),
            (
                "fixVersions",
                &edit.add_fix_versions,
                &edit.remove_fix_versions,
            ),
            (
                "versions",
                &edit.add_affects_versions,
                &edit.remove_affects_versions,
            ),
        ] {
            let ops: Vec<Value> = add
                .iter()
                .map(|v| json!({ "add": { "name": v } }))
                .chain(remove.iter().map(|v| json!({ "remove": { "name": v } })))
                .collect();
            if !ops.is_empty() {
                update.insert(field.into(), ops.into());
            }
        }
        let mut body = json!({ "fields": fields });
        if !update.is_empty() {
            body["update"] = Value::Object(update);
        }
        let query = match edit.skip_notify {
            true => vec![("notifyUsers", "false".to_owned())],
            false => Vec::new(),
        };
        self.send(
            HttpMethod::Put,
            &format!("{}/issue/{key}", self.api()),
            &query,
            Some(body),
        )
        .await?;
        Ok(())
    }

    /// `{"accountId": ...}` (Cloud) or `{"name": ...}` (Data Center) for
    /// `me`, `x`/`none` (nobody), or a search term.
    async fn user_ref(&self, user: &str) -> Result<Value> {
        let field = if self.cloud { "accountId" } else { "name" };
        Ok(json!({ field: self.user_id(user).await? }))
    }

    /// Users from a search response: account ID (Cloud) or username (Data
    /// Center), display name, email.
    fn users(&self, found: &Value) -> Vec<User> {
        found
            .as_array()
            .into_iter()
            .flatten()
            .map(|u| User {
                id: text(u, if self.cloud { "/accountId" } else { "/name" }).unwrap_or_default(),
                name: text(u, "/displayName").unwrap_or_default(),
                email: text(u, "/emailAddress"),
            })
            .filter(|u| !u.id.is_empty())
            .collect()
    }

    /// The account ID (Cloud) or username (Data Center) of `me`, `id:<id>`,
    /// a search term, or `None` for `x`/`none` (nobody). A term that matches
    /// several users and none of them exactly is [`Error::NeedsInput`].
    pub(crate) async fn user_id(&self, user: &str) -> Result<Option<String>> {
        let id = match user {
            "x" | "none" => return Ok(None),
            "me" => self.me().await?.id,
            id if id.starts_with("id:") => id[3..].to_owned(),
            query => {
                let path = format!("{}/user/search", self.api());
                let param = if self.cloud { "query" } else { "username" };
                let found = self
                    .send(HttpMethod::Get, &path, &[(param, query.to_owned())], None)
                    .await?;
                let users = self.users(&found);
                let exact: Vec<&User> = users
                    .iter()
                    .filter(|u| {
                        u.name.eq_ignore_ascii_case(query)
                            || u.id == query
                            || u.email
                                .as_deref()
                                .is_some_and(|e| e.eq_ignore_ascii_case(query))
                    })
                    .collect();
                match (users.as_slice(), exact.as_slice()) {
                    ([], _) => {
                        return Err(Error::Invalid(format!(
                            "no Jira user matches `{query}`; pass an email, a name, `me`, or `x` for unassigned"
                        )));
                    }
                    ([only], _) => only.id.clone(),
                    (_, [only]) => only.id.clone(),
                    (many, _) => {
                        return Err(Error::needs_input(
                            "user",
                            format!(
                                "`{query}` matches {} Jira users; pass one of them",
                                many.len()
                            ),
                            many.iter().map(user_choice).collect(),
                        ));
                    }
                }
            }
        };
        Ok(Some(id))
    }

    /// Users who can be assigned `key` (at most 50).
    pub async fn assignable(&self, key: &str) -> Result<Vec<User>> {
        let found = self
            .send(
                HttpMethod::Get,
                &format!("{}/user/assignable/search", self.api()),
                &[("issueKey", key.to_owned()), ("maxResults", "50".into())],
                None,
            )
            .await?;
        Ok(self.users(&found))
    }

    /// Issue type names of a project, without sub-task types.
    pub async fn issue_types(&self, project: &str) -> Result<Vec<String>> {
        let found = self
            .send(
                HttpMethod::Get,
                &format!("{}/project/{project}", self.api()),
                &[],
                None,
            )
            .await?;
        Ok(items(&found, "/issueTypes")
            .filter(|t| t.get("subtask").and_then(Value::as_bool) != Some(true))
            .filter_map(|t| text(t, "/name"))
            .collect())
    }

    /// Assigns `key`; an empty `user` is [`Error::NeedsInput`] listing the
    /// assignable users.
    pub async fn assign(&self, key: &str, user: &str) -> Result<()> {
        if user.trim().is_empty() {
            let mut choices = vec![
                Choice::new("me", "me (you)"),
                Choice::new("x", "x (unassigned)"),
            ];
            choices.extend(self.assignable(key).await?.iter().map(user_choice));
            return Err(Error::needs_input(
                "user",
                format!("pass a user to assign {key} to"),
                choices,
            ));
        }
        let body = self.user_ref(user).await?;
        self.send(
            HttpMethod::Put,
            &format!("{}/issue/{key}/assignee", self.api()),
            &[],
            Some(body),
        )
        .await?;
        Ok(())
    }

    pub async fn transitions(&self, key: &str) -> Result<Vec<Transition>> {
        let response = self
            .send(
                HttpMethod::Get,
                &format!("{}/issue/{key}/transitions", self.api()),
                &[],
                None,
            )
            .await?;
        Ok(items(&response, "/transitions")
            .map(|t| Transition {
                id: text(t, "/id").unwrap_or_default(),
                name: text(t, "/name").unwrap_or_default(),
                to: text(t, "/to/name").unwrap_or_default(),
            })
            .collect())
    }

    /// Moves the issue with the transition whose name or target status
    /// matches `to` (case-insensitive).
    pub async fn transition(
        &self,
        key: &str,
        to: &str,
        comment: Option<&str>,
        fields: &TransitionFields,
    ) -> Result<Transition> {
        let available = self.transitions(key).await?;
        let chosen = available
            .iter()
            .find(|t| t.to.eq_ignore_ascii_case(to) || t.name.eq_ignore_ascii_case(to))
            .cloned()
            .ok_or_else(|| {
                let names: Vec<String> = available.iter().map(|t| format!("{:?}", t.to)).collect();
                let names = if names.is_empty() {
                    "(none)".to_owned()
                } else {
                    names.join(", ")
                };
                let message = if to.trim().is_empty() {
                    format!("pass a status for {key}; it can move to: {names}")
                } else {
                    format!("{key} cannot move to `{to}`; it can move to: {names}")
                };
                Error::needs_input(
                    "status",
                    message,
                    available
                        .iter()
                        .map(|t| Choice::new(&t.to, format!("{} ({})", t.to, t.name)))
                        .collect(),
                )
            })?;
        let mut body = json!({ "transition": { "id": chosen.id } });
        if let Some(comment) = comment {
            body["update"] = json!({ "comment": [{ "add": { "body": self.body(comment) } }] });
        }
        if let Some(resolution) = &fields.resolution {
            body["fields"]["resolution"] = json!({ "name": resolution });
        }
        if let Some(assignee) = &fields.assignee {
            body["fields"]["assignee"] = self
                .user_ref(assignee)
                .await
                .map_err(|e| e.for_arg("assignee"))?;
        }
        self.send(
            HttpMethod::Post,
            &format!("{}/issue/{key}/transitions", self.api()),
            &[],
            Some(body),
        )
        .await?;
        Ok(chosen)
    }

    /// Adds a Markdown comment; `internal` makes it a Jira Service
    /// Management internal note (customers do not see it).
    pub async fn comment(&self, key: &str, markdown: &str, internal: bool) -> Result<Comment> {
        let mut body = json!({ "body": self.body(markdown) });
        if internal {
            body["properties"] =
                json!([{ "key": "sd.public.comment", "value": { "internal": true } }]);
        }
        let response = self
            .send(
                HttpMethod::Post,
                &format!("{}/issue/{key}/comment", self.api()),
                &[],
                Some(body),
            )
            .await?;
        Ok(Comment {
            id: text(&response, "/id").unwrap_or_default(),
            author: text(&response, "/author/displayName"),
            created: text(&response, "/created"),
            body: markdown.to_owned(),
        })
    }

    pub async fn me(&self) -> Result<User> {
        let me = self
            .send(
                HttpMethod::Get,
                &format!("{}/myself", self.api()),
                &[],
                None,
            )
            .await?;
        let id = if self.cloud {
            text(&me, "/accountId")
        } else {
            text(&me, "/name")
        };
        Ok(User {
            id: id.unwrap_or_default(),
            name: text(&me, "/displayName").unwrap_or_default(),
            email: text(&me, "/emailAddress"),
        })
    }

    pub async fn projects(&self) -> Result<Vec<Project>> {
        let response = if self.cloud {
            self.send(
                HttpMethod::Get,
                "/rest/api/3/project/search",
                &[("maxResults", "100".into())],
                None,
            )
            .await?
        } else {
            self.send(HttpMethod::Get, "/rest/api/2/project", &[], None)
                .await?
        };
        let items = response
            .get("values")
            .or(Some(&response))
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        Ok(items
            .iter()
            .map(|p| Project {
                key: text(p, "/key").unwrap_or_default(),
                name: text(p, "/name").unwrap_or_default(),
                id: text(p, "/id").unwrap_or_default(),
                project_type: text(p, "/projectTypeKey"),
            })
            .collect())
    }

    /// `board`, else the first board of `project` (or of any project).
    pub async fn board_or_first(&self, board: Option<u64>, project: Option<&str>) -> Result<u64> {
        if let Some(board) = board {
            return Ok(board);
        }
        self.boards(project).await?.first().map(|b| b.id).ok_or_else(|| {
            Error::Invalid("no board found; pass the board ID (list boards with `atl jira board list` or the task jira_board_list)".into())
        })
    }

    pub async fn boards(&self, project: Option<&str>) -> Result<Vec<Board>> {
        let mut query = vec![("maxResults", "50".to_owned())];
        if let Some(project) = project {
            query.push(("projectKeyOrId", project.to_owned()));
        }
        let response = self
            .send(HttpMethod::Get, "/rest/agile/1.0/board", &query, None)
            .await?;
        Ok(items(&response, "/values")
            .map(|b| Board {
                id: b.get("id").and_then(Value::as_u64).unwrap_or_default(),
                name: text(b, "/name").unwrap_or_default(),
                board_type: text(b, "/type"),
            })
            .collect())
    }

    /// Sprints of a board; `state` is `active`, `future`, `closed`, or a
    /// comma-separated mix.
    /// Sprints of a board, oldest first, following pagination (at most 20
    /// pages of 50).
    pub async fn sprints(&self, board: u64, state: Option<&str>) -> Result<Vec<Sprint>> {
        let mut sprints = Vec::new();
        for _ in 0..20 {
            let mut query = vec![
                ("maxResults", "50".to_owned()),
                ("startAt", sprints.len().to_string()),
            ];
            if let Some(state) = state {
                query.push(("state", state.to_owned()));
            }
            let response = self
                .send(
                    HttpMethod::Get,
                    &format!("/rest/agile/1.0/board/{board}/sprint"),
                    &query,
                    None,
                )
                .await?;
            let page: Vec<Sprint> = items(&response, "/values").map(Self::sprint_from).collect();
            let done =
                page.is_empty() || response.get("isLast").and_then(Value::as_bool) != Some(false);
            sprints.extend(page);
            if done {
                break;
            }
        }
        Ok(sprints)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct JqlCheck {
    pub valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct JqlField {
    /// The name to use in JQL.
    pub name: String,
    pub display: String,
    pub operators: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct JqlReference {
    pub fields: Vec<JqlField>,
    pub functions: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Suggestion {
    pub value: String,
    pub display: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Filter {
    pub id: String,
    pub name: String,
    pub jql: String,
    pub owner: Option<String>,
    pub favourite: bool,
    pub url: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct LinkType {
    pub name: String,
    /// Phrase from the first issue to the second ("blocks").
    pub outward: String,
    /// Phrase from the second issue to the first ("is blocked by").
    pub inward: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Attachment {
    pub id: String,
    pub filename: String,
    pub size: u64,
    pub mime_type: Option<String>,
    pub created: Option<String>,
    pub author: Option<String>,
    /// Download URL on the Jira server.
    pub content: String,
}

fn strip_tags(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut in_tag = false;
    for c in text.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            c if !in_tag => out.push(c),
            _ => {}
        }
    }
    out
}

impl Jira<'_> {
    /// Validates JQL with Jira's parser. Data Center has no parse endpoint,
    /// so an empty search reports the same errors there.
    pub async fn check_jql(&self, jql: &str) -> Result<JqlCheck> {
        if self.cloud {
            let response = self
                .send(
                    HttpMethod::Post,
                    "/rest/api/3/jql/parse",
                    &[("validation", "strict".into())],
                    Some(json!({ "queries": [jql] })),
                )
                .await?;
            let query = response.pointer("/queries/0").cloned().unwrap_or_default();
            let list = |key: &str| strings(&query, &format!("/{key}"));
            let errors = list("errors");
            return Ok(JqlCheck {
                valid: errors.is_empty(),
                errors,
                warnings: list("warnings"),
            });
        }
        match self
            .send(
                HttpMethod::Post,
                "/rest/api/2/search",
                &[],
                Some(json!({ "jql": jql, "maxResults": 0, "validateQuery": true })),
            )
            .await
        {
            Ok(response) => Ok(JqlCheck {
                valid: true,
                errors: Vec::new(),
                warnings: strings(&response, "/warningMessages"),
            }),
            Err(Error::Http(http)) if http.status == 400 => {
                let errors = http
                    .body
                    .as_ref()
                    .and_then(|b| b.get("errorMessages"))
                    .and_then(Value::as_array)
                    .map(|items| {
                        items
                            .iter()
                            .filter_map(Value::as_str)
                            .map(str::to_owned)
                            .collect()
                    })
                    .unwrap_or_else(|| vec![http.message.clone()]);
                Ok(JqlCheck {
                    valid: false,
                    errors,
                    warnings: Vec::new(),
                })
            }
            Err(err) => Err(err),
        }
    }

    /// Number of issues matching `jql` (approximate on Cloud).
    pub async fn count(&self, jql: &str) -> Result<u64> {
        let response = if self.cloud {
            self.send(
                HttpMethod::Post,
                "/rest/api/3/search/approximate-count",
                &[],
                Some(json!({ "jql": jql })),
            )
            .await?
        } else {
            self.send(
                HttpMethod::Post,
                "/rest/api/2/search",
                &[],
                Some(json!({ "jql": jql, "maxResults": 0 })),
            )
            .await?
        };
        Ok(response
            .get("count")
            .or_else(|| response.get("total"))
            .and_then(Value::as_u64)
            .unwrap_or(0))
    }

    /// Fields and functions usable in JQL on this site.
    pub async fn jql_reference(&self) -> Result<JqlReference> {
        let response = self
            .send(
                HttpMethod::Get,
                &format!("{}/jql/autocompletedata", self.api()),
                &[],
                None,
            )
            .await?;
        let fields = items(&response, "/visibleFieldNames")
            .map(|f| JqlField {
                name: text(f, "/value").unwrap_or_default(),
                display: strip_tags(&text(f, "/displayName").unwrap_or_default()),
                operators: strings(f, "/operators"),
            })
            .collect();
        let functions = items(&response, "/visibleFunctionNames")
            .filter_map(|f| text(f, "/value"))
            .collect();
        Ok(JqlReference { fields, functions })
    }

    /// Value suggestions for a field, for example `status` and `In`.
    pub async fn suggest(&self, field: &str, prefix: &str) -> Result<Vec<Suggestion>> {
        let response = self
            .send(
                HttpMethod::Get,
                &format!("{}/jql/autocompletedata/suggestions", self.api()),
                &[
                    ("fieldName", field.to_owned()),
                    ("fieldValue", prefix.to_owned()),
                ],
                None,
            )
            .await?;
        Ok(items(&response, "/results")
            .map(|r| Suggestion {
                value: text(r, "/value").unwrap_or_default(),
                display: strip_tags(&text(r, "/displayName").unwrap_or_default()),
            })
            .collect())
    }

    fn filter_from(&self, f: &Value) -> Filter {
        let id = text(f, "/id").unwrap_or_default();
        Filter {
            url: format!("{}/issues/?filter={id}", self.client.site_url(Family::Jira)),
            id,
            name: text(f, "/name").unwrap_or_default(),
            jql: text(f, "/jql").unwrap_or_default(),
            owner: text(f, "/owner/displayName"),
            favourite: f
                .get("favourite")
                .and_then(Value::as_bool)
                .unwrap_or(!self.cloud),
        }
    }

    /// Saved filters: on Cloud all filters you can see (optionally by name),
    /// on Data Center your favourite filters.
    pub async fn filters(&self, name: Option<&str>) -> Result<Vec<Filter>> {
        let items: Vec<Value> = if self.cloud {
            let mut query = vec![
                ("expand", "jql,owner,favourite".to_owned()),
                ("maxResults", "100".to_owned()),
            ];
            if let Some(name) = name {
                query.push(("filterName", name.to_owned()));
            }
            let response = self
                .send(HttpMethod::Get, "/rest/api/3/filter/search", &query, None)
                .await?;
            items(&response, "/values").cloned().collect()
        } else {
            let response = self
                .send(HttpMethod::Get, "/rest/api/2/filter/favourite", &[], None)
                .await?;
            items(&response, "").cloned().collect()
        };
        let needle = name.map(str::to_lowercase);
        Ok(items
            .iter()
            .map(|f| self.filter_from(f))
            .filter(|f| {
                needle
                    .as_ref()
                    .is_none_or(|n| f.name.to_lowercase().contains(n))
            })
            .collect())
    }

    /// One filter by numeric ID or exact (case-insensitive) name.
    pub async fn filter(&self, id_or_name: &str) -> Result<Filter> {
        if id_or_name.bytes().all(|b| b.is_ascii_digit()) {
            let response = self
                .send(
                    HttpMethod::Get,
                    &format!("{}/filter/{id_or_name}", self.api()),
                    &[],
                    None,
                )
                .await?;
            return Ok(self.filter_from(&response));
        }
        let found = self.filters(Some(id_or_name)).await?;
        if let Some(exact) = found
            .iter()
            .find(|f| f.name.eq_ignore_ascii_case(id_or_name))
        {
            return Ok(exact.clone());
        }
        if found.is_empty() {
            return Err(Error::Invalid(format!(
                "no filter named `{id_or_name}`; list them with `atl jira filter list`"
            )));
        }
        Err(Error::needs_input(
            "filter",
            format!("no filter is named exactly `{id_or_name}`; pass one of the similar filters"),
            found
                .iter()
                .take(20)
                .map(|f| Choice::new(&f.id, &f.name))
                .collect(),
        ))
    }

    pub async fn link_types(&self) -> Result<Vec<LinkType>> {
        let response = self
            .send(
                HttpMethod::Get,
                &format!("{}/issueLinkType", self.api()),
                &[],
                None,
            )
            .await?;
        Ok(items(&response, "/issueLinkTypes")
            .map(|t| LinkType {
                name: text(t, "/name").unwrap_or_default(),
                outward: text(t, "/outward").unwrap_or_default(),
                inward: text(t, "/inward").unwrap_or_default(),
            })
            .collect())
    }

    /// Links `from` and `to` so that "`from` <kind> `to`" reads true. `kind`
    /// is a link type name ("Blocks"), its outward phrase ("blocks"), or its
    /// inward phrase ("is blocked by", which swaps the issues).
    pub async fn link(&self, from: &str, to: &str, kind: &str) -> Result<LinkType> {
        let types = self.link_types().await?;
        let (chosen, swap) = types
            .iter()
            .find_map(|t| {
                if t.name.eq_ignore_ascii_case(kind) || t.outward.eq_ignore_ascii_case(kind) {
                    Some((t.clone(), false))
                } else if t.inward.eq_ignore_ascii_case(kind) {
                    Some((t.clone(), true))
                } else {
                    None
                }
            })
            .ok_or_else(|| {
                Error::needs_input(
                    "kind",
                    format!("unknown link type `{kind}`; pass a type name, its outward phrase or its inward phrase"),
                    types
                        .iter()
                        .map(|t| Choice::new(&t.name, format!("{} ({} / {})", t.name, t.outward, t.inward)))
                        .collect(),
                )
            })?;
        let (outward, inward) = if swap { (to, from) } else { (from, to) };
        let body = json!({ "type": { "name": chosen.name }, "outwardIssue": { "key": outward }, "inwardIssue": { "key": inward } });
        self.send(
            HttpMethod::Post,
            &format!("{}/issueLink", self.api()),
            &[],
            Some(body),
        )
        .await?;
        Ok(chosen)
    }

    /// Creates a copy of an issue (summary, description, type, labels,
    /// priority, components, parent) in the same project, with `options`
    /// applied.
    pub async fn clone_issue(&self, key: &str, options: &CloneOptions) -> Result<Created> {
        let fields = "summary,description,issuetype,project,labels,priority,components,parent";
        let issue = self
            .send(
                HttpMethod::Get,
                &format!("{}/issue/{key}", self.api()),
                &[("fields", fields.into())],
                None,
            )
            .await?;
        let source = issue.get("fields").cloned().unwrap_or_default();
        let mut summary = options
            .summary
            .clone()
            .or_else(|| text(&source, "/summary"))
            .unwrap_or_default();
        let mut description = source.get("description").filter(|v| !v.is_null()).cloned();
        for (find, with) in &options.replace {
            summary = summary.replace(find, with);
            if let Some(description) = &mut description {
                replace_text(description, find, with);
            }
        }
        let mut fields = json!({
            "project": { "key": text(&source, "/project/key") },
            "issuetype": { "name": text(&source, "/issuetype/name") },
            "summary": summary,
        });
        if let Some(description) = description {
            fields["description"] = description;
        }
        match options.labels.is_empty() {
            false => fields["labels"] = json!(options.labels),
            true => {
                if let Some(labels) = source.get("labels").filter(|v| !v.is_null()) {
                    fields["labels"] = labels.clone();
                }
            }
        }
        if let Some(priority) = options
            .priority
            .clone()
            .or_else(|| text(&source, "/priority/name"))
        {
            fields["priority"] = json!({ "name": priority });
        }
        if let Some(assignee) = &options.assignee {
            fields["assignee"] = self
                .user_ref(assignee)
                .await
                .map_err(|e| e.for_arg("assignee"))?;
        }
        if let Some(parent) = text(&source, "/parent/key") {
            fields["parent"] = json!({ "key": parent });
        }
        let components: Vec<Value> = items(&source, "/components")
            .filter_map(|c| text(c, "/name"))
            .map(|n| json!({ "name": n }))
            .collect();
        if !components.is_empty() {
            fields["components"] = Value::Array(components);
        }
        let response = self
            .send(
                HttpMethod::Post,
                &format!("{}/issue", self.api()),
                &[],
                Some(json!({ "fields": fields })),
            )
            .await?;
        let new_key = text(&response, "/key").ok_or_else(|| {
            Error::Invalid(format!(
                "Jira created the clone but returned no key: {response}"
            ))
        })?;
        Ok(Created {
            url: self.browse_url(&new_key),
            key: new_key,
        })
    }

    /// Logs time such as `2h 30m` or `1d`, optionally with a Markdown comment
    /// and a start time (`2026-11-09T09:00:00.000+0000`).
    pub async fn worklog(
        &self,
        key: &str,
        time_spent: &str,
        comment: Option<&str>,
        started: Option<&str>,
    ) -> Result<Value> {
        let mut body = json!({ "timeSpent": time_spent });
        if let Some(comment) = comment {
            body["comment"] = self.body(comment);
        }
        if let Some(started) = started {
            body["started"] = started.into();
        }
        let response = self
            .send(
                HttpMethod::Post,
                &format!("{}/issue/{key}/worklog", self.api()),
                &[],
                Some(body),
            )
            .await?;
        Ok(json!({ "key": key, "worklog_id": text(&response, "/id"), "time_spent": time_spent }))
    }

    /// Open epics of a project.
    pub async fn epics(&self, project: Option<&str>, limit: usize) -> Result<IssuePage> {
        let mut jql = "issuetype = Epic AND statusCategory != Done".to_owned();
        if let Some(project) = project {
            jql = format!("project = {} AND {jql}", quote(project));
        }
        jql.push_str(" ORDER BY created DESC");
        self.search(&jql, limit, None).await
    }

    /// Moves issues into an epic, or out of any epic when `epic` is `None`.
    pub async fn set_epic(&self, epic: Option<&str>, issues: &[String]) -> Result<()> {
        let target = epic.unwrap_or("none");
        self.send(
            HttpMethod::Post,
            &format!("/rest/agile/1.0/epic/{target}/issue"),
            &[],
            Some(json!({ "issues": issues })),
        )
        .await?;
        Ok(())
    }

    pub async fn sprint_add(&self, sprint: u64, issues: &[String]) -> Result<()> {
        self.send(
            HttpMethod::Post,
            &format!("/rest/agile/1.0/sprint/{sprint}/issue"),
            &[],
            Some(json!({ "issues": issues })),
        )
        .await?;
        Ok(())
    }

    fn attachment_from(a: &Value) -> Attachment {
        Attachment {
            id: text(a, "/id").unwrap_or_default(),
            filename: text(a, "/filename").unwrap_or_default(),
            size: a.get("size").and_then(Value::as_u64).unwrap_or(0),
            mime_type: text(a, "/mimeType"),
            created: text(a, "/created"),
            author: text(a, "/author/displayName"),
            content: text(a, "/content").unwrap_or_default(),
        }
    }

    pub async fn attachments(&self, key: &str) -> Result<Vec<Attachment>> {
        let issue = self
            .send(
                HttpMethod::Get,
                &format!("{}/issue/{key}", self.api()),
                &[("fields", "attachment".into())],
                None,
            )
            .await?;
        Ok(items(&issue, "/fields/attachment")
            .map(Self::attachment_from)
            .collect())
    }

    pub async fn attach(&self, key: &str, files: &[std::path::PathBuf]) -> Result<Vec<Attachment>> {
        let response = self
            .client
            .upload(
                Family::Jira,
                &format!("{}/issue/{key}/attachments", self.api()),
                files,
            )
            .await?;
        Ok(response
            .as_array()
            .into_iter()
            .flatten()
            .map(Self::attachment_from)
            .collect())
    }

    /// The attachment's metadata and bytes.
    pub async fn download(&self, id: &str) -> Result<(Attachment, Vec<u8>)> {
        let meta = Self::attachment_from(
            &self
                .send(
                    HttpMethod::Get,
                    &format!("{}/attachment/{id}", self.api()),
                    &[],
                    None,
                )
                .await?,
        );
        let bytes = self
            .client
            .download(
                Family::Jira,
                &format!("{}/attachment/content/{id}", self.api()),
            )
            .await?;
        Ok((meta, bytes))
    }
}

impl Jira<'_> {
    /// System and custom fields whose name or ID contains `filter`
    /// (case-insensitive), sorted by name.
    pub async fn fields(&self, filter: Option<&str>) -> Result<Vec<Field>> {
        let response = self
            .send(HttpMethod::Get, &format!("{}/field", self.api()), &[], None)
            .await?;
        let filter = filter.map(str::to_lowercase);
        let mut fields: Vec<Field> = response
            .as_array()
            .into_iter()
            .flatten()
            .map(|f| Field {
                id: text(f, "/id").unwrap_or_default(),
                name: text(f, "/name").unwrap_or_default(),
                custom: f.get("custom").and_then(Value::as_bool).unwrap_or(false),
                kind: text(f, "/schema/type"),
                items: text(f, "/schema/items"),
                custom_type: text(f, "/schema/custom"),
            })
            .filter(|f| {
                filter.as_ref().is_none_or(|filter| {
                    f.name.to_lowercase().contains(filter) || f.id.to_lowercase().contains(filter)
                })
            })
            .collect();
        fields.sort_by_cached_key(|f| f.name.to_lowercase());
        Ok(fields)
    }

    /// `fields` keyed by field ID with values in the server's format.
    async fn field_values(&self, values: &[(String, Value)]) -> Result<Map<String, Value>> {
        let mut out = Map::new();
        if values.is_empty() {
            return Ok(out);
        }
        let all = self.fields(None).await?;
        for (name, value) in values {
            let field = find_field(&all, name)?;
            out.insert(field.id.clone(), self.field_value(field, value).await?);
        }
        Ok(out)
    }

    /// Converts text by the field's type; JSON other than text (or text
    /// that is a JSON object or array) is sent unchanged.
    async fn field_value(&self, field: &Field, value: &Value) -> Result<Value> {
        let Value::String(text) = value else {
            return Ok(value.clone());
        };
        let trimmed = text.trim();
        if (trimmed.starts_with('{') || trimmed.starts_with('['))
            && let Ok(json) = serde_json::from_str::<Value>(trimmed)
        {
            return Ok(json);
        }
        let item = |kind: Option<&str>, part: &str| -> Option<Value> {
            match kind {
                Some("option") => Some(json!({ "value": part })),
                Some("priority" | "version" | "component" | "issuetype") => {
                    Some(json!({ "name": part }))
                }
                Some("project") => Some(json!({ "key": part })),
                Some("user") => None,
                _ => Some(Value::String(part.to_owned())),
            }
        };
        match field.kind.as_deref() {
            Some("number") => trimmed
                .parse::<f64>()
                .ok()
                .and_then(serde_json::Number::from_f64)
                .map(Value::Number)
                .ok_or_else(|| {
                    Error::Invalid(format!(
                        "field `{}` is a number and `{text}` is not; pass a number such as 3 or 2.5",
                        field.name
                    ))
                }),
            Some("user") => self.user_ref(trimmed).await,
            Some("string")
                if self.cloud
                    && field
                        .custom_type
                        .as_deref()
                        .is_some_and(|t| t.ends_with(":textarea")) =>
            {
                Ok(self.body(text))
            }
            Some("array") => {
                let mut items = Vec::new();
                for part in trimmed.split(',').map(str::trim).filter(|p| !p.is_empty()) {
                    items.push(match item(field.items.as_deref(), part) {
                        Some(value) => value,
                        None => self.user_ref(part).await?,
                    });
                }
                Ok(Value::Array(items))
            }
            kind => Ok(item(kind, trimmed).unwrap_or_else(|| Value::String(text.clone()))),
        }
    }

    /// Deletes an issue; `subtasks` also deletes its sub-tasks (Jira refuses
    /// to delete an issue with sub-tasks otherwise).
    pub async fn delete(&self, key: &str, subtasks: bool) -> Result<()> {
        let query = if subtasks {
            vec![("deleteSubtasks", "true".to_owned())]
        } else {
            Vec::new()
        };
        self.send(
            HttpMethod::Delete,
            &format!("{}/issue/{key}", self.api()),
            &query,
            None,
        )
        .await?;
        Ok(())
    }

    pub async fn watchers(&self, key: &str) -> Result<Watchers> {
        let response = self
            .send(
                HttpMethod::Get,
                &format!("{}/issue/{key}/watchers", self.api()),
                &[],
                None,
            )
            .await?;
        Ok(Watchers {
            watching: response
                .get("isWatching")
                .and_then(Value::as_bool)
                .unwrap_or(false),
            count: response
                .get("watchCount")
                .and_then(Value::as_u64)
                .unwrap_or_default(),
            watchers: items(&response, "/watchers")
                .map(|w| User {
                    id: text(w, if self.cloud { "/accountId" } else { "/name" })
                        .unwrap_or_default(),
                    name: text(w, "/displayName").unwrap_or_default(),
                    email: text(w, "/emailAddress"),
                })
                .collect(),
        })
    }

    async fn watcher_id(&self, user: &str) -> Result<String> {
        self.user_id(user).await?.ok_or_else(|| {
            Error::Invalid(format!(
                "`{user}` is not a user; pass `me`, an email, or a name"
            ))
        })
    }

    /// Adds `user` (`me`, an email, or a name) as a watcher.
    pub async fn watch(&self, key: &str, user: &str) -> Result<()> {
        let id = self.watcher_id(user).await?;
        self.send(
            HttpMethod::Post,
            &format!("{}/issue/{key}/watchers", self.api()),
            &[],
            Some(Value::String(id)),
        )
        .await?;
        Ok(())
    }

    /// Removes `user` from the watchers.
    pub async fn unwatch(&self, key: &str, user: &str) -> Result<()> {
        let id = self.watcher_id(user).await?;
        let param = if self.cloud { "accountId" } else { "username" };
        self.send(
            HttpMethod::Delete,
            &format!("{}/issue/{key}/watchers", self.api()),
            &[(param, id)],
            None,
        )
        .await?;
        Ok(())
    }
}

impl Jira<'_> {
    /// Removes the links between `from` and `to` (only those of type `kind`
    /// when given: a type name, outward or inward phrase); returns the
    /// removed link type names.
    pub async fn unlink(&self, from: &str, to: &str, kind: Option<&str>) -> Result<Vec<String>> {
        let issue = self
            .send(
                HttpMethod::Get,
                &format!("{}/issue/{from}", self.api()),
                &[("fields", "issuelinks".into())],
                None,
            )
            .await?;
        let matches_kind = |link: &Value| {
            kind.is_none_or(|kind| {
                ["/type/name", "/type/inward", "/type/outward"]
                    .iter()
                    .any(|p| text(link, p).is_some_and(|t| t.eq_ignore_ascii_case(kind)))
            })
        };
        let links: Vec<(String, String)> = items(&issue, "/fields/issuelinks")
            .filter(|link| {
                ["/inwardIssue/key", "/outwardIssue/key"]
                    .iter()
                    .any(|p| text(link, p).is_some_and(|k| k.eq_ignore_ascii_case(to)))
            })
            .filter(|link| matches_kind(link))
            .filter_map(|link| {
                Some((
                    text(link, "/id")?,
                    text(link, "/type/name").unwrap_or_default(),
                ))
            })
            .collect();
        if links.is_empty() {
            return Err(Error::Invalid(format!(
                "{from} and {to} are not linked{}; see the links with `atl api jira get {}/issue/{from}?fields=issuelinks`",
                kind.map(|k| format!(" by `{k}`")).unwrap_or_default(),
                self.api()
            )));
        }
        for (id, _) in &links {
            self.send(
                HttpMethod::Delete,
                &format!("{}/issueLink/{id}", self.api()),
                &[],
                None,
            )
            .await?;
        }
        Ok(links.into_iter().map(|(_, name)| name).collect())
    }

    /// Adds a web link ("remote link") to an issue.
    pub async fn remote_link(&self, key: &str, url: &str, title: Option<&str>) -> Result<Value> {
        let title = title.unwrap_or(url);
        let response = self
            .send(
                HttpMethod::Post,
                &format!("{}/issue/{key}/remotelink", self.api()),
                &[],
                Some(json!({ "object": { "url": url, "title": title } })),
            )
            .await?;
        Ok(json!({ "key": key, "id": response.get("id"), "url": url, "title": title }))
    }

    /// The versions ("releases") of a project, in server order.
    pub async fn releases(&self, project: &str) -> Result<Vec<Release>> {
        let parse = |v: &Value| Release {
            id: text(v, "/id").unwrap_or_default(),
            name: text(v, "/name").unwrap_or_default(),
            released: v.get("released").and_then(Value::as_bool).unwrap_or(false),
            archived: v.get("archived").and_then(Value::as_bool).unwrap_or(false),
            release_date: text(v, "/releaseDate"),
            description: text(v, "/description"),
        };
        if !self.cloud {
            let found = self
                .send(
                    HttpMethod::Get,
                    &format!("/rest/api/2/project/{project}/versions"),
                    &[],
                    None,
                )
                .await?;
            return Ok(found.as_array().into_iter().flatten().map(parse).collect());
        }
        let mut releases = Vec::new();
        for _ in 0..50 {
            let page = self
                .send(
                    HttpMethod::Get,
                    &format!("/rest/api/3/project/{project}/version"),
                    &[
                        ("startAt", releases.len().to_string()),
                        ("maxResults", "100".into()),
                    ],
                    None,
                )
                .await?;
            let values: Vec<Release> = items(&page, "/values").map(parse).collect();
            let last =
                values.is_empty() || page.get("isLast").and_then(Value::as_bool) != Some(false);
            releases.extend(values);
            if last {
                break;
            }
        }
        Ok(releases)
    }
}

/// `id:<id>` with "Name <email>" as the label.
fn user_choice(user: &User) -> Choice {
    let label = match &user.email {
        Some(email) => format!("{} <{email}>", user.name),
        None => user.name.clone(),
    };
    Choice::new(format!("id:{}", user.id), label)
}

/// The field with this ID, or the one field with this name (case-insensitive).
fn find_field<'f>(fields: &'f [Field], name: &str) -> Result<&'f Field> {
    if let Some(field) = fields.iter().find(|f| f.id == name) {
        return Ok(field);
    }
    let matches: Vec<&Field> = fields
        .iter()
        .filter(|f| f.name.eq_ignore_ascii_case(name))
        .collect();
    match matches.as_slice() {
        [field] => Ok(field),
        [] => Err(Error::Invalid(format!(
            "Jira has no field named `{name}`; list names and IDs with `atl jira field list {name}`"
        ))),
        many => Err(Error::needs_input(
            "field",
            format!(
                "{} fields are named `{name}`; pass the ID instead",
                many.len()
            ),
            many.iter().map(|f| Choice::new(&f.id, &f.name)).collect(),
        )),
    }
}

/// A field value as a person reads it: options become their value, users
/// their name, rich text Markdown, arrays lists of those.
fn simplify(value: &Value) -> Value {
    match value {
        Value::Array(items) => Value::Array(items.iter().map(simplify).collect()),
        Value::Object(map) if map.get("type").and_then(Value::as_str) == Some("doc") => {
            Value::String(markup::adf_to_markdown(value))
        }
        Value::Object(map) => ["value", "displayName", "name", "key"]
            .iter()
            .find_map(|k| map.get(*k).filter(|v| v.is_string()).cloned())
            .unwrap_or_else(|| value.clone()),
        other => other.clone(),
    }
}

/// True when all of `text` is one issue key, in any case (`abc-12`).
pub fn is_issue_key(text: &str) -> bool {
    find_issue_key(text).is_some_and(|key| key.eq_ignore_ascii_case(text))
}

/// The first Jira issue key in `text` (for example a git branch name such
/// as `feature/ABC-123-fix-login`).
pub fn find_issue_key(text: &str) -> Option<String> {
    let bytes = text.as_bytes();
    let upper = text.to_ascii_uppercase();
    let upper = upper.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        // A key starts at a word boundary with a letter.
        let boundary = i == 0 || !bytes[i - 1].is_ascii_alphanumeric();
        if boundary && upper[i].is_ascii_uppercase() {
            let mut j = i + 1;
            while j < bytes.len()
                && (upper[j].is_ascii_uppercase() || upper[j].is_ascii_digit() || upper[j] == b'_')
            {
                j += 1;
            }
            if j < bytes.len() && bytes[j] == b'-' && j > i {
                let mut k = j + 1;
                while k < bytes.len() && bytes[k].is_ascii_digit() {
                    k += 1;
                }
                let ends = k == bytes.len() || !bytes[k].is_ascii_alphanumeric();
                if k > j + 1 && ends && j - i >= 2 {
                    return Some(format!(
                        "{}-{}",
                        text[i..j].to_ascii_uppercase(),
                        &text[j + 1..k]
                    ));
                }
            }
            i = j;
        } else {
            i += 1;
        }
    }
    None
}

#[derive(Debug, Clone, Serialize)]
pub struct IssueLink {
    /// The phrase from this issue's side, for example "blocks" or "is blocked by".
    pub relation: String,
    pub key: String,
    pub summary: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Worklog {
    pub id: String,
    pub author: Option<String>,
    /// As Jira shows it, for example "2h 30m".
    pub time_spent: Option<String>,
    pub seconds: Option<u64>,
    pub started: Option<String>,
    /// Markdown.
    pub comment: Option<String>,
}

/// One changed field of one history entry.
#[derive(Debug, Clone, Serialize)]
pub struct Change {
    pub created: Option<String>,
    pub author: Option<String>,
    pub field: String,
    pub from: Option<String>,
    pub to: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Component {
    pub id: String,
    pub name: String,
    pub lead: Option<String>,
    pub description: Option<String>,
}

/// `YYYY-MM-DD` becomes midnight UTC; other text is passed on unchanged.
fn agile_date(date: &str) -> String {
    match date.len() {
        10 => format!("{date}T00:00:00.000Z"),
        _ => date.to_owned(),
    }
}

impl Jira<'_> {
    /// Every comment of an issue, oldest first, as Markdown.
    pub async fn comments(&self, key: &str, limit: usize) -> Result<Vec<Comment>> {
        let mut out = Vec::new();
        while out.len() < limit {
            let page = self
                .send(
                    HttpMethod::Get,
                    &format!("{}/issue/{key}/comment", self.api()),
                    &[
                        ("startAt", out.len().to_string()),
                        ("maxResults", "100".to_owned()),
                        ("orderBy", "created".to_owned()),
                    ],
                    None,
                )
                .await?;
            let batch: Vec<Comment> = items(&page, "/comments")
                .map(|c| Comment {
                    id: text(c, "/id").unwrap_or_default(),
                    author: text(c, "/author/displayName"),
                    created: text(c, "/created"),
                    body: Self::rich_text(c.get("body")),
                })
                .collect();
            let total = page.get("total").and_then(Value::as_u64).unwrap_or(0) as usize;
            let done = batch.is_empty();
            out.extend(batch);
            if done || out.len() >= total {
                break;
            }
        }
        out.truncate(limit);
        Ok(out)
    }

    /// Replaces a comment's text (Markdown).
    pub async fn edit_comment(&self, key: &str, id: &str, markdown: &str) -> Result<Comment> {
        let response = self
            .send(
                HttpMethod::Put,
                &format!("{}/issue/{key}/comment/{id}", self.api()),
                &[],
                Some(json!({ "body": self.body(markdown) })),
            )
            .await?;
        Ok(Comment {
            id: text(&response, "/id").unwrap_or_else(|| id.to_owned()),
            author: text(&response, "/author/displayName"),
            created: text(&response, "/created"),
            body: markdown.to_owned(),
        })
    }

    pub async fn delete_comment(&self, key: &str, id: &str) -> Result<()> {
        self.send(
            HttpMethod::Delete,
            &format!("{}/issue/{key}/comment/{id}", self.api()),
            &[],
            None,
        )
        .await?;
        Ok(())
    }

    /// Logged time of an issue, oldest first.
    pub async fn worklogs(&self, key: &str) -> Result<Vec<Worklog>> {
        let response = self
            .send(
                HttpMethod::Get,
                &format!("{}/issue/{key}/worklog", self.api()),
                &[("maxResults", "1000".to_owned())],
                None,
            )
            .await?;
        Ok(items(&response, "/worklogs")
            .map(|w| Worklog {
                id: text(w, "/id").unwrap_or_default(),
                author: text(w, "/author/displayName"),
                time_spent: text(w, "/timeSpent"),
                seconds: w.get("timeSpentSeconds").and_then(Value::as_u64),
                started: text(w, "/started"),
                comment: w
                    .get("comment")
                    .filter(|c| !c.is_null())
                    .map(|c| Self::rich_text(Some(c)))
                    .filter(|c| !c.is_empty()),
            })
            .collect())
    }

    /// Field changes of an issue, newest first.
    pub async fn history(&self, key: &str, limit: usize) -> Result<Vec<Change>> {
        let histories: Vec<Value> = if self.cloud {
            let mut all = Vec::new();
            loop {
                let page = self
                    .send(
                        HttpMethod::Get,
                        &format!("/rest/api/3/issue/{key}/changelog"),
                        &[
                            ("startAt", all.len().to_string()),
                            ("maxResults", "100".to_owned()),
                        ],
                        None,
                    )
                    .await?;
                let batch = page
                    .get("values")
                    .and_then(Value::as_array)
                    .cloned()
                    .unwrap_or_default();
                let last = page.get("isLast").and_then(Value::as_bool).unwrap_or(true);
                let empty = batch.is_empty();
                all.extend(batch);
                if last || empty || all.len() >= 2000 {
                    break;
                }
            }
            all
        } else {
            let issue = self
                .send(
                    HttpMethod::Get,
                    &format!("/rest/api/2/issue/{key}"),
                    &[
                        ("expand", "changelog".to_owned()),
                        ("fields", "summary".to_owned()),
                    ],
                    None,
                )
                .await?;
            issue
                .pointer("/changelog/histories")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default()
        };
        let mut out: Vec<Change> = histories
            .iter()
            .flat_map(|h| {
                items(h, "/items").map(move |item| Change {
                    created: text(h, "/created"),
                    author: text(h, "/author/displayName"),
                    field: text(item, "/field").unwrap_or_default(),
                    from: text(item, "/fromString").or_else(|| text(item, "/from")),
                    to: text(item, "/toString").or_else(|| text(item, "/to")),
                })
            })
            .collect();
        out.sort_by(|a, b| b.created.cmp(&a.created));
        out.truncate(limit);
        Ok(out)
    }

    pub async fn components(&self, project: &str) -> Result<Vec<Component>> {
        let response = self
            .send(
                HttpMethod::Get,
                &format!("{}/project/{project}/components", self.api()),
                &[],
                None,
            )
            .await?;
        Ok(response
            .as_array()
            .into_iter()
            .flatten()
            .map(|c| Component {
                id: text(c, "/id").unwrap_or_default(),
                name: text(c, "/name").unwrap_or_default(),
                lead: text(c, "/lead/displayName"),
                description: text(c, "/description"),
            })
            .collect())
    }

    fn sprint_from(s: &Value) -> Sprint {
        Sprint {
            id: s.get("id").and_then(Value::as_u64).unwrap_or_default(),
            name: text(s, "/name").unwrap_or_default(),
            state: text(s, "/state").unwrap_or_default(),
            start: text(s, "/startDate"),
            end: text(s, "/endDate"),
            goal: text(s, "/goal"),
        }
    }

    pub async fn sprint(&self, id: u64) -> Result<Sprint> {
        let found = self
            .send(
                HttpMethod::Get,
                &format!("/rest/agile/1.0/sprint/{id}"),
                &[],
                None,
            )
            .await?;
        Ok(Self::sprint_from(&found))
    }

    /// Creates a future sprint on a board. Dates are `YYYY-MM-DD` or ISO 8601.
    pub async fn create_sprint(
        &self,
        board: u64,
        name: &str,
        start: Option<&str>,
        end: Option<&str>,
        goal: Option<&str>,
    ) -> Result<Sprint> {
        let mut body = json!({ "name": name, "originBoardId": board });
        if let Some(start) = start {
            body["startDate"] = agile_date(start).into();
        }
        if let Some(end) = end {
            body["endDate"] = agile_date(end).into();
        }
        if let Some(goal) = goal {
            body["goal"] = goal.into();
        }
        let created = self
            .send(HttpMethod::Post, "/rest/agile/1.0/sprint", &[], Some(body))
            .await?;
        Ok(Self::sprint_from(&created))
    }

    /// Starts a future sprint. The start defaults to the stored start, then
    /// now; the end to the stored end (Jira needs both).
    pub async fn start_sprint(
        &self,
        id: u64,
        start: Option<&str>,
        end: Option<&str>,
    ) -> Result<Sprint> {
        let current = self.sprint(id).await?;
        let now =
            || crate::time::now().map(|secs| crate::time::utc_with_millis(secs as i64 * 1000));
        let start = start.map(agile_date).or(current.start).or_else(now).ok_or_else(|| {
            Error::Invalid(format!(
                "sprint {id} has no start date and this platform has no clock; pass a start date (YYYY-MM-DD)"
            ))
        })?;
        let end = end.map(agile_date).or(current.end).ok_or_else(|| {
            Error::Invalid(format!(
                "sprint {id} has no end date; pass --end YYYY-MM-DD (Jira needs a start and an end to start a sprint)"
            ))
        })?;
        let updated = self
            .send(
                HttpMethod::Post,
                &format!("/rest/agile/1.0/sprint/{id}"),
                &[],
                Some(json!({ "state": "active", "startDate": start, "endDate": end })),
            )
            .await?;
        Ok(Self::sprint_from(&updated))
    }

    /// Completes an active sprint.
    pub async fn close_sprint(&self, id: u64) -> Result<Sprint> {
        let updated = self
            .send(
                HttpMethod::Post,
                &format!("/rest/agile/1.0/sprint/{id}"),
                &[],
                Some(json!({ "state": "closed" })),
            )
            .await?;
        Ok(Self::sprint_from(&updated))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filters_become_jql() {
        let cases = [
            (
                IssueQuery::default(),
                "assignee = currentUser() AND resolution = Unresolved ORDER BY updated DESC",
            ),
            (
                IssueQuery {
                    project: Some("ABC".into()),
                    status: vec!["In Progress".into(), "~Done".into()],
                    assignee: Some("me".into()),
                    labels: vec!["a\"b".into()],
                    created: Some("-7d".into()),
                    updated: Some("week".into()),
                    order_by: Some("priority".into()),
                    reverse: true,
                    ..Default::default()
                },
                "project = \"ABC\" AND status IN (\"In Progress\") AND status NOT IN (\"Done\") AND labels IN (\"a\\\"b\") AND assignee = currentUser() AND created >= -7d AND updated >= startOfWeek() ORDER BY priority ASC",
            ),
            (
                IssueQuery {
                    assignee: Some("x".into()),
                    jql: Some("sprint in openSprints() order by rank".into()),
                    ..Default::default()
                },
                "assignee IS EMPTY AND (sprint in openSprints()) order by rank",
            ),
            (
                IssueQuery {
                    jql: Some("ORDER BY created".into()),
                    ..Default::default()
                },
                "assignee = currentUser() AND resolution = Unresolved ORDER BY created",
            ),
            (
                IssueQuery {
                    resolution: vec!["unresolved".into(), "~Won't Do".into()],
                    parent: Some("ABC-1".into()),
                    watching: true,
                    history: true,
                    sprint: Some("current".into()),
                    created_before: Some("2026-01-01".into()),
                    updated_before: Some("-30d".into()),
                    ..Default::default()
                },
                "resolution IN (Unresolved) AND resolution NOT IN (\"Won't Do\") AND parent = \"ABC-1\" AND watcher = currentUser() AND issuekey IN issueHistory() AND sprint IN openSprints() AND created < \"2026-01-01\" AND updated < -30d ORDER BY lastViewed DESC",
            ),
            (
                IssueQuery {
                    sprint: Some("42".into()),
                    ..Default::default()
                },
                "sprint = 42 ORDER BY updated DESC",
            ),
        ];
        for (query, expected) in cases {
            assert_eq!(query.to_jql(), expected);
        }
    }

    #[test]
    fn issue_keys_are_found_in_branch_names() {
        let cases = [
            ("feature/ABC-123-fix-login", Some("ABC-123")),
            ("abc-9_typo", Some("ABC-9")),
            ("PROJ2-7", Some("PROJ2-7")),
            ("v1.2.3", None),
            ("main", None),
            ("x-1", None),
        ];
        for (branch, expected) in cases {
            assert_eq!(find_issue_key(branch).as_deref(), expected, "{branch}");
        }
    }
}
