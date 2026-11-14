//! Follows Atlassian's pagination hints page by page.
//!
//! Detection order (first match wins): `Link: <...>; rel="next"` header;
//! `_links.next` (Confluence); top-level `next` URL (Bitbucket Cloud);
//! `links.next` (Admin); `nextPageToken` (Jira enhanced search);
//! `isLastPage`/`nextPageStart` (Bitbucket Data Center); and
//! `startAt`/`maxResults`/`total`/`isLast` (classic Jira).

use crate::client::{Client, PreparedBody, PreparedRequest, Response};
use crate::error::Result;
use serde_json::Value;
use std::time::Duration;
use url::Url;

pub struct Pager<'a> {
    client: &'a Client,
    next: Option<PreparedRequest>,
    pages: usize,
    limit: usize,
    delay: Duration,
}

impl<'a> Pager<'a> {
    /// The wait between pages unless a caller asks for another.
    pub const DEFAULT_DELAY: Duration = Duration::from_millis(100);

    /// Fetches at most `limit` pages (at least one), waiting `delay` between requests.
    pub fn new(client: &'a Client, first: PreparedRequest, limit: usize, delay: Duration) -> Self {
        Self {
            client,
            next: Some(first),
            pages: 0,
            limit: limit.max(1),
            delay,
        }
    }

    /// Returns the next page body, or `None` after the last page or the limit.
    pub async fn next_page(&mut self) -> Result<Option<Value>> {
        if self.pages >= self.limit {
            return Ok(None);
        }
        let Some(request) = self.next.take() else {
            return Ok(None);
        };
        if self.pages > 0 && !self.delay.is_zero() {
            crate::rt::sleep(self.delay).await;
        }
        let response = self.client.execute(&request).await?;
        self.pages += 1;
        let body = response.json().unwrap_or(Value::Null);
        // The credentials only go to the server they belong to: a `next` link
        // to another origin ends the paging instead of being followed.
        self.next = next_request(&request, &response, &body).filter(|next| {
            next.url.origin() == request.url.origin()
                && (next.url != request.url || next.body != request.body)
        });
        Ok(Some(body))
    }

    /// True when the page limit stopped the pager and more pages exist.
    pub fn truncated(&self) -> bool {
        self.pages >= self.limit && self.next.is_some()
    }
}

pub(crate) fn next_request(
    previous: &PreparedRequest,
    response: &Response,
    body: &Value,
) -> Option<PreparedRequest> {
    if let Some(url) = link_header_next(previous, response) {
        return Some(previous.with_url(url));
    }
    if let Some(next) = body.pointer("/_links/next").and_then(Value::as_str) {
        return resolve_link(
            previous,
            body.pointer("/_links/base").and_then(Value::as_str),
            next,
        )
        .map(|url| previous.with_url(url));
    }
    if let Some(next) = body.get("next").and_then(Value::as_str) {
        return Url::parse(next).ok().map(|url| previous.with_url(url));
    }
    if let Some(next) = body.pointer("/links/next").and_then(Value::as_str) {
        if next.starts_with("http") || next.starts_with('/') {
            return resolve_link(previous, None, next).map(|url| previous.with_url(url));
        }
        return Some(with_param(
            previous,
            "cursor",
            Value::String(next.to_owned()),
        ));
    }
    if let Some(token) = body.get("nextPageToken").and_then(Value::as_str) {
        return Some(with_param(
            previous,
            "nextPageToken",
            Value::String(token.to_owned()),
        ));
    }
    if body.get("isLastPage").and_then(Value::as_bool) == Some(false)
        && let Some(start) = body.get("nextPageStart").and_then(Value::as_u64)
    {
        return Some(with_param(previous, "start", start.into()));
    }
    let start_at = body.get("startAt").and_then(Value::as_u64)?;
    if body.get("isLast").and_then(Value::as_bool) == Some(true) {
        return None;
    }
    let count = [
        "values", "issues", "results", "comments", "worklogs", "data",
    ]
    .iter()
    .find_map(|key| body.get(*key).and_then(Value::as_array))
    .map(Vec::len)?;
    if count == 0 {
        return None;
    }
    let next_start = start_at + count as u64;
    if let Some(total) = body.get("total").and_then(Value::as_u64)
        && next_start >= total
    {
        return None;
    }
    if body.get("total").is_none() && body.get("isLast").is_none() {
        // Without total or isLast a short page is the last page.
        let max = body.get("maxResults").and_then(Value::as_u64)?;
        if (count as u64) < max {
            return None;
        }
    }
    Some(with_param(previous, "startAt", next_start.into()))
}

fn link_header_next(previous: &PreparedRequest, response: &Response) -> Option<Url> {
    let header = response.headers.get(reqwest::header::LINK)?.to_str().ok()?;
    header.split(',').find_map(|part| {
        let (target, params) = part.split_once(';')?;
        if !(params.contains("rel=\"next\"") || params.contains("rel=next")) {
            return None;
        }
        let target = target.trim().trim_start_matches('<').trim_end_matches('>');
        resolve_link(previous, None, target)
    })
}

/// Resolves a next link that may be absolute, relative to `_links.base`, or
/// relative to the site origin. Confluence v1 links are relative to the base
/// (which includes `/wiki`), Confluence v2 links already include `/wiki`.
fn resolve_link(previous: &PreparedRequest, base: Option<&str>, next: &str) -> Option<Url> {
    if next.starts_with("http://") || next.starts_with("https://") {
        return Url::parse(next).ok();
    }
    if let Some(base) = base.and_then(|base| Url::parse(base).ok()) {
        let base_path = base.path().trim_end_matches('/');
        if !base_path.is_empty() && !next.starts_with(&format!("{base_path}/")) {
            return Url::parse(&format!("{}{next}", base.as_str().trim_end_matches('/'))).ok();
        }
        return base.join(next).ok();
    }
    previous.url.join(next).ok()
}

/// Sets a pagination parameter: in the JSON body for POST searches that
/// carry their parameters there, otherwise in the query string.
fn with_param(previous: &PreparedRequest, name: &str, value: Value) -> PreparedRequest {
    let mut next = previous.clone();
    if let PreparedBody::Json(Value::Object(map)) = &mut next.body {
        map.insert(name.to_owned(), value);
        return next;
    }
    let text = match &value {
        Value::String(text) => text.clone(),
        other => other.to_string(),
    };
    let kept: Vec<(String, String)> = next
        .url
        .query_pairs()
        .filter(|(key, _)| key != name)
        .map(|(k, v)| (k.into_owned(), v.into_owned()))
        .collect();
    next.url
        .query_pairs_mut()
        .clear()
        .extend_pairs(kept)
        .append_pair(name, &text);
    next
}
