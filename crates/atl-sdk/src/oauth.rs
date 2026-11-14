//! OAuth 2.0 authorization-code sign-in for Atlassian Cloud (Jira and
//! Confluence, "3LO") and Bitbucket Cloud: authorization URLs, the code
//! exchange, token refresh, and the sites a token can reach.
//!
//! The user registers their own OAuth app; both providers require the
//! client secret for the token requests.

use crate::catalog::HttpMethod;
use crate::client::{PreparedBody, PreparedRequest, http_client, send};
use crate::config::{Auth, Family};
use crate::error::{Error, Result};
use serde_json::{Value, json};
use url::Url;

/// Who issues the tokens.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Provider {
    /// auth.atlassian.com, for Jira and Confluence Cloud.
    Atlassian,
    /// bitbucket.org, for Bitbucket Cloud.
    Bitbucket,
}

/// `ATL_OAUTH_AUTH_SERVER`, else `https://auth.atlassian.com`.
fn atlassian_auth_server() -> String {
    std::env::var("ATL_OAUTH_AUTH_SERVER")
        .ok()
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| "https://auth.atlassian.com".into())
        .trim_end_matches('/')
        .to_owned()
}

/// `ATL_OAUTH_API_SERVER`, else `https://api.atlassian.com`.
pub(crate) fn api_server() -> String {
    std::env::var("ATL_OAUTH_API_SERVER")
        .ok()
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| "https://api.atlassian.com".into())
        .trim_end_matches('/')
        .to_owned()
}

impl Provider {
    pub fn for_family(family: Family) -> Option<Provider> {
        match family {
            Family::Jira | Family::Confluence => Some(Self::Atlassian),
            Family::Bitbucket => Some(Self::Bitbucket),
            _ => None,
        }
    }

    pub fn token_url(self) -> String {
        match self {
            Self::Atlassian => format!("{}/oauth/token", atlassian_auth_server()),
            Self::Bitbucket => "https://bitbucket.org/site/oauth2/access_token".into(),
        }
    }

    /// The page that asks the user to approve the app. `scopes` is ignored
    /// by Bitbucket, whose consumer settings hold the scopes.
    pub fn authorize_url(
        self,
        client_id: &str,
        redirect_uri: &str,
        scopes: &str,
        state: &str,
    ) -> String {
        let (base, mut pairs) = match self {
            Self::Atlassian => (
                format!("{}/authorize", atlassian_auth_server()),
                vec![
                    ("audience", "api.atlassian.com"),
                    ("scope", scopes),
                    ("prompt", "consent"),
                ],
            ),
            Self::Bitbucket => ("https://bitbucket.org/site/oauth2/authorize".into(), vec![]),
        };
        pairs.extend([
            ("client_id", client_id),
            ("redirect_uri", redirect_uri),
            ("state", state),
            ("response_type", "code"),
        ]);
        let mut url = Url::parse(&base).expect("the authorization server URL is valid");
        url.query_pairs_mut().extend_pairs(pairs);
        url.into()
    }
}

/// The scopes `atl` asks for by default (see docs/auth.md for the app setup).
pub fn default_scopes(families: &[Family]) -> String {
    let mut scopes = Vec::new();
    if families.contains(&Family::Jira) {
        scopes.extend([
            "read:jira-work",
            "write:jira-work",
            "read:jira-user",
            "read:servicedesk-request",
            "write:servicedesk-request",
            "read:board-scope:jira-software",
            "read:sprint:jira-software",
            "write:sprint:jira-software",
            "read:issue:jira-software",
            "write:issue:jira-software",
        ]);
    }
    if families.contains(&Family::Confluence) {
        scopes.extend([
            "read:confluence-content.all",
            "write:confluence-content",
            "read:confluence-space.summary",
            "search:confluence",
            "read:confluence-user",
            "read:page:confluence",
            "write:page:confluence",
            "read:space:confluence",
            "read:comment:confluence",
            "write:comment:confluence",
            "read:label:confluence",
            "write:label:confluence",
            "read:attachment:confluence",
            "write:attachment:confluence",
        ]);
    }
    scopes.push("offline_access");
    scopes.join(" ")
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tokens {
    pub access_token: String,
    pub refresh_token: Option<String>,
    /// Seconds until the access token expires.
    pub expires_in: Option<u64>,
}

/// Posts a token request: JSON for Atlassian, a form with HTTP Basic
/// client authentication for Bitbucket.
async fn token_request(
    token_url: &str,
    client_id: &str,
    client_secret: &str,
    grant: &[(&str, &str)],
) -> Result<Tokens> {
    let url = Url::parse(token_url).map_err(|err| {
        Error::Invalid(format!("the OAuth token URL `{token_url}` is invalid ({err}); log in again with `atl auth login --method oauth`"))
    })?;
    let (body, auth) = if token_url.ends_with("/site/oauth2/access_token") {
        (
            PreparedBody::Form(
                grant
                    .iter()
                    .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
                    .collect(),
            ),
            Some(Auth::Basic {
                username: client_id.to_owned(),
                secret: client_secret.to_owned(),
            }),
        )
    } else {
        let mut body = json!({ "client_id": client_id, "client_secret": client_secret });
        for (key, value) in grant {
            body[*key] = Value::String((*value).to_owned());
        }
        (PreparedBody::Json(body), None)
    };
    let headers = match body {
        PreparedBody::Json(_) => vec![("Content-Type".to_owned(), "application/json".to_owned())],
        _ => Vec::new(),
    };
    let request = PreparedRequest {
        headers,
        body,
        ..PreparedRequest::new(HttpMethod::Post, url, auth)
    };
    let response = send(&http_client(), &request, 1).await?.value();
    let access_token = response
        .get("access_token")
        .and_then(Value::as_str)
        .filter(|t| !t.is_empty())
        .ok_or_else(|| {
            Error::Invalid(format!(
                "the OAuth server returned no access token: {response}"
            ))
        })?;
    Ok(Tokens {
        access_token: access_token.to_owned(),
        refresh_token: response
            .get("refresh_token")
            .and_then(Value::as_str)
            .map(str::to_owned),
        expires_in: response.get("expires_in").and_then(Value::as_u64),
    })
}

/// Exchanges the code from the callback for tokens.
pub async fn exchange_code(
    token_url: &str,
    client_id: &str,
    client_secret: &str,
    code: &str,
    redirect_uri: &str,
) -> Result<Tokens> {
    token_request(
        token_url,
        client_id,
        client_secret,
        &[
            ("grant_type", "authorization_code"),
            ("code", code),
            ("redirect_uri", redirect_uri),
        ],
    )
    .await
}

/// The `Auth::OAuth` value for fresh tokens.
pub fn auth_from(
    tokens: Tokens,
    client_id: &str,
    client_secret: &str,
    token_url: &str,
    previous_refresh: &str,
) -> Auth {
    Auth::OAuth {
        access_token: tokens.access_token,
        refresh_token: tokens
            .refresh_token
            .unwrap_or_else(|| previous_refresh.to_owned()),
        expires_at: match (crate::time::now(), tokens.expires_in) {
            (Some(now), Some(seconds)) => now + seconds,
            _ => 0,
        },
        client_id: client_id.to_owned(),
        client_secret: client_secret.to_owned(),
        token_url: token_url.to_owned(),
    }
}

/// New tokens for an `Auth::OAuth` value, keeping the old refresh token
/// when the server sends none.
pub async fn refresh(auth: &Auth) -> Result<Auth> {
    let Auth::OAuth {
        refresh_token,
        client_id,
        client_secret,
        token_url,
        ..
    } = auth
    else {
        return Err(Error::Invalid("only OAuth logins can be refreshed".into()));
    };
    if refresh_token.is_empty() {
        return Err(Error::Invalid(
            "the OAuth access token expired and there is no refresh token (the app needs the offline_access scope); log in again with `atl auth login --method oauth`".into(),
        ));
    }
    let tokens = token_request(
        token_url,
        client_id,
        client_secret,
        &[
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token),
        ],
    )
    .await?;
    Ok(auth_from(
        tokens,
        client_id,
        client_secret,
        token_url,
        refresh_token,
    ))
}

/// A site an Atlassian token can reach.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct Site {
    /// The cloud ID used in api.atlassian.com/ex/... URLs.
    pub id: String,
    pub url: String,
    pub name: String,
}

pub async fn accessible_resources(access_token: &str) -> Result<Vec<Site>> {
    let url = format!("{}/oauth/token/accessible-resources", api_server());
    let parsed = Url::parse(&url).map_err(|err| {
        Error::Invalid(format!(
            "`{url}` is invalid ({err}); check ATL_OAUTH_API_SERVER"
        ))
    })?;
    let token = Auth::Bearer {
        token: access_token.to_owned(),
    };
    let request = PreparedRequest::new(HttpMethod::Get, parsed, Some(token));
    let response = send(&http_client(), &request, 1).await?.value();
    let text = |v: &Value, key: &str| {
        v.get(key)
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned()
    };
    Ok(response
        .as_array()
        .into_iter()
        .flatten()
        .map(|site| Site {
            id: text(site, "id"),
            url: text(site, "url").trim_end_matches('/').to_owned(),
            name: text(site, "name"),
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn authorization_urls_carry_the_client_state_and_scopes() {
        let url = Provider::Atlassian.authorize_url(
            "cid",
            "http://localhost:8976/callback",
            "read:jira-work offline_access",
            "s1",
        );
        let parsed = Url::parse(&url).expect("url");
        let pairs: std::collections::HashMap<_, _> = parsed.query_pairs().into_owned().collect();
        assert_eq!(parsed.path(), "/authorize");
        assert_eq!(pairs["audience"], "api.atlassian.com");
        assert_eq!(pairs["scope"], "read:jira-work offline_access");
        assert_eq!(pairs["redirect_uri"], "http://localhost:8976/callback");
        assert_eq!(
            (pairs["state"].as_str(), pairs["response_type"].as_str()),
            ("s1", "code")
        );
        let url =
            Provider::Bitbucket.authorize_url("key", "http://localhost:8976/callback", "", "s2");
        assert!(
            url.starts_with("https://bitbucket.org/site/oauth2/authorize?client_id=key"),
            "{url}"
        );
        assert!(
            default_scopes(&[Family::Jira]).ends_with("write:issue:jira-software offline_access")
        );
    }
}
