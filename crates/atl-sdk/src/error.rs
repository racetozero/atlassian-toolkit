use serde_json::Value;
use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The caller passed a value the operation cannot accept.
    #[error("{0}")]
    Invalid(String),
    /// The service, resource, or operation does not exist.
    #[error("{0}")]
    Unknown(String),
    /// The connection is read-only and the request writes.
    #[error("{0}")]
    ReadOnly(String),
    /// `--dry-run` stopped a write. Holds the request that was not sent.
    #[error("dry run: {} {} was not sent", .0["method"].as_str().unwrap_or(""), .0["url"].as_str().unwrap_or(""))]
    DryRun(Box<Value>),
    /// No credential is configured for the product.
    #[error("{0}")]
    MissingCredentials(String),
    /// A value is missing or matches several things; `choices` lists the
    /// valid values. The CLI asks for one in a terminal; agents retry with one.
    #[error("{}", .0.text())]
    NeedsInput(Box<NeedsInput>),
    /// The server answered with a non-success status. Boxed because it is
    /// the largest variant and only exists on the failure path.
    #[error("{} {} returned HTTP {}: {}", .0.method, .0.url, .0.status, .0.message)]
    Http(Box<HttpError>),
    #[error("request to {url} failed: {source}")]
    Transport {
        url: String,
        #[source]
        source: reqwest::Error,
    },
    #[error(
        "config file {path} is invalid: {message}. Fix the file or delete it and run `atl auth login`"
    )]
    Config { path: PathBuf, message: String },
    #[error("{context}: {source}")]
    Io {
        context: String,
        #[source]
        source: std::io::Error,
    },
}

/// Every value of [`Error::code`].
pub const CODES: &[&str] = &[
    "auth",
    "api",
    "invalid",
    "unknown",
    "read_only",
    "dry_run",
    "input_required",
    "internal",
];

impl Error {
    pub(crate) fn io(context: String, source: std::io::Error) -> Self {
        Error::Io { context, source }
    }

    /// A stable machine-readable code, one of [`CODES`]. The CLI, MCP server
    /// and language SDKs use it; the SDKs' error types are generated from
    /// [`CODES`].
    pub fn code(&self) -> &'static str {
        match self {
            _ if self.is_auth() => "auth",
            Error::Http(_) => "api",
            Error::Invalid(_) | Error::Config { .. } => "invalid",
            Error::Unknown(_) => "unknown",
            Error::ReadOnly(_) => "read_only",
            Error::DryRun(_) => "dry_run",
            Error::NeedsInput(_) => "input_required",
            _ => "internal",
        }
    }

    /// `{"code", "message", "status"?, "details"?}`.
    pub fn to_json(&self) -> Value {
        let mut out = serde_json::json!({ "code": self.code(), "message": self.to_string() });
        if let Error::Http(http) = self {
            out["status"] = http.status.into();
            if let Some(body) = &http.body {
                out["details"] = body.clone();
            }
        }
        if let Error::NeedsInput(need) = self {
            out["input"] = serde_json::json!({ "arg": need.arg, "choices": need.choices });
        }
        out
    }

    pub fn needs_input(arg: &str, message: String, choices: Vec<Choice>) -> Error {
        Error::NeedsInput(Box::new(NeedsInput {
            arg: arg.to_owned(),
            message,
            choices,
        }))
    }

    /// Renames the argument of a [`Error::NeedsInput`] (for tasks whose
    /// argument has another name than the SDK method's); other errors pass
    /// through.
    pub fn for_arg(self, arg: &str) -> Error {
        match self {
            Error::NeedsInput(mut need) => {
                need.arg = arg.to_owned();
                Error::NeedsInput(need)
            }
            other => other,
        }
    }

    /// True when the server rejected the credential (HTTP 401 or 403) or none
    /// was configured.
    pub fn is_auth(&self) -> bool {
        matches!(self, Error::MissingCredentials(_))
            || matches!(self, Error::Http(http) if matches!(http.status, 401 | 403))
    }
}

/// One valid value for a [`NeedsInput`] argument.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct Choice {
    /// What to pass back as the argument.
    pub value: String,
    /// What a person reads.
    pub label: String,
}

impl Choice {
    pub fn new(value: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
        }
    }
}

#[derive(Debug)]
pub struct NeedsInput {
    /// The task argument to fill in, for example `status`.
    pub arg: String,
    pub message: String,
    pub choices: Vec<Choice>,
}

impl NeedsInput {
    /// The message and up to 20 choice values, for plain-text readers.
    pub fn text(&self) -> String {
        if self.choices.is_empty() {
            return self.message.clone();
        }
        // The label when it starts with the value ("Done (Finish)"), else both.
        let mut values: Vec<String> = self
            .choices
            .iter()
            .take(20)
            .map(|c| match c.label.starts_with(&c.value) {
                true => c.label.clone(),
                false => format!("{} ({})", c.value, c.label),
            })
            .collect();
        let more = self.choices.len().saturating_sub(20);
        let more = if more > 0 {
            format!(" and {more} more")
        } else {
            String::new()
        };
        values.dedup();
        format!("{}; choices: {}{more}", self.message, values.join(", "))
    }
}

#[derive(Debug)]
pub struct HttpError {
    pub status: u16,
    pub method: String,
    pub url: String,
    pub message: String,
    /// The parsed JSON error body, when the server sent JSON.
    pub body: Option<Value>,
    /// Seconds from the `Retry-After` header.
    pub retry_after: Option<u64>,
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_code_is_listed() {
        let errors = [
            Error::MissingCredentials(String::new()),
            Error::Invalid(String::new()),
            Error::Unknown(String::new()),
            Error::ReadOnly(String::new()),
            Error::needs_input("x", String::new(), Vec::new()),
            Error::io(String::new(), std::io::Error::other("x")),
        ];
        for err in errors {
            assert!(CODES.contains(&err.code()), "{}", err.code());
        }
    }
}
