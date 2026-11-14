//! `atl trello`: boards, lists and cards. Boards and lists can be named;
//! cards are a URL, short link or ID.

use crate::Exit;
use crate::output::{Col, bold, col, dim, flex};
use crate::tasks::{Ctx, key_and_text, read_text, s};
use anyhow::Result;
use atl_sdk::Error;
use clap::Subcommand;
use serde_json::json;
use std::path::PathBuf;

#[derive(Subcommand, Debug)]
pub enum TrelloCommand {
    /// Your open boards.
    #[command(subcommand)]
    Board(BoardCommand),
    /// Lists of a board.
    #[command(subcommand)]
    List(ListCommand),
    /// Cards: list, view, create, edit, move, comment, archive.
    #[command(subcommand)]
    Card(CardCommand),
    /// Checklists of a card: list, create, check, uncheck.
    #[command(subcommand)]
    Checklist(ChecklistCommand),
}

#[derive(Subcommand, Debug)]
pub enum ChecklistCommand {
    /// Checklists of a card with their items.
    #[command(visible_alias = "ls")]
    List {
        /// Card URL, short link or ID.
        card: String,
    },
    /// Add a checklist: `atl trello checklist create CARD "Release" --item "Tag" --item "Notes"`.
    Create {
        /// Card URL, short link or ID.
        card: String,
        /// Checklist name.
        name: String,
        /// Item text; repeatable.
        #[arg(long)]
        item: Vec<String>,
    },
    /// Tick an item (name or ID).
    Check {
        /// Card URL, short link or ID.
        card: String,
        /// Checklist item name or ID.
        item: String,
    },
    /// Untick an item (name or ID).
    Uncheck {
        /// Card URL, short link or ID.
        card: String,
        /// Checklist item name or ID.
        item: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum BoardCommand {
    /// List your open boards.
    #[command(visible_alias = "ls")]
    List,
}

#[derive(Subcommand, Debug)]
pub enum ListCommand {
    /// Open lists of a board.
    #[command(visible_alias = "ls")]
    List {
        /// Board name, URL, short link or ID (default: [trello] board in .atl.toml).
        #[arg(long, short = 'b')]
        board: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
pub enum CardCommand {
    /// Cards of a board or one list, or your cards with --mine.
    #[command(visible_alias = "ls")]
    List {
        /// Board name, URL, short link or ID (default: `[trello] board` in .atl.toml).
        #[arg(long, short = 'b')]
        board: Option<String>,
        /// List name or ID.
        #[arg(long, short = 'l')]
        list: Option<String>,
        /// Your cards on every board.
        #[arg(long)]
        mine: bool,
    },
    /// Show a card with its description and latest comments.
    View {
        /// Card URL, short link or ID.
        card: String,
    },
    /// Create a card in a list.
    Create {
        /// Board name, URL, short link or ID (default: `[trello] board` in .atl.toml).
        #[arg(long, short = 'b')]
        board: Option<String>,
        /// List name (with a board) or list ID.
        #[arg(long, short = 'l')]
        list: String,
        /// Card name.
        #[arg(long, short = 'n')]
        name: String,
        /// Description (Markdown).
        #[arg(long, short = 'd')]
        desc: Option<String>,
        /// Read the description from a file, or - for stdin.
        #[arg(long)]
        desc_file: Option<PathBuf>,
        /// Due date, for example 2026-12-01.
        #[arg(long)]
        due: Option<String>,
    },
    /// Move a card to another list (on its own board unless --board is given).
    Move {
        /// Card URL, short link or ID.
        card: String,
        /// List name or ID to move the card to.
        #[arg(long, short = 'l')]
        list: String,
        /// Board name, URL, short link or ID (default: `[trello] board` in .atl.toml).
        #[arg(long, short = 'b')]
        board: Option<String>,
    },
    /// Comment: `atl trello card comment CARD "text"` or --body-file.
    Comment {
        /// `CARD [TEXT]`.
        #[arg(required = true, num_args = 1..=2, value_names = ["CARD", "TEXT"])]
        args: Vec<String>,
        /// Read the text from a file (`-` for stdin).
        #[arg(long)]
        body_file: Option<PathBuf>,
    },
    /// Archive (close) a card.
    Archive {
        /// Card URL, short link or ID.
        card: String,
    },
    /// Change a card: name, description, due date, done, labels, members.
    Edit {
        /// Card URL, short link or ID.
        card: String,
        /// New card name.
        #[arg(long, short = 'n')]
        name: Option<String>,
        /// Description (Markdown).
        #[arg(long, short = 'd')]
        desc: Option<String>,
        /// Read the description from a file, or - for stdin.
        #[arg(long)]
        desc_file: Option<PathBuf>,
        /// Due date, for example 2026-12-01 or 2026-12-01T17:00:00Z.
        #[arg(long, conflicts_with = "no_due")]
        due: Option<String>,
        /// Remove the due date.
        #[arg(long)]
        no_due: bool,
        /// Mark the due date complete.
        #[arg(long, conflicts_with = "undone")]
        done: bool,
        /// Mark the due date not complete.
        #[arg(long)]
        undone: bool,
        /// Label name or color on the card's board; repeatable.
        #[arg(long)]
        add_label: Vec<String>,
        /// Label to remove (name or color).
        #[arg(long)]
        remove_label: Vec<String>,
        /// Member username, full name, or me; repeatable.
        #[arg(long)]
        add_member: Vec<String>,
        /// Member to remove (username, full name, or `me`).
        #[arg(long)]
        remove_member: Vec<String>,
    },
}

const CARD_COLUMNS: [Col; 4] = [
    col("LINK", "short_link"),
    col("LIST", "list"),
    col("DUE", "due"),
    flex("NAME", "name"),
];

pub async fn run(ctx: &Ctx<'_>, command: TrelloCommand) -> Result<Exit> {
    match command {
        TrelloCommand::Board(BoardCommand::List) => {
            ctx.list(
                "trello_board_list",
                json!({}),
                &[col("LINK", "short_link"), flex("NAME", "name")],
            )
            .await
        }
        TrelloCommand::List(ListCommand::List { board }) => {
            let board = ctx.repo.board(board).ok_or_else(|| {
                Error::Invalid("pass --board NAME (or set [trello] board in .atl.toml)".into())
            })?;
            ctx.list(
                "trello_list_list",
                json!({ "board": board }),
                &[col("ID", "id"), flex("NAME", "name")],
            )
            .await
        }
        TrelloCommand::Card(CardCommand::List { board, list, mine }) => {
            ctx.list(
                "trello_card_list",
                json!({ "board": ctx.repo.board(board), "list": list, "mine": mine }),
                &CARD_COLUMNS,
            )
            .await
        }
        TrelloCommand::Card(CardCommand::View { card }) => {
            let tty = ctx.out.tty;
            ctx.item("trello_card_view", json!({ "card": card }), |c| {
                let mut out = format!(
                    "{}  {}\n{}\n{}\n\n{}\n",
                    bold(s(c, "name"), tty),
                    s(c, "list"),
                    dim(
                        &format!(
                            "{} · due {}",
                            s(c, "board"),
                            c["due"].as_str().unwrap_or("-")
                        ),
                        tty
                    ),
                    dim(s(c, "url"), tty),
                    match s(c, "description") {
                        "" => "(no description)",
                        text => text,
                    }
                );
                for comment in c["comments"].as_array().into_iter().flatten() {
                    out.push_str(&format!(
                        "\n{} · {}\n{}\n",
                        bold(comment["author"].as_str().unwrap_or("unknown"), tty),
                        s(comment, "date").chars().take(10).collect::<String>(),
                        s(comment, "text")
                    ));
                }
                out
            })
            .await
        }
        TrelloCommand::Card(CardCommand::Create {
            board,
            list,
            name,
            desc,
            desc_file,
            due,
        }) => {
            let description = read_text(desc, desc_file.as_deref())?;
            ctx.item(
                "trello_card_create",
                json!({
                    "board": ctx.repo.board(board),
                    "list": list,
                    "name": name,
                    "description": description,
                    "due": due,
                }),
                |c| format!("Created {} {}", s(c, "name"), s(c, "url")),
            )
            .await
        }
        TrelloCommand::Card(CardCommand::Move { card, list, board }) => {
            ctx.item(
                "trello_card_move",
                json!({ "card": card, "list": list, "board": board }),
                |c| format!("Moved {} to {list}", s(c, "name")),
            )
            .await
        }
        TrelloCommand::Card(CardCommand::Comment { args, body_file }) => {
            let (card, text) = key_and_text(
                args,
                body_file.as_deref(),
                |_| true,
                |card| {
                    card.ok_or_else(|| {
                        Error::Invalid("pass `CARD TEXT` or `CARD --body-file <path>`".into())
                    })
                },
            )?;
            ctx.item(
                "trello_card_comment",
                json!({ "card": card, "text": text }),
                |_| format!("Commented on {card}"),
            )
            .await
        }
        TrelloCommand::Card(CardCommand::Archive { card }) => {
            ctx.item("trello_card_archive", json!({ "card": card }), |c| {
                format!("Archived {}", s(c, "name"))
            })
            .await
        }
        TrelloCommand::Card(CardCommand::Edit {
            card,
            name,
            desc,
            desc_file,
            due,
            no_due,
            done,
            undone,
            add_label,
            remove_label,
            add_member,
            remove_member,
        }) => {
            let description = read_text(desc, desc_file.as_deref())?;
            let due = match no_due {
                true => Some(String::new()),
                false => due,
            };
            let done = match (done, undone) {
                (true, _) => Some(true),
                (_, true) => Some(false),
                _ => None,
            };
            ctx.item(
                "trello_card_edit",
                json!({
                    "card": card, "name": name, "description": description, "due": due, "done": done,
                    "add_labels": add_label, "remove_labels": remove_label,
                    "add_members": add_member, "remove_members": remove_member,
                }),
                |c| format!("Updated {} {}", s(c, "name"), s(c, "url")),
            )
            .await
        }
        TrelloCommand::Checklist(ChecklistCommand::List { card }) => {
            let lists = ctx
                .task("trello_checklist_list", json!({ "card": card }))
                .await?;
            let tty = ctx.out.tty;
            ctx.out.item(lists, serde_json::Value::Null, |lists| {
                let mut out = String::new();
                for list in lists.as_array().into_iter().flatten() {
                    out.push_str(&bold(s(list, "name"), tty));
                    out.push('\n');
                    for item in list["items"].as_array().into_iter().flatten() {
                        let mark = if item["done"] == true { "[x]" } else { "[ ]" };
                        out.push_str(&format!(
                            "  {mark} {}  {}\n",
                            s(item, "name"),
                            dim(s(item, "id"), tty)
                        ));
                    }
                }
                if out.is_empty() {
                    out = "The card has no checklists".into();
                }
                out
            })
        }
        TrelloCommand::Checklist(ChecklistCommand::Create { card, name, item }) => {
            ctx.item(
                "trello_checklist_create",
                json!({ "card": card, "name": name, "items": item }),
                |c| {
                    format!(
                        "Added checklist {} with {} item(s)",
                        s(c, "name"),
                        c["items"].as_array().map_or(0, Vec::len)
                    )
                },
            )
            .await
        }
        TrelloCommand::Checklist(ChecklistCommand::Check { card, item }) => {
            ctx.item(
                "trello_checklist_check",
                json!({ "card": card, "item": item, "done": true }),
                |i| format!("[x] {}", s(i, "name")),
            )
            .await
        }
        TrelloCommand::Checklist(ChecklistCommand::Uncheck { card, item }) => {
            ctx.item(
                "trello_checklist_check",
                json!({ "card": card, "item": item, "done": false }),
                |i| format!("[ ] {}", s(i, "name")),
            )
            .await
        }
    }
}
