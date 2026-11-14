//! Task-level Confluence API that works the same on Cloud (REST v2, with v1
//! for CQL search) and Data Center (`/rest/api/content`).

use crate::catalog::HttpMethod;
use crate::client::Client;
use crate::config::{Deployment, Family};
use crate::error::{Error, Result};
use crate::json::{items, text};
use crate::markup;
use crate::ql::quote;
use percent_encoding::{NON_ALPHANUMERIC, utf8_percent_encode};
use serde::Serialize;
use serde_json::{Value, json};
use std::collections::BTreeMap;

/// How a page is named on the command line. Task arguments deserialize it
/// from that text, so a bad reference fails before the task runs.
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(try_from = "String")]
pub enum PageRef {
    Id(String),
    Title { space: String, title: String },
}

impl TryFrom<String> for PageRef {
    type Error = Error;
    fn try_from(text: String) -> Result<Self> {
        Self::parse(&text)
    }
}

impl schemars::JsonSchema for PageRef {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        "PageRef".into()
    }
    fn inline_schema() -> bool {
        true
    }
    fn json_schema(_: &mut schemars::SchemaGenerator) -> schemars::Schema {
        schemars::json_schema!({ "type": "string" })
    }
}

impl PageRef {
    /// Accepts a numeric ID, a page URL (`.../pages/123/...` or
    /// `...?pageId=123`), or `SPACE:Title`.
    pub fn parse(text: &str) -> Result<PageRef> {
        let text = text.trim();
        if !text.is_empty() && text.bytes().all(|b| b.is_ascii_digit()) {
            return Ok(PageRef::Id(text.to_owned()));
        }
        if text.starts_with("http://") || text.starts_with("https://") {
            let digits = |rest: &str| rest.bytes().take_while(u8::is_ascii_digit).count();
            for marker in ["/pages/", "pageId="] {
                if let Some(at) = text.find(marker) {
                    let rest = &text[at + marker.len()..];
                    let n = digits(rest);
                    if n > 0 {
                        return Ok(PageRef::Id(rest[..n].to_owned()));
                    }
                }
            }
            return Err(Error::Invalid(format!(
                "cannot find a page ID in `{text}`; use a URL containing /pages/<id>/ or pageId=<id>, the numeric ID, or SPACE:Title"
            )));
        }
        if let Some((space, title)) = text.split_once(':')
            && !space.is_empty()
            && !title.is_empty()
            && !space.contains(' ')
        {
            return Ok(PageRef::Title {
                space: space.to_owned(),
                title: title.to_owned(),
            });
        }
        Err(Error::Invalid(format!(
            "`{text}` is not a page reference; use a numeric ID, a page URL, or SPACE:Title (for example DOCS:Handbook)"
        )))
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Page {
    pub id: String,
    pub title: String,
    pub space: Option<String>,
    /// ID of the parent page (none for a page at the top of a space).
    pub parent: Option<String>,
    pub version: u64,
    pub url: String,
    /// The body as Markdown.
    pub markdown: String,
    /// The body in storage format, for lossless edits.
    #[serde(skip)]
    pub storage: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct PageSummary {
    pub id: String,
    pub title: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Space {
    pub key: String,
    pub name: String,
    pub id: Option<String>,
    #[serde(rename = "type")]
    pub space_type: Option<String>,
    pub url: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SearchHit {
    pub id: String,
    #[serde(rename = "type")]
    pub content_type: String,
    pub title: String,
    pub space: Option<String>,
    pub excerpt: Option<String>,
    pub url: String,
}

pub struct Confluence<'a> {
    client: &'a Client,
    cloud: bool,
    /// Cloud space ID -> key: an export or push reads many pages of one space.
    space_keys: std::sync::Mutex<std::collections::HashMap<String, Option<String>>>,
}

impl<'a> Confluence<'a> {
    pub fn new(client: &'a Client) -> Self {
        Self {
            client,
            cloud: client.deployment(Family::Confluence) == Deployment::Cloud,
            space_keys: Default::default(),
        }
    }

    /// The key of a Cloud space, fetched once per space.
    async fn space_key(&self, space_id: &str) -> Result<Option<String>> {
        if let Some(key) = self
            .space_keys
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .get(space_id)
        {
            return Ok(key.clone());
        }
        let found = self
            .send(
                HttpMethod::Get,
                &format!("/wiki/api/v2/spaces/{space_id}"),
                &[],
                None,
            )
            .await?;
        let key = text(&found, "/key");
        self.space_keys
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .insert(space_id.to_owned(), key.clone());
        Ok(key)
    }

    async fn send(
        &self,
        method: HttpMethod,
        path: &str,
        query: &[(&str, String)],
        body: Option<Value>,
    ) -> Result<Value> {
        self.client
            .send_json(Family::Confluence, method, path, query, body)
            .await
    }

    /// Browser URL for a `_links.webui` path.
    /// A site URL for a `_links.webui` or download path; absolute URLs pass through.
    fn web_url(&self, webui: &str) -> String {
        if webui.starts_with("http") {
            return webui.to_owned();
        }
        let site = self.client.site_url(Family::Confluence);
        if self.cloud {
            format!("{site}/wiki{webui}")
        } else {
            format!("{site}{webui}")
        }
    }

    async fn space_id(&self, key: &str) -> Result<String> {
        let found = self
            .send(
                HttpMethod::Get,
                "/wiki/api/v2/spaces",
                &[("keys", key.to_owned())],
                None,
            )
            .await?;
        text(&found, "/results/0/id").ok_or_else(|| {
            Error::Invalid(format!(
                "no Confluence space has key `{key}`; list keys with `atl confluence space list`"
            ))
        })
    }

    /// The page ID for a reference.
    pub async fn resolve(&self, page: &PageRef) -> Result<String> {
        let (space, title) = match page {
            PageRef::Id(id) => return Ok(id.clone()),
            PageRef::Title { space, title } => (space, title),
        };
        let found = if self.cloud {
            let space_id = self.space_id(space).await?;
            self.send(
                HttpMethod::Get,
                "/wiki/api/v2/pages",
                &[("space-id", space_id), ("title", title.clone())],
                None,
            )
            .await?
        } else {
            self.send(
                HttpMethod::Get,
                "/rest/api/content",
                &[("spaceKey", space.clone()), ("title", title.clone())],
                None,
            )
            .await?
        };
        text(&found, "/results/0/id").ok_or_else(|| {
            Error::Invalid(format!(
                "no page titled `{title}` in space `{space}`; titles are exact and case-sensitive"
            ))
        })
    }

    pub async fn page(&self, page: &PageRef) -> Result<Page> {
        self.page_at(page, None).await
    }

    /// The page, or an older `version` of it.
    pub async fn page_at(&self, page: &PageRef, version: Option<u64>) -> Result<Page> {
        let id = self.resolve(page).await?;
        if self.cloud {
            let mut query = vec![("body-format", "storage".to_owned())];
            if let Some(version) = version {
                query.push(("version", version.to_string()));
            }
            let found = self
                .send(
                    HttpMethod::Get,
                    &format!("/wiki/api/v2/pages/{id}"),
                    &query,
                    None,
                )
                .await?;
            let space = match text(&found, "/spaceId") {
                Some(space_id) => self.space_key(&space_id).await?,
                None => None,
            };
            let storage = text(&found, "/body/storage/value").unwrap_or_default();
            Ok(Page {
                id,
                title: text(&found, "/title").unwrap_or_default(),
                space,
                parent: text(&found, "/parentId"),
                version: found
                    .pointer("/version/number")
                    .and_then(Value::as_u64)
                    .unwrap_or(1),
                url: self.web_url(&text(&found, "/_links/webui").unwrap_or_default()),
                markdown: markup::storage_to_markdown(&storage),
                storage,
            })
        } else {
            let mut query = vec![("expand", "body.storage,version,space,ancestors".to_owned())];
            if let Some(version) = version {
                query.push(("status", "historical".into()));
                query.push(("version", version.to_string()));
            }
            let found = self
                .send(
                    HttpMethod::Get,
                    &format!("/rest/api/content/{id}"),
                    &query,
                    None,
                )
                .await?;
            let storage = text(&found, "/body/storage/value").unwrap_or_default();
            Ok(Page {
                id,
                title: text(&found, "/title").unwrap_or_default(),
                space: text(&found, "/space/key"),
                parent: found
                    .get("ancestors")
                    .and_then(Value::as_array)
                    .and_then(|a| a.last())
                    .and_then(|a| text(a, "/id")),
                version: found
                    .pointer("/version/number")
                    .and_then(Value::as_u64)
                    .unwrap_or(1),
                url: self.web_url(&text(&found, "/_links/webui").unwrap_or_default()),
                markdown: markup::storage_to_markdown(&storage),
                storage,
            })
        }
    }

    fn summary(&self, value: &Value) -> PageSummary {
        PageSummary {
            id: text(value, "/id").unwrap_or_default(),
            title: text(value, "/title").unwrap_or_default(),
            url: self.web_url(&text(value, "/_links/webui").unwrap_or_default()),
        }
    }

    /// Creates a page from storage-format XHTML (see
    /// [`markup::markdown_to_storage`]).
    pub async fn create(
        &self,
        space: &str,
        title: &str,
        parent: Option<&PageRef>,
        storage: String,
    ) -> Result<PageSummary> {
        let parent_id = match parent {
            Some(parent) => Some(self.resolve(parent).await?),
            None => None,
        };
        let created = if self.cloud {
            let mut body = json!({
                "spaceId": self.space_id(space).await?,
                "status": "current",
                "title": title,
                "body": { "representation": "storage", "value": storage },
            });
            if let Some(parent_id) = parent_id {
                body["parentId"] = parent_id.into();
            }
            self.send(HttpMethod::Post, "/wiki/api/v2/pages", &[], Some(body))
                .await?
        } else {
            let mut body = json!({
                "type": "page",
                "title": title,
                "space": { "key": space },
                "body": { "storage": { "value": storage, "representation": "storage" } },
            });
            if let Some(parent_id) = parent_id {
                body["ancestors"] = json!([{ "id": parent_id }]);
            }
            self.send(HttpMethod::Post, "/rest/api/content", &[], Some(body))
                .await?
        };
        Ok(self.summary(&created))
    }

    /// Replaces the title and/or body; the version number is bumped for you.
    pub async fn update(
        &self,
        page: &PageRef,
        title: Option<&str>,
        storage: Option<String>,
    ) -> Result<PageSummary> {
        if title.is_none() && storage.is_none() {
            return Err(Error::Invalid(
                "nothing to change; pass --title, --body or --file".into(),
            ));
        }
        let current = self.page(page).await?;
        let title = title.unwrap_or(&current.title);
        let storage = storage.unwrap_or(current.storage);
        let version = current.version + 1;
        let updated = if self.cloud {
            let body = json!({
                "id": current.id,
                "status": "current",
                "title": title,
                "body": { "representation": "storage", "value": storage },
                "version": { "number": version },
            });
            self.send(
                HttpMethod::Put,
                &format!("/wiki/api/v2/pages/{}", current.id),
                &[],
                Some(body),
            )
            .await?
        } else {
            let body = json!({
                "id": current.id,
                "type": "page",
                "title": title,
                "version": { "number": version },
                "body": { "storage": { "value": storage, "representation": "storage" } },
            });
            self.send(
                HttpMethod::Put,
                &format!("/rest/api/content/{}", current.id),
                &[],
                Some(body),
            )
            .await?
        };
        Ok(self.summary(&updated))
    }

    pub async fn delete(&self, page: &PageRef) -> Result<String> {
        let id = self.resolve(page).await?;
        let path = if self.cloud {
            format!("/wiki/api/v2/pages/{id}")
        } else {
            format!("/rest/api/content/{id}")
        };
        self.send(HttpMethod::Delete, &path, &[], None).await?;
        Ok(id)
    }

    pub async fn children(&self, page: &PageRef) -> Result<Vec<PageSummary>> {
        let id = self.resolve(page).await?;
        let found = if self.cloud {
            self.send(
                HttpMethod::Get,
                &format!("/wiki/api/v2/pages/{id}/children"),
                &[("limit", "250".into())],
                None,
            )
            .await?
        } else {
            self.send(
                HttpMethod::Get,
                &format!("/rest/api/content/{id}/child/page"),
                &[("limit", "200".into())],
                None,
            )
            .await?
        };
        Ok(items(&found, "/results").map(|p| self.summary(p)).collect())
    }

    /// Top-level pages of a space.
    pub async fn root_pages(&self, space: &str, limit: usize) -> Result<Vec<PageSummary>> {
        let found = if self.cloud {
            let space_id = self.space_id(space).await?;
            self.send(
                HttpMethod::Get,
                &format!("/wiki/api/v2/spaces/{space_id}/pages"),
                &[("depth", "root".into()), ("limit", limit.to_string())],
                None,
            )
            .await?
        } else {
            self.send(
                HttpMethod::Get,
                &format!("/rest/api/space/{space}/content/page"),
                &[("depth", "root".into()), ("limit", limit.to_string())],
                None,
            )
            .await?
        };
        Ok(items(&found, "/results").map(|p| self.summary(p)).collect())
    }

    pub async fn spaces(&self, limit: usize) -> Result<Vec<Space>> {
        let path = if self.cloud {
            "/wiki/api/v2/spaces"
        } else {
            "/rest/api/space"
        };
        let found = self
            .send(HttpMethod::Get, path, &[("limit", limit.to_string())], None)
            .await?;
        Ok(items(&found, "/results")
            .map(|s| Space {
                key: text(s, "/key").unwrap_or_default(),
                name: text(s, "/name").unwrap_or_default(),
                id: if self.cloud { text(s, "/id") } else { None },
                space_type: text(s, "/type"),
                url: self.web_url(&text(s, "/_links/webui").unwrap_or_default()),
            })
            .collect())
    }

    /// CQL search, for example `text ~ "deploy" AND type = page`.
    pub async fn search(&self, cql: &str, limit: usize) -> Result<Vec<SearchHit>> {
        let query = [("cql", cql.to_owned()), ("limit", limit.to_string())];
        if self.cloud {
            let found = self
                .send(HttpMethod::Get, "/wiki/rest/api/search", &query, None)
                .await?;
            Ok(items(&found, "/results")
                .map(|r| SearchHit {
                    id: text(r, "/content/id").unwrap_or_default(),
                    content_type: text(r, "/content/type")
                        .or_else(|| text(r, "/entityType"))
                        .unwrap_or_default(),
                    title: text(r, "/content/title")
                        .or_else(|| text(r, "/title"))
                        .unwrap_or_default(),
                    space: text(r, "/resultGlobalContainer/title"),
                    excerpt: text(r, "/excerpt").filter(|e| !e.is_empty()),
                    url: self.web_url(
                        &text(r, "/url")
                            .or_else(|| text(r, "/content/_links/webui"))
                            .unwrap_or_default(),
                    ),
                })
                .collect())
        } else {
            let mut query = query.to_vec();
            query.push(("expand", "space".into()));
            let found = self
                .send(HttpMethod::Get, "/rest/api/content/search", &query, None)
                .await?;
            Ok(items(&found, "/results")
                .map(|r| SearchHit {
                    id: text(r, "/id").unwrap_or_default(),
                    content_type: text(r, "/type").unwrap_or_default(),
                    title: text(r, "/title").unwrap_or_default(),
                    space: text(r, "/space/key"),
                    excerpt: None,
                    url: self.web_url(&text(r, "/_links/webui").unwrap_or_default()),
                })
                .collect())
        }
    }
}

/// One node of a page tree (see [`Confluence::tree`]).
#[derive(Debug, Clone, Serialize)]
pub struct TreeNode {
    pub id: String,
    pub title: String,
    /// page, folder, whiteboard, database or embed (Data Center: page).
    #[serde(rename = "type")]
    pub kind: String,
    /// 1 for direct children of the root.
    pub depth: usize,
    /// ID of the parent node.
    pub parent: String,
    pub url: String,
}

impl Confluence<'_> {
    /// Direct children of a page or (Cloud) folder: pages and, on Cloud,
    /// folders and other content, following pagination.
    async fn direct_children(
        &self,
        id: &str,
        kind: &str,
    ) -> Result<Vec<(String, String, String, String)>> {
        let mut out = Vec::new();
        if self.cloud {
            let collection = if kind == "folder" { "folders" } else { "pages" };
            let mut next = Some(format!(
                "/wiki/api/v2/{collection}/{id}/direct-children?limit=250"
            ));
            while let Some(path) = next.take() {
                let found = self.send(HttpMethod::Get, &path, &[], None).await?;
                for child in items(&found, "/results") {
                    let child_id = text(child, "/id").unwrap_or_default();
                    let child_kind = text(child, "/type").unwrap_or_else(|| "page".into());
                    let url = match child_kind.as_str() {
                        "page" => {
                            self.web_url(&format!("/pages/viewpage.action?pageId={child_id}"))
                        }
                        _ => String::new(),
                    };
                    out.push((
                        child_id,
                        text(child, "/title").unwrap_or_default(),
                        child_kind,
                        url,
                    ));
                }
                next = text(&found, "/_links/next");
            }
            return Ok(out);
        }
        loop {
            let found = self
                .send(
                    HttpMethod::Get,
                    &format!("/rest/api/content/{id}/child/page"),
                    &[("limit", "200".into()), ("start", out.len().to_string())],
                    None,
                )
                .await?;
            let before = out.len();
            for child in items(&found, "/results") {
                let summary = self.summary(child);
                out.push((summary.id, summary.title, "page".to_owned(), summary.url));
            }
            if out.len() - before < 200 {
                return Ok(out);
            }
        }
    }

    /// The pages (and on Cloud folders and other content) under `root`,
    /// depth first: each node is followed by its descendants. `max_depth`
    /// 1 gives the direct children; `None` walks the whole tree.
    pub async fn tree(&self, root: &PageRef, max_depth: Option<usize>) -> Result<Vec<TreeNode>> {
        let root = self.resolve(root).await?;
        let mut out = Vec::new();
        let node = |(id, title, kind, url): (String, String, String, String),
                    depth: usize,
                    parent: &str| TreeNode {
            id,
            title,
            kind,
            depth,
            parent: parent.to_owned(),
            url,
        };
        let mut stack: Vec<TreeNode> = self
            .direct_children(&root, "page")
            .await?
            .into_iter()
            .rev()
            .map(|child| node(child, 1, &root))
            .collect();
        while let Some(current) = stack.pop() {
            let descend = max_depth.is_none_or(|max| current.depth < max)
                && matches!(current.kind.as_str(), "page" | "folder");
            if descend {
                let children = self.direct_children(&current.id, &current.kind).await?;
                stack.extend(
                    children
                        .into_iter()
                        .rev()
                        .map(|child| node(child, current.depth + 1, &current.id)),
                );
            }
            out.push(current);
        }
        Ok(out)
    }

    /// Pages whose title contains the words of `title`, optionally in one space.
    pub async fn find(
        &self,
        title: &str,
        space: Option<&str>,
        limit: usize,
    ) -> Result<Vec<SearchHit>> {
        let mut cql = format!("type = page AND title ~ {}", quote(title));
        if let Some(space) = space {
            cql.push_str(&format!(" AND space = {}", quote(space)));
        }
        self.search(&cql, limit).await
    }
}

/// CQL for plain words, optionally limited to a space and a content type.
pub fn text_cql(words: &str, space: Option<&str>, content_type: Option<&str>) -> String {
    let mut clauses = vec![format!("text ~ {}", quote(words))];
    if let Some(space) = space {
        clauses.push(format!("space = {}", quote(space)));
    }
    if let Some(kind) = content_type {
        clauses.push(format!("type = {}", quote(kind)));
    }
    clauses.join(" AND ")
}

#[derive(Debug, Clone, Serialize)]
pub struct PageAttachment {
    pub id: String,
    pub title: String,
    pub size: Option<u64>,
    pub media_type: Option<String>,
    /// Download URL on the Confluence server.
    pub download: String,
}

impl Confluence<'_> {
    pub async fn attachments(&self, page: &PageRef) -> Result<Vec<PageAttachment>> {
        let id = self.resolve(page).await?;
        let found = if self.cloud {
            self.send(
                HttpMethod::Get,
                &format!("/wiki/api/v2/pages/{id}/attachments"),
                &[("limit", "250".into())],
                None,
            )
            .await?
        } else {
            self.send(
                HttpMethod::Get,
                &format!("/rest/api/content/{id}/child/attachment"),
                &[("limit", "200".into())],
                None,
            )
            .await?
        };
        Ok(items(&found, "/results")
            .map(|a| self.attachment_from(a))
            .collect())
    }

    fn attachment_from(&self, a: &Value) -> PageAttachment {
        PageAttachment {
            id: text(a, "/id").unwrap_or_default(),
            title: text(a, "/title").unwrap_or_default(),
            size: a
                .get("fileSize")
                .or_else(|| a.pointer("/extensions/fileSize"))
                .and_then(Value::as_u64),
            media_type: text(a, "/mediaType").or_else(|| text(a, "/extensions/mediaType")),
            download: self.web_url(
                &text(a, "/downloadLink")
                    .or_else(|| text(a, "/_links/download"))
                    .unwrap_or_default(),
            ),
        }
    }

    pub async fn attach(
        &self,
        page: &PageRef,
        files: &[std::path::PathBuf],
    ) -> Result<Vec<PageAttachment>> {
        let id = self.resolve(page).await?;
        let path = format!("{}/content/{id}/child/attachment", self.v1());
        let response = self.client.upload(Family::Confluence, &path, files).await?;
        Ok(items(&response, "/results")
            .map(|a| self.attachment_from(a))
            .collect())
    }

    /// Uploads files, replacing attachments with the same name (new
    /// versions of them) instead of failing.
    pub async fn attach_replace(
        &self,
        page: &PageRef,
        files: &[std::path::PathBuf],
    ) -> Result<Vec<PageAttachment>> {
        let id = self.resolve(page).await?;
        if self.cloud {
            let response = self
                .client
                .upload_with(
                    Family::Confluence,
                    HttpMethod::Put,
                    &format!("/wiki/rest/api/content/{id}/child/attachment"),
                    files,
                )
                .await?;
            return Ok(items(&response, "/results")
                .map(|a| self.attachment_from(a))
                .collect());
        }
        // Data Center has no create-or-update: post new data to the existing one.
        let existing = self.attachments(&PageRef::Id(id.clone())).await?;
        let mut out = Vec::new();
        for file in files {
            let name = file
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default();
            let path = match existing.iter().find(|a| a.title == name) {
                Some(found) => format!("/rest/api/content/{id}/child/attachment/{}/data", found.id),
                None => format!("/rest/api/content/{id}/child/attachment"),
            };
            let response = self
                .client
                .upload(Family::Confluence, &path, std::slice::from_ref(file))
                .await?;
            match response.get("results") {
                Some(_) => {
                    out.extend(items(&response, "/results").map(|a| self.attachment_from(a)))
                }
                None => out.push(self.attachment_from(&response)),
            }
        }
        Ok(out)
    }

    pub async fn delete_attachment(&self, id: &str) -> Result<()> {
        let path = if self.cloud {
            format!("/wiki/api/v2/attachments/{id}")
        } else {
            format!("/rest/api/content/{id}")
        };
        self.send(HttpMethod::Delete, &path, &[], None).await?;
        Ok(())
    }

    /// Bytes of an attachment, by its `download` URL.
    pub async fn download(&self, url: &str) -> Result<Vec<u8>> {
        self.client.download(Family::Confluence, url).await
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct PageComment {
    pub id: String,
    /// Display name (account ID on Cloud when the name lookup fails).
    pub author: Option<String>,
    pub created: Option<String>,
    /// The comment as Markdown.
    pub body: String,
    /// The comment this one replies to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<String>,
    /// For inline comments, the page text they are attached to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selection: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PageVersion {
    pub number: u64,
    pub author: Option<String>,
    pub created: Option<String>,
    pub message: Option<String>,
    pub minor_edit: bool,
}

/// Where [`Confluence::move_page`] puts the page relative to the target.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MovePosition {
    /// The last child of the target.
    Append,
    /// The sibling before the target (Cloud only).
    Before,
    /// The sibling after the target (Cloud only).
    After,
}

impl MovePosition {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Append => "append",
            Self::Before => "before",
            Self::After => "after",
        }
    }
}

impl std::str::FromStr for MovePosition {
    type Err = Error;

    fn from_str(text: &str) -> Result<Self> {
        match text {
            "append" => Ok(Self::Append),
            "before" => Ok(Self::Before),
            "after" => Ok(Self::After),
            _ => Err(Error::Invalid(format!(
                "move position `{text}` is unknown; use append (child of the target), before or after"
            ))),
        }
    }
}

impl Confluence<'_> {
    /// REST v1 prefix: `/wiki/rest/api` (Cloud) or `/rest/api` (Data Center).
    fn v1(&self) -> &'static str {
        if self.cloud {
            "/wiki/rest/api"
        } else {
            "/rest/api"
        }
    }

    /// Display names of Cloud account IDs; empty on Data Center or on error.
    async fn names(&self, ids: Vec<String>) -> BTreeMap<String, String> {
        let mut ids = ids;
        ids.sort_unstable();
        ids.dedup();
        if !self.cloud || ids.is_empty() {
            return BTreeMap::new();
        }
        let Ok(found) = self
            .send(
                HttpMethod::Get,
                "/wiki/rest/api/user/bulk",
                &[("accountId", ids.join(","))],
                None,
            )
            .await
        else {
            return BTreeMap::new();
        };
        items(&found, "/results")
            .filter_map(|u| Some((text(u, "/accountId")?, text(u, "/displayName")?)))
            .collect()
    }

    /// Footer comments (or with `inline`, inline comments), oldest first,
    /// with Markdown bodies.
    pub async fn comments(
        &self,
        page: &PageRef,
        limit: usize,
        inline: bool,
    ) -> Result<Vec<PageComment>> {
        let id = self.resolve(page).await?;
        let limit = limit.clamp(1, 250).to_string();
        if self.cloud {
            let kind = if inline {
                "inline-comments"
            } else {
                "footer-comments"
            };
            let found = self
                .send(
                    HttpMethod::Get,
                    &format!("/wiki/api/v2/pages/{id}/{kind}"),
                    &[("body-format", "storage".into()), ("limit", limit)],
                    None,
                )
                .await?;
            let mut comments: Vec<PageComment> = items(&found, "/results")
                .map(|c| PageComment {
                    id: text(c, "/id").unwrap_or_default(),
                    author: text(c, "/version/authorId"),
                    created: text(c, "/version/createdAt"),
                    body: markup::storage_to_markdown(
                        &text(c, "/body/storage/value").unwrap_or_default(),
                    ),
                    parent: text(c, "/parentCommentId"),
                    selection: text(c, "/properties/inlineOriginalSelection"),
                })
                .collect();
            let names = self
                .names(comments.iter().filter_map(|c| c.author.clone()).collect())
                .await;
            for comment in &mut comments {
                if let Some(name) = comment.author.as_ref().and_then(|id| names.get(id)) {
                    comment.author = Some(name.clone());
                }
            }
            return Ok(comments);
        }
        let found = self
            .send(
                HttpMethod::Get,
                &format!("/rest/api/content/{id}/child/comment"),
                &[
                    (
                        "expand",
                        "body.storage,history,ancestors,extensions.inlineProperties".into(),
                    ),
                    ("limit", limit),
                    ("location", if inline { "inline" } else { "footer" }.into()),
                    ("depth", "all".into()),
                ],
                None,
            )
            .await?;
        Ok(items(&found, "/results")
            .map(|c| PageComment {
                id: text(c, "/id").unwrap_or_default(),
                author: text(c, "/history/createdBy/displayName"),
                created: text(c, "/history/createdDate"),
                body: markup::storage_to_markdown(
                    &text(c, "/body/storage/value").unwrap_or_default(),
                ),
                parent: c
                    .get("ancestors")
                    .and_then(Value::as_array)
                    .and_then(|a| a.last())
                    .and_then(|a| text(a, "/id")),
                selection: text(c, "/extensions/inlineProperties/originalSelection"),
            })
            .collect())
    }

    /// Adds a footer comment written in Markdown, or a reply to the
    /// comment `parent`.
    pub async fn comment(
        &self,
        page: &PageRef,
        markdown: &str,
        parent: Option<&str>,
    ) -> Result<PageComment> {
        let id = self.resolve(page).await?;
        let storage = markup::markdown_to_storage(markdown);
        let created = if self.cloud {
            let mut body = json!({ "body": { "representation": "storage", "value": storage } });
            match parent {
                Some(parent) => body["parentCommentId"] = parent.into(),
                None => body["pageId"] = id.clone().into(),
            }
            self.send(
                HttpMethod::Post,
                "/wiki/api/v2/footer-comments",
                &[],
                Some(body),
            )
            .await?
        } else {
            let mut body = json!({
                "type": "comment",
                "container": { "id": id, "type": "page" },
                "body": { "storage": { "value": storage, "representation": "storage" } },
            });
            if let Some(parent) = parent {
                body["ancestors"] = json!([{ "id": parent }]);
            }
            self.send(HttpMethod::Post, "/rest/api/content", &[], Some(body))
                .await?
        };
        Ok(PageComment {
            id: text(&created, "/id").unwrap_or_default(),
            author: None,
            created: text(&created, "/version/createdAt")
                .or_else(|| text(&created, "/history/createdDate")),
            body: markdown.to_owned(),
            parent: parent.map(str::to_owned),
            selection: None,
        })
    }

    /// Deletes a footer or inline comment.
    pub async fn delete_comment(&self, id: &str) -> Result<()> {
        self.send(
            HttpMethod::Delete,
            &format!("{}/content/{id}", self.v1()),
            &[],
            None,
        )
        .await?;
        Ok(())
    }

    pub async fn labels(&self, page: &PageRef) -> Result<Vec<String>> {
        let id = self.resolve(page).await?;
        let path = if self.cloud {
            format!("/wiki/api/v2/pages/{id}/labels")
        } else {
            format!("/rest/api/content/{id}/label")
        };
        let found = self
            .send(HttpMethod::Get, &path, &[("limit", "200".into())], None)
            .await?;
        Ok(items(&found, "/results")
            .filter_map(|l| text(l, "/name"))
            .collect())
    }

    /// Adds and removes labels, then returns the page's labels.
    pub async fn change_labels(
        &self,
        page: &PageRef,
        add: &[String],
        remove: &[String],
    ) -> Result<Vec<String>> {
        if add.is_empty() && remove.is_empty() {
            return Err(Error::Invalid(
                "nothing to change; pass labels to add or remove".into(),
            ));
        }
        let id = self.resolve(page).await?;
        let page = PageRef::Id(id.clone());
        if !add.is_empty() {
            let body: Vec<Value> = add
                .iter()
                .map(|name| json!({ "prefix": "global", "name": name }))
                .collect();
            self.send(
                HttpMethod::Post,
                &format!("{}/content/{id}/label", self.v1()),
                &[],
                Some(Value::Array(body)),
            )
            .await?;
        }
        for name in remove {
            let label = utf8_percent_encode(name, NON_ALPHANUMERIC);
            self.send(
                HttpMethod::Delete,
                &format!("{}/content/{id}/label/{label}", self.v1()),
                &[],
                None,
            )
            .await?;
        }
        self.labels(&page).await
    }

    /// Moves a page next to or under `target`. Data Center supports only
    /// [`MovePosition::Append`] (a new parent).
    pub async fn move_page(
        &self,
        page: &PageRef,
        position: MovePosition,
        target: &PageRef,
    ) -> Result<PageSummary> {
        let target = self.resolve(target).await?;
        if self.cloud {
            let id = self.resolve(page).await?;
            let moved = self
                .send(
                    HttpMethod::Put,
                    &format!(
                        "/wiki/rest/api/content/{id}/move/{}/{target}",
                        position.as_str()
                    ),
                    &[],
                    None,
                )
                .await?;
            let mut summary = self.summary(&moved);
            if summary.id.is_empty() {
                summary.id = id;
            }
            return Ok(summary);
        }
        if position != MovePosition::Append {
            return Err(Error::Invalid(
                "Confluence Data Center can only give a page a new parent; pass --parent instead of --before or --after".into(),
            ));
        }
        let current = self.page(page).await?;
        let moved = self
            .send(
                HttpMethod::Put,
                &format!("/rest/api/content/{}", current.id),
                &[],
                Some(json!({
                    "id": current.id,
                    "type": "page",
                    "title": current.title,
                    "version": { "number": current.version + 1 },
                    "ancestors": [{ "id": target }],
                    "body": { "storage": { "value": current.storage, "representation": "storage" } },
                })),
            )
            .await?;
        Ok(self.summary(&moved))
    }

    /// The page's versions, newest first.
    pub async fn versions(&self, page: &PageRef, limit: usize) -> Result<Vec<PageVersion>> {
        let id = self.resolve(page).await?;
        let limit = limit.clamp(1, 200).to_string();
        if self.cloud {
            let found = self
                .send(
                    HttpMethod::Get,
                    &format!("/wiki/api/v2/pages/{id}/versions"),
                    &[("limit", limit), ("sort", "-modified-date".into())],
                    None,
                )
                .await?;
            let mut versions: Vec<PageVersion> = items(&found, "/results")
                .map(|v| PageVersion {
                    number: v.get("number").and_then(Value::as_u64).unwrap_or_default(),
                    author: text(v, "/authorId"),
                    created: text(v, "/createdAt"),
                    message: text(v, "/message").filter(|m| !m.is_empty()),
                    minor_edit: v.get("minorEdit").and_then(Value::as_bool).unwrap_or(false),
                })
                .collect();
            let names = self
                .names(versions.iter().filter_map(|v| v.author.clone()).collect())
                .await;
            for version in &mut versions {
                if let Some(name) = version.author.as_ref().and_then(|id| names.get(id)) {
                    version.author = Some(name.clone());
                }
            }
            return Ok(versions);
        }
        let found = self
            .send(
                HttpMethod::Get,
                &format!("/rest/experimental/content/{id}/version"),
                &[("limit", limit)],
                None,
            )
            .await
            .map_err(|err| match err {
                Error::Http(http) if http.status == 404 => Error::Invalid(
                    "this Confluence server has no version list API (it needs Data Center 7 or later); open the page history in the browser".into(),
                ),
                other => other,
            })?;
        Ok(items(&found, "/results")
            .map(|v| PageVersion {
                number: v.get("number").and_then(Value::as_u64).unwrap_or_default(),
                author: text(v, "/by/displayName"),
                created: text(v, "/when"),
                message: text(v, "/message").filter(|m| !m.is_empty()),
                minor_edit: v.get("minorEdit").and_then(Value::as_bool).unwrap_or(false),
            })
            .collect())
    }
}

impl Confluence<'_> {
    /// Blog posts of a space, newest first.
    pub async fn blog_posts(&self, space: &str, limit: usize) -> Result<Vec<PageSummary>> {
        let found = if self.cloud {
            let id = self.space_id(space).await?;
            self.send(
                HttpMethod::Get,
                &format!("/wiki/api/v2/spaces/{id}/blogposts"),
                &[
                    ("limit", limit.clamp(1, 250).to_string()),
                    ("sort", "-created-date".into()),
                ],
                None,
            )
            .await?
        } else {
            self.send(
                HttpMethod::Get,
                "/rest/api/content",
                &[
                    ("type", "blogpost".into()),
                    ("spaceKey", space.to_owned()),
                    ("limit", limit.clamp(1, 200).to_string()),
                    ("orderby", "history.createdDate desc".into()),
                ],
                None,
            )
            .await?
        };
        Ok(items(&found, "/results").map(|b| self.summary(b)).collect())
    }

    /// One blog post by ID, with its body as Markdown.
    pub async fn blog_post(&self, id: &str) -> Result<Page> {
        if !self.cloud {
            // Data Center serves blog posts from the content API, like pages.
            return self.page(&PageRef::Id(id.to_owned())).await;
        }
        let found = self
            .send(
                HttpMethod::Get,
                &format!("/wiki/api/v2/blogposts/{id}"),
                &[("body-format", "storage".into())],
                None,
            )
            .await?;
        let storage = text(&found, "/body/storage/value").unwrap_or_default();
        let space = match text(&found, "/spaceId") {
            Some(space_id) => self.space_key(&space_id).await?,
            None => None,
        };
        Ok(Page {
            id: id.to_owned(),
            title: text(&found, "/title").unwrap_or_default(),
            space,
            parent: None,
            version: found
                .pointer("/version/number")
                .and_then(Value::as_u64)
                .unwrap_or(1),
            url: self.web_url(&text(&found, "/_links/webui").unwrap_or_default()),
            markdown: markup::storage_to_markdown(&storage),
            storage,
        })
    }

    /// Publishes a blog post from storage-format XHTML.
    pub async fn create_blog_post(
        &self,
        space: &str,
        title: &str,
        storage: String,
    ) -> Result<PageSummary> {
        let created = if self.cloud {
            let body = json!({
                "spaceId": self.space_id(space).await?,
                "status": "current",
                "title": title,
                "body": { "representation": "storage", "value": storage },
            });
            self.send(HttpMethod::Post, "/wiki/api/v2/blogposts", &[], Some(body))
                .await?
        } else {
            let body = json!({
                "type": "blogpost",
                "title": title,
                "space": { "key": space },
                "body": { "storage": { "value": storage, "representation": "storage" } },
            });
            self.send(HttpMethod::Post, "/rest/api/content", &[], Some(body))
                .await?
        };
        Ok(self.summary(&created))
    }

    /// Creates a space with a plain-text description.
    pub async fn create_space(
        &self,
        key: &str,
        name: &str,
        description: Option<&str>,
    ) -> Result<Space> {
        let created = if self.cloud {
            let mut body = json!({ "key": key, "name": name });
            if let Some(description) = description {
                body["description"] = json!({ "value": description, "representation": "plain" });
            }
            self.send(HttpMethod::Post, "/wiki/api/v2/spaces", &[], Some(body))
                .await?
        } else {
            let mut body = json!({ "key": key, "name": name });
            if let Some(description) = description {
                body["description"] =
                    json!({ "plain": { "value": description, "representation": "plain" } });
            }
            self.send(HttpMethod::Post, "/rest/api/space", &[], Some(body))
                .await?
        };
        Ok(Space {
            key: text(&created, "/key").unwrap_or_else(|| key.to_owned()),
            name: text(&created, "/name").unwrap_or_else(|| name.to_owned()),
            id: text(&created, "/id").or_else(|| {
                created
                    .get("id")
                    .and_then(Value::as_u64)
                    .map(|id| id.to_string())
            }),
            space_type: text(&created, "/type"),
            url: self.web_url(&text(&created, "/_links/webui").unwrap_or_default()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn page_references_accept_ids_urls_and_titles() {
        let id = |s: &str| Ok(PageRef::Id(s.into()));
        let cases: [(&str, Result<PageRef, ()>); 6] = [
            ("12345", id("12345")),
            (
                "https://acme.atlassian.net/wiki/spaces/DOCS/pages/98765/Handbook",
                id("98765"),
            ),
            (
                "https://wiki.corp/pages/viewpage.action?pageId=42",
                id("42"),
            ),
            (
                "DOCS:Getting started: day one",
                Ok(PageRef::Title {
                    space: "DOCS".into(),
                    title: "Getting started: day one".into(),
                }),
            ),
            ("https://acme.atlassian.net/wiki/spaces/DOCS", Err(())),
            ("just words", Err(())),
        ];
        for (input, expected) in cases {
            assert_eq!(PageRef::parse(input).map_err(|_| ()), expected, "{input}");
        }
    }
}
