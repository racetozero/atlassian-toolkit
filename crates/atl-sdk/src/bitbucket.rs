//! Task-level Bitbucket API that works on Cloud (`api.bitbucket.org/2.0`,
//! repositories under a workspace) and Data Center (`/rest/api/latest`,
//! repositories under a project).

use crate::catalog::HttpMethod;
use crate::client::Client;
use crate::config::{Deployment, Family};
use crate::error::{Error, Result};
use crate::json::{items, text};
use serde::Serialize;
use serde_json::{Value, json};
use std::collections::HashSet;

/// `owner/slug`: a workspace (Cloud) or project key (Data Center) and a
/// repository slug.
/// Task arguments deserialize it from that text and echo it back the same way.
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(try_from = "String", into = "String")]
pub struct RepoRef {
    pub owner: String,
    pub slug: String,
}

impl TryFrom<String> for RepoRef {
    type Error = Error;
    fn try_from(text: String) -> Result<Self> {
        Self::parse(&text)
    }
}

impl From<RepoRef> for String {
    fn from(repo: RepoRef) -> String {
        repo.to_string()
    }
}

impl schemars::JsonSchema for RepoRef {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        "RepoRef".into()
    }
    fn inline_schema() -> bool {
        true
    }
    fn json_schema(_: &mut schemars::SchemaGenerator) -> schemars::Schema {
        schemars::json_schema!({ "type": "string" })
    }
}

impl RepoRef {
    pub fn parse(text: &str) -> Result<RepoRef> {
        match text.trim_matches('/').split_once('/') {
            Some((owner, slug)) if !owner.is_empty() && !slug.is_empty() && !slug.contains('/') => {
                Ok(RepoRef {
                    owner: owner.to_owned(),
                    slug: slug.trim_end_matches(".git").to_owned(),
                })
            }
            _ => Err(Error::Invalid(format!(
                "`{text}` is not a repository; pass WORKSPACE/REPO (Cloud) or PROJECT/REPO (Data Center), for example acme/website"
            ))),
        }
    }

    /// The repository of a git remote URL on Bitbucket, for example
    /// `git@bitbucket.org:acme/web.git` or
    /// `https://bitbucket.corp/scm/PROJ/web.git`.
    pub fn from_remote(url: &str) -> Option<RepoRef> {
        let path = match url.split_once("://") {
            Some((_, rest)) => rest.split_once('/')?.1,
            None => url.split_once(':')?.1,
        };
        let path = path.trim_prefix("scm/");
        let mut parts = path.trim_matches('/').rsplitn(3, '/');
        let slug = parts.next()?;
        let owner = parts.next()?;
        RepoRef::parse(&format!("{owner}/{slug}")).ok()
    }
}

impl std::fmt::Display for RepoRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.owner, self.slug)
    }
}

#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct Repo {
    /// `owner/slug`.
    pub repo: String,
    pub name: String,
    pub description: Option<String>,
    pub default_branch: Option<String>,
    pub private: Option<bool>,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct Branch {
    pub name: String,
    pub commit: Option<String>,
}

#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct PullRequest {
    pub id: u64,
    pub title: String,
    /// OPEN, MERGED, DECLINED or SUPERSEDED.
    pub state: String,
    pub author: Option<String>,
    pub source: String,
    pub destination: String,
    pub updated: Option<String>,
    pub description: String,
    pub reviewers: Vec<String>,
    pub approved_by: Vec<String>,
    pub url: String,
    /// `owner/slug` of the repository the source branch is in (differs
    /// from the target repository for pull requests from forks).
    pub source_repo: Option<String>,
    /// True for a draft (not yet ready for review).
    pub draft: bool,
}

#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct Pipeline {
    pub build_number: u64,
    pub uuid: Option<String>,
    /// PENDING, IN_PROGRESS or COMPLETED.
    pub state: String,
    /// SUCCESSFUL, FAILED, STOPPED or ERROR once completed.
    pub result: Option<String>,
    pub branch: Option<String>,
    pub created: Option<String>,
    pub url: String,
}

/// Data Center timestamps are epoch milliseconds; shown as UTC ISO 8601.
pub(crate) fn epoch_ms(value: Option<&Value>) -> Option<String> {
    value?.as_i64().map(crate::time::utc)
}

pub struct Bitbucket<'a> {
    client: &'a Client,
    cloud: bool,
}

impl<'a> Bitbucket<'a> {
    pub fn new(client: &'a Client) -> Self {
        Self {
            client,
            cloud: client.deployment(Family::Bitbucket) == Deployment::Cloud,
        }
    }

    async fn send(
        &self,
        method: HttpMethod,
        path: &str,
        query: &[(&str, String)],
        body: Option<Value>,
    ) -> Result<Value> {
        self.client
            .send_json(Family::Bitbucket, method, path, query, body)
            .await
    }

    fn repo_path(&self, repo: &RepoRef) -> String {
        if self.cloud {
            format!("/2.0/repositories/{}/{}", repo.owner, repo.slug)
        } else {
            format!(
                "/rest/api/latest/projects/{}/repos/{}",
                repo.owner, repo.slug
            )
        }
    }

    fn page_size(&self, limit: usize) -> String {
        limit
            .clamp(1, if self.cloud { 50 } else { 100 })
            .to_string()
    }

    fn repo_from(&self, r: &Value) -> Repo {
        if self.cloud {
            return Repo {
                repo: text(r, "/full_name").unwrap_or_default(),
                name: text(r, "/name").unwrap_or_default(),
                description: text(r, "/description"),
                default_branch: text(r, "/mainbranch/name"),
                private: r.get("is_private").and_then(Value::as_bool),
                url: text(r, "/links/html/href").unwrap_or_default(),
            };
        }
        Repo {
            repo: format!(
                "{}/{}",
                text(r, "/project/key").unwrap_or_default(),
                text(r, "/slug").unwrap_or_default()
            ),
            name: text(r, "/name").unwrap_or_default(),
            description: text(r, "/description"),
            default_branch: None,
            private: r.get("public").and_then(Value::as_bool).map(|p| !p),
            url: text(r, "/links/self/0/href").unwrap_or_default(),
        }
    }

    /// Repositories of a workspace or project; without one, the
    /// repositories you are a member of (Cloud) or can see (Data Center).
    pub async fn repos(&self, owner: Option<&str>, limit: usize) -> Result<Vec<Repo>> {
        let size = self.page_size(limit);
        let response = match (self.cloud, owner) {
            (true, Some(owner)) => {
                self.send(
                    HttpMethod::Get,
                    &format!("/2.0/repositories/{owner}"),
                    &[("pagelen", size)],
                    None,
                )
                .await?
            }
            (true, None) => {
                self.send(
                    HttpMethod::Get,
                    "/2.0/repositories",
                    &[("role", "member".into()), ("pagelen", size)],
                    None,
                )
                .await?
            }
            (false, Some(owner)) => {
                self.send(
                    HttpMethod::Get,
                    &format!("/rest/api/latest/projects/{owner}/repos"),
                    &[("limit", size)],
                    None,
                )
                .await?
            }
            (false, None) => {
                self.send(
                    HttpMethod::Get,
                    "/rest/api/latest/repos",
                    &[("limit", size)],
                    None,
                )
                .await?
            }
        };
        Ok(items(&response, "/values")
            .map(|r| self.repo_from(r))
            .collect())
    }

    pub async fn repo(&self, repo: &RepoRef) -> Result<Repo> {
        let path = self.repo_path(repo);
        let mut found = self.repo_from(&self.send(HttpMethod::Get, &path, &[], None).await?);
        if !self.cloud {
            let branch = self
                .send(
                    HttpMethod::Get,
                    &format!("{path}/default-branch"),
                    &[],
                    None,
                )
                .await?;
            found.default_branch = text(&branch, "/displayId");
        }
        Ok(found)
    }

    /// Branches whose name contains `filter`.
    pub async fn branches(
        &self,
        repo: &RepoRef,
        filter: Option<&str>,
        limit: usize,
    ) -> Result<Vec<Branch>> {
        let size = self.page_size(limit);
        let (path, mut query) = if self.cloud {
            (
                format!("{}/refs/branches", self.repo_path(repo)),
                vec![("pagelen", size), ("sort", "-target.date".into())],
            )
        } else {
            (
                format!("{}/branches", self.repo_path(repo)),
                vec![("limit", size), ("orderBy", "MODIFICATION".into())],
            )
        };
        if let Some(filter) = filter {
            if self.cloud {
                query.push(("q", format!("name ~ {}", crate::ql::quote(filter))));
            } else {
                query.push(("filterText", filter.to_owned()));
            }
        }
        let response = self.send(HttpMethod::Get, &path, &query, None).await?;
        Ok(items(&response, "/values")
            .map(|b| Branch {
                name: text(b, if self.cloud { "/name" } else { "/displayId" }).unwrap_or_default(),
                commit: text(
                    b,
                    if self.cloud {
                        "/target/hash"
                    } else {
                        "/latestCommit"
                    },
                ),
            })
            .collect())
    }

    fn pr_from(&self, p: &Value) -> PullRequest {
        let names = |list: &str, name: &str, keep: &dyn Fn(&Value) -> bool| -> Vec<String> {
            p.get(list)
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .filter(|v| keep(v))
                .filter_map(|v| text(v, name))
                .collect()
        };
        let approved = |v: &Value| v.get("approved").and_then(Value::as_bool) == Some(true);
        if self.cloud {
            return PullRequest {
                id: p.get("id").and_then(Value::as_u64).unwrap_or_default(),
                title: text(p, "/title").unwrap_or_default(),
                state: text(p, "/state").unwrap_or_default(),
                author: text(p, "/author/display_name"),
                source: text(p, "/source/branch/name").unwrap_or_default(),
                destination: text(p, "/destination/branch/name").unwrap_or_default(),
                updated: text(p, "/updated_on"),
                description: text(p, "/description").unwrap_or_default(),
                reviewers: names("reviewers", "/display_name", &|_| true),
                approved_by: names("participants", "/user/display_name", &approved),
                url: text(p, "/links/html/href").unwrap_or_default(),
                source_repo: text(p, "/source/repository/full_name"),
                draft: p.get("draft").and_then(Value::as_bool).unwrap_or(false),
            };
        }
        PullRequest {
            id: p.get("id").and_then(Value::as_u64).unwrap_or_default(),
            title: text(p, "/title").unwrap_or_default(),
            state: text(p, "/state").unwrap_or_default(),
            author: text(p, "/author/user/displayName"),
            source: text(p, "/fromRef/displayId").unwrap_or_default(),
            destination: text(p, "/toRef/displayId").unwrap_or_default(),
            updated: epoch_ms(p.get("updatedDate")),
            description: text(p, "/description").unwrap_or_default(),
            reviewers: names("reviewers", "/user/displayName", &|_| true),
            approved_by: names("reviewers", "/user/displayName", &approved),
            url: text(p, "/links/self/0/href").unwrap_or_default(),
            source_repo: match (
                text(p, "/fromRef/repository/project/key"),
                text(p, "/fromRef/repository/slug"),
            ) {
                (Some(key), Some(slug)) => Some(format!("{key}/{slug}")),
                _ => None,
            },
            draft: p.get("draft").and_then(Value::as_bool).unwrap_or(false),
        }
    }

    fn pr_path(&self, repo: &RepoRef, id: u64) -> String {
        let kind = if self.cloud {
            "pullrequests"
        } else {
            "pull-requests"
        };
        format!("{}/{kind}/{id}", self.repo_path(repo))
    }

    /// Pull requests in `state`: open, merged, declined, or all.
    pub async fn pull_requests(
        &self,
        repo: &RepoRef,
        state: &str,
        limit: usize,
    ) -> Result<Vec<PullRequest>> {
        let state = state.to_ascii_uppercase();
        let states: &[&str] = match state.as_str() {
            "OPEN" | "MERGED" | "DECLINED" => &[state.as_str()],
            "ALL" if self.cloud => &["OPEN", "MERGED", "DECLINED", "SUPERSEDED"],
            "ALL" => &["ALL"],
            _ => {
                return Err(Error::Invalid(format!(
                    "pull request state `{state}` is unknown; use open, merged, declined or all"
                )));
            }
        };
        let (path, mut query) = if self.cloud {
            (
                format!("{}/pullrequests", self.repo_path(repo)),
                vec![("pagelen", self.page_size(limit))],
            )
        } else {
            (
                format!("{}/pull-requests", self.repo_path(repo)),
                vec![("limit", self.page_size(limit))],
            )
        };
        query.extend(states.iter().map(|s| ("state", (*s).to_owned())));
        let response = self.send(HttpMethod::Get, &path, &query, None).await?;
        Ok(items(&response, "/values")
            .map(|p| self.pr_from(p))
            .collect())
    }

    pub async fn pull_request(&self, repo: &RepoRef, id: u64) -> Result<PullRequest> {
        let pr = self
            .send(HttpMethod::Get, &self.pr_path(repo, id), &[], None)
            .await?;
        Ok(self.pr_from(&pr))
    }

    /// Opens a pull request from `source` into `destination` (default: the
    /// repository's main branch).
    pub async fn create_pull_request(
        &self,
        repo: &RepoRef,
        title: &str,
        source: &str,
        destination: Option<&str>,
        description: Option<&str>,
        draft: bool,
    ) -> Result<PullRequest> {
        let body = if self.cloud {
            let mut body = json!({
                "title": title,
                "source": { "branch": { "name": source } },
                "description": description.unwrap_or_default(),
                "draft": draft,
            });
            if let Some(destination) = destination {
                body["destination"] = json!({ "branch": { "name": destination } });
            }
            body
        } else {
            let destination = match destination {
                Some(branch) => branch.to_owned(),
                None => self.repo(repo).await?.default_branch.ok_or_else(|| {
                    Error::Invalid(format!("{repo} has no default branch; pass --to <branch>"))
                })?,
            };
            json!({
                "title": title,
                "description": description.unwrap_or_default(),
                "fromRef": { "id": format!("refs/heads/{source}") },
                "toRef": { "id": format!("refs/heads/{destination}") },
                "draft": draft,
            })
        };
        let kind = if self.cloud {
            "pullrequests"
        } else {
            "pull-requests"
        };
        let created = self
            .send(
                HttpMethod::Post,
                &format!("{}/{kind}", self.repo_path(repo)),
                &[],
                Some(body),
            )
            .await?;
        Ok(self.pr_from(&created))
    }

    /// Merges a pull request. `strategy` is a Cloud merge strategy
    /// (merge_commit, squash, fast_forward); Data Center uses the
    /// repository's setting.
    pub async fn merge(
        &self,
        repo: &RepoRef,
        id: u64,
        strategy: Option<&str>,
    ) -> Result<PullRequest> {
        let path = format!("{}/merge", self.pr_path(repo, id));
        let merged = if self.cloud {
            let mut body = json!({});
            if let Some(strategy) = strategy {
                body["merge_strategy"] = strategy.into();
            }
            self.send(HttpMethod::Post, &path, &[], Some(body)).await?
        } else {
            // Data Center refuses a merge without the current version.
            let pr = self
                .send(HttpMethod::Get, &self.pr_path(repo, id), &[], None)
                .await?;
            let version = pr.get("version").and_then(Value::as_u64).unwrap_or(0);
            self.send(
                HttpMethod::Post,
                &path,
                &[("version", version.to_string())],
                None,
            )
            .await?
        };
        Ok(self.pr_from(&merged))
    }

    pub async fn approve(&self, repo: &RepoRef, id: u64) -> Result<()> {
        self.send(
            HttpMethod::Post,
            &format!("{}/approve", self.pr_path(repo, id)),
            &[],
            None,
        )
        .await?;
        Ok(())
    }

    /// Adds a comment; Bitbucket renders it as Markdown. Returns the comment ID.
    pub async fn comment(&self, repo: &RepoRef, id: u64, markdown: &str) -> Result<String> {
        let body = if self.cloud {
            json!({ "content": { "raw": markdown } })
        } else {
            json!({ "text": markdown })
        };
        let created = self
            .send(
                HttpMethod::Post,
                &format!("{}/comments", self.pr_path(repo, id)),
                &[],
                Some(body),
            )
            .await?;
        Ok(created
            .get("id")
            .map(|id| id.to_string())
            .unwrap_or_default())
    }

    /// The pull request's unified diff.
    pub async fn diff(&self, repo: &RepoRef, id: u64) -> Result<String> {
        let path = if self.cloud {
            format!("{}/diff", self.pr_path(repo, id))
        } else {
            format!("{}.diff", self.pr_path(repo, id))
        };
        let response = self.send(HttpMethod::Get, &path, &[], None).await?;
        Ok(response
            .get("text")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned())
    }

    fn require_cloud(&self) -> Result<()> {
        if self.cloud {
            return Ok(());
        }
        Err(Error::Invalid(
            "Bitbucket Data Center has no Pipelines; use your CI server (for example Bamboo or Jenkins), or log in to Bitbucket Cloud".into(),
        ))
    }

    fn pipeline_from(&self, repo: &RepoRef, p: &Value) -> Pipeline {
        let build_number = p.get("build_number").and_then(Value::as_u64).unwrap_or(0);
        Pipeline {
            build_number,
            uuid: text(p, "/uuid"),
            state: text(p, "/state/name").unwrap_or_default(),
            result: text(p, "/state/result/name"),
            branch: text(p, "/target/ref_name"),
            created: text(p, "/created_on"),
            url: format!(
                "https://bitbucket.org/{}/{}/pipelines/results/{build_number}",
                repo.owner, repo.slug
            ),
        }
    }

    /// The latest pipelines, newest first (Cloud only).
    pub async fn pipelines(&self, repo: &RepoRef, limit: usize) -> Result<Vec<Pipeline>> {
        self.require_cloud()?;
        let response = self
            .send(
                HttpMethod::Get,
                &format!("{}/pipelines", self.repo_path(repo)),
                &[
                    ("pagelen", self.page_size(limit)),
                    ("sort", "-created_on".into()),
                ],
                None,
            )
            .await?;
        Ok(items(&response, "/values")
            .map(|p| self.pipeline_from(repo, p))
            .collect())
    }

    /// Starts the pipeline of `branch`, or its custom pipeline `custom`
    /// (Cloud only).
    pub async fn run_pipeline(
        &self,
        repo: &RepoRef,
        branch: &str,
        custom: Option<&str>,
    ) -> Result<Pipeline> {
        self.require_cloud()?;
        let mut target =
            json!({ "type": "pipeline_ref_target", "ref_type": "branch", "ref_name": branch });
        if let Some(pattern) = custom {
            target["selector"] = json!({ "type": "custom", "pattern": pattern });
        }
        let started = self
            .send(
                HttpMethod::Post,
                &format!("{}/pipelines", self.repo_path(repo)),
                &[],
                Some(json!({ "target": target })),
            )
            .await?;
        Ok(self.pipeline_from(repo, &started))
    }
}

#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct PipelineStep {
    pub uuid: String,
    pub name: String,
    /// PENDING, IN_PROGRESS or COMPLETED.
    pub state: String,
    pub result: Option<String>,
    pub seconds: Option<u64>,
}

#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct PipelineDetail {
    #[serde(flatten)]
    pub pipeline: Pipeline,
    pub steps: Vec<PipelineStep>,
}

/// True when a user object has `who` as its name, nickname, slug, email,
/// account ID or UUID (case-insensitive).
fn is_user(user: &Value, who: &str) -> bool {
    [
        "/display_name",
        "/nickname",
        "/uuid",
        "/account_id",
        "/name",
        "/slug",
        "/displayName",
        "/emailAddress",
    ]
    .iter()
    .any(|p| text(user, p).is_some_and(|v| v.eq_ignore_ascii_case(who)))
}

impl Bitbucket<'_> {
    /// The pull request as the server returns it.
    async fn raw_pr(&self, repo: &RepoRef, id: u64) -> Result<Value> {
        self.send(HttpMethod::Get, &self.pr_path(repo, id), &[], None)
            .await
    }

    fn version(pr: &Value) -> String {
        pr.get("version")
            .and_then(Value::as_u64)
            .unwrap_or(0)
            .to_string()
    }

    /// Declines (rejects) a pull request. Cloud cannot reopen it afterwards.
    pub async fn decline(&self, repo: &RepoRef, id: u64) -> Result<PullRequest> {
        let path = format!("{}/decline", self.pr_path(repo, id));
        let declined = if self.cloud {
            self.send(HttpMethod::Post, &path, &[], None).await?
        } else {
            let version = Self::version(&self.raw_pr(repo, id).await?);
            self.send(HttpMethod::Post, &path, &[("version", version)], None)
                .await?
        };
        Ok(self.pr_from(&declined))
    }

    /// Marks the pull request as needing changes by you ("Request changes"
    /// on Cloud, "Needs work" on Data Center).
    pub async fn request_changes(&self, repo: &RepoRef, id: u64) -> Result<()> {
        if self.cloud {
            self.send(
                HttpMethod::Post,
                &format!("{}/request-changes", self.pr_path(repo, id)),
                &[],
                None,
            )
            .await?;
            return Ok(());
        }
        let slug = self.current_user_slug().await?;
        self.send(
            HttpMethod::Put,
            &format!("{}/participants/{slug}", self.pr_path(repo, id)),
            &[],
            Some(json!({ "user": { "name": slug }, "approved": false, "status": "NEEDS_WORK" })),
        )
        .await?;
        Ok(())
    }

    /// The Data Center user slug of the signed-in user.
    async fn current_user_slug(&self) -> Result<String> {
        let name = self
            .client
            .download(Family::Bitbucket, "/plugins/servlet/applinks/whoami")
            .await?;
        let name = String::from_utf8_lossy(&name).trim().to_owned();
        if name.is_empty() {
            return Err(Error::Invalid(
                "Bitbucket did not report a signed-in user; check the login with `atl auth status --verify`".into(),
            ));
        }
        let user = self
            .send(
                HttpMethod::Get,
                &format!("/rest/api/latest/users/{name}"),
                &[],
                None,
            )
            .await?;
        Ok(text(&user, "/slug").unwrap_or(name))
    }

    /// Changes the title, description (Markdown) and/or destination branch.
    pub async fn edit_pull_request(
        &self,
        repo: &RepoRef,
        id: u64,
        title: Option<&str>,
        description: Option<&str>,
        destination: Option<&str>,
    ) -> Result<PullRequest> {
        if title.is_none() && description.is_none() && destination.is_none() {
            return Err(Error::Invalid(
                "nothing to change; pass --title, --body or --to".into(),
            ));
        }
        let pr = self.raw_pr(repo, id).await?;
        let title = title.map(str::to_owned).or_else(|| text(&pr, "/title"));
        let body = if self.cloud {
            let mut body = json!({ "title": title });
            if let Some(description) = description {
                body["description"] = description.into();
            }
            if let Some(branch) = destination {
                body["destination"] = json!({ "branch": { "name": branch } });
            }
            body
        } else {
            let mut body = json!({
                "version": pr.get("version").cloned().unwrap_or(json!(0)),
                "title": title,
                "description": description.map(str::to_owned).or_else(|| text(&pr, "/description")).unwrap_or_default(),
                "reviewers": Self::dc_reviewers(&pr, &[]),
            });
            if let Some(branch) = destination {
                body["toRef"] = json!({ "id": format!("refs/heads/{branch}") });
            }
            body
        };
        let updated = self
            .send(HttpMethod::Put, &self.pr_path(repo, id), &[], Some(body))
            .await?;
        Ok(self.pr_from(&updated))
    }

    /// Data Center reviewer list for an update: the current reviewers minus
    /// `remove`, as `[{"user": {"name": ...}}]`.
    fn dc_reviewers(pr: &Value, remove: &[String]) -> Vec<Value> {
        items(pr, "/reviewers")
            .filter_map(|r| r.get("user"))
            .filter(|u| !remove.iter().any(|who| is_user(u, who)))
            .filter_map(|u| text(u, "/name"))
            .map(|name| json!({ "user": { "name": name } }))
            .collect()
    }

    /// A Cloud workspace member matching `who` (display name, nickname,
    /// account ID or UUID); `{uuid}` is used as given.
    async fn cloud_member(&self, workspace: &str, who: &str) -> Result<Value> {
        if who.starts_with('{') && who.ends_with('}') {
            return Ok(json!({ "uuid": who }));
        }
        let mut path = format!("/2.0/workspaces/{workspace}/members");
        let mut query = vec![("pagelen", "100".to_owned())];
        for _ in 0..5 {
            let page = self.send(HttpMethod::Get, &path, &query, None).await?;
            if let Some(user) = items(&page, "/values")
                .filter_map(|m| m.get("user"))
                .find(|u| is_user(u, who))
            {
                return Ok(user.clone());
            }
            match text(&page, "/next") {
                Some(next) => {
                    path = next;
                    query.clear();
                }
                None => break,
            }
        }
        Err(Error::Invalid(format!(
            "no member of workspace `{workspace}` matches `{who}`; pass a display name, nickname, account ID or {{uuid}}"
        )))
    }

    /// The Data Center user name (slug) matching `who`.
    async fn dc_user(&self, who: &str) -> Result<String> {
        let found = self
            .send(
                HttpMethod::Get,
                "/rest/api/latest/users",
                &[("filter", who.to_owned()), ("limit", "2".into())],
                None,
            )
            .await?;
        items(&found, "/values")
            .next()
            .and_then(|u| text(u, "/name"))
            .ok_or_else(|| {
                Error::Invalid(format!(
                    "no Bitbucket user matches `{who}`; pass a username, name or email"
                ))
            })
    }

    /// Adds and removes reviewers, keeping the others.
    pub async fn set_reviewers(
        &self,
        repo: &RepoRef,
        id: u64,
        add: &[String],
        remove: &[String],
    ) -> Result<PullRequest> {
        if add.is_empty() && remove.is_empty() {
            return Err(Error::Invalid(
                "nothing to change; pass reviewers to add or remove".into(),
            ));
        }
        let pr = self.raw_pr(repo, id).await?;
        let current: Vec<&Value> = items(&pr, "/reviewers")
            .map(|r| {
                if self.cloud {
                    r
                } else {
                    r.get("user").unwrap_or(r)
                }
            })
            .collect();
        if let Some(missing) = remove
            .iter()
            .find(|who| !current.iter().any(|u| is_user(u, who)))
        {
            return Err(Error::Invalid(format!(
                "`{missing}` is not a reviewer of #{id}; list them with `atl bitbucket pr reviewers {id}`"
            )));
        }
        let body = if self.cloud {
            let mut uuids: Vec<String> = current
                .iter()
                .filter(|u| !remove.iter().any(|who| is_user(u, who)))
                .filter_map(|u| text(u, "/uuid"))
                .collect();
            for who in add {
                if let Some(uuid) = text(&self.cloud_member(&repo.owner, who).await?, "/uuid")
                    && !uuids.contains(&uuid)
                {
                    uuids.push(uuid);
                }
            }
            json!({
                "title": text(&pr, "/title"),
                "reviewers": uuids.iter().map(|uuid| json!({ "uuid": uuid })).collect::<Vec<_>>(),
            })
        } else {
            let mut reviewers = Self::dc_reviewers(&pr, remove);
            for who in add {
                let name = self.dc_user(who).await?;
                let entry = json!({ "user": { "name": name } });
                if !reviewers.contains(&entry) {
                    reviewers.push(entry);
                }
            }
            json!({
                "version": pr.get("version").cloned().unwrap_or(json!(0)),
                "title": text(&pr, "/title"),
                "description": text(&pr, "/description").unwrap_or_default(),
                "reviewers": reviewers,
            })
        };
        let updated = self
            .send(HttpMethod::Put, &self.pr_path(repo, id), &[], Some(body))
            .await?;
        Ok(self.pr_from(&updated))
    }

    /// A pipeline by build number or UUID, with its steps (Cloud only).
    pub async fn pipeline(&self, repo: &RepoRef, id: &str) -> Result<PipelineDetail> {
        self.require_cloud()?;
        let base = format!("{}/pipelines", self.repo_path(repo));
        let found = self
            .send(HttpMethod::Get, &format!("{base}/{id}"), &[], None)
            .await
            .map_err(|err| match err {
                Error::Http(http) if http.status == 404 => Error::Invalid(format!(
                    "{repo} has no pipeline `{id}`; pass a build number or the UUID from `atl bitbucket pipeline list`"
                )),
                other => other,
            })?;
        let pipeline = self.pipeline_from(repo, &found);
        let uuid = pipeline.uuid.clone().unwrap_or_else(|| id.to_owned());
        let steps = self
            .send(
                HttpMethod::Get,
                &format!("{base}/{uuid}/steps"),
                &[("pagelen", "100".into())],
                None,
            )
            .await?;
        Ok(PipelineDetail {
            pipeline,
            steps: items(&steps, "/values")
                .map(|s| PipelineStep {
                    uuid: text(s, "/uuid").unwrap_or_default(),
                    name: text(s, "/name").unwrap_or_else(|| "(unnamed step)".into()),
                    state: text(s, "/state/name").unwrap_or_default(),
                    result: text(s, "/state/result/name"),
                    seconds: s.get("duration_in_seconds").and_then(Value::as_u64),
                })
                .collect(),
        })
    }

    /// Step logs of a pipeline: one step by name or UUID, or every step
    /// under a `== name (state) ==` header.
    pub async fn pipeline_logs(
        &self,
        repo: &RepoRef,
        id: &str,
        step: Option<&str>,
    ) -> Result<String> {
        let detail = self.pipeline(repo, id).await?;
        let uuid = detail
            .pipeline
            .uuid
            .clone()
            .unwrap_or_else(|| id.to_owned());
        let steps: Vec<&PipelineStep> = match step {
            Some(wanted) => {
                let found = detail
                    .steps
                    .iter()
                    .find(|s| s.name.eq_ignore_ascii_case(wanted) || s.uuid == wanted)
                    .ok_or_else(|| {
                        Error::Invalid(format!(
                            "pipeline {id} has no step `{wanted}`; steps: {}",
                            detail
                                .steps
                                .iter()
                                .map(|s| format!("{:?}", s.name))
                                .collect::<Vec<_>>()
                                .join(", ")
                        ))
                    })?;
                vec![found]
            }
            None => detail.steps.iter().collect(),
        };
        let mut out = String::new();
        for s in &steps {
            let bytes = self
                .client
                .download(
                    Family::Bitbucket,
                    &format!(
                        "{}/pipelines/{uuid}/steps/{}/log",
                        self.repo_path(repo),
                        s.uuid
                    ),
                )
                .await?;
            if step.is_none() {
                let state = s.result.as_deref().unwrap_or(&s.state);
                out.push_str(&format!("== {} ({state}) ==\n", s.name));
            }
            out.push_str(&String::from_utf8_lossy(&bytes));
            if !out.ends_with('\n') {
                out.push('\n');
            }
        }
        Ok(out)
    }
}

#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct PrComment {
    pub id: String,
    pub author: Option<String>,
    pub created: Option<String>,
    /// Markdown.
    pub body: String,
    /// File of an inline comment.
    pub path: Option<String>,
    /// Line of an inline comment (in the new version of the file when known).
    pub line: Option<u64>,
    /// ID of the comment this one answers.
    pub parent: Option<String>,
    pub resolved: bool,
}

#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct BuildStatus {
    pub name: String,
    pub key: Option<String>,
    /// SUCCESSFUL, FAILED, INPROGRESS, STOPPED (Cloud), CANCELLED or UNKNOWN (Data Center).
    pub state: String,
    pub url: Option<String>,
    pub description: Option<String>,
}

impl BuildStatus {
    pub fn failed(&self) -> bool {
        matches!(self.state.as_str(), "FAILED" | "STOPPED" | "CANCELLED")
    }

    pub fn pending(&self) -> bool {
        matches!(self.state.as_str(), "INPROGRESS" | "UNKNOWN")
    }
}

#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct Commit {
    pub hash: String,
    pub author: Option<String>,
    pub date: Option<String>,
    /// The first line of the message.
    pub message: String,
}

/// Most pages the comment and commit lists follow.
const MAX_PAGES: usize = 20;

impl Bitbucket<'_> {
    /// The pull request's comments, oldest first, with replies flattened
    /// (`parent` names the comment they answer).
    pub async fn pr_comments(
        &self,
        repo: &RepoRef,
        id: u64,
        limit: usize,
    ) -> Result<Vec<PrComment>> {
        let mut out = Vec::new();
        if self.cloud {
            let mut path = format!("{}/comments", self.pr_path(repo, id));
            let mut query = vec![("pagelen", "100".to_owned())];
            for _ in 0..MAX_PAGES {
                let page = self.send(HttpMethod::Get, &path, &query, None).await?;
                for c in items(&page, "/values") {
                    if c.get("deleted").and_then(Value::as_bool) == Some(true) {
                        continue;
                    }
                    out.push(PrComment {
                        id: text(c, "/id").unwrap_or_default(),
                        author: text(c, "/user/display_name"),
                        created: text(c, "/created_on"),
                        body: text(c, "/content/raw").unwrap_or_default(),
                        path: text(c, "/inline/path"),
                        line: c
                            .pointer("/inline/to")
                            .and_then(Value::as_u64)
                            .or_else(|| c.pointer("/inline/from").and_then(Value::as_u64)),
                        parent: text(c, "/parent/id"),
                        resolved: c.get("resolution").is_some_and(|r| !r.is_null()),
                    });
                }
                match text(&page, "/next") {
                    Some(next) if out.len() < limit => {
                        path = next;
                        query = Vec::new();
                    }
                    _ => break,
                }
            }
        } else {
            fn walk(c: &Value, parent: Option<String>, anchor: &Value, out: &mut Vec<PrComment>) {
                let id = text(c, "/id").unwrap_or_default();
                out.push(PrComment {
                    id: id.clone(),
                    author: text(c, "/author/displayName"),
                    created: epoch_ms(c.get("createdDate")),
                    body: text(c, "/text").unwrap_or_default(),
                    path: text(anchor, "/path"),
                    line: anchor.get("line").and_then(Value::as_u64),
                    parent,
                    resolved: text(c, "/state").as_deref() == Some("RESOLVED"),
                });
                for reply in items(c, "/comments") {
                    walk(reply, Some(id.clone()), anchor, out);
                }
            }
            let path = format!("{}/activities", self.pr_path(repo, id));
            // Replies can show up as activities of their own; walk only the
            // comments that no other comment contains.
            let mut found: Vec<(Value, Value)> = Vec::new();
            let mut start = 0;
            for _ in 0..MAX_PAGES {
                let page = self
                    .send(
                        HttpMethod::Get,
                        &path,
                        &[("limit", "100".into()), ("start", start.to_string())],
                        None,
                    )
                    .await?;
                for activity in items(&page, "/values") {
                    if text(activity, "/action").as_deref() == Some("COMMENTED")
                        && let Some(comment) = activity.get("comment")
                    {
                        let anchor = activity
                            .get("commentAnchor")
                            .cloned()
                            .unwrap_or(Value::Null);
                        found.push((comment.clone(), anchor));
                    }
                }
                match page.get("nextPageStart").and_then(Value::as_u64) {
                    Some(next)
                        if page.get("isLastPage") == Some(&Value::Bool(false))
                            && found.len() < limit =>
                    {
                        start = next
                    }
                    _ => break,
                }
            }
            fn nested(c: &Value, ids: &mut HashSet<String>) {
                for reply in items(c, "/comments") {
                    ids.insert(text(reply, "/id").unwrap_or_default());
                    nested(reply, ids);
                }
            }
            let mut replies = HashSet::new();
            for (comment, _) in &found {
                nested(comment, &mut replies);
            }
            // A comment shows up once per activity on it; walk each top-level one once.
            let mut walked = HashSet::new();
            for (comment, anchor) in &found {
                let id = text(comment, "/id").unwrap_or_default();
                if !replies.contains(&id) && walked.insert(id) {
                    walk(comment, None, anchor, &mut out);
                }
            }
            // The activity feed is newest first.
            out.sort_by(|a, b| a.created.cmp(&b.created));
        }
        out.truncate(limit);
        Ok(out)
    }

    /// Build statuses of the pull request's latest commit.
    pub async fn pr_checks(&self, repo: &RepoRef, id: u64) -> Result<Vec<BuildStatus>> {
        let (path, query) = if self.cloud {
            (
                format!("{}/statuses", self.pr_path(repo, id)),
                vec![("pagelen", "100".to_owned())],
            )
        } else {
            let pr = self.raw_pr(repo, id).await?;
            let commit = text(&pr, "/fromRef/latestCommit").ok_or_else(|| {
                Error::Invalid(format!(
                    "pull request #{id} of {repo} reports no latest commit"
                ))
            })?;
            (
                format!("/rest/build-status/latest/commits/{commit}"),
                vec![("limit", "100".to_owned())],
            )
        };
        let page = self.send(HttpMethod::Get, &path, &query, None).await?;
        Ok(items(&page, "/values")
            .map(|b| BuildStatus {
                name: text(b, "/name")
                    .or_else(|| text(b, "/key"))
                    .unwrap_or_default(),
                key: text(b, "/key"),
                state: text(b, "/state").unwrap_or_default(),
                url: text(b, "/url"),
                description: text(b, "/description"),
            })
            .collect())
    }

    /// Removes your approval.
    pub async fn unapprove(&self, repo: &RepoRef, id: u64) -> Result<()> {
        self.send(
            HttpMethod::Delete,
            &format!("{}/approve", self.pr_path(repo, id)),
            &[],
            None,
        )
        .await?;
        Ok(())
    }

    /// Marks the pull request as a draft, or (`draft: false`) ready for review.
    pub async fn set_draft(&self, repo: &RepoRef, id: u64, draft: bool) -> Result<PullRequest> {
        let pr = self.raw_pr(repo, id).await?;
        let body = if self.cloud {
            json!({ "title": text(&pr, "/title"), "draft": draft })
        } else {
            json!({
                "version": pr.get("version").cloned().unwrap_or(json!(0)),
                "title": text(&pr, "/title"),
                "description": text(&pr, "/description").unwrap_or_default(),
                "reviewers": Self::dc_reviewers(&pr, &[]),
                "draft": draft,
            })
        };
        let updated = self
            .send(HttpMethod::Put, &self.pr_path(repo, id), &[], Some(body))
            .await?;
        Ok(self.pr_from(&updated))
    }

    /// Creates a branch from `from` (a branch, tag or commit; default: the
    /// repository's main branch).
    pub async fn create_branch(
        &self,
        repo: &RepoRef,
        name: &str,
        from: Option<&str>,
    ) -> Result<Branch> {
        let from = match from {
            Some(from) => from.to_owned(),
            None if self.cloud => self.repo(repo).await?.default_branch.ok_or_else(|| {
                Error::Invalid(format!(
                    "{repo} has no main branch; pass --from <branch or commit>"
                ))
            })?,
            None => {
                let default = self
                    .send(
                        HttpMethod::Get,
                        &format!("{}/default-branch", self.repo_path(repo)),
                        &[],
                        None,
                    )
                    .await?;
                text(&default, "/id").ok_or_else(|| {
                    Error::Invalid(format!(
                        "{repo} has no default branch; pass --from <branch or commit>"
                    ))
                })?
            }
        };
        if self.cloud {
            let created = self
                .send(
                    HttpMethod::Post,
                    &format!("{}/refs/branches", self.repo_path(repo)),
                    &[],
                    Some(json!({ "name": name, "target": { "hash": from } })),
                )
                .await?;
            return Ok(Branch {
                name: text(&created, "/name").unwrap_or_else(|| name.to_owned()),
                commit: text(&created, "/target/hash"),
            });
        }
        let created = self
            .send(
                HttpMethod::Post,
                &format!("{}/branches", self.repo_path(repo)),
                &[],
                Some(json!({ "name": name, "startPoint": from })),
            )
            .await?;
        Ok(Branch {
            name: text(&created, "/displayId").unwrap_or_else(|| name.to_owned()),
            commit: text(&created, "/latestCommit"),
        })
    }

    pub async fn delete_branch(&self, repo: &RepoRef, name: &str) -> Result<()> {
        if self.cloud {
            let encoded =
                percent_encoding::utf8_percent_encode(name, percent_encoding::NON_ALPHANUMERIC);
            self.send(
                HttpMethod::Delete,
                &format!("{}/refs/branches/{encoded}", self.repo_path(repo)),
                &[],
                None,
            )
            .await?;
        } else {
            self.send(
                HttpMethod::Delete,
                &format!(
                    "/rest/branch-utils/latest/projects/{}/repos/{}/branches",
                    repo.owner, repo.slug
                ),
                &[],
                Some(json!({ "name": format!("refs/heads/{name}") })),
            )
            .await?;
        }
        Ok(())
    }

    /// Recent commits of `branch` (default: the main branch), newest first.
    pub async fn commits(
        &self,
        repo: &RepoRef,
        branch: Option<&str>,
        limit: usize,
    ) -> Result<Vec<Commit>> {
        let first_line = |message: Option<String>| {
            message
                .unwrap_or_default()
                .lines()
                .next()
                .unwrap_or_default()
                .to_owned()
        };
        let mut out = Vec::new();
        if self.cloud {
            let mut path = match branch {
                Some(branch) => format!(
                    "{}/commits/{}",
                    self.repo_path(repo),
                    percent_encoding::utf8_percent_encode(
                        branch,
                        percent_encoding::NON_ALPHANUMERIC
                    )
                ),
                None => format!("{}/commits", self.repo_path(repo)),
            };
            let mut query = vec![("pagelen", self.page_size(limit))];
            for _ in 0..MAX_PAGES {
                let page = self.send(HttpMethod::Get, &path, &query, None).await?;
                out.extend(items(&page, "/values").map(|c| Commit {
                    hash: text(c, "/hash").unwrap_or_default(),
                    author: text(c, "/author/user/display_name").or_else(|| text(c, "/author/raw")),
                    date: text(c, "/date"),
                    message: first_line(text(c, "/message")),
                }));
                match text(&page, "/next") {
                    Some(next) if out.len() < limit => {
                        path = next;
                        query = Vec::new();
                    }
                    _ => break,
                }
            }
        } else {
            let mut query = vec![("limit", self.page_size(limit))];
            if let Some(branch) = branch {
                query.push(("until", format!("refs/heads/{branch}")));
            }
            let page = self
                .send(
                    HttpMethod::Get,
                    &format!("{}/commits", self.repo_path(repo)),
                    &query,
                    None,
                )
                .await?;
            out.extend(items(&page, "/values").map(|c| Commit {
                hash: text(c, "/id").unwrap_or_default(),
                author: text(c, "/author/displayName").or_else(|| text(c, "/author/name")),
                date: epoch_ms(c.get("authorTimestamp")),
                message: first_line(text(c, "/message")),
            }));
        }
        out.truncate(limit);
        Ok(out)
    }

    /// Stops a running pipeline (Cloud only).
    pub async fn stop_pipeline(&self, repo: &RepoRef, id: &str) -> Result<Pipeline> {
        let found = self.pipeline(repo, id).await?.pipeline;
        let uuid = found.uuid.clone().unwrap_or_else(|| id.to_owned());
        self.send(
            HttpMethod::Post,
            &format!("{}/pipelines/{uuid}/stopPipeline", self.repo_path(repo)),
            &[],
            None,
        )
        .await?;
        Ok(self.pipeline(repo, &uuid).await?.pipeline)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repositories_come_from_names_and_git_remotes() {
        let repo = |owner: &str, slug: &str| {
            Some(RepoRef {
                owner: owner.into(),
                slug: slug.into(),
            })
        };
        assert_eq!(RepoRef::parse("acme/web").ok(), repo("acme", "web"));
        assert!(RepoRef::parse("web").is_err());
        assert!(RepoRef::parse("a/b/c").is_err());
        for (remote, expected) in [
            ("git@bitbucket.org:acme/web.git", repo("acme", "web")),
            (
                "https://ada@bitbucket.org/acme/web.git",
                repo("acme", "web"),
            ),
            (
                "https://bitbucket.corp/scm/PROJ/web.git",
                repo("PROJ", "web"),
            ),
            (
                "ssh://git@bitbucket.corp:7999/proj/web.git",
                repo("proj", "web"),
            ),
            (
                "https://bitbucket.corp/bitbucket/scm/PROJ/web.git",
                repo("PROJ", "web"),
            ),
            ("/local/path", None),
        ] {
            assert_eq!(RepoRef::from_remote(remote), expected, "{remote}");
        }
    }

    #[test]
    fn data_center_epoch_milliseconds_become_utc_dates() {
        assert_eq!(
            epoch_ms(Some(&json!(1_700_000_000_123_i64))).as_deref(),
            Some("2023-11-14T22:13:20Z")
        );
        assert_eq!(
            epoch_ms(Some(&json!(0))).as_deref(),
            Some("1970-01-01T00:00:00Z")
        );
        assert_eq!(epoch_ms(None), None);
    }
}
