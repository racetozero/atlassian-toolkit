//! Rust SDK for the Atlassian suite.
//!
//! Every public REST operation of Jira (platform, Software, Service
//! Management), Confluence (v2 and v1), Bitbucket, Trello and the Atlassian
//! Admin APIs is available on Atlassian Cloud and, where Atlassian publishes
//! an OpenAPI document, on Data Center. Operations come from Atlassian's
//! OpenAPI documents, normalized at build time into [`catalog`]s.
//!
//! ```no_run
//! # async fn demo() -> atl_sdk::Result<()> {
//! use atl_sdk::{CallArgs, Client};
//!
//! // Credentials come from `atl auth login` (or ATL_JIRA_URL / ATL_JIRA_TOKEN).
//! let client = Client::from_env(None)?;
//! let issue = client
//!     .call("jira", "getIssue", &CallArgs::default().param("issueIdOrKey", "PROJ-1"))
//!     .await?;
//! println!("{}", issue.json().unwrap_or_default());
//! # Ok(())
//! # }
//! ```

pub mod admin;
pub mod bitbucket;
pub mod catalog;
pub mod client;
pub mod config;
pub mod confluence;
pub mod docs;
pub mod error;
pub mod jira;
pub mod jsm;
mod json;
pub mod markup;
pub mod oauth;
pub mod paginate;
mod ql;
mod rt;
pub mod schema;
pub mod tasks;
pub mod time;
pub mod trello;

pub use catalog::{Catalog, HttpMethod, Operation, Param, ParamLocation, ServiceDef};
pub use client::{
    CallArgs, Client, DebugLevel, PreparedBody, PreparedRequest, Response, cloud_id, gateway_url,
    set_debug, verify,
};
pub use config::{Auth, Config, Connection, Deployment, Family, Profile};
pub use error::{Error, HttpError, Result};
pub use paginate::Pager;
