//! Trello tasks.

use super::*;
use crate::client::Client;
use crate::error::Result;
use crate::trello::Trello;
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct TrelloEditArgs {
    /// Card URL, short link or ID.
    card: String,
    name: Option<String>,
    /// Description (Markdown).
    description: Option<String>,
    /// Due date or date-time; an empty string removes it.
    due: Option<String>,
    /// Mark the due date complete (true) or not (false).
    done: Option<bool>,
    /// Label names or colors on the card's board.
    #[serde(default)]
    add_labels: Vec<String>,
    #[serde(default)]
    remove_labels: Vec<String>,
    /// Members by username, full name, or me.
    #[serde(default)]
    add_members: Vec<String>,
    #[serde(default)]
    remove_members: Vec<String>,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct ChecklistCreateArgs {
    card: String,
    name: String,
    /// Items to add, in order.
    #[serde(default)]
    items: Vec<String>,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct CheckArgs {
    card: String,
    /// Item name or ID.
    item: String,
    /// false unticks the item.
    #[serde(default = "default_true")]
    done: bool,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct TrelloCardsArgs {
    /// Board name, URL, short link or ID.
    board: Option<String>,
    /// List name or ID on the board.
    list: Option<String>,
    /// Your cards on every board instead.
    #[serde(default)]
    mine: bool,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct TrelloCardArgs {
    /// Card URL, short link or ID.
    card: String,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct TrelloCreateArgs {
    /// Board name, URL, short link or ID (needed when `list` is a name).
    board: Option<String>,
    /// List name or ID.
    list: String,
    name: String,
    /// Description (Markdown).
    description: Option<String>,
    /// Due date, for example 2026-12-01 or 2026-12-01T17:00:00Z.
    due: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct TrelloMoveArgs {
    card: String,
    /// Target list name or ID.
    list: String,
    /// Board of the list (default: the card's board).
    board: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct TrelloCommentArgs {
    card: String,
    text: String,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct TrelloBoardArgs {
    /// Board name, URL, short link or ID.
    board: String,
}

pub(super) static TASKS: &[TaskDef] = &[
    task!("trello_board_list", true, Empty => Vec<crate::trello::Board>, "Your open Trello boards."),
    task!(
        "trello_list_list",
        true,
        TrelloBoardArgs => Vec<crate::trello::List>,
        "Open lists of a Trello board (board by name, URL, short link or ID)."
    ),
    task!(
        "trello_card_list",
        true,
        TrelloCardsArgs => Vec<crate::trello::Card>,
        "Cards of a Trello board or one of its lists, or your cards on every board (`mine`)."
    ),
    task!(
        "trello_card_view",
        true,
        TrelloCardArgs => crate::trello::CardDetail,
        "One Trello card with list, board, description and latest comments."
    ),
    task!(
        "trello_card_create",
        false,
        TrelloCreateArgs => crate::trello::Card,
        "Create a Trello card in a list (by name with `board`, or by list ID)."
    ),
    task!(
        "trello_card_move",
        false,
        TrelloMoveArgs => crate::trello::Card,
        "Move a Trello card to another list (by default on the card's own board)."
    ),
    task!(
        "trello_card_comment",
        false,
        TrelloCommentArgs => crate::trello::CardComment,
        "Add a comment to a Trello card."
    ),
    task!(
        "trello_card_archive",
        false,
        TrelloCardArgs => crate::trello::Card,
        "Archive (close) a Trello card."
    ),
    task!(
        "trello_card_edit",
        false,
        TrelloEditArgs => crate::trello::Card,
        "Change a Trello card: name, description, due date, done, labels (by name or color) and members (username, name or me)."
    ),
    task!(
        "trello_checklist_list",
        true,
        TrelloCardArgs => Vec<crate::trello::Checklist>,
        "Checklists of a Trello card with their items and whether each is done."
    ),
    task!(
        "trello_checklist_create",
        false,
        ChecklistCreateArgs => crate::trello::Checklist,
        "Add a checklist with items to a Trello card."
    ),
    task!(
        "trello_checklist_check",
        false,
        CheckArgs => crate::trello::CheckItem,
        "Tick (or with done: false, untick) a Trello checklist item by name or ID."
    ),
];

pub(super) async fn run(client: &Client, name: &str, args: Value) -> Result<Value> {
    let trello = Trello::new(client);
    match name {
        "trello_board_list" => {
            let _: Empty = parse(name, args)?;
            output(name, trello.boards().await?)
        }
        "trello_list_list" => {
            let a: TrelloBoardArgs = parse(name, args)?;
            output(name, trello.lists(&a.board).await?)
        }
        "trello_card_list" => {
            let a: TrelloCardsArgs = parse(name, args)?;
            output(
                name,
                trello
                    .cards(a.board.as_deref(), a.list.as_deref(), a.mine)
                    .await?,
            )
        }
        "trello_card_view" => {
            let a: TrelloCardArgs = parse(name, args)?;
            output(name, trello.card(&a.card).await?)
        }
        "trello_card_create" => {
            let a: TrelloCreateArgs = parse(name, args)?;
            output(
                name,
                trello
                    .create_card(
                        a.board.as_deref(),
                        &a.list,
                        &a.name,
                        a.description.as_deref(),
                        a.due.as_deref(),
                    )
                    .await?,
            )
        }
        "trello_card_move" => {
            let a: TrelloMoveArgs = parse(name, args)?;
            output(
                name,
                trello
                    .move_card(&a.card, &a.list, a.board.as_deref())
                    .await?,
            )
        }
        "trello_card_comment" => {
            let a: TrelloCommentArgs = parse(name, args)?;
            output(name, trello.comment(&a.card, &a.text).await?)
        }
        "trello_card_edit" => {
            let a: TrelloEditArgs = parse(name, args)?;
            let edit = crate::trello::CardEdit {
                name: a.name,
                description: a.description,
                due: a.due,
                done: a.done,
                add_labels: a.add_labels,
                remove_labels: a.remove_labels,
                add_members: a.add_members,
                remove_members: a.remove_members,
            };
            output(name, trello.edit_card(&a.card, &edit).await?)
        }
        "trello_checklist_list" => {
            let a: TrelloCardArgs = parse(name, args)?;
            output(name, trello.checklists(&a.card).await?)
        }
        "trello_checklist_create" => {
            let a: ChecklistCreateArgs = parse(name, args)?;
            output(
                name,
                trello.create_checklist(&a.card, &a.name, &a.items).await?,
            )
        }
        "trello_checklist_check" => {
            let a: CheckArgs = parse(name, args)?;
            output(name, trello.check_item(&a.card, &a.item, a.done).await?)
        }
        "trello_card_archive" => {
            let a: TrelloCardArgs = parse(name, args)?;
            output(name, trello.archive(&a.card).await?)
        }
        _ => Err(unknown(name)),
    }
}
