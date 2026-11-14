//! Profiles, credentials, and the config file.
//!
//! The config file is TOML at `$ATL_CONFIG_DIR/config.toml`, else
//! `$XDG_CONFIG_HOME/atl/config.toml`, else `~/.config/atl/config.toml`:
//!
//! ```toml
//! default_profile = "default"
//!
//! [profiles.default.jira]
//! deployment = "cloud"
//! base_url = "https://acme.atlassian.net"
//! account = "Ada Lovelace"
//! auth = { kind = "basic", username = "ada@acme.com", secret = "ATATT..." }
//! ```
//!
//! The file holds the secrets, so it is written with mode 0600 on Unix.
//!
//! Environment variables `ATL_<FAMILY>_URL`, `ATL_<FAMILY>_USER`,
//! `ATL_<FAMILY>_TOKEN` and `ATL_<FAMILY>_DEPLOYMENT` (family in upper snake
//! case, for example `ATL_ADMIN_SCIM_TOKEN`) override the file.

use crate::error::Error;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::str::FromStr;

/// A set of services that share one server and one credential.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Family {
    Jira,
    Confluence,
    Bitbucket,
    Trello,
    Admin,
    AdminScim,
}

impl Family {
    pub const ALL: [Family; 6] = [
        Family::Jira,
        Family::Confluence,
        Family::Bitbucket,
        Family::Trello,
        Family::Admin,
        Family::AdminScim,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Jira => "jira",
            Self::Confluence => "confluence",
            Self::Bitbucket => "bitbucket",
            Self::Trello => "trello",
            Self::Admin => "admin",
            Self::AdminScim => "admin-scim",
        }
    }

    /// Prefix of the environment variables that override this family.
    pub fn env_prefix(self) -> &'static str {
        match self {
            Self::Jira => "ATL_JIRA",
            Self::Confluence => "ATL_CONFLUENCE",
            Self::Bitbucket => "ATL_BITBUCKET",
            Self::Trello => "ATL_TRELLO",
            Self::Admin => "ATL_ADMIN",
            Self::AdminScim => "ATL_ADMIN_SCIM",
        }
    }

    /// The Cloud base URL when it does not depend on the customer's site.
    pub fn default_cloud_url(self) -> Option<&'static str> {
        match self {
            Self::Jira | Self::Confluence => None,
            Self::Bitbucket => Some("https://api.bitbucket.org"),
            Self::Trello => Some("https://api.trello.com"),
            Self::Admin | Self::AdminScim => Some("https://api.atlassian.com"),
        }
    }

    /// A clearly fake base URL for dry runs without credentials.
    pub(crate) fn placeholder_url(self, deployment: Deployment) -> &'static str {
        match (deployment, self.default_cloud_url()) {
            (Deployment::Cloud, Some(url)) => url,
            (Deployment::Cloud, None) => "https://your-domain.atlassian.net",
            (Deployment::DataCenter, _) => match self {
                Self::Confluence => "https://confluence.example.com",
                Self::Bitbucket => "https://bitbucket.example.com",
                _ => "https://jira.example.com",
            },
        }
    }
}

impl fmt::Display for Family {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for Family {
    type Err = Error;

    fn from_str(text: &str) -> Result<Self, Error> {
        Family::ALL
            .into_iter()
            .find(|family| family.as_str() == text)
            .ok_or_else(|| {
                Error::Invalid(format!(
                    "unknown product family {text:?}; use one of: {}",
                    Family::ALL.map(Family::as_str).join(", ")
                ))
            })
    }
}

/// Where the product runs.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Deployment {
    #[default]
    Cloud,
    /// Self-hosted Data Center or Server ("on-prem").
    DataCenter,
}

impl Deployment {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Cloud => "cloud",
            Self::DataCenter => "data-center",
        }
    }

    /// Hosts that belong to Atlassian are Cloud; anything else is assumed to
    /// be a self-hosted Data Center server.
    pub fn infer(base_url: &str) -> Deployment {
        let host = url::Url::parse(base_url)
            .ok()
            .and_then(|url| url.host_str().map(str::to_ascii_lowercase))
            .unwrap_or_default();
        let cloud = [
            "atlassian.net",
            "atlassian.com",
            "bitbucket.org",
            "trello.com",
            "jira.com",
        ]
        .iter()
        .any(|suffix| host == *suffix || host.ends_with(&format!(".{suffix}")));
        if cloud {
            Deployment::Cloud
        } else {
            Deployment::DataCenter
        }
    }
}

impl fmt::Display for Deployment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for Deployment {
    type Err = Error;

    fn from_str(text: &str) -> Result<Self, Error> {
        match text.to_ascii_lowercase().as_str() {
            "cloud" => Ok(Self::Cloud),
            "data-center" | "datacenter" | "dc" | "server" | "on-prem" | "onprem" => {
                Ok(Self::DataCenter)
            }
            _ => Err(Error::Invalid(format!(
                "unknown deployment {text:?}; use `cloud` or `data-center`"
            ))),
        }
    }
}

/// How requests authenticate.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum Auth {
    /// HTTP Basic: Cloud email plus API token, or Data Center username plus
    /// password.
    Basic { username: String, secret: String },
    /// HTTP Bearer: Data Center personal access token, Admin API key,
    /// Bitbucket access token, or an OAuth access token.
    Bearer { token: String },
    /// Trello API key plus user token.
    Trello { key: String, token: String },
    /// OAuth 2.0 tokens from a browser sign-in (`atl auth login --method
    /// oauth`), with what is needed to refresh them. Sent as Bearer.
    #[serde(rename = "oauth")]
    OAuth {
        access_token: String,
        #[serde(default)]
        refresh_token: String,
        /// Expiry of the access token, in seconds since 1970 (0 = unknown).
        #[serde(default)]
        expires_at: u64,
        client_id: String,
        #[serde(default)]
        client_secret: String,
        token_url: String,
    },
    /// A browser session's `Cookie` header value (Data Center behind SSO).
    Cookie { cookie: String },
    /// HTTP Basic from the user's `.netrc` entry for the server's host,
    /// read at each run (see [`netrc_login`]).
    Netrc {},
    /// No credential: anonymous access to public sites.
    None {},
}

impl Auth {
    /// The `Authorization` header with the secret replaced by `***`.
    pub(crate) fn masked_header(&self) -> &'static str {
        match self {
            Self::Basic { .. } | Self::Netrc {} => "Basic ***",
            Self::Bearer { .. } | Self::OAuth { .. } => "Bearer ***",
            Self::Trello { .. } => "OAuth ***",
            Self::Cookie { .. } | Self::None {} => "",
        }
    }

    /// True when the credential has its secret (netrc and anonymous
    /// credentials need none).
    pub fn has_secret(&self) -> bool {
        match self {
            Self::Basic { secret, .. } => !secret.is_empty(),
            Self::Bearer { token } | Self::Trello { token, .. } => !token.is_empty(),
            Self::OAuth {
                access_token,
                refresh_token,
                ..
            } => !access_token.is_empty() || !refresh_token.is_empty(),
            Self::Cookie { cookie } => !cookie.is_empty(),
            Self::Netrc {} | Self::None {} => true,
        }
    }

    /// A short description that reveals at most the last four secret characters.
    pub fn describe(&self) -> String {
        let tail = |secret: &str| {
            if secret.is_empty() {
                return "(empty)".to_owned();
            }
            let chars: Vec<char> = secret.chars().collect();
            let tail: String = chars[chars.len().saturating_sub(4)..].iter().collect();
            format!("…{tail}")
        };
        match self {
            Self::Basic { username, secret } => format!("basic {username} {}", tail(secret)),
            Self::Bearer { token } => format!("bearer {}", tail(token)),
            Self::Trello { key, token } => {
                format!("trello key {} token {}", tail(key), tail(token))
            }
            Self::OAuth {
                client_id,
                access_token,
                ..
            } => format!(
                "oauth client {} token {}",
                tail(client_id),
                tail(access_token)
            ),
            Self::Cookie { cookie } => format!("cookie {}", tail(cookie)),
            Self::Netrc {} => "netrc".into(),
            Self::None {} => "none (anonymous)".into(),
        }
    }
}

impl fmt::Debug for Auth {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.describe())
    }
}

/// One family's server and credential.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Connection {
    pub deployment: Deployment,
    pub base_url: String,
    pub auth: Auth,
    /// Display name the server returned at login, cached for `auth status`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub account: Option<String>,
    /// Browser URL of the site when `base_url` is the api.atlassian.com
    /// gateway (scoped tokens).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub site_url: Option<String>,
    /// Refuse every write through this connection.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub read_only: bool,
    /// PEM client certificate for mutual TLS (it may also hold the key).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub client_cert: Option<PathBuf>,
    /// PEM private key of the client certificate, when not in `client_cert`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub client_key: Option<PathBuf>,
    /// PEM CA certificates to trust in addition to the system roots.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ca_cert: Option<PathBuf>,
    /// A `Cookie` header added to every request (`ATL_<FAMILY>_COOKIE`), for
    /// gateways in front of a token login.
    #[serde(skip)]
    pub extra_cookie: Option<String>,
}

impl Connection {
    pub fn new(deployment: Deployment, base_url: impl Into<String>, auth: Auth) -> Self {
        Self {
            deployment,
            base_url: base_url.into(),
            auth,
            account: None,
            site_url: None,
            read_only: false,
            client_cert: None,
            client_key: None,
            ca_cert: None,
            extra_cookie: None,
        }
    }

    /// True when requests need their own TLS settings.
    pub(crate) fn has_tls(&self) -> bool {
        self.client_cert.is_some() || self.ca_cert.is_some()
    }

    /// A copy with a netrc credential read from the netrc file. A login
    /// without its secret is an error.
    pub fn resolve_secret(&self) -> Result<Connection, String> {
        if self.auth == (Auth::Netrc {}) {
            let host = url::Url::parse(&self.base_url)
                .ok()
                .and_then(|u| u.host_str().map(str::to_owned))
                .unwrap_or_default();
            let path = netrc_path();
            let (username, secret) = netrc_login(path.as_deref(), &host)?.ok_or_else(|| {
                format!(
                    "{} has no entry for host {host}; add `machine {host} login USER password TOKEN`",
                    path.as_deref().map_or("~/.netrc".into(), |p| p.display().to_string())
                )
            })?;
            return Ok(Connection {
                auth: Auth::Basic { username, secret },
                ..self.clone()
            });
        }
        if !self.auth.has_secret() {
            return Err("the config file has no secret for this login".into());
        }
        Ok(self.clone())
    }
}

/// `$NETRC`, else `~/.netrc` (`~/_netrc` on Windows).
pub(crate) fn netrc_path() -> Option<PathBuf> {
    if let Some(path) = std::env::var_os("NETRC").filter(|p| !p.is_empty()) {
        return Some(PathBuf::from(path));
    }
    #[cfg(feature = "native")]
    let home = dirs::home_dir();
    #[cfg(not(feature = "native"))]
    let home = std::env::var_os("HOME").map(PathBuf::from);
    let name = if cfg!(windows) { "_netrc" } else { ".netrc" };
    home.map(|home| home.join(name))
}

/// The login and password for `host` in a netrc file (`machine HOST login
/// USER password SECRET`, or the `default` entry). A missing file is `None`.
pub fn netrc_login(path: Option<&Path>, host: &str) -> Result<Option<(String, String)>, String> {
    let Some(path) = path else { return Ok(None) };
    let file = match std::fs::File::open(path) {
        Ok(file) => file,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(err) => return Err(format!("cannot read {} ({err})", path.display())),
    };
    let netrc = netrc::Netrc::parse(std::io::BufReader::new(file)).map_err(|err| {
        format!(
            "{} is not a valid netrc file ({err:?}); fix it, or set NETRC to another file",
            path.display()
        )
    })?;
    let machine = netrc
        .hosts
        .into_iter()
        .find(|(name, _)| name.eq_ignore_ascii_case(host))
        .map(|(_, machine)| machine)
        .or(netrc.default);
    Ok(machine.map(|m| (m.login, m.password.unwrap_or_default())))
}

/// One profile: a connection per product family, stored as
/// `[profiles.<name>.<family>]` tables.
pub type Profile = BTreeMap<Family, Connection>;

/// True for a flag value other than "", 0, false, no, off, n or f (any
/// case): the rule clap uses for boolean flags, for every `ATL_*` switch.
pub fn flag_value(value: &str) -> bool {
    !matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "" | "0" | "false" | "no" | "off" | "n" | "f"
    )
}

/// True when the environment variable `name` is set to an "on" value (see [`flag_value`]).
pub fn env_flag(name: &str) -> bool {
    std::env::var(name).is_ok_and(|value| flag_value(&value))
}

pub const DEFAULT_PROFILE: &str = "default";

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_profile: Option<String>,
    #[serde(default)]
    pub profiles: BTreeMap<String, Profile>,
    /// `atl ALIAS ...` expands to these words (see `atl alias`).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub aliases: BTreeMap<String, String>,
    /// The file this config was loaded from; clients save refreshed OAuth
    /// tokens there.
    #[serde(skip)]
    pub path: Option<PathBuf>,
}

impl Config {
    /// The config file path (see module docs).
    pub fn path() -> Result<PathBuf, Error> {
        if let Some(dir) = std::env::var_os("ATL_CONFIG_DIR").filter(|dir| !dir.is_empty()) {
            return Ok(PathBuf::from(dir).join("config.toml"));
        }
        if let Some(dir) = std::env::var_os("XDG_CONFIG_HOME").filter(|dir| !dir.is_empty()) {
            return Ok(PathBuf::from(dir).join("atl").join("config.toml"));
        }
        #[cfg(feature = "native")]
        let home = dirs::home_dir();
        #[cfg(not(feature = "native"))]
        let home = std::env::var_os("HOME").map(PathBuf::from);
        home
            .map(|home| home.join(".config").join("atl").join("config.toml"))
            .ok_or_else(|| {
                Error::Invalid(
                    "cannot find a home directory for the config file; set ATL_CONFIG_DIR to a writable directory".into(),
                )
            })
    }

    /// Loads the config file; a missing file is an empty config.
    pub fn load() -> Result<Config, Error> {
        Self::load_from(&Self::path()?)
    }

    pub fn load_from(path: &Path) -> Result<Config, Error> {
        match std::fs::read_to_string(path) {
            Ok(text) => {
                let mut config: Config = toml::from_str(&text).map_err(|err| Error::Config {
                    path: path.to_path_buf(),
                    message: err.to_string(),
                })?;
                config.path = Some(path.to_path_buf());
                Ok(config)
            }
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(Config {
                path: Some(path.to_path_buf()),
                ..Config::default()
            }),
            Err(err) => Err(Error::io(format!("cannot read {}", path.display()), err)),
        }
    }

    pub fn save(&self) -> Result<PathBuf, Error> {
        let path = Self::path()?;
        self.save_to(&path)?;
        Ok(path)
    }

    /// Writes atomically (temporary file plus rename) with mode 0600 on Unix,
    /// because the file holds secrets.
    pub fn save_to(&self, path: &Path) -> Result<(), Error> {
        let dir = path.parent().unwrap_or(Path::new("."));
        std::fs::create_dir_all(dir)
            .map_err(|err| Error::io(format!("cannot create {}", dir.display()), err))?;
        let text = toml::to_string_pretty(self)
            .map_err(|err| Error::Invalid(format!("cannot encode config: {err}")))?;
        let tmp = path.with_extension("toml.tmp");
        let mut options = std::fs::OpenOptions::new();
        options.write(true).create(true).truncate(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let mut file = options
            .open(&tmp)
            .map_err(|err| Error::io(format!("cannot write {}", tmp.display()), err))?;
        file.write_all(text.as_bytes())
            .and_then(|()| file.sync_all())
            .map_err(|err| Error::io(format!("cannot write {}", tmp.display()), err))?;
        std::fs::rename(&tmp, path)
            .map_err(|err| Error::io(format!("cannot replace {}", path.display()), err))
    }

    /// Replaces one stored connection's credential (for example refreshed
    /// OAuth tokens) in the config file at `path`; a missing connection is
    /// left alone.
    pub(crate) fn store_auth(
        path: &Path,
        profile: &str,
        family: Family,
        auth: &Auth,
    ) -> Result<(), Error> {
        let mut config = Config::load_from(path)?;
        let Some(connection) = config
            .profiles
            .get_mut(profile)
            .and_then(|stored| stored.get_mut(&family))
        else {
            return Ok(());
        };
        connection.auth = auth.clone();
        config.save_to(path)
    }

    /// The profile to use: the argument, else `ATL_PROFILE`, else
    /// `default_profile`, else `default`.
    pub fn profile_name(&self, requested: Option<&str>) -> String {
        requested
            .map(str::to_owned)
            .or_else(|| {
                std::env::var("ATL_PROFILE")
                    .ok()
                    .filter(|name| !name.is_empty())
            })
            .or_else(|| self.default_profile.clone())
            .unwrap_or_else(|| DEFAULT_PROFILE.to_owned())
    }

    /// The stored connection for `family`, with environment overrides
    /// applied. Returns `None` when neither the file nor the environment
    /// supplies a credential.
    pub fn connection(&self, profile: &str, family: Family) -> Option<Connection> {
        let stored = self
            .profiles
            .get(profile)
            .and_then(|p| p.get(&family))
            .cloned();
        let var = |suffix: &str| {
            std::env::var(format!("{}_{suffix}", family.env_prefix()))
                .ok()
                .filter(|value| !value.is_empty())
        };
        let (url, user, token, deployment) =
            (var("URL"), var("USER"), var("TOKEN"), var("DEPLOYMENT"));
        // A URL alone uses the netrc entry for its host, when there is one.
        // A broken netrc file still selects it, so the request reports why.
        let netrc = |url: &str| {
            let host = url::Url::parse(url).ok()?.host_str()?.to_owned();
            if let Ok(None) = netrc_login(netrc_path().as_deref(), &host) {
                return None;
            }
            Some(Connection::new(Deployment::infer(url), url, Auth::Netrc {}))
        };
        let mut connection = match (stored, token) {
            (stored, Some(token)) => {
                let auth = match (family, user) {
                    (Family::Trello, Some(key)) => Auth::Trello { key, token },
                    (_, Some(username)) => Auth::Basic {
                        username,
                        secret: token,
                    },
                    (_, None) => Auth::Bearer { token },
                };
                // A token without a URL never goes to a placeholder host: without
                // a stored login or a default (Trello, Admin) there is no connection.
                let base_url = url
                    .clone()
                    .or_else(|| stored.as_ref().map(|s| s.base_url.clone()))
                    .or_else(|| family.default_cloud_url().map(str::to_owned))?;
                Connection::new(Deployment::infer(&base_url), base_url, auth)
            }
            (Some(stored), None) => stored,
            (None, None) => netrc(url.as_deref()?)?,
        };
        if let Some(path) = var("CLIENT_CERT") {
            connection.client_cert = Some(path.into());
        }
        if let Some(path) = var("CLIENT_KEY") {
            connection.client_key = Some(path.into());
        }
        if let Some(path) = var("CA_CERT") {
            connection.ca_cert = Some(path.into());
        }
        connection.extra_cookie = var("COOKIE");
        if let Some(url) = url {
            connection.deployment = Deployment::infer(&url);
            connection.base_url = url;
        }
        if let Some(deployment) = deployment.and_then(|d| d.parse().ok()) {
            connection.deployment = deployment;
        }
        Some(connection)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flags_follow_clap_boolish_rule() {
        for on in ["1", "true", "YES", "on", "y", "anything"] {
            assert!(flag_value(on), "{on}");
        }
        for off in ["", " ", "0", "false", "No", "OFF", "n", "f"] {
            assert!(!flag_value(off), "{off:?}");
        }
    }

    #[test]
    fn netrc_entries_match_hosts_with_a_default_and_skip_macros() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("netrc");
        std::fs::write(
            &path,
            "machine other.example login x password y\n\
             macdef init\nmachine jira.corp.example login evil password evil\n\n\
             machine JIRA.corp.example\n  login ada\n  account a\n  password s3cret\n\
             default login anon password guest\n",
        )
        .expect("netrc");
        let login = |host: &str| netrc_login(Some(&path), host).expect("read");
        assert_eq!(
            login("jira.corp.example"),
            Some(("ada".into(), "s3cret".into()))
        );
        assert_eq!(login("other.example"), Some(("x".into(), "y".into())));
        assert_eq!(
            login("unknown.example"),
            Some(("anon".into(), "guest".into()))
        );
        assert_eq!(
            netrc_login(Some(&dir.path().join("missing")), "x").expect("missing"),
            None
        );
    }

    #[test]
    fn new_credentials_and_tls_paths_round_trip_through_toml() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("config.toml");
        let mut profile = Profile::default();
        profile.insert(
            Family::Jira,
            Connection {
                client_cert: Some("/certs/me.pem".into()),
                ca_cert: Some("/certs/ca.pem".into()),
                ..Connection::new(
                    Deployment::DataCenter,
                    "https://jira.corp",
                    Auth::Cookie {
                        cookie: "JSESSIONID=1".into(),
                    },
                )
            },
        );
        profile.insert(
            Family::Confluence,
            Connection::new(Deployment::DataCenter, "https://wiki.corp", Auth::Netrc {}),
        );
        profile.insert(
            Family::Bitbucket,
            Connection::new(Deployment::DataCenter, "https://git.corp", Auth::None {}),
        );
        let mut config = Config::default();
        config.profiles.insert("default".into(), profile);
        config.save_to(&path).expect("save");
        config.path = Some(path.clone());
        assert_eq!(Config::load_from(&path).expect("load"), config);
        let text = std::fs::read_to_string(&path).expect("read");
        assert!(
            text.contains("kind = \"netrc\"") && text.contains("client_cert = \"/certs/me.pem\""),
            "{text}"
        );
    }

    #[test]
    fn config_round_trips_through_toml_with_private_permissions() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("nested").join("config.toml");
        let mut config = Config {
            default_profile: Some("work".into()),
            ..Config::default()
        };
        let mut profile = Profile::default();
        profile.insert(
            Family::AdminScim,
            Connection {
                read_only: true,
                ..Connection::new(
                    Deployment::Cloud,
                    "https://api.atlassian.com",
                    Auth::Bearer {
                        token: "secret-token".into(),
                    },
                )
            },
        );
        config.profiles.insert("work".into(), profile);
        config.save_to(&path).expect("save");

        config.path = Some(path.clone());
        assert_eq!(Config::load_from(&path).expect("load"), config);
        let text = std::fs::read_to_string(&path).expect("read");
        assert!(text.contains("[profiles.work.admin-scim]"), "{text}");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(&path)
                .expect("metadata")
                .permissions()
                .mode();
            assert_eq!(mode & 0o777, 0o600);
        }
    }

    #[test]
    fn a_login_without_its_secret_asks_for_a_new_login() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("config.toml");
        std::fs::write(
            &path,
            "[profiles.work.jira]\ndeployment = \"cloud\"\nbase_url = \"https://acme.atlassian.net\"\nauth = { kind = \"basic\", username = \"ada@acme.com\", secret = \"\" }\n",
        )
        .expect("write");
        let loaded = Config::load_from(&path).expect("load");
        let jira = &loaded.profiles["work"][&Family::Jira];
        assert_eq!(jira.auth.describe(), "basic ada@acme.com (empty)");
        let client = crate::Client::new(&loaded, Some("work"));
        let request = client
            .prepare_raw(
                Family::Jira,
                crate::HttpMethod::Get,
                "/rest/api/3/myself",
                &serde_json::Map::new(),
                None,
            )
            .expect("prepare");
        let message = request.missing_credentials.expect("missing");
        assert!(message.contains("run `atl auth login`"), "{message}");
    }

    #[test]
    fn deployment_is_inferred_from_the_host() {
        assert_eq!(
            Deployment::infer("https://acme.atlassian.net"),
            Deployment::Cloud
        );
        assert_eq!(
            Deployment::infer("https://api.bitbucket.org"),
            Deployment::Cloud
        );
        assert_eq!(
            Deployment::infer("https://jira.corp.example:8443/jira"),
            Deployment::DataCenter
        );
    }

    #[test]
    fn debug_output_never_contains_the_secret() {
        let auth = Auth::Basic {
            username: "ada".into(),
            secret: "supersecretvalue".into(),
        };
        let text = format!("{auth:?}");
        assert!(!text.contains("supersecret"), "{text}");
        assert!(text.ends_with("…alue"), "{text}");
    }
}
