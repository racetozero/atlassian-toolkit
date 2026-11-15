//! Task-level Trello API (`https://api.trello.com/1`): boards, lists and
//! cards, named the way people see them (names, short links, URLs).

use crate::catalog::HttpMethod;
use crate::client::Client;
use crate::config::Family;
use crate::error::{Error, Result};
use crate::json::{items, text};
use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
#[schemars(rename = "TrelloBoard")]
pub struct Board {
    pub id: String,
    pub name: String,
    pub short_link: Option<String>,
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
#[schemars(rename = "TrelloList")]
pub struct List {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct Card {
    pub id: String,
    pub name: String,
    /// The list's name when known, else its ID.
    pub list: Option<String>,
    pub due: Option<String>,
    pub labels: Vec<String>,
    pub closed: bool,
    pub short_link: Option<String>,
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct CheckItem {
    pub id: String,
    pub name: String,
    pub done: bool,
}

#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct Checklist {
    pub id: String,
    pub name: String,
    pub items: Vec<CheckItem>,
}

/// Changes for [`Trello::edit_card`]; `None` leaves a value as it is.
#[derive(Debug, Clone, Default)]
pub struct CardEdit {
    pub name: Option<String>,
    pub description: Option<String>,
    /// A date or date-time; an empty string removes the due date.
    pub due: Option<String>,
    pub done: Option<bool>,
    pub add_labels: Vec<String>,
    pub remove_labels: Vec<String>,
    pub add_members: Vec<String>,
    pub remove_members: Vec<String>,
}

#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct CardComment {
    pub author: Option<String>,
    pub date: Option<String>,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct CardDetail {
    #[serde(flatten)]
    pub card: Card,
    pub board: Option<String>,
    /// The card description (Trello stores Markdown).
    pub description: String,
    /// Latest comments first.
    pub comments: Vec<CardComment>,
}

fn is_id(text: &str) -> bool {
    text.len() == 24 && text.bytes().all(|b| b.is_ascii_hexdigit())
}

/// The short link of a `trello.com/b/<link>/...` or `/c/<link>/...` URL, or
/// the text itself when it is an ID or an 8-character short link.
fn id_or_link(text: &str, kind: char) -> Option<String> {
    let text = text.trim();
    if let Some(rest) = text.split(&format!("trello.com/{kind}/")).nth(1) {
        return rest.split('/').next().map(str::to_owned);
    }
    let short_link = text.len() == 8 && text.bytes().all(|b| b.is_ascii_alphanumeric());
    (is_id(text) || short_link).then(|| text.to_owned())
}

pub struct Trello<'a> {
    client: &'a Client,
}

impl<'a> Trello<'a> {
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    async fn send(
        &self,
        method: HttpMethod,
        path: &str,
        query: &[(&str, String)],
    ) -> Result<Value> {
        self.client
            .send_json(Family::Trello, method, path, query, None)
            .await
    }

    fn card_from(c: &Value, lists: &[List]) -> Card {
        let list = text(c, "/idList").map(|id| {
            lists
                .iter()
                .find(|l| l.id == id)
                .map_or(id, |l| l.name.clone())
        });
        Card {
            id: text(c, "/id").unwrap_or_default(),
            name: text(c, "/name").unwrap_or_default(),
            list,
            due: text(c, "/due"),
            labels: items(c, "/labels")
                .filter_map(|l| text(l, "/name"))
                .collect(),
            closed: c.get("closed").and_then(Value::as_bool).unwrap_or(false),
            short_link: text(c, "/shortLink"),
            url: text(c, "/url"),
        }
    }

    /// Your open boards.
    pub async fn boards(&self) -> Result<Vec<Board>> {
        let found = self
            .send(
                HttpMethod::Get,
                "/1/members/me/boards",
                &[
                    ("filter", "open".into()),
                    ("fields", "name,url,shortLink".into()),
                ],
            )
            .await?;
        Ok(items(&found, "")
            .map(|b| Board {
                id: text(b, "/id").unwrap_or_default(),
                name: text(b, "/name").unwrap_or_default(),
                short_link: text(b, "/shortLink"),
                url: text(b, "/url"),
            })
            .collect())
    }

    /// A board ID from an ID, URL, short link, or exact name. Short links
    /// are checked against your boards because a name can look like one.
    async fn board_id(&self, board: &str) -> Result<String> {
        if is_id(board.trim()) || board.contains("trello.com/b/") {
            return Ok(id_or_link(board, 'b').unwrap_or_default());
        }
        let boards = self.boards().await?;
        boards
            .iter()
            .find(|b| {
                b.name.eq_ignore_ascii_case(board.trim())
                    || b.short_link.as_deref() == Some(board.trim())
            })
            .map(|b| b.id.clone())
            .ok_or_else(|| {
                Error::Invalid(format!(
                    "you have no open Trello board named `{board}`; boards: {}",
                    boards
                        .iter()
                        .map(|b| format!("{:?}", b.name))
                        .collect::<Vec<_>>()
                        .join(", ")
                ))
            })
    }

    fn card_id(card: &str) -> Result<String> {
        id_or_link(card, 'c').ok_or_else(|| {
            Error::Invalid(format!(
                "`{card}` is not a Trello card; pass its URL (https://trello.com/c/...), short link or ID"
            ))
        })
    }

    pub async fn lists(&self, board: &str) -> Result<Vec<List>> {
        let id = self.board_id(board).await?;
        let found = self
            .send(
                HttpMethod::Get,
                &format!("/1/boards/{id}/lists"),
                &[("filter", "open".into())],
            )
            .await?;
        Ok(items(&found, "")
            .map(|l| List {
                id: text(l, "/id").unwrap_or_default(),
                name: text(l, "/name").unwrap_or_default(),
            })
            .collect())
    }

    /// A list ID on `board` from an ID or a name.
    async fn list_id(&self, board: &str, list: &str) -> Result<String> {
        if is_id(list) {
            return Ok(list.to_owned());
        }
        let lists = self.lists(board).await?;
        lists
            .iter()
            .find(|l| l.name.eq_ignore_ascii_case(list.trim()))
            .map(|l| l.id.clone())
            .ok_or_else(|| {
                Error::Invalid(format!(
                    "board `{board}` has no list named `{list}`; lists: {}",
                    lists
                        .iter()
                        .map(|l| format!("{:?}", l.name))
                        .collect::<Vec<_>>()
                        .join(", ")
                ))
            })
    }

    /// Cards on a board (optionally one list), or your cards with `mine`.
    pub async fn cards(
        &self,
        board: Option<&str>,
        list: Option<&str>,
        mine: bool,
    ) -> Result<Vec<Card>> {
        if mine {
            let found = self
                .send(HttpMethod::Get, "/1/members/me/cards", &[])
                .await?;
            return Ok(items(&found, "").map(|c| Self::card_from(c, &[])).collect());
        }
        let board = board.ok_or_else(|| {
            Error::Invalid(
                "pass --board NAME (or set [trello] board in .atl.toml), or --mine for your cards"
                    .into(),
            )
        })?;
        let lists = self.lists(board).await?;
        let path = match list {
            Some(list) => format!("/1/lists/{}/cards", self.list_id(board, list).await?),
            None => format!("/1/boards/{}/cards", self.board_id(board).await?),
        };
        let found = self.send(HttpMethod::Get, &path, &[]).await?;
        Ok(items(&found, "")
            .map(|c| Self::card_from(c, &lists))
            .collect())
    }

    pub async fn card(&self, card: &str) -> Result<CardDetail> {
        let id = Self::card_id(card)?;
        let found = self
            .send(
                HttpMethod::Get,
                &format!("/1/cards/{id}"),
                &[
                    ("list", "true".into()),
                    ("board", "true".into()),
                    ("actions", "commentCard".into()),
                    ("actions_limit", "20".into()),
                ],
            )
            .await?;
        let mut summary = Self::card_from(&found, &[]);
        summary.list = text(&found, "/list/name").or(summary.list);
        Ok(CardDetail {
            card: summary,
            board: text(&found, "/board/name"),
            description: text(&found, "/desc").unwrap_or_default(),
            comments: items(&found, "/actions")
                .map(|a| CardComment {
                    author: text(a, "/memberCreator/fullName"),
                    date: text(a, "/date"),
                    text: text(a, "/data/text").unwrap_or_default(),
                })
                .collect(),
        })
    }

    pub async fn create_card(
        &self,
        board: Option<&str>,
        list: &str,
        name: &str,
        desc: Option<&str>,
        due: Option<&str>,
    ) -> Result<Card> {
        let list_id = match board {
            Some(board) => self.list_id(board, list).await?,
            None if is_id(list) => list.to_owned(),
            None => {
                return Err(Error::Invalid(format!(
                    "pass --board to find the list `{list}` by name, or pass the list ID"
                )));
            }
        };
        let mut query = vec![("idList", list_id), ("name", name.to_owned())];
        if let Some(desc) = desc {
            query.push(("desc", desc.to_owned()));
        }
        if let Some(due) = due {
            query.push(("due", due.to_owned()));
        }
        let created = self.send(HttpMethod::Post, "/1/cards", &query).await?;
        Ok(Self::card_from(&created, &[]))
    }

    /// Moves a card to a list on its own board (or on `board`).
    pub async fn move_card(&self, card: &str, list: &str, board: Option<&str>) -> Result<Card> {
        let id = Self::card_id(card)?;
        let list_id = if is_id(list) {
            list.to_owned()
        } else {
            let board = match board {
                Some(board) => board.to_owned(),
                None => text(
                    &self
                        .send(
                            HttpMethod::Get,
                            &format!("/1/cards/{id}"),
                            &[("fields", "idBoard".into())],
                        )
                        .await?,
                    "/idBoard",
                )
                .unwrap_or_default(),
            };
            self.list_id(&board, list).await?
        };
        let moved = self
            .send(
                HttpMethod::Put,
                &format!("/1/cards/{id}"),
                &[("idList", list_id)],
            )
            .await?;
        let mut card = Self::card_from(&moved, &[]);
        card.list = Some(list.to_owned());
        Ok(card)
    }

    pub async fn comment(&self, card: &str, text_body: &str) -> Result<CardComment> {
        let id = Self::card_id(card)?;
        let created = self
            .send(
                HttpMethod::Post,
                &format!("/1/cards/{id}/actions/comments"),
                &[("text", text_body.to_owned())],
            )
            .await?;
        Ok(CardComment {
            author: text(&created, "/memberCreator/fullName"),
            date: text(&created, "/date"),
            text: text_body.to_owned(),
        })
    }

    /// Archives (closes) a card; Trello can restore it later.
    pub async fn archive(&self, card: &str) -> Result<Card> {
        let id = Self::card_id(card)?;
        let closed = self
            .send(
                HttpMethod::Put,
                &format!("/1/cards/{id}"),
                &[("closed", "true".into())],
            )
            .await?;
        Ok(Self::card_from(&closed, &[]))
    }
}

impl Trello<'_> {
    /// `(id, names...)` of each label or member of the card's board.
    async fn board_things(&self, board: &str, kind: &str) -> Result<Vec<(String, Vec<String>)>> {
        let found = self
            .send(HttpMethod::Get, &format!("/1/boards/{board}/{kind}"), &[])
            .await?;
        Ok(items(&found, "")
            .map(|v| {
                let names = ["/name", "/color", "/username", "/fullName"]
                    .iter()
                    .filter_map(|p| text(v, p))
                    .collect();
                (text(v, "/id").unwrap_or_default(), names)
            })
            .collect())
    }

    fn pick(things: &[(String, Vec<String>)], wanted: &str, what: &str) -> Result<String> {
        things
            .iter()
            .find(|(id, names)| {
                id == wanted || names.iter().any(|n| n.eq_ignore_ascii_case(wanted))
            })
            .map(|(id, _)| id.clone())
            .ok_or_else(|| {
                let known: Vec<String> = things
                    .iter()
                    .filter_map(|(_, n)| n.first().cloned())
                    .collect();
                Error::Invalid(format!(
                    "the card's board has no {what} `{wanted}`; use one of: {}",
                    if known.is_empty() {
                        "(none)".to_owned()
                    } else {
                        known.join(", ")
                    }
                ))
            })
    }

    /// Renames, describes, schedules, completes, labels and assigns a card.
    pub async fn edit_card(&self, card: &str, edit: &CardEdit) -> Result<Card> {
        let id = Self::card_id(card)?;
        let mut query: Vec<(&str, String)> = Vec::new();
        if let Some(name) = &edit.name {
            query.push(("name", name.clone()));
        }
        if let Some(description) = &edit.description {
            query.push(("desc", description.clone()));
        }
        if let Some(due) = &edit.due {
            query.push((
                "due",
                if due.is_empty() {
                    "null".into()
                } else {
                    due.clone()
                },
            ));
        }
        if let Some(done) = edit.done {
            query.push(("dueComplete", done.to_string()));
        }
        let labels = !edit.add_labels.is_empty() || !edit.remove_labels.is_empty();
        let members = !edit.add_members.is_empty() || !edit.remove_members.is_empty();
        if query.is_empty() && !labels && !members {
            return Err(Error::Invalid(
                "nothing to change; pass --name, --desc, --due, --no-due, --done, --undone, --add-label, --remove-label, --add-member or --remove-member".into(),
            ));
        }
        if !query.is_empty() {
            self.send(HttpMethod::Put, &format!("/1/cards/{id}"), &query)
                .await?;
        }
        if labels || members {
            let board = text(
                &self
                    .send(
                        HttpMethod::Get,
                        &format!("/1/cards/{id}"),
                        &[("fields", "idBoard".into())],
                    )
                    .await?,
                "/idBoard",
            )
            .unwrap_or_default();
            if labels {
                let known = self.board_things(&board, "labels").await?;
                for label in &edit.add_labels {
                    let label = Self::pick(&known, label, "label")?;
                    self.send(
                        HttpMethod::Post,
                        &format!("/1/cards/{id}/idLabels"),
                        &[("value", label)],
                    )
                    .await?;
                }
                for label in &edit.remove_labels {
                    let label = Self::pick(&known, label, "label")?;
                    self.send(
                        HttpMethod::Delete,
                        &format!("/1/cards/{id}/idLabels/{label}"),
                        &[],
                    )
                    .await?;
                }
            }
            if members {
                let known = self.board_things(&board, "members").await?;
                let me = match edit
                    .add_members
                    .iter()
                    .chain(&edit.remove_members)
                    .any(|m| m == "me")
                {
                    true => text(
                        &self
                            .send(HttpMethod::Get, "/1/members/me", &[("fields", "id".into())])
                            .await?,
                        "/id",
                    ),
                    false => None,
                };
                let member = |who: &String| match (who.as_str(), &me) {
                    ("me", Some(me)) => Ok(me.clone()),
                    _ => Self::pick(&known, who, "member"),
                };
                for who in &edit.add_members {
                    self.send(
                        HttpMethod::Post,
                        &format!("/1/cards/{id}/idMembers"),
                        &[("value", member(who)?)],
                    )
                    .await?;
                }
                for who in &edit.remove_members {
                    self.send(
                        HttpMethod::Delete,
                        &format!("/1/cards/{id}/idMembers/{}", member(who)?),
                        &[],
                    )
                    .await?;
                }
            }
        }
        Ok(self.card(&id).await?.card)
    }

    pub async fn checklists(&self, card: &str) -> Result<Vec<Checklist>> {
        let id = Self::card_id(card)?;
        let found = self
            .send(HttpMethod::Get, &format!("/1/cards/{id}/checklists"), &[])
            .await?;
        Ok(items(&found, "")
            .map(|c| Checklist {
                id: text(c, "/id").unwrap_or_default(),
                name: text(c, "/name").unwrap_or_default(),
                items: items(c, "/checkItems")
                    .map(|i| CheckItem {
                        id: text(i, "/id").unwrap_or_default(),
                        name: text(i, "/name").unwrap_or_default(),
                        done: text(i, "/state").as_deref() == Some("complete"),
                    })
                    .collect(),
            })
            .collect())
    }

    /// Adds a checklist with these items to a card.
    pub async fn create_checklist(
        &self,
        card: &str,
        name: &str,
        entries: &[String],
    ) -> Result<Checklist> {
        let id = Self::card_id(card)?;
        let created = self
            .send(
                HttpMethod::Post,
                &format!("/1/cards/{id}/checklists"),
                &[("name", name.to_owned())],
            )
            .await?;
        let checklist = text(&created, "/id").unwrap_or_default();
        let mut out = Checklist {
            id: checklist.clone(),
            name: name.to_owned(),
            items: Vec::new(),
        };
        for entry in entries {
            let item = self
                .send(
                    HttpMethod::Post,
                    &format!("/1/checklists/{checklist}/checkItems"),
                    &[("name", entry.clone())],
                )
                .await?;
            out.items.push(CheckItem {
                id: text(&item, "/id").unwrap_or_default(),
                name: entry.clone(),
                done: false,
            });
        }
        Ok(out)
    }

    /// Ticks (or, with `done: false`, unticks) an item by name or ID.
    pub async fn check_item(&self, card: &str, item: &str, done: bool) -> Result<CheckItem> {
        let id = Self::card_id(card)?;
        let all: Vec<CheckItem> = self
            .checklists(&id)
            .await?
            .into_iter()
            .flat_map(|c| c.items)
            .collect();
        let found: Vec<&CheckItem> = all
            .iter()
            .filter(|i| i.id == item || i.name.eq_ignore_ascii_case(item))
            .collect();
        let chosen = match found.as_slice() {
            [one] => (*one).clone(),
            [] => {
                return Err(Error::Invalid(format!(
                    "the card has no checklist item `{item}`; items: {}",
                    all.iter()
                        .map(|i| format!("{:?}", i.name))
                        .collect::<Vec<_>>()
                        .join(", ")
                )));
            }
            many => {
                return Err(Error::needs_input(
                    "item",
                    format!("several checklist items are named `{item}`; pass the item ID"),
                    many.iter()
                        .map(|i| crate::error::Choice {
                            value: i.id.clone(),
                            label: i.name.clone(),
                        })
                        .collect(),
                ));
            }
        };
        let state = if done { "complete" } else { "incomplete" };
        self.send(
            HttpMethod::Put,
            &format!("/1/cards/{id}/checkItem/{}", chosen.id),
            &[("state", state.into())],
        )
        .await?;
        Ok(CheckItem { done, ..chosen })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn boards_and_cards_are_named_by_url_short_link_or_id() {
        let id = "0123456789abcdef01234567";
        assert_eq!(id_or_link(id, 'b').as_deref(), Some(id));
        assert_eq!(id_or_link("AbCd1234", 'c').as_deref(), Some("AbCd1234"));
        assert_eq!(
            id_or_link("https://trello.com/c/AbCd1234/12-write-docs", 'c').as_deref(),
            Some("AbCd1234")
        );
        assert_eq!(
            id_or_link("https://trello.com/b/XyZ98765/roadmap", 'b').as_deref(),
            Some("XyZ98765")
        );
        assert_eq!(id_or_link("Roadmap", 'b'), None);
        assert_eq!(id_or_link("Road map", 'b'), None);
    }
}
