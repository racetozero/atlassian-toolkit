//! `atl admin`: one Atlassian organization's directories, managed users,
//! groups, domains and audit log (organization API key).

use crate::Exit;
use crate::output::{col, flex};
use crate::tasks::{Ctx, confirm};
use anyhow::Result;
use clap::{Args, Subcommand};
use serde_json::json;

#[derive(Args, Debug)]
pub struct AdminArgs {
    /// Organization ID or name. Default: [admin] org in .atl.toml, then the
    /// only organization the API key can see.
    #[arg(long, global = true)]
    org: Option<String>,
    #[command(subcommand)]
    command: AdminCommand,
}

#[derive(Subcommand, Debug)]
enum AdminCommand {
    /// Organizations the API key can see.
    #[command(subcommand)]
    Org(ListOnly),
    /// User directories.
    #[command(subcommand)]
    Directory(ListOnly),
    /// Managed users: list, suspend, restore.
    #[command(subcommand)]
    User(UserCommand),
    /// Groups: list, add-member, remove-member.
    #[command(subcommand)]
    Group(GroupCommand),
    /// Domains with their verification status.
    #[command(subcommand)]
    Domain(ListOnly),
    /// Audit log events, newest first.
    #[command(subcommand)]
    Event(EventCommand),
}

#[derive(Subcommand, Debug)]
enum ListOnly {
    /// List them.
    #[command(visible_alias = "ls")]
    List,
}

#[derive(Subcommand, Debug)]
enum SearchOnly {
    /// List them, optionally matching --search.
    #[command(visible_alias = "ls")]
    List {
        /// Only results that contain this text.
        #[arg(long, short = 's')]
        search: Option<String>,
        /// Maximum number of results.
        #[arg(long, default_value_t = 50)]
        limit: usize,
    },
}

#[derive(Subcommand, Debug)]
enum GroupCommand {
    /// List groups, optionally matching --search.
    #[command(visible_alias = "ls")]
    List {
        /// Only results that contain this text.
        #[arg(long, short = 's')]
        search: Option<String>,
        /// Maximum number of results.
        #[arg(long, default_value_t = 50)]
        limit: usize,
    },
    /// Add a managed user (email or account ID) to a group (name or ID).
    AddMember {
        /// Group name or ID.
        group: String,
        /// Email or account ID.
        user: String,
    },
    /// Remove a managed user from a group (asks first; --yes skips the question).
    RemoveMember {
        /// Group name or ID.
        group: String,
        /// Email or account ID.
        user: String,
    },
}

#[derive(Subcommand, Debug)]
enum UserCommand {
    /// Managed users of every directory, optionally matching --search (name or email).
    #[command(visible_alias = "ls")]
    List {
        /// Only results that contain this text.
        #[arg(long, short = 's')]
        search: Option<String>,
        /// Maximum number of results.
        #[arg(long, default_value_t = 50)]
        limit: usize,
    },
    /// Suspend a user's access to every product (asks first; --yes skips).
    Suspend {
        /// Account ID or email.
        user: String,
        /// Directory ID (default: the directory where the user is found).
        #[arg(long)]
        directory: Option<String>,
    },
    /// Restore a suspended user's access.
    Restore {
        /// Account ID or email.
        user: String,
        /// Directory ID (default: every directory).
        #[arg(long)]
        directory: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
enum EventCommand {
    /// Audit log events filtered by words, dates and action.
    #[command(visible_alias = "ls")]
    List {
        /// Only results that contain this text.
        #[arg(long, short = 's')]
        search: Option<String>,
        /// Start date or time, for example 2026-11-01.
        #[arg(long)]
        from: Option<String>,
        /// End date or time.
        #[arg(long)]
        to: Option<String>,
        /// Event action, for example user_added_to_group.
        #[arg(long)]
        action: Option<String>,
        /// Maximum number of results.
        #[arg(long, default_value_t = 50)]
        limit: usize,
    },
}

const USER_COLUMNS: [crate::output::Col; 5] = [
    col("ACCOUNT", "account_id"),
    col("STATUS", "status"),
    col("EMAIL", "email"),
    col("DIRECTORY", "directory"),
    flex("NAME", "name"),
];

pub async fn run(ctx: &Ctx<'_>, args: AdminArgs) -> Result<Exit> {
    let org = ctx.repo.org(args.org);
    match args.command {
        AdminCommand::Org(ListOnly::List) => {
            ctx.list(
                "admin_org_list",
                json!({}),
                &[col("ID", "id"), flex("NAME", "name")],
            )
            .await
        }
        AdminCommand::Directory(ListOnly::List) => {
            ctx.list(
                "admin_directory_list",
                json!({ "org": org }),
                &[col("ID", "id"), flex("NAME", "name")],
            )
            .await
        }
        AdminCommand::User(UserCommand::List { search, limit }) => {
            ctx.list(
                "admin_user_list",
                json!({ "org": org, "search": search, "limit": limit }),
                &USER_COLUMNS,
            )
            .await
        }
        AdminCommand::User(UserCommand::Suspend { user, directory }) => {
            confirm(ctx.yes,
                &format!("suspend {user}'s access to every product"),
            )?;
            ctx.item(
                "admin_user_suspend",
                json!({ "org": org, "user": user, "directory": directory }),
                |_| format!("Suspended {user}"),
            )
            .await
        }
        AdminCommand::User(UserCommand::Restore { user, directory }) => {
            ctx.item(
                "admin_user_restore",
                json!({ "org": org, "user": user, "directory": directory }),
                |_| format!("Restored {user}"),
            )
            .await
        }
        AdminCommand::Group(GroupCommand::AddMember { group, user }) => {
            ctx.item(
                "admin_group_add_member",
                json!({ "org": org, "group": group, "user": user }),
                |m| format!("Added {} to {}", crate::tasks::s(m, "user"), crate::tasks::s(m, "group")),
            )
            .await
        }
        AdminCommand::Group(GroupCommand::RemoveMember { group, user }) => {
            confirm(ctx.yes, &format!("remove {user} from group {group}"))?;
            ctx.item(
                "admin_group_remove_member",
                json!({ "org": org, "group": group, "user": user }),
                |m| format!("Removed {} from {}", crate::tasks::s(m, "user"), crate::tasks::s(m, "group")),
            )
            .await
        }
        AdminCommand::Group(GroupCommand::List { search, limit }) => {
            ctx.list(
                "admin_group_list",
                json!({ "org": org, "search": search, "limit": limit }),
                &[
                    col("ID", "id"),
                    col("USERS", "users"),
                    col("DIRECTORY", "directory"),
                    flex("NAME", "name"),
                ],
            )
            .await
        }
        AdminCommand::Domain(ListOnly::List) => {
            ctx.list(
                "admin_domain_list",
                json!({ "org": org }),
                &[col("STATUS", "status"), flex("DOMAIN", "name")],
            )
            .await
        }
        AdminCommand::Event(EventCommand::List {
            search,
            from,
            to,
            action,
            limit,
        }) => {
            ctx.list(
                "admin_event_list",
                json!({ "org": org, "search": search, "from": from, "to": to, "action": action, "limit": limit }),
                &[
                    col("TIME", "time"),
                    col("ACTION", "action"),
                    col("ACTOR", "actor"),
                    flex("MESSAGE", "message"),
                ],
            )
            .await
        }
    }
}
