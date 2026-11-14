//! Task-level Jira Service Management API (`/rest/servicedeskapi`), the same
//! on Cloud and Data Center. It uses the Jira credential.

use crate::catalog::HttpMethod;
use crate::client::Client;
use crate::config::Family;
use crate::error::{Error, Result};
use crate::jira::{IssueSummary, Jira};
use crate::json::{items, text};
use serde::Serialize;
use serde_json::{Value, json};

#[derive(Debug, Clone, Serialize)]
pub struct Desk {
    pub id: String,
    pub project: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct RequestType {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Queue {
    pub id: String,
    pub name: String,
    pub jql: Option<String>,
    pub issues: Option<u64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Request {
    pub key: String,
    pub summary: String,
    pub status: String,
    #[serde(rename = "type")]
    pub request_type: Option<String>,
    pub reporter: Option<String>,
    pub created: Option<String>,
    pub url: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct RequestComment {
    pub id: String,
    pub author: Option<String>,
    pub created: Option<String>,
    /// False for internal notes that customers cannot see.
    pub public: bool,
    pub body: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct RequestDetail {
    #[serde(flatten)]
    pub request: Request,
    pub description: String,
    pub comments: Vec<RequestComment>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RequestTransition {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Approval {
    pub id: String,
    pub name: String,
    /// pending, approved or declined.
    pub decision: String,
    /// True when you are an approver who has not answered yet.
    pub can_answer: bool,
    pub approvers: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Sla {
    pub name: String,
    /// ongoing, paused, met or breached.
    pub state: String,
    pub breached: bool,
    /// Time left (negative when breached), as Jira shows it.
    pub remaining: Option<String>,
    pub goal: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RequestAttachment {
    pub filename: String,
    pub author: Option<String>,
    pub created: Option<String>,
    pub size: Option<u64>,
    pub mime_type: Option<String>,
    pub url: Option<String>,
}

fn attachment_from(a: &Value) -> RequestAttachment {
    RequestAttachment {
        filename: text(a, "/filename").unwrap_or_default(),
        author: text(a, "/author/displayName"),
        created: text(a, "/created/iso8601"),
        size: a.get("size").and_then(Value::as_u64),
        mime_type: text(a, "/mimeType"),
        url: text(a, "/_links/content"),
    }
}

/// The value of one request field (`summary`, `description`).
fn field(request: &Value, id: &str) -> Option<String> {
    request
        .get("requestFieldValues")
        .and_then(Value::as_array)?
        .iter()
        .find(|f| f.get("fieldId").and_then(Value::as_str) == Some(id))
        .and_then(|f| text(f, "/value"))
}

fn by_id_or_name<T>(
    items: Vec<T>,
    wanted: &str,
    keys: impl Fn(&T) -> (&str, &str),
    what: &str,
    list_with: &str,
) -> Result<T> {
    let mut names = Vec::new();
    for item in items {
        let (id, name) = keys(&item);
        if id == wanted || name.eq_ignore_ascii_case(wanted) {
            return Ok(item);
        }
        names.push(format!("{name:?} ({id})"));
    }
    Err(Error::Invalid(format!(
        "no {what} matches `{wanted}`; available: {}; list them with `{list_with}`",
        if names.is_empty() {
            "(none)".to_owned()
        } else {
            names.join(", ")
        }
    )))
}

const BASE: &str = "/rest/servicedeskapi";

pub struct ServiceDesk<'a> {
    client: &'a Client,
}

impl<'a> ServiceDesk<'a> {
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    async fn send(
        &self,
        method: HttpMethod,
        path: &str,
        query: &[(&str, String)],
        body: Option<Value>,
    ) -> Result<Value> {
        self.client
            .send_json(Family::Jira, method, &format!("{BASE}{path}"), query, body)
            .await
    }

    fn request_from(&self, r: &Value) -> Request {
        let key = text(r, "/issueKey").unwrap_or_default();
        Request {
            url: text(r, "/_links/web")
                .unwrap_or_else(|| crate::jira::Jira::new(self.client).browse_url(&key)),
            summary: field(r, "summary").unwrap_or_default(),
            status: text(r, "/currentStatus/status").unwrap_or_default(),
            request_type: text(r, "/requestType/name"),
            reporter: text(r, "/reporter/displayName"),
            created: text(r, "/createdDate/iso8601"),
            key,
        }
    }

    pub async fn desks(&self) -> Result<Vec<Desk>> {
        let response = self
            .send(
                HttpMethod::Get,
                "/servicedesk",
                &[("limit", "100".into())],
                None,
            )
            .await?;
        Ok(items(&response, "/values")
            .map(|d| Desk {
                id: text(d, "/id").unwrap_or_default(),
                project: text(d, "/projectKey").unwrap_or_default(),
                name: text(d, "/projectName").unwrap_or_default(),
            })
            .collect())
    }

    /// The service desk ID for an ID or a project key.
    pub async fn desk_id(&self, desk: &str) -> Result<String> {
        if desk.bytes().all(|b| b.is_ascii_digit()) {
            return Ok(desk.to_owned());
        }
        let found = by_id_or_name(
            self.desks().await?,
            desk,
            |d| (d.id.as_str(), d.project.as_str()),
            "service desk",
            "atl jsm desk list",
        )?;
        Ok(found.id)
    }

    pub async fn request_types(&self, desk: &str) -> Result<Vec<RequestType>> {
        let id = self.desk_id(desk).await?;
        let response = self
            .send(
                HttpMethod::Get,
                &format!("/servicedesk/{id}/requesttype"),
                &[("limit", "100".into())],
                None,
            )
            .await?;
        Ok(items(&response, "/values")
            .map(|t| RequestType {
                id: text(t, "/id").unwrap_or_default(),
                name: text(t, "/name").unwrap_or_default(),
                description: text(t, "/description"),
            })
            .collect())
    }

    pub async fn queues(&self, desk: &str) -> Result<Vec<Queue>> {
        let id = self.desk_id(desk).await?;
        let response = self
            .send(
                HttpMethod::Get,
                &format!("/servicedesk/{id}/queue"),
                &[("includeCount", "true".into()), ("limit", "100".into())],
                None,
            )
            .await?;
        Ok(items(&response, "/values")
            .map(|q| Queue {
                id: text(q, "/id").unwrap_or_default(),
                name: text(q, "/name").unwrap_or_default(),
                jql: text(q, "/jql"),
                issues: q.get("issueCount").and_then(Value::as_u64),
            })
            .collect())
    }

    /// Issues in a queue, by queue ID or name.
    pub async fn queue_issues(
        &self,
        desk: &str,
        queue: &str,
        limit: usize,
    ) -> Result<Vec<IssueSummary>> {
        let id = self.desk_id(desk).await?;
        let queue = by_id_or_name(
            self.queues(&id).await?,
            queue,
            |q| (q.id.as_str(), q.name.as_str()),
            "queue",
            "atl jsm queue list",
        )?;
        let response = self
            .send(
                HttpMethod::Get,
                &format!("/servicedesk/{id}/queue/{}/issue", queue.id),
                &[("limit", limit.clamp(1, 50).to_string())],
                None,
            )
            .await?;
        let jira = Jira::new(self.client);
        Ok(items(&response, "/values")
            .map(|i| jira.summary(i))
            .collect())
    }

    /// Requests you can see; `status` is open, closed, or all.
    pub async fn requests(
        &self,
        desk: Option<&str>,
        status: &str,
        search: Option<&str>,
        mine: bool,
        limit: usize,
    ) -> Result<Vec<Request>> {
        let status = match status.to_ascii_lowercase().as_str() {
            "open" => "OPEN_REQUESTS",
            "closed" => "CLOSED_REQUESTS",
            "all" => "ALL_REQUESTS",
            other => {
                return Err(Error::Invalid(format!(
                    "request status `{other}` is unknown; use open, closed or all"
                )));
            }
        };
        let ownership = if mine {
            "OWNED_REQUESTS"
        } else {
            "ALL_REQUESTS"
        };
        let mut query = vec![
            ("requestStatus", status.to_owned()),
            ("requestOwnership", ownership.to_owned()),
            ("expand", "requestType".to_owned()),
            ("limit", limit.clamp(1, 100).to_string()),
        ];
        if let Some(desk) = desk {
            query.push(("serviceDeskId", self.desk_id(desk).await?));
        }
        if let Some(search) = search {
            query.push(("searchTerm", search.to_owned()));
        }
        let response = self.send(HttpMethod::Get, "/request", &query, None).await?;
        Ok(items(&response, "/values")
            .map(|r| self.request_from(r))
            .collect())
    }

    /// One request with its description and latest `comments` comments.
    pub async fn request(&self, key: &str, comments: usize) -> Result<RequestDetail> {
        let request = self
            .send(
                HttpMethod::Get,
                &format!("/request/{key}"),
                &[("expand", "requestType".into())],
                None,
            )
            .await?;
        let mut list = Vec::new();
        if comments > 0 {
            let response = self
                .send(
                    HttpMethod::Get,
                    &format!("/request/{key}/comment"),
                    &[("limit", "100".into())],
                    None,
                )
                .await?;
            list = items(&response, "/values")
                .map(|c| RequestComment {
                    id: text(c, "/id").unwrap_or_default(),
                    author: text(c, "/author/displayName"),
                    created: text(c, "/created/iso8601"),
                    public: c.get("public").and_then(Value::as_bool).unwrap_or(true),
                    body: text(c, "/body").unwrap_or_default(),
                })
                .collect();
            list.drain(..list.len().saturating_sub(comments));
        }
        Ok(RequestDetail {
            description: field(&request, "description").unwrap_or_default(),
            request: self.request_from(&request),
            comments: list,
        })
    }

    /// Raises a request; `request_type` is an ID or a name.
    pub async fn create(
        &self,
        desk: &str,
        request_type: &str,
        summary: &str,
        description: Option<&str>,
        on_behalf_of: Option<&str>,
    ) -> Result<Request> {
        let desk = self.desk_id(desk).await?;
        let type_id = if request_type.bytes().all(|b| b.is_ascii_digit()) {
            request_type.to_owned()
        } else {
            by_id_or_name(
                self.request_types(&desk).await?,
                request_type,
                |t| (t.id.as_str(), t.name.as_str()),
                "request type",
                "atl jsm desk types <desk>",
            )?
            .id
        };
        let mut fields = json!({ "summary": summary });
        if let Some(description) = description {
            fields["description"] = description.into();
        }
        let mut body = json!({
            "serviceDeskId": desk,
            "requestTypeId": type_id,
            "requestFieldValues": fields,
        });
        if let Some(customer) = on_behalf_of {
            body["raiseOnBehalfOf"] = customer.into();
        }
        let created = self
            .send(HttpMethod::Post, "/request", &[], Some(body))
            .await?;
        Ok(self.request_from(&created))
    }

    /// Adds a reply (public) or an internal note.
    pub async fn comment(&self, key: &str, body: &str, internal: bool) -> Result<RequestComment> {
        let created = self
            .send(
                HttpMethod::Post,
                &format!("/request/{key}/comment"),
                &[],
                Some(json!({ "body": body, "public": !internal })),
            )
            .await?;
        Ok(RequestComment {
            id: text(&created, "/id").unwrap_or_default(),
            author: text(&created, "/author/displayName"),
            created: text(&created, "/created/iso8601"),
            public: !internal,
            body: body.to_owned(),
        })
    }

    pub async fn transitions(&self, key: &str) -> Result<Vec<RequestTransition>> {
        let response = self
            .send(
                HttpMethod::Get,
                &format!("/request/{key}/transition"),
                &[],
                None,
            )
            .await?;
        Ok(items(&response, "/values")
            .map(|t| RequestTransition {
                id: text(t, "/id").unwrap_or_default(),
                name: text(t, "/name").unwrap_or_default(),
            })
            .collect())
    }

    /// Runs the customer transition named `name` (case-insensitive) or with that ID.
    pub async fn transition(
        &self,
        key: &str,
        name: &str,
        comment: Option<&str>,
    ) -> Result<RequestTransition> {
        let chosen = by_id_or_name(
            self.transitions(key).await?,
            name,
            |t| (t.id.as_str(), t.name.as_str()),
            "transition",
            &format!("atl jsm request transitions {key}"),
        )?;
        let mut body = json!({ "id": chosen.id });
        if let Some(comment) = comment {
            body["additionalComment"] = json!({ "body": comment });
        }
        self.send(
            HttpMethod::Post,
            &format!("/request/{key}/transition"),
            &[],
            Some(body),
        )
        .await?;
        Ok(chosen)
    }
}

impl ServiceDesk<'_> {
    pub async fn approvals(&self, key: &str) -> Result<Vec<Approval>> {
        let response = self
            .send(
                HttpMethod::Get,
                &format!("/request/{key}/approval"),
                &[],
                None,
            )
            .await?;
        Ok(items(&response, "/values")
            .map(|a| Approval {
                id: a
                    .get("id")
                    .map(|v| v.as_str().map_or_else(|| v.to_string(), str::to_owned))
                    .unwrap_or_default(),
                name: text(a, "/name").unwrap_or_default(),
                decision: text(a, "/finalDecision").unwrap_or_default(),
                can_answer: a
                    .get("canAnswerApproval")
                    .and_then(Value::as_bool)
                    .unwrap_or(false),
                approvers: items(a, "/approvers")
                    .filter_map(|x| text(x, "/approver/displayName"))
                    .collect(),
            })
            .collect())
    }

    /// Approves or declines (`approve: false`) an approval; without an ID,
    /// the one pending approval you can answer.
    pub async fn answer_approval(
        &self,
        key: &str,
        approval: Option<&str>,
        approve: bool,
    ) -> Result<Approval> {
        let id = match approval {
            Some(id) => id.to_owned(),
            None => {
                let open: Vec<Approval> = self
                    .approvals(key)
                    .await?
                    .into_iter()
                    .filter(|a| a.can_answer && a.decision == "pending")
                    .collect();
                match open.as_slice() {
                    [] => {
                        return Err(Error::Invalid(format!(
                            "{key} has no pending approval that you can answer; list them with `atl jsm request approvals {key}`"
                        )));
                    }
                    [only] => only.id.clone(),
                    many => {
                        return Err(Error::needs_input(
                            "approval",
                            format!("{key} has several pending approvals; pass the approval ID"),
                            many.iter()
                                .map(|a| crate::error::Choice {
                                    value: a.id.clone(),
                                    label: a.name.clone(),
                                })
                                .collect(),
                        ));
                    }
                }
            }
        };
        let decision = if approve { "approve" } else { "decline" };
        let answered = self
            .send(
                HttpMethod::Post,
                &format!("/request/{key}/approval/{id}"),
                &[],
                Some(json!({ "decision": decision })),
            )
            .await?;
        Ok(Approval {
            id,
            name: text(&answered, "/name").unwrap_or_default(),
            decision: text(&answered, "/finalDecision").unwrap_or_else(|| format!("{decision}d")),
            can_answer: false,
            approvers: Vec::new(),
        })
    }

    async fn participant_names(&self, key: &str) -> Result<Vec<String>> {
        let response = self
            .send(
                HttpMethod::Get,
                &format!("/request/{key}/participant"),
                &[("limit", "100".into())],
                None,
            )
            .await?;
        Ok(items(&response, "/values")
            .filter_map(|u| text(u, "/displayName"))
            .collect())
    }

    /// Adds and removes participants (`me`, an email or a name) and returns
    /// the participants afterwards.
    pub async fn participants(
        &self,
        key: &str,
        add: &[String],
        remove: &[String],
    ) -> Result<Vec<String>> {
        let jira = Jira::new(self.client);
        let cloud = self.client.deployment(Family::Jira) == crate::config::Deployment::Cloud;
        for (users, method) in [(add, HttpMethod::Post), (remove, HttpMethod::Delete)] {
            if users.is_empty() {
                continue;
            }
            let mut ids = Vec::new();
            for user in users {
                ids.push(jira.user_id(user).await?.ok_or_else(|| {
                    Error::Invalid(format!(
                        "`{user}` is not a user; pass me, an email or a name"
                    ))
                })?);
            }
            let body = match cloud {
                true => json!({ "accountIds": ids }),
                false => json!({ "usernames": ids }),
            };
            self.send(
                method,
                &format!("/request/{key}/participant"),
                &[],
                Some(body),
            )
            .await?;
        }
        self.participant_names(key).await
    }

    pub async fn slas(&self, key: &str) -> Result<Vec<Sla>> {
        let response = self
            .send(HttpMethod::Get, &format!("/request/{key}/sla"), &[], None)
            .await?;
        Ok(items(&response, "/values")
            .map(|s| {
                let ongoing = s.get("ongoingCycle").filter(|c| !c.is_null());
                let last = s
                    .get("completedCycles")
                    .and_then(Value::as_array)
                    .and_then(|c| c.last());
                let cycle = ongoing.or(last).cloned().unwrap_or(Value::Null);
                let breached = cycle
                    .get("breached")
                    .and_then(Value::as_bool)
                    .unwrap_or(false);
                let state = match (ongoing, breached) {
                    (_, true) => "breached",
                    (Some(c), false) if c.get("paused").and_then(Value::as_bool) == Some(true) => {
                        "paused"
                    }
                    (Some(_), false) => "ongoing",
                    (None, false) if last.is_some() => "met",
                    (None, false) => "ongoing",
                };
                Sla {
                    name: text(s, "/name").unwrap_or_default(),
                    state: state.to_owned(),
                    breached,
                    remaining: text(&cycle, "/remainingTime/friendly"),
                    goal: text(&cycle, "/goalDuration/friendly"),
                }
            })
            .collect())
    }

    pub async fn attachments(&self, key: &str) -> Result<Vec<RequestAttachment>> {
        let response = self
            .send(
                HttpMethod::Get,
                &format!("/request/{key}/attachment"),
                &[("limit", "100".into())],
                None,
            )
            .await?;
        Ok(items(&response, "/values").map(attachment_from).collect())
    }

    /// Attaches files to a request: each is uploaded to the request's desk as
    /// a temporary file, then all are attached in one step. `public: false`
    /// keeps them (and the comment JSM adds) internal.
    pub async fn attach(
        &self,
        key: &str,
        files: &[std::path::PathBuf],
        public: bool,
    ) -> Result<Vec<RequestAttachment>> {
        let request = self
            .send(HttpMethod::Get, &format!("/request/{key}"), &[], None)
            .await?;
        let desk = text(&request, "/serviceDeskId").ok_or_else(|| {
            Error::Invalid(format!("{key} is not a service desk request; attach to Jira issues with `atl jira issue attach`"))
        })?;
        let mut ids = Vec::new();
        for file in files {
            let mut upload = self.client.prepare_raw(
                Family::Jira,
                HttpMethod::Post,
                &format!("{BASE}/servicedesk/{desk}/attachTemporaryFile"),
                &serde_json::Map::new(),
                None,
            )?;
            // Older JSM servers still require the experimental opt-in.
            upload
                .headers
                .push(("X-ExperimentalApi".into(), "opt-in".into()));
            upload.set_multipart(vec![file.clone()], Vec::new());
            let uploaded = self.client.execute(&upload).await?.value();
            ids.extend(
                items(&uploaded, "/temporaryAttachments")
                    .filter_map(|t| text(t, "/temporaryAttachmentId")),
            );
        }
        let attached = self
            .send(
                HttpMethod::Post,
                &format!("/request/{key}/attachment"),
                &[],
                Some(json!({ "temporaryAttachmentIds": ids, "public": public })),
            )
            .await?;
        Ok(items(&attached, "/attachments/values")
            .map(attachment_from)
            .collect())
    }
}
