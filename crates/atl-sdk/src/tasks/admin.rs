//! Atlassian Admin tasks.

use super::*;
use crate::admin::{Admin, EventQuery};
use crate::client::Client;
use crate::error::Result;
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct GroupMemberArgs {
    /// Organization ID or name (default: the only one).
    org: Option<String>,
    /// Group ID or name.
    group: String,
    /// Email or account ID.
    user: String,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct OrgArgs {
    /// Organization ID or name (default: the only one the API key sees).
    org: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct AdminSearchArgs {
    org: Option<String>,
    /// Words to match in names and emails.
    search: Option<String>,
    #[serde(default = "default_limit")]
    limit: usize,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct AdminUserArgs {
    org: Option<String>,
    /// Account ID or email.
    user: String,
    /// Directory ID (default: the directory where the user is found).
    directory: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct AdminEventArgs {
    org: Option<String>,
    /// Words to find in the events.
    search: Option<String>,
    /// Start date or time, for example 2026-11-01.
    from: Option<String>,
    /// End date or time.
    to: Option<String>,
    /// Event action, for example user_added_to_group.
    action: Option<String>,
    #[serde(default = "default_limit")]
    limit: usize,
}

pub(super) static TASKS: &[TaskDef] = &[
    task!(
        "admin_org_list",
        true,
        Empty => Vec<crate::admin::Org>,
        "Organizations the Atlassian Admin API key can see."
    ),
    task!(
        "admin_directory_list",
        true,
        OrgArgs => Vec<crate::admin::Directory>,
        "User directories of an Atlassian organization."
    ),
    task!(
        "admin_user_list",
        true,
        AdminSearchArgs => Vec<crate::admin::AdminUser>,
        "Managed users of an Atlassian organization (every directory), optionally matching `search`."
    ),
    task!(
        "admin_user_suspend",
        false,
        AdminUserArgs => crate::admin::AdminUser,
        "Suspend a managed user's access to every product (by account ID or email).",
        destructive
    ),
    task!(
        "admin_user_restore",
        false,
        AdminUserArgs => crate::admin::AdminUser,
        "Restore a suspended managed user's access (by account ID or email)."
    ),
    task!(
        "admin_group_add_member",
        false,
        GroupMemberArgs => Membership,
        "Add a managed user (email or account ID) to a group (name or ID) of an Atlassian organization."
    ),
    task!(
        "admin_group_remove_member",
        false,
        GroupMemberArgs => Membership,
        "Remove a managed user from a group of an Atlassian organization.",
        destructive
    ),
    task!(
        "admin_group_list",
        true,
        AdminSearchArgs => Vec<crate::admin::Group>,
        "Groups of an Atlassian organization (every directory), optionally matching `search`."
    ),
    task!(
        "admin_domain_list",
        true,
        OrgArgs => Vec<crate::admin::Domain>,
        "Domains of an Atlassian organization with their verification status."
    ),
    task!(
        "admin_event_list",
        true,
        AdminEventArgs => Vec<crate::admin::Event>,
        "Audit log events of an Atlassian organization, newest first, filtered by words, dates and action."
    ),
];

pub(super) async fn run(client: &Client, name: &str, args: Value) -> Result<Value> {
    let admin = Admin::new(client);
    match name {
        "admin_group_add_member" => {
            output(name, membership(&admin, parse(name, args)?, true).await?)
        }
        "admin_group_remove_member" => {
            output(name, membership(&admin, parse(name, args)?, false).await?)
        }
        "admin_org_list" => {
            let _: Empty = parse(name, args)?;
            output(name, admin.orgs().await?)
        }
        "admin_directory_list" => {
            let a: OrgArgs = parse(name, args)?;
            output(name, admin.directories(a.org.as_deref()).await?)
        }
        "admin_user_list" => {
            let a: AdminSearchArgs = parse(name, args)?;
            output(
                name,
                admin
                    .users(a.org.as_deref(), a.search.as_deref(), a.limit)
                    .await?,
            )
        }
        "admin_user_suspend" => {
            let a: AdminUserArgs = parse(name, args)?;
            output(
                name,
                admin
                    .suspend(a.org.as_deref(), &a.user, a.directory.as_deref())
                    .await?,
            )
        }
        "admin_user_restore" => {
            let a: AdminUserArgs = parse(name, args)?;
            output(
                name,
                admin
                    .restore(a.org.as_deref(), &a.user, a.directory.as_deref())
                    .await?,
            )
        }
        "admin_group_list" => {
            let a: AdminSearchArgs = parse(name, args)?;
            output(
                name,
                admin
                    .groups(a.org.as_deref(), a.search.as_deref(), a.limit)
                    .await?,
            )
        }
        "admin_domain_list" => {
            let a: OrgArgs = parse(name, args)?;
            output(name, admin.domains(a.org.as_deref()).await?)
        }
        "admin_event_list" => {
            let a: AdminEventArgs = parse(name, args)?;
            let query = EventQuery {
                search: a.search,
                from: a.from,
                to: a.to,
                action: a.action,
                limit: a.limit,
            };
            output(name, admin.events(a.org.as_deref(), &query).await?)
        }
        _ => Err(unknown(name)),
    }
}

async fn membership(admin: &Admin<'_>, a: GroupMemberArgs, add: bool) -> Result<Membership> {
    let (group, user) = admin
        .group_membership(a.org.as_deref(), &a.group, &a.user, add)
        .await?;
    Ok(Membership {
        group: group.name,
        group_id: group.id,
        user: user.email.or(user.name),
        account_id: user.account_id,
        member: add,
    })
}

/// A user's membership of a group after an add or a remove.
#[derive(Serialize, JsonSchema)]
struct Membership {
    group: String,
    group_id: String,
    /// The user's email, or their name when the email is hidden.
    user: Option<String>,
    account_id: String,
    /// True after an add, false after a remove.
    member: bool,
}
