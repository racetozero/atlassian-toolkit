//! Task-level Atlassian Admin API (`https://api.atlassian.com/admin`):
//! organizations, directories, managed users, groups, domains and the audit
//! log, for one organization at a time.

use crate::catalog::HttpMethod;
use crate::client::Client;
use crate::config::Family;
use crate::error::{Error, Result};
use crate::json::{items, text};
use serde::Serialize;
use serde_json::{Value, json};

#[derive(Debug, Clone, Serialize)]
pub struct Org {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Directory {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AdminUser {
    pub account_id: String,
    pub name: Option<String>,
    pub email: Option<String>,
    /// Account status, for example active or suspended.
    pub status: Option<String>,
    pub membership: Option<String>,
    pub directory: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Group {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub users: Option<u64>,
    pub directory: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Domain {
    pub id: String,
    pub name: String,
    /// Verification status of the domain claim.
    pub status: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Event {
    pub id: String,
    pub time: Option<String>,
    pub action: Option<String>,
    pub actor: Option<String>,
    pub message: Option<String>,
}

/// Audit log filters.
#[derive(Debug, Clone, Default)]
pub struct EventQuery {
    /// Words to find in the events.
    pub search: Option<String>,
    /// Start date or time, for example 2026-11-01 or 2026-11-01T09:00:00Z.
    pub from: Option<String>,
    /// End date or time.
    pub to: Option<String>,
    /// Event action, for example user_added_to_group.
    pub action: Option<String>,
    pub limit: usize,
}

/// Most pages the list functions follow.
const MAX_PAGES: usize = 10;

pub struct Admin<'a> {
    client: &'a Client,
}

impl<'a> Admin<'a> {
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    async fn send(
        &self,
        method: HttpMethod,
        path: &str,
        query: &[(&str, String)],
    ) -> Result<Value> {
        self.client
            .send_json(Family::Admin, method, path, query, None)
            .await
    }

    async fn send_body(&self, method: HttpMethod, path: &str, body: Value) -> Result<Value> {
        self.client
            .send_json(Family::Admin, method, path, &[], Some(body))
            .await
    }

    /// The `data` items of `path` and its following pages (`links.next`),
    /// up to `limit` items.
    async fn pages(
        &self,
        path: &str,
        query: &[(&str, String)],
        limit: usize,
    ) -> Result<Vec<Value>> {
        let mut out = Vec::new();
        let mut next = Some((path.to_owned(), query.to_vec()));
        for _ in 0..MAX_PAGES {
            let Some((path, query)) = next.take() else {
                break;
            };
            let page = self.send(HttpMethod::Get, &path, &query).await?;
            out.extend(items(&page, "/data").cloned());
            if out.len() >= limit {
                out.truncate(limit);
                break;
            }
            next = text(&page, "/links/next").map(|url| (url, Vec::new()));
        }
        Ok(out)
    }

    pub async fn orgs(&self) -> Result<Vec<Org>> {
        Ok(self
            .pages("/admin/v1/orgs", &[], 500)
            .await?
            .iter()
            .map(|o| Org {
                id: text(o, "/id").unwrap_or_default(),
                name: text(o, "/attributes/name").unwrap_or_default(),
            })
            .collect())
    }

    /// The organization ID for an ID or name, or the only organization.
    async fn org_id(&self, org: Option<&str>) -> Result<String> {
        let orgs = self.orgs().await?;
        let listed = || {
            orgs.iter()
                .map(|o| format!("{} ({})", o.name, o.id))
                .collect::<Vec<_>>()
                .join(", ")
        };
        match org {
            Some(wanted) => orgs
                .iter()
                .find(|o| o.id == wanted || o.name.eq_ignore_ascii_case(wanted))
                .map(|o| o.id.clone())
                .ok_or_else(|| {
                    Error::Invalid(format!(
                        "the Admin API key cannot see an organization `{wanted}`; organizations: {}",
                        listed()
                    ))
                }),
            None => match orgs.as_slice() {
                [only] => Ok(only.id.clone()),
                [] => Err(Error::Invalid(
                    "the Admin API key sees no organization; create the key in admin.atlassian.com > Settings > API keys".into(),
                )),
                _ => Err(Error::Invalid(format!(
                    "the Admin API key sees several organizations; pass --org (or set [admin] org in .atl.toml): {}",
                    listed()
                ))),
            },
        }
    }

    pub async fn directories(&self, org: Option<&str>) -> Result<Vec<Directory>> {
        let org = self.org_id(org).await?;
        self.org_directories(&org).await
    }

    async fn org_directories(&self, org: &str) -> Result<Vec<Directory>> {
        Ok(self
            .pages(&format!("/admin/v2/orgs/{org}/directories"), &[], 500)
            .await?
            .iter()
            .map(|d| Directory {
                id: text(d, "/directoryId").unwrap_or_default(),
                name: text(d, "/name").unwrap_or_default(),
            })
            .collect())
    }

    async fn directory_users(
        &self,
        org: &str,
        directory: &str,
        search: Option<&str>,
        limit: usize,
    ) -> Result<Vec<AdminUser>> {
        let mut query = vec![("limit", limit.clamp(1, 100).to_string())];
        if let Some(search) = search {
            query.push(("searchTerm", search.to_owned()));
        }
        Ok(self
            .pages(
                &format!("/admin/v2/orgs/{org}/directories/{directory}/users"),
                &query,
                limit,
            )
            .await?
            .iter()
            .map(|u| AdminUser {
                account_id: text(u, "/accountId").unwrap_or_default(),
                name: text(u, "/name"),
                email: text(u, "/email"),
                status: text(u, "/accountStatus").or_else(|| text(u, "/status")),
                membership: text(u, "/membershipStatus"),
                directory: directory.to_owned(),
            })
            .collect())
    }

    /// Managed users in every directory, optionally matching `search`.
    pub async fn users(
        &self,
        org: Option<&str>,
        search: Option<&str>,
        limit: usize,
    ) -> Result<Vec<AdminUser>> {
        let org = self.org_id(org).await?;
        let mut out = Vec::new();
        for directory in self.org_directories(&org).await? {
            if out.len() >= limit {
                break;
            }
            out.extend(
                self.directory_users(&org, &directory.id, search, limit - out.len())
                    .await?,
            );
        }
        Ok(out)
    }

    /// The user with this account ID or email, searched in `directory` or
    /// in every directory.
    async fn find_user(&self, org: &str, who: &str, directory: Option<&str>) -> Result<AdminUser> {
        let directories: Vec<String> = match directory {
            Some(directory) => vec![directory.to_owned()],
            None => self
                .org_directories(org)
                .await?
                .into_iter()
                .map(|d| d.id)
                .collect(),
        };
        for directory in &directories {
            let found = self.directory_users(org, directory, Some(who), 20).await?;
            if let Some(user) = found.into_iter().find(|u| {
                u.account_id == who
                    || u.email
                        .as_deref()
                        .is_some_and(|e| e.eq_ignore_ascii_case(who))
            }) {
                return Ok(user);
            }
        }
        Err(Error::Invalid(format!(
            "no managed user has the account ID or email `{who}`; find users with `atl admin user list --search ...`"
        )))
    }

    async fn set_access(
        &self,
        org: Option<&str>,
        who: &str,
        directory: Option<&str>,
        action: &str,
        status: &str,
    ) -> Result<AdminUser> {
        let org = self.org_id(org).await?;
        let mut user = self.find_user(&org, who, directory).await?;
        self.send(
            HttpMethod::Post,
            &format!(
                "/admin/v2/orgs/{org}/directories/{}/users/{}/{action}",
                user.directory, user.account_id
            ),
            &[],
        )
        .await?;
        user.status = Some(status.to_owned());
        Ok(user)
    }

    /// Suspends the user's access to every product of the organization.
    pub async fn suspend(
        &self,
        org: Option<&str>,
        user: &str,
        directory: Option<&str>,
    ) -> Result<AdminUser> {
        self.set_access(org, user, directory, "suspend", "suspended")
            .await
    }

    pub async fn restore(
        &self,
        org: Option<&str>,
        user: &str,
        directory: Option<&str>,
    ) -> Result<AdminUser> {
        self.set_access(org, user, directory, "restore", "active")
            .await
    }

    /// Groups in every directory, optionally matching `search`.
    pub async fn groups(
        &self,
        org: Option<&str>,
        search: Option<&str>,
        limit: usize,
    ) -> Result<Vec<Group>> {
        let org = self.org_id(org).await?;
        let mut out = Vec::new();
        for directory in self.org_directories(&org).await? {
            if out.len() >= limit {
                break;
            }
            let mut query = vec![("limit", limit.clamp(1, 100).to_string())];
            if let Some(search) = search {
                query.push(("searchTerm", search.to_owned()));
            }
            let found = self
                .pages(
                    &format!("/admin/v2/orgs/{org}/directories/{}/groups", directory.id),
                    &query,
                    limit - out.len(),
                )
                .await?;
            out.extend(found.iter().map(|g| Group {
                id: text(g, "/id").unwrap_or_default(),
                name: text(g, "/name").unwrap_or_default(),
                description: text(g, "/description"),
                users: g.pointer("/counts/users").and_then(Value::as_u64),
                directory: directory.id.clone(),
            }));
        }
        Ok(out)
    }

    pub async fn domains(&self, org: Option<&str>) -> Result<Vec<Domain>> {
        let org = self.org_id(org).await?;
        Ok(self
            .pages(&format!("/admin/v1/orgs/{org}/domains"), &[], 500)
            .await?
            .iter()
            .map(|d| Domain {
                id: text(d, "/id").unwrap_or_default(),
                name: text(d, "/attributes/name").unwrap_or_default(),
                status: text(d, "/attributes/claim/status"),
            })
            .collect())
    }

    /// Audit log events, newest first.
    pub async fn events(&self, org: Option<&str>, query: &EventQuery) -> Result<Vec<Event>> {
        let org = self.org_id(org).await?;
        let mut params = vec![("limit", query.limit.clamp(1, 200).to_string())];
        for (name, value) in [
            ("q", &query.search),
            ("from", &query.from),
            ("to", &query.to),
            ("action", &query.action),
        ] {
            if let Some(value) = value {
                params.push((name, value.clone()));
            }
        }
        Ok(self
            .pages(
                &format!("/admin/v1/orgs/{org}/events"),
                &params,
                query.limit,
            )
            .await?
            .iter()
            .map(|e| Event {
                id: text(e, "/id").unwrap_or_default(),
                time: text(e, "/attributes/time"),
                action: text(e, "/attributes/action"),
                actor: text(e, "/attributes/actor/name")
                    .or_else(|| text(e, "/attributes/actor/email")),
                message: text(e, "/message/content"),
            })
            .collect())
    }
}

impl Admin<'_> {
    /// A group by ID or exact name (case-insensitive) in any directory.
    async fn find_group(&self, org: &str, group: &str) -> Result<Group> {
        let found = self.groups(Some(org), Some(group), 200).await?;
        let mut matches = found
            .into_iter()
            .filter(|g| g.id == group || g.name.eq_ignore_ascii_case(group));
        matches.next().ok_or_else(|| {
            Error::Invalid(format!(
                "no group has the ID or name `{group}`; find groups with `atl admin group list --search ...`"
            ))
        })
    }

    /// Adds (or with `add: false` removes) a managed user to a group.
    pub async fn group_membership(
        &self,
        org: Option<&str>,
        group: &str,
        user: &str,
        add: bool,
    ) -> Result<(Group, AdminUser)> {
        let org = self.org_id(org).await?;
        let group = self.find_group(&org, group).await?;
        let user = self.find_user(&org, user, Some(&group.directory)).await?;
        let base = format!(
            "/admin/v2/orgs/{org}/directories/{}/groups/{}/memberships",
            group.directory, group.id
        );
        match add {
            true => {
                self.send_body(
                    HttpMethod::Post,
                    &base,
                    json!({ "accountId": user.account_id }),
                )
                .await?;
            }
            false => {
                self.send(
                    HttpMethod::Delete,
                    &format!("{base}/{}", user.account_id),
                    &[],
                )
                .await?;
            }
        }
        Ok((group, user))
    }
}
