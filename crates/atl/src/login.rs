//! `atl auth login`: an interactive terminal form, plus the shared logic that
//! turns a login request into verified, saved connections.

use anyhow::Result;
use atl_sdk::{Auth, Config, Connection, Deployment, Error, Family};
use ratatui::Frame;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::layout::Constraint;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Paragraph, Wrap};
use serde_json::{Value, json};
use std::mem::DropGuard;
use std::path::PathBuf;

/// What the user signs in to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum Product {
    /// One Atlassian Cloud site: Jira and Confluence with one API token.
    CloudSite,
    Jira,
    Confluence,
    Bitbucket,
    Trello,
    Admin,
    AdminScim,
}

impl Product {
    const ALL: [Product; 7] = [
        Product::CloudSite,
        Product::Jira,
        Product::Confluence,
        Product::Bitbucket,
        Product::Trello,
        Product::Admin,
        Product::AdminScim,
    ];

    fn label(self) -> &'static str {
        match self {
            Self::CloudSite => "Atlassian Cloud site (Jira + Confluence)",
            Self::Jira => "Jira (platform, Software, Service Management)",
            Self::Confluence => "Confluence",
            Self::Bitbucket => "Bitbucket",
            Self::Trello => "Trello",
            Self::Admin => "Atlassian Admin (organization API key)",
            Self::AdminScim => "Atlassian Admin user provisioning (SCIM key)",
        }
    }

    pub fn families(self) -> &'static [Family] {
        match self {
            Self::CloudSite => &[Family::Jira, Family::Confluence],
            Self::Jira => &[Family::Jira],
            Self::Confluence => &[Family::Confluence],
            Self::Bitbucket => &[Family::Bitbucket],
            Self::Trello => &[Family::Trello],
            Self::Admin => &[Family::Admin],
            Self::AdminScim => &[Family::AdminScim],
        }
    }

    fn cloud_only(self) -> bool {
        matches!(
            self,
            Self::CloudSite | Self::Trello | Self::Admin | Self::AdminScim
        )
    }

    fn default_url(self, deployment: Deployment) -> &'static str {
        match deployment {
            Deployment::Cloud => self.families()[0].default_cloud_url().unwrap_or(""),
            Deployment::DataCenter => "",
        }
    }

    fn url_hint(self, deployment: Deployment) -> &'static str {
        match (self, deployment) {
            (Self::CloudSite | Self::Jira | Self::Confluence, Deployment::Cloud) => {
                "https://your-site.atlassian.net"
            }
            (Self::Confluence, Deployment::DataCenter) => {
                "https://confluence.example.com (include any context path)"
            }
            (Self::Bitbucket, Deployment::DataCenter) => "https://bitbucket.example.com",
            (_, Deployment::DataCenter) => "https://jira.example.com (include any context path)",
            _ => "",
        }
    }
}

/// How the credential is sent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum AuthMethod {
    /// Atlassian account email + API token (HTTP Basic).
    ApiToken,
    /// Scoped API token, sent through api.atlassian.com with the site's cloud ID.
    ScopedToken,
    /// Data Center personal access token (HTTP Bearer).
    Pat,
    /// Data Center username + password (HTTP Basic).
    Password,
    /// Any bearer token: Admin API key, Bitbucket access token, OAuth access token.
    Bearer,
    /// Trello API key + token.
    TrelloKey,
    /// OAuth 2.0 sign-in in the browser with your own app (--user CLIENT_ID,
    /// client secret on stdin). Cloud sites and Bitbucket Cloud.
    #[value(name = "oauth")]
    OAuth,
    /// A browser session's Cookie header (Data Center behind SSO), on --token-stdin.
    Cookie,
    /// The login and password in ~/.netrc (or $NETRC) for the server's host, read at each run.
    Netrc,
    /// No credential: anonymous access to a public site.
    None,
}

impl AuthMethod {
    fn available(product: Product, deployment: Deployment) -> &'static [AuthMethod] {
        use AuthMethod::*;
        match (product, deployment) {
            (Product::CloudSite, Deployment::Cloud) => &[ApiToken, ScopedToken],
            (Product::Jira | Product::Confluence, Deployment::Cloud) => {
                &[ApiToken, ScopedToken, None]
            }
            (Product::Bitbucket, Deployment::Cloud) => &[ApiToken, Bearer],
            (Product::Jira | Product::Confluence | Product::Bitbucket, Deployment::DataCenter) => {
                &[Pat, Password, Cookie, Netrc, None]
            }
            (Product::Trello, _) => &[TrelloKey],
            (Product::Admin | Product::AdminScim, _) => &[Bearer],
            (Product::CloudSite, Deployment::DataCenter) => &[Pat, Password],
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::ApiToken => "Email + API token",
            Self::ScopedToken => "Email + scoped API token (via api.atlassian.com)",
            Self::Pat => "Personal access token",
            Self::Password => "Username + password",
            Self::Bearer => "Bearer token / API key",
            Self::TrelloKey => "Trello API key + token",
            Self::OAuth => "OAuth 2.0 in the browser (your own app)",
            Self::Cookie => "Session cookie (SSO gateway)",
            Self::Netrc => "~/.netrc entry",
            Self::None => "None (anonymous)",
        }
    }

    fn user_label(self) -> Option<&'static str> {
        match self {
            Self::ApiToken | Self::ScopedToken => Some("Email"),
            Self::Password => Some("Username"),
            Self::TrelloKey => Some("API key"),
            Self::OAuth => Some("Client ID"),
            Self::Pat | Self::Bearer | Self::Cookie | Self::Netrc | Self::None => None,
        }
    }

    fn secret_label(self) -> &'static str {
        match self {
            Self::ApiToken | Self::ScopedToken => "API token",
            Self::Pat => "Access token",
            Self::Password => "Password",
            Self::Bearer => "Token / API key",
            Self::TrelloKey => "Token",
            Self::OAuth => "Client secret",
            Self::Cookie => "Cookie header",
            Self::Netrc | Self::None => "",
        }
    }

    /// False for methods that store no secret.
    fn needs_secret(self) -> bool {
        !matches!(self, Self::Netrc | Self::None)
    }

    fn hint(self) -> &'static str {
        match self {
            Self::ApiToken | Self::ScopedToken => {
                "Create a token at https://id.atlassian.com/manage-profile/security/api-tokens"
            }
            Self::Pat => {
                "Create one in the product: profile picture > Profile > Personal Access Tokens"
            }
            Self::Password => "Prefer a personal access token where your server supports it",
            Self::Bearer => "Admin keys: admin.atlassian.com > Settings > API keys",
            Self::TrelloKey => "Get the key and token at https://trello.com/power-ups/admin",
            Self::OAuth => "Register an OAuth app with the callback http://localhost:8976/callback",
            Self::Cookie => {
                "Copy the Cookie request header from your browser's developer tools (Network tab)"
            }
            Self::Netrc => {
                "Reads ~/.netrc (or $NETRC) at each run: machine HOST login USER password TOKEN"
            }
            Self::None => "Anonymous: only what the site shows without signing in",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoginRequest {
    pub product: Product,
    pub deployment: Deployment,
    pub url: String,
    pub method: AuthMethod,
    pub user: String,
    pub secret: String,
    /// Overrides the profile chosen by `--profile`.
    pub profile: Option<String>,
    /// Store the connection as read-only: every write is refused.
    pub read_only: bool,
    /// OAuth: the local callback port (the app's callback URL must match).
    pub oauth_port: u16,
    /// OAuth: scopes to request instead of [`atl_sdk::oauth::default_scopes`].
    pub oauth_scopes: Option<String>,
    /// Mutual TLS client certificate, its key, and extra CA certificates.
    pub client_cert: Option<std::path::PathBuf>,
    pub client_key: Option<std::path::PathBuf>,
    pub ca_cert: Option<std::path::PathBuf>,
}

/// The default OAuth callback port: http://localhost:8976/callback.
pub const OAUTH_PORT: u16 = 8976;

impl LoginRequest {
    pub fn from_flags(
        product: Product,
        url: Option<String>,
        method: Option<AuthMethod>,
        user: Option<String>,
        secret: String,
        deployment: Option<&str>,
        read_only: bool,
    ) -> Result<Self, Error> {
        let url = url.unwrap_or_else(|| product.default_url(Deployment::Cloud).to_owned());
        let oauth = method == Some(AuthMethod::OAuth);
        let deployment = match deployment {
            Some(text) => text.parse()?,
            None if product.cloud_only() || oauth => Deployment::Cloud,
            None => Deployment::infer(&normalize_url(&url)),
        };
        let methods = AuthMethod::available(product, deployment);
        let method = method.unwrap_or(methods[0]);
        let oauth_ok = deployment == Deployment::Cloud
            && matches!(
                product,
                Product::CloudSite | Product::Jira | Product::Confluence | Product::Bitbucket
            );
        if oauth && !oauth_ok {
            return Err(Error::Invalid(format!(
                "--method oauth works for cloud-site, jira, confluence and bitbucket on Atlassian Cloud, not for {} on {deployment}",
                product.label()
            )));
        }
        if !oauth && !methods.contains(&method) {
            return Err(Error::Invalid(format!(
                "--method {method:?} does not apply to {} on {deployment}; use one of: {}",
                product.label(),
                methods
                    .iter()
                    .map(|m| format!("{m:?}"))
                    .collect::<Vec<_>>()
                    .join(", ")
            )));
        }
        let request = Self {
            product,
            deployment,
            url,
            method,
            user: user.unwrap_or_default(),
            secret,
            profile: None,
            read_only,
            oauth_port: OAUTH_PORT,
            oauth_scopes: None,
            client_cert: None,
            client_key: None,
            ca_cert: None,
        };
        request.validate().map_err(Error::Invalid)?;
        Ok(request)
    }

    fn validate(&self) -> Result<(), String> {
        let url = normalize_url(&self.url);
        // OAuth finds the site from the token; a URL only picks among sites.
        let oauth_without_url = url.is_empty() && self.method == AuthMethod::OAuth;
        if url.is_empty() && !oauth_without_url {
            return Err(format!(
                "enter the URL, for example {}",
                self.product.url_hint(self.deployment)
            ));
        }
        if !oauth_without_url
            && url::Url::parse(&url)
                .map(|u| u.host_str().is_none())
                .unwrap_or(true)
        {
            return Err(format!(
                "`{}` is not a URL; enter one such as {}",
                self.url,
                self.product.url_hint(self.deployment)
            ));
        }
        if let Some(label) = self.method.user_label()
            && self.user.trim().is_empty()
        {
            return Err(format!(
                "enter the {} (pass --user in scripts)",
                label.to_ascii_lowercase()
            ));
        }
        if self.secret.is_empty() && self.method.needs_secret() {
            return Err(format!(
                "enter the {} (pipe it to --token-stdin in scripts)",
                self.method.secret_label().to_ascii_lowercase()
            ));
        }
        Ok(())
    }
}

/// Adds `https://` when missing and drops trailing slashes.
fn normalize_url(text: &str) -> String {
    let text = text.trim().trim_end_matches('/');
    if text.is_empty() || text.contains("://") {
        text.to_owned()
    } else {
        format!("https://{text}")
    }
}

#[derive(Debug)]
pub struct LoginOutcome {
    profile: Option<String>,
    connections: Vec<(Family, Connection)>,
    /// Families of a multi-product login that failed verification.
    skipped: Vec<(Family, String)>,
}

impl LoginOutcome {
    pub fn profile_or(&self, fallback: &str) -> String {
        self.profile.clone().unwrap_or_else(|| fallback.to_owned())
    }

    pub fn summary(&self) -> Value {
        let mut out = serde_json::Map::new();
        for (family, connection) in &self.connections {
            out.insert(
                family.as_str().into(),
                json!({
                    "deployment": connection.deployment.as_str(),
                    "base_url": connection.base_url,
                    "account": connection.account,
                }),
            );
        }
        for (family, error) in &self.skipped {
            out.insert(family.as_str().into(), json!({ "skipped": error }));
        }
        Value::Object(out)
    }
}

/// Builds the connections for a request and, when `verify` is true, checks
/// each one against the server.
pub async fn perform(request: &LoginRequest, verify: bool) -> Result<LoginOutcome> {
    let candidates = match request.method {
        AuthMethod::OAuth => oauth_connections(request).await?,
        _ => token_connections(request).await?,
    };
    let mut outcome = LoginOutcome {
        profile: request.profile.clone(),
        connections: Vec::new(),
        skipped: Vec::new(),
    };
    let mut first_error = None;
    for (family, mut connection) in candidates {
        connection.read_only = request.read_only;
        connection.client_cert = request.client_cert.clone();
        connection.client_key = request.client_key.clone();
        connection.ca_cert = request.ca_cert.clone();
        if verify {
            match atl_sdk::verify(family, &connection).await {
                Ok(account) => connection.account = Some(account),
                Err(err) => {
                    outcome.skipped.push((family, err.to_string()));
                    first_error.get_or_insert(err);
                    continue;
                }
            }
        }
        outcome.connections.push((family, connection));
    }
    match (outcome.connections.is_empty(), first_error) {
        (true, Some(err)) => Err(err.into()),
        _ => Ok(outcome),
    }
}

/// Connections for a token, password or key login.
async fn token_connections(request: &LoginRequest) -> Result<Vec<(Family, Connection)>> {
    let mut base = normalize_url(&request.url);
    if request.deployment == Deployment::Cloud {
        // People paste Confluence links; Cloud paths already start with /wiki.
        base = base.trim_end_matches("/wiki").to_owned();
    }
    let auth = match request.method {
        AuthMethod::ApiToken | AuthMethod::ScopedToken | AuthMethod::Password => Auth::Basic {
            username: request.user.trim().to_owned(),
            secret: request.secret.clone(),
        },
        AuthMethod::Pat | AuthMethod::Bearer => Auth::Bearer {
            token: request.secret.clone(),
        },
        AuthMethod::TrelloKey => Auth::Trello {
            key: request.user.trim().to_owned(),
            token: request.secret.clone(),
        },
        AuthMethod::Cookie => Auth::Cookie {
            cookie: request.secret.trim().to_owned(),
        },
        AuthMethod::Netrc => Auth::Netrc {},
        AuthMethod::None => Auth::None {},
        AuthMethod::OAuth => unreachable!("perform runs the OAuth sign-in"),
    };
    let cloud_id = match request.method {
        AuthMethod::ScopedToken => Some(atl_sdk::cloud_id(&base).await?),
        _ => None,
    };
    Ok(request
        .product
        .families()
        .iter()
        .map(|&family| {
            let base_url = match &cloud_id {
                Some(id) => atl_sdk::gateway_url(family, id),
                None => base.clone(),
            };
            let connection = Connection {
                site_url: cloud_id.as_ref().map(|_| base.clone()),
                ..Connection::new(request.deployment, base_url, auth.clone())
            };
            (family, connection)
        })
        .collect())
}

/// Signs in with OAuth 2.0 in the browser: listens on 127.0.0.1 for the
/// callback, exchanges the code, and builds the connections.
async fn oauth_connections(request: &LoginRequest) -> Result<Vec<(Family, Connection)>> {
    use atl_sdk::oauth::{self, Provider};
    let families = request.product.families();
    let provider = Provider::for_family(families[0]).ok_or_else(|| {
        Error::Invalid("--method oauth works for cloud-site, jira, confluence and bitbucket".into())
    })?;
    let port = request.oauth_port;
    let redirect = format!("http://localhost:{port}/callback");
    let listener = tokio::net::TcpListener::bind(("127.0.0.1", port))
        .await
        .map_err(|err| {
            Error::Invalid(format!(
                "cannot listen on 127.0.0.1:{port} for the sign-in callback ({err}); free the port or pass --callback-port N (the app's callback URL must use the same port)"
            ))
        })?;
    let state = random_state();
    let scopes = request
        .oauth_scopes
        .clone()
        .unwrap_or_else(|| oauth::default_scopes(families));
    let client_id = request.user.trim();
    let url = provider.authorize_url(client_id, &redirect, &scopes, &state);
    eprintln!("Open this URL to sign in (waiting up to 5 minutes for {redirect}):\n{url}");
    if !atl_sdk::config::env_flag("ATL_NO_BROWSER") {
        let _ = open::that_detached(&url);
    }
    let code = tokio::time::timeout(
        std::time::Duration::from_secs(300),
        wait_for_code(&listener, &state),
    )
    .await
    .map_err(|_| {
        Error::Invalid(
            "no sign-in arrived within 5 minutes; run `atl auth login --method oauth` again".into(),
        )
    })??;
    let token_url = provider.token_url();
    let tokens =
        oauth::exchange_code(&token_url, client_id, &request.secret, &code, &redirect).await?;
    let access_token = tokens.access_token.clone();
    let auth = oauth::auth_from(tokens, client_id, &request.secret, &token_url, "");
    if provider == Provider::Bitbucket {
        return Ok(vec![(
            Family::Bitbucket,
            Connection::new(Deployment::Cloud, "https://api.bitbucket.org", auth),
        )]);
    }
    let sites = oauth::accessible_resources(&access_token).await?;
    let wanted = normalize_url(&request.url);
    let host = |url: &str| {
        url::Url::parse(url)
            .ok()
            .and_then(|u| u.host_str().map(str::to_ascii_lowercase))
    };
    let listed = || {
        sites
            .iter()
            .map(|s| s.url.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    };
    let site = match (wanted.is_empty(), sites.as_slice()) {
        (_, []) => {
            return Err(Error::Invalid(
                "the sign-in worked, but the token reaches no site; check that the app has Jira or Confluence scopes and that you chose a site on the consent page".into(),
            )
            .into());
        }
        (true, [only]) => only,
        (true, _) => {
            return Err(Error::Invalid(format!(
                "the token reaches several sites; run the login again with --url, one of: {}",
                listed()
            ))
            .into());
        }
        (false, _) => sites
            .iter()
            .find(|s| host(&s.url) == host(&wanted))
            .ok_or_else(|| {
                Error::Invalid(format!(
                    "the token does not reach {wanted}; it reaches: {}",
                    listed()
                ))
            })?,
    };
    Ok(families
        .iter()
        .map(|&family| {
            let connection = Connection {
                site_url: Some(site.url.clone()),
                ..Connection::new(
                    Deployment::Cloud,
                    atl_sdk::gateway_url(family, &site.id),
                    auth.clone(),
                )
            };
            (family, connection)
        })
        .collect())
}

/// 128 unpredictable bits for the OAuth `state` parameter.
fn random_state() -> String {
    use std::hash::{BuildHasher, RandomState};
    let now = std::time::SystemTime::now();
    let a = RandomState::new().hash_one(now);
    let b = RandomState::new().hash_one((now, std::process::id()));
    format!("{a:016x}{b:016x}")
}

/// Serves the callback page until the sign-in request arrives; other paths
/// (the browser's favicon request) get 404.
async fn wait_for_code(listener: &tokio::net::TcpListener, state: &str) -> Result<String> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    loop {
        let (mut socket, _) = listener
            .accept()
            .await
            .map_err(|err| Error::Invalid(format!("the sign-in callback failed ({err})")))?;
        // The request line can arrive in several TCP segments.
        let mut buffer = vec![0u8; 16 * 1024];
        let mut read = 0;
        while read < buffer.len() && !buffer[..read].windows(2).any(|w| w == b"\r\n") {
            match socket.read(&mut buffer[read..]).await {
                Ok(0) | Err(_) => break,
                Ok(n) => read += n,
            }
        }
        let head = String::from_utf8_lossy(&buffer[..read]);
        let line = head.lines().next().unwrap_or_default();
        let (status, page, result) = match parse_callback(line, state) {
            Ok(None) => ("404 Not Found", "Not found.".to_owned(), None),
            Ok(Some(code)) => (
                "200 OK",
                "Signed in to atl. You can close this tab and return to the terminal.".to_owned(),
                Some(Ok(code)),
            ),
            Err(err) => (
                "400 Bad Request",
                format!("atl could not sign in: {err}"),
                Some(Err(err)),
            ),
        };
        let page = page.replace('&', "&amp;").replace('<', "&lt;");
        let response = format!(
            "HTTP/1.1 {status}\r\nContent-Type: text/html; charset=utf-8\r\nConnection: close\r\n\r\n<!doctype html><title>atl</title><p style=\"font-family:sans-serif\">{page}</p>"
        );
        let _ = socket.write_all(response.as_bytes()).await;
        let _ = socket.shutdown().await;
        if let Some(result) = result {
            return Ok(result?);
        }
    }
}

/// The authorization code in a callback request line such as
/// `GET /callback?code=X&state=Y HTTP/1.1`; `None` for other paths.
fn parse_callback(line: &str, state: &str) -> Result<Option<String>, Error> {
    let target = line.split_whitespace().nth(1).unwrap_or_default();
    let Ok(url) = url::Url::parse(&format!("http://localhost{target}")) else {
        return Ok(None);
    };
    if url.path() != "/callback" {
        return Ok(None);
    }
    let query: std::collections::HashMap<String, String> = url.query_pairs().into_owned().collect();
    if let Some(error) = query.get("error") {
        let detail = query
            .get("error_description")
            .map(String::as_str)
            .unwrap_or_default();
        return Err(Error::Invalid(format!(
            "the sign-in was refused ({error}{}{detail}); approve the app on the consent page, or check its scopes and callback URL",
            if detail.is_empty() { "" } else { ": " }
        )));
    }
    if query.get("state").map(String::as_str) != Some(state) {
        return Err(Error::Invalid(
            "the sign-in callback has the wrong state, so it did not come from this login; run the login again".into(),
        ));
    }
    query.get("code").cloned().map(Some).ok_or_else(|| {
        Error::Invalid("the sign-in callback has no code; run the login again".into())
    })
}

/// Writes the outcome into `profile` and saves the config file. The first
/// profile ever saved becomes the default.
pub fn save(config: &mut Config, profile: &str, outcome: &LoginOutcome) -> Result<PathBuf> {
    let stored = config.profiles.entry(profile.to_owned()).or_default();
    for (family, connection) in &outcome.connections {
        stored.insert(*family, connection.clone());
    }
    if config.default_profile.is_none() {
        config.default_profile = Some(profile.to_owned());
    }
    Ok(config.save()?)
}

// ---------------------------------------------------------------------------
// The form.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Field {
    Product,
    Deployment,
    Url,
    Method,
    User,
    Secret,
    Profile,
    Access,
    Submit,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Status {
    Idle,
    Working(String),
    Error(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Action {
    None,
    Submit,
    Cancel,
}

/// Form state. Pure: key events in, state and actions out, so it is tested
/// without a terminal.
#[derive(Debug, Clone)]
struct Form {
    product: usize,
    deployment: Deployment,
    url: String,
    method: usize,
    user: String,
    secret: String,
    profile: String,
    focus: Field,
    show_secret: bool,
    read_only: bool,
    status: Status,
}

impl Form {
    fn new(profile: &str, config: &Config) -> Self {
        let mut form = Self {
            product: 0,
            deployment: Deployment::Cloud,
            url: String::new(),
            method: 0,
            user: String::new(),
            secret: String::new(),
            profile: profile.to_owned(),
            focus: Field::Url,
            show_secret: false,
            read_only: false,
            status: Status::Idle,
        };
        // Start from the profile's Jira login when there is one, so a token
        // rotation only needs the new token.
        if let Some(existing) = config
            .profiles
            .get(profile)
            .and_then(|p| p.get(&Family::Jira))
        {
            form.product = if existing.deployment == Deployment::Cloud {
                0
            } else {
                1
            };
            form.deployment = existing.deployment;
            // A scoped-token login stores the api.atlassian.com gateway as
            // the base URL and the site in `site_url`.
            form.url = existing
                .site_url
                .clone()
                .unwrap_or_else(|| existing.base_url.clone());
            if existing.site_url.is_some()
                && let Some(index) = form
                    .methods()
                    .iter()
                    .position(|m| *m == AuthMethod::ScopedToken)
            {
                form.method = index;
            }
            if let Auth::Basic { username, .. } = &existing.auth {
                form.user = username.clone();
            }
            form.focus = Field::Secret;
        }
        form
    }

    fn product(&self) -> Product {
        Product::ALL[self.product]
    }

    fn methods(&self) -> &'static [AuthMethod] {
        AuthMethod::available(self.product(), self.deployment)
    }

    fn method(&self) -> AuthMethod {
        let methods = self.methods();
        methods[self.method.min(methods.len() - 1)]
    }

    fn fields(&self) -> Vec<Field> {
        let mut fields = vec![Field::Product];
        if !self.product().cloud_only() {
            fields.push(Field::Deployment);
        }
        fields.extend([Field::Url, Field::Method]);
        if self.method().user_label().is_some() {
            fields.push(Field::User);
        }
        if self.method().needs_secret() {
            fields.push(Field::Secret);
        }
        fields.extend([Field::Profile, Field::Access, Field::Submit]);
        fields
    }

    fn move_focus(&mut self, forward: bool) {
        if self.focus == Field::Url && !self.product().cloud_only() && !self.url.trim().is_empty() {
            let inferred = Deployment::infer(&normalize_url(&self.url));
            if inferred != self.deployment {
                self.deployment = inferred;
                self.method = 0;
            }
        }
        let fields = self.fields();
        let index = fields.iter().position(|f| *f == self.focus).unwrap_or(0);
        let next = if forward {
            (index + 1) % fields.len()
        } else {
            (index + fields.len() - 1) % fields.len()
        };
        self.focus = fields[next];
    }

    fn cycle(&mut self, forward: bool) {
        let step = |index: usize, len: usize| {
            if forward {
                (index + 1) % len
            } else {
                (index + len - 1) % len
            }
        };
        match self.focus {
            Field::Product => {
                let old_default = self.product().default_url(self.deployment);
                self.product = step(self.product, Product::ALL.len());
                if self.product().cloud_only() {
                    self.deployment = Deployment::Cloud;
                }
                self.method = 0;
                if self.url.is_empty() || self.url == old_default {
                    self.url = self.product().default_url(self.deployment).to_owned();
                }
            }
            Field::Deployment => {
                let old_default = self.product().default_url(self.deployment);
                self.deployment = match self.deployment {
                    Deployment::Cloud => Deployment::DataCenter,
                    Deployment::DataCenter => Deployment::Cloud,
                };
                self.method = 0;
                if self.url == old_default {
                    self.url = self.product().default_url(self.deployment).to_owned();
                }
            }
            Field::Access => self.read_only = !self.read_only,
            Field::Method => {
                self.method = step(
                    self.method.min(self.methods().len() - 1),
                    self.methods().len(),
                )
            }
            _ => {}
        }
    }

    fn text_mut(&mut self) -> Option<&mut String> {
        match self.focus {
            Field::Url => Some(&mut self.url),
            Field::User => Some(&mut self.user),
            Field::Secret => Some(&mut self.secret),
            Field::Profile => Some(&mut self.profile),
            _ => None,
        }
    }

    fn handle(&mut self, key: KeyEvent) -> Action {
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        match key.code {
            KeyCode::Esc => return Action::Cancel,
            KeyCode::Char('c') if ctrl => return Action::Cancel,
            KeyCode::Char('r') if ctrl => self.show_secret = !self.show_secret,
            KeyCode::Char('u') if ctrl => {
                if let Some(text) = self.text_mut() {
                    text.clear();
                }
            }
            KeyCode::Tab | KeyCode::Down => self.move_focus(true),
            KeyCode::BackTab | KeyCode::Up => self.move_focus(false),
            KeyCode::Enter if self.focus == Field::Submit => return Action::Submit,
            KeyCode::Enter => self.move_focus(true),
            KeyCode::Left => self.cycle(false),
            KeyCode::Right => self.cycle(true),
            KeyCode::Char(' ')
                if matches!(
                    self.focus,
                    Field::Product | Field::Deployment | Field::Method | Field::Access
                ) =>
            {
                self.cycle(true)
            }
            KeyCode::Backspace => {
                if let Some(text) = self.text_mut() {
                    text.pop();
                }
            }
            KeyCode::Char(c) if !ctrl => {
                if let Some(text) = self.text_mut() {
                    text.push(c);
                }
            }
            _ => {}
        }
        if matches!(self.status, Status::Error(_)) {
            self.status = Status::Idle;
        }
        Action::None
    }

    fn request(&self) -> Result<LoginRequest, String> {
        let request = LoginRequest {
            product: self.product(),
            deployment: self.deployment,
            url: self.url.clone(),
            method: self.method(),
            user: self.user.clone(),
            secret: self.secret.clone(),
            profile: Some(self.profile.trim().to_owned()).filter(|p| !p.is_empty()),
            read_only: self.read_only,
            oauth_port: OAUTH_PORT,
            oauth_scopes: None,
            client_cert: None,
            client_key: None,
            ca_cert: None,
        };
        request.validate()?;
        Ok(request)
    }

    fn render(&self, frame: &mut Frame) {
        let area = frame
            .area()
            .centered(Constraint::Length(84), Constraint::Length(22));
        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .title(Line::from(" atl auth login ").style(Style::new().add_modifier(Modifier::BOLD)))
            .title_bottom(Line::from(" Tab/↑↓ move · ←/→ change · Enter next/confirm · Ctrl-R show token · Esc cancel ").centered());
        let inner = block.inner(area);
        frame.render_widget(block, area);

        let method = self.method();
        let mut lines: Vec<Line> = Vec::new();
        for field in self.fields() {
            let focused = field == self.focus;
            let (label, value, choice) = match field {
                Field::Product => ("Product", self.product().label().to_owned(), true),
                Field::Deployment => (
                    "Deployment",
                    match self.deployment {
                        Deployment::Cloud => "Cloud".to_owned(),
                        Deployment::DataCenter => "Data Center / Server (on-prem)".to_owned(),
                    },
                    true,
                ),
                Field::Url => ("URL", self.url.clone(), false),
                Field::Method => ("Auth method", method.label().to_owned(), true),
                Field::User => ("User", self.user.clone(), false),
                Field::Secret => (
                    method.secret_label(),
                    if self.show_secret {
                        self.secret.clone()
                    } else {
                        "•".repeat(self.secret.chars().count())
                    },
                    false,
                ),
                Field::Profile => ("Profile", self.profile.clone(), false),
                Field::Access => (
                    "Access",
                    if self.read_only {
                        "Read-only (writes refused; good for agents)".to_owned()
                    } else {
                        "Read and write".to_owned()
                    },
                    true,
                ),
                Field::Submit => {
                    lines.push(Line::default());
                    let style = if focused {
                        Style::new()
                            .fg(Color::Black)
                            .bg(Color::Green)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::new().fg(Color::Green)
                    };
                    lines.push(
                        Line::from(Span::styled("  [ Verify and save ]  ", style)).centered(),
                    );
                    continue;
                }
            };
            let label = match field {
                Field::User => method.user_label().unwrap_or("User"),
                _ => label,
            };
            let label_style = if focused {
                Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD)
            } else {
                Style::new().fg(Color::Gray)
            };
            let marker = if focused { "› " } else { "  " };
            let value_span = if choice {
                Span::styled(
                    format!("◀ {value} ▶"),
                    if focused {
                        Style::new().add_modifier(Modifier::BOLD)
                    } else {
                        Style::new()
                    },
                )
            } else if value.is_empty() && field == Field::Url {
                Span::styled(
                    self.product().url_hint(self.deployment),
                    Style::new().fg(Color::DarkGray),
                )
            } else {
                let cursor = if focused { "▏" } else { "" };
                Span::raw(format!("{value}{cursor}"))
            };
            lines.push(Line::from(vec![
                Span::styled(marker, label_style),
                Span::styled(format!("{label:<14}"), label_style),
                value_span,
            ]));
        }
        lines.push(Line::default());
        lines.push(Line::from(Span::styled(
            method.hint(),
            Style::new().fg(Color::DarkGray),
        )));
        lines.push(Line::from(Span::styled(
            format!(
                "Saved to {}",
                Config::path()
                    .map(|p| p.display().to_string())
                    .unwrap_or_default()
            ),
            Style::new().fg(Color::DarkGray),
        )));
        match &self.status {
            Status::Idle => {}
            Status::Working(text) => lines.push(Line::from(Span::styled(
                text.clone(),
                Style::new().fg(Color::Yellow),
            ))),
            Status::Error(text) => lines.push(Line::from(Span::styled(
                text.clone(),
                Style::new().fg(Color::Red),
            ))),
        }
        frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), inner);
    }
}

/// Runs the form until the user saves (returns the verified outcome) or
/// cancels (returns `None`). Verification failures stay in the form.
pub async fn run_form(profile: &str, config: &Config) -> Result<Option<LoginOutcome>> {
    let mut form = Form::new(profile, config);
    // Restores the terminal on every way out, including a dropped future.
    let mut terminal = DropGuard::new(ratatui::init(), |_| ratatui::restore());
    async {
        loop {
            terminal.draw(|frame| form.render(frame))?;
            let event = tokio::task::block_in_place(event::read)?;
            let Event::Key(key) = event else { continue };
            if key.kind != KeyEventKind::Press {
                continue;
            }
            match form.handle(key) {
                Action::None => {}
                Action::Cancel => return Ok(None),
                Action::Submit => match form.request() {
                    Err(message) => form.status = Status::Error(message),
                    Ok(request) => {
                        form.status = Status::Working("Verifying with the server…".into());
                        terminal.draw(|frame| form.render(frame))?;
                        let verify = perform(&request, true);
                        tokio::pin!(verify);
                        // Esc or Ctrl-C stops waiting and returns to the form.
                        let result = loop {
                            tokio::select! {
                                result = &mut verify => break Some(result),
                                () = tokio::time::sleep(std::time::Duration::from_millis(50)) => {
                                    if event::poll(std::time::Duration::ZERO)?
                                        && let Event::Key(key) = event::read()?
                                        && (key.code == KeyCode::Esc
                                            || (key.code == KeyCode::Char('c')
                                                && key.modifiers.contains(KeyModifiers::CONTROL)))
                                    {
                                        break None;
                                    }
                                }
                            }
                        };
                        match result {
                            Some(Ok(outcome)) => return Ok(Some(outcome)),
                            Some(Err(err)) => form.status = Status::Error(format!("{err:#}")),
                            None => {
                                form.status = Status::Error(
                                    "verification cancelled; press Enter to try again".into(),
                                )
                            }
                        }
                    }
                },
            }
        }
    }
    .await
}

#[cfg(test)]
mod tests {
    use super::*;

    fn press(form: &mut Form, code: KeyCode) -> Action {
        form.handle(KeyEvent::new(code, KeyModifiers::NONE))
    }

    fn type_text(form: &mut Form, text: &str) {
        for c in text.chars() {
            press(form, KeyCode::Char(c));
        }
    }

    #[test]
    fn keyboard_fills_a_data_center_jira_token_login() {
        let mut form = Form::new("work", &Config::default());
        assert_eq!(form.focus, Field::Url);
        press(&mut form, KeyCode::Up); // Product
        press(&mut form, KeyCode::Right); // Jira
        press(&mut form, KeyCode::Down); // Deployment
        press(&mut form, KeyCode::Down); // URL
        type_text(&mut form, "jira.corp.example/");
        press(&mut form, KeyCode::Tab); // Method; leaving URL infers Data Center
        assert_eq!(form.deployment, Deployment::DataCenter);
        assert_eq!(form.method(), AuthMethod::Pat);
        assert!(!form.fields().contains(&Field::User), "tokens need no user");
        press(&mut form, KeyCode::Tab); // Secret
        type_text(&mut form, "pat-123");
        press(&mut form, KeyCode::Enter); // Profile
        press(&mut form, KeyCode::Enter); // Access
        press(&mut form, KeyCode::Right); // Read-only
        press(&mut form, KeyCode::Enter); // Submit
        assert_eq!(press(&mut form, KeyCode::Enter), Action::Submit);

        let request = form.request().expect("valid");
        assert_eq!(
            request,
            LoginRequest {
                product: Product::Jira,
                deployment: Deployment::DataCenter,
                url: "jira.corp.example/".into(),
                method: AuthMethod::Pat,
                user: String::new(),
                secret: "pat-123".into(),
                profile: Some("work".into()),
                read_only: true,
                oauth_port: OAUTH_PORT,
                oauth_scopes: None,
                client_cert: None,
                client_key: None,
                ca_cert: None,
            }
        );
        assert_eq!(normalize_url(&request.url), "https://jira.corp.example");
    }

    #[test]
    fn parse_callback_returns_the_code_for_the_right_state_only() {
        let line = |q: &str| format!("GET /callback?{q} HTTP/1.1");
        assert_eq!(
            parse_callback(&line("code=c1&state=s1"), "s1").expect("ok"),
            Some("c1".to_owned())
        );
        assert_eq!(
            parse_callback("GET /favicon.ico HTTP/1.1", "s1").expect("ok"),
            None
        );
        let err = parse_callback(&line("code=c1&state=evil"), "s1").expect_err("state");
        assert!(err.to_string().contains("wrong state"), "{err}");
        let err = parse_callback(
            &line("error=access_denied&error_description=User+said+no"),
            "s1",
        )
        .expect_err("refused");
        assert!(
            err.to_string().contains("(access_denied: User said no)"),
            "{err}"
        );
    }

    #[test]
    fn cloud_only_products_force_cloud_and_fill_the_default_url() {
        let mut form = Form::new("default", &Config::default());
        form.focus = Field::Product;
        for _ in 0..4 {
            press(&mut form, KeyCode::Right);
        }
        assert_eq!(form.product(), Product::Trello);
        assert_eq!(form.url, "https://api.trello.com");
        assert_eq!(form.method(), AuthMethod::TrelloKey);
        assert!(!form.fields().contains(&Field::Deployment));

        form.focus = Field::Submit;
        assert_eq!(
            form.request().expect_err("missing key"),
            "enter the api key (pass --user in scripts)"
        );
    }
}
