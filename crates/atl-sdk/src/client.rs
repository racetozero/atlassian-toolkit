//! Building, sending, and verifying requests.

use crate::catalog::{self, Catalog, HttpMethod, Operation, ParamLocation, ServiceDef};
use crate::config::{Auth, Config, Connection, Deployment, Family};
use crate::error::{Error, HttpError, Result};
use percent_encoding::{AsciiSet, CONTROLS, utf8_percent_encode};
use serde_json::{Map, Value, json};
use std::path::PathBuf;
use std::time::Duration;
use url::Url;

/// Characters escaped inside one path segment.
const SEGMENT: &AsciiSet = &CONTROLS
    .add(b' ')
    .add(b'"')
    .add(b'#')
    .add(b'%')
    .add(b'/')
    .add(b'<')
    .add(b'>')
    .add(b'?')
    .add(b'[')
    .add(b'\\')
    .add(b']')
    .add(b'^')
    .add(b'`')
    .add(b'{')
    .add(b'|')
    .add(b'}');

/// Like [`SEGMENT`] but keeps `/`, for parameters that are file paths
/// (Bitbucket `.../src/{commit}/{path}`).
const MULTI_SEGMENT: &AsciiSet = &SEGMENT.remove(b'/').remove(b'[').remove(b']');

/// Inputs for one operation call. Deserializes from
/// `{"params": {...}, "body": ..., "files": ["path", ...]}` (all optional).
#[derive(Debug, Clone, Default, serde::Deserialize)]
#[serde(default)]
pub struct CallArgs {
    /// Parameter values keyed by API name (`issueIdOrKey`) or CLI flag
    /// (`issue-id-or-key`). Arrays become repeated query parameters.
    pub params: Map<String, Value>,
    /// JSON body; for multipart operations, extra form fields as an object;
    /// for `text/plain` operations, a JSON string.
    pub body: Option<Value>,
    /// Files for multipart operations, sent as form field `file`.
    pub files: Vec<PathBuf>,
}

impl CallArgs {
    pub fn param(mut self, name: &str, value: impl Into<Value>) -> Self {
        self.params.insert(name.to_owned(), value.into());
        self
    }

    pub fn body(mut self, body: Value) -> Self {
        self.body = Some(body);
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum PreparedBody {
    None,
    Json(Value),
    Text(String),
    Form(Vec<(String, String)>),
    Multipart {
        files: Vec<PathBuf>,
        fields: Vec<(String, String)>,
    },
}

/// A fully built request that has not been sent.
#[derive(Debug, Clone)]
pub struct PreparedRequest {
    pub method: HttpMethod,
    pub url: Url,
    pub headers: Vec<(String, String)>,
    pub body: PreparedBody,
    pub(crate) auth: Option<Auth>,
    /// Set when no credential exists; sending then fails with this message.
    pub(crate) missing_credentials: Option<String>,
    /// Set for read-only connections; sending a write then fails.
    pub(crate) read_only: bool,
    /// The family whose connection built the request (for OAuth refresh).
    pub(crate) family: Option<Family>,
}

impl PreparedRequest {
    /// A request outside any connection (login, OAuth, verification): no
    /// family, never read-only, sent with `auth` as given.
    pub(crate) fn new(method: HttpMethod, url: Url, auth: Option<Auth>) -> Self {
        Self {
            method,
            url,
            headers: Vec::new(),
            body: PreparedBody::None,
            auth,
            missing_credentials: None,
            read_only: false,
            family: None,
        }
    }

    /// Makes this a multipart upload. Jira and Confluence reject multipart
    /// requests without `X-Atlassian-Token: no-check` (cross-site request
    /// forgery protection), so the header comes with the body.
    pub(crate) fn set_multipart(&mut self, files: Vec<PathBuf>, fields: Vec<(String, String)>) {
        self.headers
            .push(("X-Atlassian-Token".into(), "no-check".into()));
        self.body = PreparedBody::Multipart { files, fields };
    }

    /// A JSON description with secrets masked, for `--dry-run`.
    pub fn describe(&self) -> Value {
        let mut headers = Map::new();
        match &self.auth {
            Some(Auth::Cookie { .. }) => {
                headers.insert("cookie".into(), "***".into());
            }
            Some(Auth::None {}) | None => {}
            Some(auth) => {
                headers.insert("authorization".into(), auth.masked_header().into());
            }
        }
        for (name, value) in &self.headers {
            let shown = match name.eq_ignore_ascii_case("cookie") {
                true => "***".to_owned(),
                false => value.clone(),
            };
            headers.insert(name.to_ascii_lowercase(), shown.into());
        }
        let body = match &self.body {
            PreparedBody::None => Value::Null,
            PreparedBody::Json(value) => value.clone(),
            PreparedBody::Text(text) => text.clone().into(),
            PreparedBody::Form(pairs) => pairs
                .iter()
                .map(|(k, v)| (k.clone(), Value::from(v.clone())))
                .collect::<Map<_, _>>()
                .into(),
            PreparedBody::Multipart { files, fields } => json!({
                "files": files.iter().map(|f| f.display().to_string()).collect::<Vec<_>>(),
                "fields": fields.iter().map(|(k, v)| (k.clone(), Value::from(v.clone()))).collect::<Map<_, _>>(),
            }),
        };
        let mut out = json!({ "method": self.method.as_str(), "url": self.url.as_str(), "headers": headers, "body": body });
        if let Some(message) = &self.missing_credentials {
            out["warning"] = message.clone().into();
        }
        out
    }

    pub(crate) fn with_url(&self, url: Url) -> Self {
        Self {
            url,
            ..self.clone()
        }
    }
}

/// A received response with a success status.
#[derive(Debug, Clone)]
pub struct Response {
    pub status: u16,
    pub headers: reqwest::header::HeaderMap,
    pub body: Vec<u8>,
}

impl Response {
    pub fn content_type(&self) -> &str {
        self.headers
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .unwrap_or("")
    }

    /// The body parsed as JSON, or `None` for empty and non-JSON bodies.
    pub fn json(&self) -> Option<Value> {
        if self.body.is_empty() {
            return None;
        }
        serde_json::from_slice(&self.body).ok()
    }

    /// The JSON body, or `{"status": n}` for an empty body and
    /// `{"status": n, "text": "..."}` for a non-JSON one.
    pub fn value(&self) -> Value {
        self.json().unwrap_or_else(|| match self.body.is_empty() {
            true => json!({ "status": self.status }),
            false => json!({ "status": self.status, "text": String::from_utf8_lossy(&self.body) }),
        })
    }
}

/// Per-family HTTP clients with their own TLS settings.
type TlsClients =
    [std::sync::OnceLock<std::result::Result<reqwest::Client, String>>; Family::ALL.len()];

/// Per-family cache of resolved credentials.
type SecretSlots = [std::sync::Mutex<Option<std::result::Result<Auth, String>>>; Family::ALL.len()];
/// One OAuth refresh at a time per family: refresh tokens rotate, so a second
/// refresh with the same old token would fail.
type RefreshLocks = [async_lock::Mutex<()>; Family::ALL.len()];

/// Sends requests for every service with one HTTP connection pool.
#[derive(Debug, Clone)]
pub struct Client {
    /// Built on the first request: commands that send nothing (help,
    /// --dry-run, schema) skip the TLS setup. Clones share one pool.
    http: std::sync::Arc<std::sync::OnceLock<reqwest::Client>>,
    /// HTTP clients of families with client certificates or extra CAs.
    tls_http: std::sync::Arc<TlsClients>,
    profile: String,
    connections: [Option<Connection>; Family::ALL.len()],
    /// Credentials resolved on the first request of each family (netrc),
    /// and refreshed OAuth tokens.
    secrets: std::sync::Arc<SecretSlots>,
    refreshing: std::sync::Arc<RefreshLocks>,
    /// The config file the connections came from; refreshed OAuth tokens
    /// are saved there.
    config_path: Option<PathBuf>,
    deployment_override: Option<Deployment>,
    /// Refuse writes for every family (`ATL_READ_ONLY=1`).
    read_only: bool,
    /// Retries for rate limits and brief outages (`ATL_MAX_RETRIES`, default 3).
    max_retries: u32,
    /// Stop at the first write with [`Error::DryRun`]; reads still run.
    dry_run: bool,
}

fn family_index(family: Family) -> usize {
    Family::ALL
        .iter()
        .position(|candidate| *candidate == family)
        .expect("ALL lists every family")
}

// One client per `Client`, never one per process: pooled connections belong
// to the tokio runtime that opened them, and SDK callers may run several.
pub(crate) fn http_client() -> reqwest::Client {
    client_builder()
        .build()
        .expect("the HTTP client initializes")
}

/// The user agent and timeouts every client shares.
fn client_builder() -> reqwest::ClientBuilder {
    let builder = reqwest::Client::builder();
    // Browsers set the user agent and own connection timeouts.
    #[cfg(feature = "native")]
    let builder = builder
        .user_agent(concat!("atl/", env!("CARGO_PKG_VERSION")))
        .connect_timeout(Duration::from_secs(20))
        // A server that accepts the connection but stops sending fails
        // instead of hanging every caller (TUI, MCP, the language SDKs).
        .read_timeout(Duration::from_secs(60));
    builder
}

/// An HTTP client with the connection's client certificate and extra CA
/// certificates (mutual TLS, private CAs).
pub(crate) fn tls_client(
    family: Family,
    connection: &Connection,
) -> std::result::Result<reqwest::Client, String> {
    #[cfg(not(feature = "native"))]
    {
        let _ = (family, connection);
        Err("client certificates need the native SDK; browsers manage TLS themselves".into())
    }
    #[cfg(feature = "native")]
    {
        let prefix = family.env_prefix();
        let read =
            |path: &PathBuf, what: &str, flag: &str| {
                std::fs::read(path).map_err(|err| {
                format!(
                    "cannot read the {what} {} ({err}); pass a PEM file to {flag} (or {prefix}_{})",
                    path.display(),
                    flag.trim_start_matches("--").replace('-', "_").to_ascii_uppercase()
                )
            })
            };
        let mut builder = client_builder();
        if let Some(cert) = &connection.client_cert {
            let mut pem = read(cert, "client certificate", "--client-cert")?;
            if let Some(key) = &connection.client_key {
                pem.push(b'\n');
                pem.extend(read(key, "client key", "--client-key")?);
            }
            let identity = reqwest::Identity::from_pem(&pem).map_err(|err| {
                format!(
                    "cannot use the client certificate {} ({err}); pass a PEM certificate with --client-cert and its PEM private key with --client-key (or {prefix}_CLIENT_CERT/_CLIENT_KEY)",
                    cert.display()
                )
            })?;
            builder = builder.identity(identity);
        }
        if let Some(ca) = &connection.ca_cert {
            let certs = reqwest::Certificate::from_pem_bundle(&read(ca, "CA certificate", "--ca-cert")?)
                .map_err(|err| {
                    format!(
                        "cannot use the CA certificates {} ({err}); pass a PEM bundle with --ca-cert (or {prefix}_CA_CERT)",
                        ca.display()
                    )
                })?;
            builder = builder.tls_certs_merge(certs);
        }
        builder
            .build()
            .map_err(|err| format!("cannot set up TLS for {family} ({err})"))
    }
}

const DEFAULT_RETRIES: u32 = 3;

fn retries_from_env() -> u32 {
    std::env::var("ATL_MAX_RETRIES")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(DEFAULT_RETRIES)
}

impl Client {
    /// Uses the profile's connections from `config`, with environment
    /// overrides (see [`crate::config`]).
    pub fn new(config: &Config, profile: Option<&str>) -> Self {
        let profile = config.profile_name(profile);
        let connections = Family::ALL.map(|family| config.connection(&profile, family));
        let read_only = crate::config::env_flag("ATL_READ_ONLY");
        Self {
            http: Default::default(),
            tls_http: Default::default(),
            profile,
            connections,
            secrets: Default::default(),
            refreshing: Default::default(),
            config_path: config.path.clone(),
            deployment_override: None,
            read_only,
            max_retries: retries_from_env(),
            dry_run: false,
        }
    }

    /// Loads the config file and calls [`Client::new`].
    pub fn from_env(profile: Option<&str>) -> Result<Self> {
        Ok(Self::new(&Config::load()?, profile))
    }

    /// A client with explicit connections, for programs that manage their
    /// own credentials.
    pub fn with_connections(connections: impl IntoIterator<Item = (Family, Connection)>) -> Self {
        let mut slots: [Option<Connection>; Family::ALL.len()] = Default::default();
        for (family, connection) in connections {
            slots[family_index(family)] = Some(connection);
        }
        Self {
            http: Default::default(),
            tls_http: Default::default(),
            profile: "custom".into(),
            connections: slots,
            secrets: Default::default(),
            refreshing: Default::default(),
            config_path: None,
            deployment_override: None,
            read_only: false,
            max_retries: retries_from_env(),
            dry_run: false,
        }
    }

    /// Forces a deployment for every family (CLI `--deployment`).
    pub fn set_deployment_override(&mut self, deployment: Option<Deployment>) {
        self.deployment_override = deployment;
    }

    pub fn set_max_retries(&mut self, retries: u32) {
        self.max_retries = retries;
    }

    /// Makes [`Client::execute`] return [`Error::DryRun`] with the request
    /// instead of sending a write. Reads still run, so a task can look up
    /// what it needs (transitions, versions) before the write it would send.
    pub fn set_dry_run(&mut self, dry_run: bool) {
        self.dry_run = dry_run;
    }

    /// Refuses every write request when true (in addition to connections
    /// stored with `read_only = true`).
    pub fn set_read_only(&mut self, read_only: bool) {
        self.read_only = read_only;
    }

    /// True when writes to `family` are refused.
    pub fn is_read_only(&self, family: Family) -> bool {
        self.read_only || self.connection(family).is_some_and(|c| c.read_only)
    }

    /// The browser URL of the family's site (differs from the API base URL
    /// for scoped tokens, which go through api.atlassian.com).
    pub fn site_url(&self, family: Family) -> String {
        match self.connection(family) {
            Some(c) => c
                .site_url
                .clone()
                .unwrap_or_else(|| c.base_url.clone())
                .trim_end_matches('/')
                .to_owned(),
            None => family.placeholder_url(self.deployment(family)).to_owned(),
        }
    }

    pub fn profile(&self) -> &str {
        &self.profile
    }

    pub fn connection(&self, family: Family) -> Option<&Connection> {
        self.connections[family_index(family)].as_ref()
    }

    pub fn deployment(&self, family: Family) -> Deployment {
        self.deployment_override
            .or_else(|| self.connection(family).map(|c| c.deployment))
            .unwrap_or_default()
    }

    /// The catalog that serves `service` for the configured deployment.
    pub fn catalog_for(&self, service: &ServiceDef) -> Result<&'static Catalog> {
        let deployment = self.deployment(service.family);
        match service.catalog_key(deployment) {
            Some(key) => Ok(catalog::catalog(key)),
            None => Err(Error::Unknown(format!(
                "service `{}` is not available on {deployment}: {}",
                service.name,
                service
                    .data_center_note
                    .unwrap_or("no OpenAPI document exists for it")
            ))),
        }
    }

    /// Looks up a service and an operation by `operationId`, CLI name, or
    /// `<resource>.<method>`.
    pub fn resolve(
        &self,
        service: &str,
        operation: &str,
    ) -> Result<(&'static ServiceDef, &'static Catalog, &'static Operation)> {
        let def = catalog::service(service).ok_or_else(|| {
            Error::Unknown(format!(
                "unknown service `{service}`; use one of: {}",
                catalog::services()
                    .iter()
                    .map(|s| s.name)
                    .collect::<Vec<_>>()
                    .join(", ")
            ))
        })?;
        let catalog = self.catalog_for(def)?;
        let op = catalog.find(operation).ok_or_else(|| {
            Error::Unknown(format!(
                "service `{service}` has no operation `{operation}`; list them with `atl api {service} --help` or search with the MCP tool atlassian_search_operations"
            ))
        })?;
        Ok((def, catalog, op))
    }

    /// The `ATL_<FAMILY>_COOKIE` header, when set.
    fn extra_cookie(&self, family: Family) -> Option<(String, String)> {
        let cookie = self.connection(family)?.extra_cookie.clone()?;
        Some(("Cookie".into(), cookie))
    }

    fn base(&self, family: Family) -> (String, Option<Auth>, Option<String>, bool) {
        let read_only = self.is_read_only(family);
        match self.connection(family) {
            Some(connection) => {
                let base = connection.base_url.trim_end_matches('/').to_owned();
                let mut cached = self.secrets[family_index(family)]
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                let secret =
                    cached.get_or_insert_with(|| connection.resolve_secret().map(|c| c.auth));
                match secret {
                    Ok(auth) => (base, Some(auth.clone()), None, read_only),
                    Err(reason) => (
                        base,
                        None,
                        Some(format!(
                            "cannot use the {family} login of profile `{}` ({reason}); run `atl auth login` again",
                            self.profile
                        )),
                        read_only,
                    ),
                }
            }
            None => {
                let prefix = family.env_prefix();
                let message = format!(
                    "no {family} credentials in profile `{}`; run `atl auth login` or set {prefix}_URL and {prefix}_TOKEN",
                    self.profile
                );
                (
                    family.placeholder_url(self.deployment(family)).to_owned(),
                    None,
                    Some(message),
                    read_only,
                )
            }
        }
    }

    /// Validates `args` against the operation and builds the request.
    pub fn prepare(
        &self,
        service: &ServiceDef,
        catalog: &Catalog,
        op: &Operation,
        args: &CallArgs,
    ) -> Result<PreparedRequest> {
        let params = catalog.params_of(op);
        for key in args.params.keys() {
            if !params.iter().any(|p| p.name == *key || p.flag == *key) {
                let valid: Vec<&str> = params.iter().map(|p| p.name.as_ref()).collect();
                return Err(Error::Invalid(format!(
                    "operation `{}` has no parameter `{key}`; valid parameters: {}",
                    op.id,
                    if valid.is_empty() {
                        "(none)".to_owned()
                    } else {
                        valid.join(", ")
                    }
                )));
            }
        }

        let mut path = op.path.to_string();
        let mut query: Vec<(String, String)> = Vec::new();
        let mut headers: Vec<(String, String)> = Vec::new();
        for param in params {
            let value = args
                .params
                .get(param.name.as_ref())
                .or_else(|| args.params.get(param.flag.as_ref()))
                .filter(|value| !value.is_null());
            let Some(value) = value else {
                if param.required {
                    return Err(Error::Invalid(format!(
                        "operation `{}` needs parameter `{}` (CLI flag --{}); see `atl schema {}.{}`",
                        op.id, param.name, param.flag, service.name, op.id
                    )));
                }
                continue;
            };
            match param.location {
                ParamLocation::Path => {
                    let text = match value {
                        Value::Array(items) => {
                            items.iter().map(scalar).collect::<Vec<_>>().join(",")
                        }
                        other => scalar(other),
                    };
                    let set = if param.name == "path" {
                        MULTI_SEGMENT
                    } else {
                        SEGMENT
                    };
                    let encoded = utf8_percent_encode(&text, set).to_string();
                    path = path.replace(&format!("{{{}}}", param.name), &encoded);
                }
                ParamLocation::Query => match value {
                    Value::Array(items) => query.extend(
                        items
                            .iter()
                            .map(|item| (param.name.to_string(), scalar(item))),
                    ),
                    other => query.push((param.name.to_string(), scalar(other))),
                },
                ParamLocation::Header => headers.push((param.name.to_string(), scalar(value))),
            }
        }

        let body = match (&op.body, &args.body, args.files.is_empty()) {
            (None, None, true) => PreparedBody::None,
            (None, _, _) => {
                return Err(Error::Invalid(format!(
                    "operation `{}` takes no request body; remove --json/--upload (or `body`)",
                    op.id
                )));
            }
            (Some(spec), body, _) if spec.is_multipart() => {
                let fields = match body {
                    None => Vec::new(),
                    Some(Value::Object(map)) => {
                        map.iter().map(|(k, v)| (k.clone(), scalar(v))).collect()
                    }
                    Some(_) => {
                        return Err(Error::Invalid(format!(
                            "operation `{}` is a multipart upload; pass files with --upload and extra form fields as a JSON object",
                            op.id
                        )));
                    }
                };
                if args.files.is_empty() && spec.required {
                    return Err(Error::Invalid(format!(
                        "operation `{}` uploads files; pass at least one with --upload <path>",
                        op.id
                    )));
                }
                PreparedBody::Multipart {
                    files: args.files.clone(),
                    fields,
                }
            }
            (Some(_), _, false) => {
                return Err(Error::Invalid(format!(
                    "operation `{}` does not take file uploads; remove --upload",
                    op.id
                )));
            }
            (Some(spec), None, true) if spec.required => {
                return Err(Error::Invalid(format!(
                    "operation `{}` needs a request body; pass --json '<json>' (see `atl schema {}.{}`)",
                    op.id, service.name, op.id
                )));
            }
            (Some(_), None, true) => PreparedBody::None,
            (Some(spec), Some(body), true) => match spec.content_type.as_ref() {
                "application/x-www-form-urlencoded" => match body {
                    Value::Object(map) => PreparedBody::Form(
                        map.iter().map(|(k, v)| (k.clone(), scalar(v))).collect(),
                    ),
                    _ => {
                        return Err(Error::Invalid(format!(
                            "operation `{}` sends a form; pass the fields as a JSON object",
                            op.id
                        )));
                    }
                },
                "text/plain" => PreparedBody::Text(scalar(body)),
                _ => PreparedBody::Json(body.clone()),
            },
        };
        if let Some(spec) = &op.body
            && !matches!(body, PreparedBody::None | PreparedBody::Multipart { .. })
        {
            headers.push(("Content-Type".into(), spec.content_type.to_string()));
        }

        let (base, auth, missing_credentials, read_only) = self.base(service.family);
        let mut url = Url::parse(&format!("{base}{path}")).map_err(|err| {
            Error::Invalid(format!(
                "base URL `{base}` for {} is not a valid URL ({err}); run `atl auth login` to fix it",
                service.family
            ))
        })?;
        if !query.is_empty() {
            url.query_pairs_mut().extend_pairs(query);
        }
        headers.extend(self.extra_cookie(service.family));
        let mut request = PreparedRequest {
            method: op.method,
            url,
            headers,
            body: PreparedBody::None,
            auth,
            missing_credentials,
            read_only,
            family: Some(service.family),
        };
        match body {
            PreparedBody::Multipart { files, fields } => request.set_multipart(files, fields),
            body => request.body = body,
        }
        Ok(request)
    }

    /// Builds a request for any path on a family's server (`atl api`).
    pub fn prepare_raw(
        &self,
        family: Family,
        method: HttpMethod,
        path: &str,
        query: &Map<String, Value>,
        body: Option<Value>,
    ) -> Result<PreparedRequest> {
        let (base, auth, missing_credentials, read_only) = self.base(family);
        let joined = if path.starts_with("http://") || path.starts_with("https://") {
            // Credentials only go to the configured server.
            let origin = |u: &str| Url::parse(u).ok().map(|u| u.origin());
            if missing_credentials.is_none() && origin(path) != origin(&base) {
                return Err(Error::Invalid(format!(
                    "`{path}` is not on the {family} server {base}; pass a path such as /rest/api/3/myself"
                )));
            }
            path.to_owned()
        } else {
            format!("{base}/{}", path.trim_start_matches('/'))
        };
        let mut url = Url::parse(&joined).map_err(|err| {
            Error::Invalid(format!(
                "`{joined}` is not a valid URL ({err}); pass a path such as /rest/api/3/myself"
            ))
        })?;
        for (key, value) in query {
            match value {
                Value::Array(items) => items.iter().for_each(|item| {
                    url.query_pairs_mut().append_pair(key, &scalar(item));
                }),
                other => {
                    url.query_pairs_mut().append_pair(key, &scalar(other));
                }
            }
        }
        let mut headers = Vec::new();
        let body = match body {
            Some(body) => {
                headers.push(("Content-Type".into(), "application/json".into()));
                PreparedBody::Json(body)
            }
            None => PreparedBody::None,
        };
        headers.extend(self.extra_cookie(family));
        Ok(PreparedRequest {
            method,
            url,
            headers,
            body,
            auth,
            missing_credentials,
            read_only,
            family: Some(family),
        })
    }

    /// Sends the request. Non-success statuses become [`Error::Http`].
    pub async fn execute(&self, request: &PreparedRequest) -> Result<Response> {
        // A dry run stops at the first write. Without a login it cannot send
        // the reads either, so it shows the first request, whatever it is,
        // against the placeholder URL.
        if self.dry_run
            && (request.missing_credentials.is_some()
                || !request.method.is_safe_for(request.url.path()))
        {
            return Err(Error::DryRun(Box::new(request.describe())));
        }
        if let Some(message) = &request.missing_credentials {
            return Err(Error::MissingCredentials(message.clone()));
        }
        if request.read_only && !request.method.is_safe_for(request.url.path()) {
            return Err(Error::ReadOnly(format!(
                "{} {} is a write and this client is read-only (set by `atl mcp --read-only`, ATL_READ_ONLY, or a login with --read-only); remove that setting to allow writes",
                request.method,
                request.url.path()
            )));
        }
        let http = match request.family.and_then(|f| Some((f, self.connection(f)?))) {
            Some((family, connection)) if connection.has_tls() => self.tls_http
                [family_index(family)]
            .get_or_init(|| tls_client(family, connection))
            .as_ref()
            .map_err(|reason| Error::Invalid(reason.clone()))?,
            _ => self.http.get_or_init(http_client),
        };
        let mut request = std::borrow::Cow::Borrowed(request);
        let mut refreshed = false;
        if let (Some(family), Some(auth)) = (request.family, request.auth.clone())
            && oauth_expiring(&auth)
        {
            request.to_mut().auth = Some(self.refresh_oauth(family, &auth).await?);
            refreshed = true;
        }
        match send(http, &request, self.max_retries).await {
            Err(Error::Http(error))
                if error.status == 401
                    && !refreshed
                    && matches!(&request.auth, Some(Auth::OAuth { refresh_token, .. }) if !refresh_token.is_empty()) =>
            {
                let (Some(family), Some(auth)) = (request.family, request.auth.clone()) else {
                    return Err(Error::Http(error));
                };
                let mut retry = request.into_owned();
                retry.auth = Some(self.refresh_oauth(family, &auth).await?);
                send(http, &retry, self.max_retries).await
            }
            other => other,
        }
    }

    /// Refreshes an OAuth credential, caches it for this client, and saves
    /// it to the config file the client was loaded from.
    async fn refresh_oauth(&self, family: Family, auth: &Auth) -> Result<Auth> {
        let _one_at_a_time = self.refreshing[family_index(family)].lock().await;
        // A request that waited here finds the token the other one refreshed.
        let cached = self.secrets[family_index(family)]
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone();
        if let (
            Some(Ok(
                current @ Auth::OAuth {
                    access_token: new, ..
                },
            )),
            Auth::OAuth {
                access_token: old, ..
            },
        ) = (&cached, auth)
            && new != old
            && !oauth_expiring(current)
        {
            return Ok(current.clone());
        }
        let fresh = crate::oauth::refresh(auth).await?;
        *self.secrets[family_index(family)]
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(Ok(fresh.clone()));
        // The request still succeeds without the save, but refresh tokens
        // rotate: the next process would hold a used one, so say so.
        if let Some(path) = &self.config_path
            && let Err(err) = Config::store_auth(path, &self.profile, family, &fresh)
        {
            #[cfg(feature = "native")]
            eprintln!(
                "atl: refreshed the {family} OAuth token but cannot save it ({err}); the next run may need `atl auth login` again"
            );
            #[cfg(not(feature = "native"))]
            let _ = err;
        }
        Ok(fresh)
    }

    /// Resolves, prepares, and sends one operation.
    pub async fn call(&self, service: &str, operation: &str, args: &CallArgs) -> Result<Response> {
        let (def, catalog, op) = self.resolve(service, operation)?;
        let request = self.prepare(def, catalog, op, args)?;
        self.execute(&request).await
    }
}

impl Client {
    /// Uploads files as multipart field `file` to a path (attachments),
    /// adding the `X-Atlassian-Token: no-check` header Jira and Confluence
    /// require.
    pub async fn upload(
        &self,
        family: Family,
        path: &str,
        files: &[std::path::PathBuf],
    ) -> Result<Value> {
        self.upload_with(family, HttpMethod::Post, path, files)
            .await
    }

    /// [`Client::upload`] with another method (Confluence Cloud creates or
    /// updates attachments with PUT).
    pub async fn upload_with(
        &self,
        family: Family,
        method: HttpMethod,
        path: &str,
        files: &[std::path::PathBuf],
    ) -> Result<Value> {
        let mut request = self.prepare_raw(family, method, path, &Map::new(), None)?;
        request.set_multipart(files.to_vec(), Vec::new());
        Ok(self.execute(&request).await?.value())
    }

    /// Sends any request to a service's server (`atl api`), by service name.
    pub async fn request(
        &self,
        service: &str,
        method: HttpMethod,
        path: &str,
        query: &Map<String, Value>,
        body: Option<Value>,
    ) -> Result<Response> {
        let def = catalog::service(service).ok_or_else(|| {
            Error::Unknown(format!(
                "unknown service `{service}`; use one of: {}",
                catalog::services()
                    .iter()
                    .map(|s| s.name)
                    .collect::<Vec<_>>()
                    .join(", ")
            ))
        })?;
        let request = self.prepare_raw(def.family, method, path, query, body)?;
        self.execute(&request).await
    }

    /// Downloads raw bytes from a path or a URL on the family's server.
    pub async fn download(&self, family: Family, path_or_url: &str) -> Result<Vec<u8>> {
        let request = self.prepare_raw(family, HttpMethod::Get, path_or_url, &Map::new(), None)?;
        Ok(self.execute(&request).await?.body)
    }

    /// A client from JSON connections, the format the language SDKs use:
    /// `{"jira": {"deployment": "cloud", "base_url": "https://x.atlassian.net",
    /// "auth": {"kind": "basic", "username": "...", "secret": "..."}}}`.
    pub fn from_connections_json(value: Value) -> Result<Client> {
        let map: std::collections::BTreeMap<Family, Connection> = serde_json::from_value(value).map_err(|err| {
            Error::Invalid(format!(
                "connections are invalid ({err}); pass an object keyed by jira, confluence, bitbucket, trello, admin or admin-scim, each with deployment, base_url and auth {{kind: basic|bearer|trello, ...}}"
            ))
        })?;
        Ok(Client::with_connections(map))
    }
}

/// True for an OAuth access token that expires within a minute and can be
/// refreshed.
fn oauth_expiring(auth: &Auth) -> bool {
    match auth {
        Auth::OAuth {
            refresh_token,
            expires_at,
            ..
        } => {
            !refresh_token.is_empty()
                && *expires_at > 0
                && crate::time::now().is_some_and(|now| now + 60 >= *expires_at)
        }
        _ => false,
    }
}

impl Client {
    /// Sends a request to a path on a family's server and returns the JSON
    /// body (or `{"status": n}` for empty and non-JSON bodies). Used by the
    /// task-level APIs in [`crate::jira`] and [`crate::confluence`].
    pub async fn send_json(
        &self,
        family: Family,
        method: HttpMethod,
        path: &str,
        query: &[(&str, String)],
        body: Option<Value>,
    ) -> Result<Value> {
        let mut request = self.prepare_raw(family, method, path, &Map::new(), body)?;
        if !query.is_empty() {
            request
                .url
                .query_pairs_mut()
                .extend_pairs(query.iter().map(|(k, v)| (*k, v.as_str())));
        }
        Ok(self.execute(&request).await?.value())
    }
}

fn scalar(value: &Value) -> String {
    match value {
        Value::String(text) => text.clone(),
        Value::Null => String::new(),
        other => other.to_string(),
    }
}

/// Retries 429 for every method and 502/503/504 or connection failures for
/// methods that are safe to repeat, waiting `Retry-After` (at most 60 s) or an
/// exponential backoff with jitter.
pub(crate) async fn send(
    http: &reqwest::Client,
    request: &PreparedRequest,
    max_retries: u32,
) -> Result<Response> {
    let repeatable = request.method.is_repeatable();
    let mut attempt = 0;
    loop {
        #[cfg(feature = "native")]
        let started = debug::request(request);
        let result = send_once(http, request).await;
        #[cfg(feature = "native")]
        debug::response(request, &result, started);
        let wait = match &result {
            Err(Error::Http(http))
                if http.status == 429 || (repeatable && matches!(http.status, 502..=504)) =>
            {
                Some(http.retry_after)
            }
            Err(Error::Transport { source, .. }) if repeatable && transient(source) => Some(None),
            _ => None,
        };
        match wait {
            Some(retry_after) if attempt < max_retries => {
                let backoff = 500u64 << attempt.min(6); // at most 32 s
                let jitter =
                    (u64::from(attempt) * 137 + request.url.as_str().len() as u64 * 31) % 250;
                let delay = retry_after.map_or(backoff + jitter, |secs| secs.min(60) * 1000);
                #[cfg(feature = "native")]
                debug::retry(delay, attempt + 1, max_retries);
                crate::rt::sleep(Duration::from_millis(delay)).await;
                attempt += 1;
            }
            _ => return result,
        }
    }
}

/// How much HTTP traffic to print to standard error.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DebugLevel {
    Off,
    /// One line per request, response and retry, with timings.
    Requests,
    /// Also headers and bodies (secrets masked, long bodies cut).
    Full,
}

/// Overrides `ATL_DEBUG` for this process (the CLI's `--debug` flag).
pub fn set_debug(level: DebugLevel) {
    #[cfg(feature = "native")]
    debug::LEVEL.store(level as u8, std::sync::atomic::Ordering::Relaxed);
    #[cfg(not(feature = "native"))]
    let _ = level;
}

/// The `ATL_DEBUG` logger: `1` (or `true`, `requests`) and `full` (or `2`).
#[cfg(feature = "native")]
mod debug {
    use super::{DebugLevel, PreparedRequest, Response};
    use crate::error::Error;
    use std::sync::atomic::{AtomicU8, Ordering};
    use std::time::Instant;

    /// `UNSET` until the first request reads `ATL_DEBUG`.
    const UNSET: u8 = u8::MAX;
    pub(super) static LEVEL: AtomicU8 = AtomicU8::new(UNSET);
    const BODY_LIMIT: usize = 4000;

    fn level() -> DebugLevel {
        let mut raw = LEVEL.load(Ordering::Relaxed);
        if raw == UNSET {
            let level = match std::env::var("ATL_DEBUG")
                .unwrap_or_default()
                .to_ascii_lowercase()
                .as_str()
            {
                "1" | "true" | "requests" => DebugLevel::Requests,
                "2" | "full" => DebugLevel::Full,
                _ => DebugLevel::Off,
            };
            raw = level as u8;
            LEVEL.store(raw, Ordering::Relaxed);
        }
        match raw {
            1 => DebugLevel::Requests,
            2 => DebugLevel::Full,
            _ => DebugLevel::Off,
        }
    }

    fn body_text(bytes: &[u8], content_type: &str) -> String {
        match std::str::from_utf8(bytes) {
            Ok("") => String::new(),
            Ok(text) if text.chars().count() > BODY_LIMIT => {
                let cut: String = text.chars().take(BODY_LIMIT).collect();
                format!("{cut}… ({} bytes in all)", bytes.len())
            }
            Ok(text) => text.to_owned(),
            Err(_) => format!("<{} bytes of {content_type}>", bytes.len()),
        }
    }

    pub(super) fn request(request: &PreparedRequest) -> Option<Instant> {
        let level = level();
        if level == DebugLevel::Off {
            return None;
        }
        eprintln!("atl: > {} {}", request.method, request.url);
        if level == DebugLevel::Full {
            let described = request.describe();
            for (name, value) in described["headers"].as_object().into_iter().flatten() {
                eprintln!("atl: >   {name}: {}", value.as_str().unwrap_or_default());
            }
            if !described["body"].is_null() {
                let body = serde_json::to_string(&described["body"]).unwrap_or_default();
                eprintln!(
                    "atl: >   {}",
                    body_text(body.as_bytes(), "application/json")
                );
            }
        }
        Some(Instant::now())
    }

    pub(super) fn response(
        request: &PreparedRequest,
        result: &Result<Response, Error>,
        started: Option<Instant>,
    ) {
        let Some(started) = started else {
            return;
        };
        let ms = started.elapsed().as_millis();
        let full = level() == DebugLevel::Full;
        match result {
            Ok(response) => {
                eprintln!("atl: < {} {} ({ms} ms)", response.status, request.url);
                if full {
                    for (name, value) in &response.headers {
                        let shown = match name.as_str() {
                            "set-cookie" => "***",
                            _ => value.to_str().unwrap_or("<binary>"),
                        };
                        eprintln!("atl: <   {name}: {shown}");
                    }
                    let body = body_text(&response.body, response.content_type());
                    if !body.is_empty() {
                        eprintln!("atl: <   {body}");
                    }
                }
            }
            Err(Error::Http(error)) => {
                eprintln!("atl: < {} {} ({ms} ms)", error.status, request.url);
                if full && let Some(body) = &error.body {
                    eprintln!("atl: <   {body}");
                } else if full {
                    eprintln!("atl: <   {}", error.message);
                }
            }
            Err(error) => eprintln!("atl: < error: {error} ({ms} ms)"),
        }
    }

    pub(super) fn retry(delay_ms: u64, attempt: u32, max: u32) {
        if level() != DebugLevel::Off {
            eprintln!("atl: … retrying in {delay_ms} ms (attempt {attempt} of {max})");
        }
    }
}

fn transient(error: &reqwest::Error) -> bool {
    #[cfg(feature = "native")]
    return error.is_connect() || error.is_timeout();
    #[cfg(not(feature = "native"))]
    return error.is_timeout();
}

async fn send_once(http: &reqwest::Client, request: &PreparedRequest) -> Result<Response> {
    let method = reqwest::Method::from_bytes(request.method.as_str().as_bytes())
        .expect("HTTP methods are valid tokens");
    let mut builder = http
        .request(method, request.url.clone())
        .header(reqwest::header::ACCEPT, "application/json, */*;q=0.5");
    builder = match &request.auth {
        Some(Auth::Basic { username, secret }) => builder.basic_auth(username, Some(secret)),
        Some(
            Auth::Bearer { token }
            | Auth::OAuth {
                access_token: token,
                ..
            },
        ) => builder.bearer_auth(token),
        Some(Auth::Trello { key, token }) => builder.header(
            reqwest::header::AUTHORIZATION,
            format!("OAuth oauth_consumer_key=\"{key}\", oauth_token=\"{token}\""),
        ),
        // Netrc is resolved to Basic before requests are built.
        Some(Auth::Cookie { .. } | Auth::Netrc {} | Auth::None {}) | None => builder,
    };
    // One Cookie header: the session cookie and any extra cookies, joined.
    let mut cookies: Vec<&str> = Vec::new();
    if let Some(Auth::Cookie { cookie }) = &request.auth {
        cookies.push(cookie);
    }
    for (name, value) in &request.headers {
        match name.eq_ignore_ascii_case("cookie") {
            true => cookies.push(value),
            false => builder = builder.header(name, value),
        }
    }
    if !cookies.is_empty() {
        builder = builder.header(reqwest::header::COOKIE, cookies.join("; "));
    }
    builder = match &request.body {
        PreparedBody::None => builder,
        PreparedBody::Json(value) => {
            builder.body(serde_json::to_vec(value).expect("JSON values serialize"))
        }
        PreparedBody::Text(text) => builder.body(text.clone()),
        PreparedBody::Form(pairs) => builder.form(pairs),
        PreparedBody::Multipart { files, fields } => {
            let mut form = reqwest::multipart::Form::new();
            for (name, value) in fields {
                form = form.text(name.clone(), value.clone());
            }
            #[cfg(not(feature = "native"))]
            if !files.is_empty() {
                return Err(Error::Invalid(
                    "file uploads need the native SDK; WebAssembly builds cannot read local files"
                        .into(),
                ));
            }
            #[cfg(feature = "native")]
            for file in files {
                let part = reqwest::multipart::Part::file(file).await.map_err(|err| {
                    Error::io(
                        format!(
                            "cannot read upload {}; check the --upload path",
                            file.display()
                        ),
                        err,
                    )
                })?;
                form = form.part("file", part);
            }
            builder.multipart(form)
        }
    };
    let url = request.url.to_string();
    let response = builder.send().await.map_err(|source| Error::Transport {
        url: url.clone(),
        source,
    })?;
    let status = response.status().as_u16();
    let headers = response.headers().clone();
    let body: Vec<u8> = response
        .bytes()
        .await
        .map_err(|source| Error::Transport {
            url: url.clone(),
            source,
        })?
        .into();
    if (200..300).contains(&status) {
        return Ok(Response {
            status,
            headers,
            body,
        });
    }
    let retry_after = headers
        .get(reqwest::header::RETRY_AFTER)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.trim().parse::<u64>().ok());
    let json: Option<Value> = serde_json::from_slice(&body).ok();
    let message = json.as_ref().and_then(error_message).unwrap_or_else(|| {
        let text = String::from_utf8_lossy(&body);
        let text = text.trim();
        if text.is_empty() {
            "empty response body".to_owned()
        } else {
            text.chars().take(500).collect()
        }
    });
    Err(Error::Http(Box::new(HttpError {
        status,
        method: request.method.to_string(),
        url,
        message,
        body: json,
        retry_after,
    })))
}

/// Pulls a readable message out of the error formats Atlassian products use.
fn error_message(body: &Value) -> Option<String> {
    let mut parts: Vec<String> = Vec::new();
    // Jira: {"errorMessages": [...], "errors": {"field": "message"}}
    if let Some(messages) = body.get("errorMessages").and_then(Value::as_array) {
        parts.extend(messages.iter().filter_map(Value::as_str).map(str::to_owned));
    }
    if let Some(errors) = body.get("errors") {
        match errors {
            Value::Object(map) => {
                parts.extend(map.iter().map(|(k, v)| format!("{k}: {}", scalar(v))))
            }
            // Confluence v2 and Bitbucket DC: {"errors": [{"title"|"message": ...}]}
            Value::Array(items) => parts.extend(items.iter().filter_map(|item| {
                item.get("message")
                    .or_else(|| item.get("title"))
                    .or_else(|| item.get("detail"))
                    .and_then(Value::as_str)
                    .map(str::to_owned)
            })),
            _ => {}
        }
    }
    // Bitbucket Cloud: {"error": {"message": ...}}; Confluence v1 and others: {"message": ...}
    for pointer in ["/error/message", "/message", "/detail", "/error"] {
        if parts.is_empty()
            && let Some(text) = body.pointer(pointer).and_then(Value::as_str)
        {
            parts.push(text.to_owned());
        }
    }
    (!parts.is_empty()).then(|| parts.join("; "))
}

/// Checks a connection by calling the product's "current user" endpoint and
/// returns the account name the server reports.
pub async fn verify(family: Family, connection: &Connection) -> Result<String> {
    let connection = &connection.resolve_secret().map_err(Error::Invalid)?;
    let (path, pointers): (&str, &[&str]) = match (family, connection.deployment) {
        (Family::Jira, Deployment::Cloud) => {
            ("/rest/api/3/myself", &["/displayName", "/emailAddress"])
        }
        (Family::Jira, Deployment::DataCenter) => {
            ("/rest/api/2/myself", &["/displayName", "/name"])
        }
        (Family::Confluence, Deployment::Cloud) => (
            "/wiki/rest/api/user/current",
            &["/displayName", "/publicName"],
        ),
        (Family::Confluence, Deployment::DataCenter) => {
            ("/rest/api/user/current", &["/displayName", "/username"])
        }
        (Family::Bitbucket, Deployment::Cloud) => ("/2.0/user", &["/display_name", "/username"]),
        (Family::Bitbucket, Deployment::DataCenter) => {
            ("/rest/api/latest/profile/recent/repos?limit=1", &[])
        }
        (Family::Trello, _) => ("/1/members/me", &["/fullName", "/username"]),
        (Family::Admin, _) => ("/admin/v1/orgs", &["/data/0/attributes/name"]),
        (Family::AdminScim, _) => {
            return Ok(
                "not verified (SCIM keys are scoped to one directory; calls need --directory-id)"
                    .into(),
            );
        }
    };
    // Anonymous logins check a public endpoint instead of the current user.
    let anonymous = connection.auth == (Auth::None {});
    let path = match (anonymous, family, connection.deployment) {
        (false, ..) => path,
        (true, Family::Jira, Deployment::Cloud) => "/rest/api/3/serverInfo",
        (true, Family::Jira, Deployment::DataCenter) => "/rest/api/2/serverInfo",
        (true, Family::Confluence, Deployment::Cloud) => "/wiki/rest/api/space?limit=1",
        (true, Family::Confluence, Deployment::DataCenter) => "/rest/api/space?limit=1",
        (true, Family::Bitbucket, Deployment::DataCenter) => {
            "/rest/api/latest/application-properties"
        }
        (true, ..) => {
            return Err(Error::Invalid(format!(
                "anonymous access (--method none) works for Jira, Confluence and Bitbucket Data Center, not {family}"
            )));
        }
    };
    let url = Url::parse(&format!(
        "{}{path}",
        connection.base_url.trim_end_matches('/')
    ))
    .map_err(|err| {
        Error::Invalid(format!(
            "`{}` is not a valid URL ({err}); enter a URL such as https://acme.atlassian.net",
            connection.base_url
        ))
    })?;
    let request = PreparedRequest {
        headers: connection
            .extra_cookie
            .iter()
            .map(|cookie| ("Cookie".to_owned(), cookie.clone()))
            .collect(),
        ..PreparedRequest::new(HttpMethod::Get, url, Some(connection.auth.clone()))
    };
    let http = match connection.has_tls() {
        true => tls_client(family, connection).map_err(Error::Invalid)?,
        false => http_client(),
    };
    let response = send(&http, &request, 1).await?;
    let body = response.json().unwrap_or(Value::Null);
    if anonymous {
        return Ok("anonymous".into());
    }
    // Confluence answers anonymous callers with 200 and an anonymous user.
    if body.get("type").and_then(Value::as_str) == Some("anonymous") {
        return Err(Error::Http(Box::new(HttpError {
            status: 401,
            method: "GET".into(),
            url: request.url.to_string(),
            message:
                "the server treated the request as anonymous; check the email/username and token"
                    .into(),
            body: Some(body),
            retry_after: None,
        })));
    }
    let name = pointers
        .iter()
        .find_map(|pointer| body.pointer(pointer).and_then(Value::as_str))
        .map(str::to_owned)
        .or_else(|| {
            response
                .headers
                .get("x-ausername")
                .and_then(|value| value.to_str().ok())
                .map(str::to_owned)
        })
        .unwrap_or_else(|| "authenticated".to_owned());
    Ok(name)
}

/// Finds the Cloud ID of an Atlassian Cloud site, needed for scoped API
/// tokens and OAuth tokens, which go through `api.atlassian.com`.
pub async fn cloud_id(site: &str) -> Result<String> {
    let url = format!("{}/_edge/tenant_info", site.trim_end_matches('/'));
    let parsed = Url::parse(&url).map_err(|err| Error::Invalid(format!("`{site}` is not a valid site URL ({err}); enter a URL such as https://acme.atlassian.net")))?;
    let request = PreparedRequest::new(HttpMethod::Get, parsed, None);
    let response = send(&http_client(), &request, 1).await?;
    response
        .json()
        .and_then(|body| {
            body.get("cloudId")
                .and_then(Value::as_str)
                .map(str::to_owned)
        })
        .ok_or_else(|| {
            Error::Invalid(format!(
                "{url} returned no cloudId; check that `{site}` is an Atlassian Cloud site"
            ))
        })
}

/// The `api.atlassian.com` gateway base URL for a Cloud site.
pub fn gateway_url(family: Family, cloud_id: &str) -> String {
    let product = match family {
        Family::Confluence => "confluence",
        _ => "jira",
    };
    format!("{}/ex/{product}/{cloud_id}", crate::oauth::api_server())
}
