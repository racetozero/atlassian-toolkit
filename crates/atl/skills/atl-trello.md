---
name: atl-trello
description: Work with Trello through the `atl` CLI. Covers boards, lists, cards (create, edit, move, comment, archive, labels, members, due dates) and checklists. Builds on the `atl` skill.
---

# Trello with atl

Add `--agent` to every command (see the `atl` skill for output and errors). A board is a name, URL, short link or ID (`[trello] board` in `.atl.toml` sets a default). A card is a URL, short link or ID.

## Boards and cards

- `atl trello board list --agent` · `atl trello list list -b BOARD --agent`
- `atl trello card list -b BOARD [-l LIST] [--mine] --agent` · `atl trello card view CARD --agent` (description and comments)
- `atl trello card create -b BOARD -l LIST -n NAME [-d MD] [--due DATE] --agent`
- `atl trello card edit CARD [-n NAME] [-d MD] [--due DATE | --no-due] [--done | --undone] [--add-label L] [--remove-label L] [--add-member U] [--remove-member U] --agent`
  - A label is a name or a color. A member is a username, a full name, or `me`.
- `atl trello card move CARD -l LIST --agent` · `atl trello card comment CARD "text" --agent` · `atl trello card archive CARD --agent`

## Checklists

- `atl trello checklist list CARD --agent`
- `atl trello checklist create CARD NAME [--item TEXT]... --agent`
- `atl trello checklist check CARD ITEM --agent` · `atl trello checklist uncheck CARD ITEM --agent`. ITEM is an item name or ID.
