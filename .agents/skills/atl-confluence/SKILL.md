---
name: atl-confluence
description: Read, write and organize Confluence pages, blog posts, spaces, comments, labels and attachments as Markdown through the `atl` CLI, and keep a page tree in git as Markdown files (export and push), on Cloud and Data Center. Builds on the `atl` skill.
---

# Confluence with atl

Add `--agent` to every command (see the `atl` skill for output and errors). REF is a page ID, a page URL, or `SPACE:Title`.

## Read

- `atl confluence page get REF --agent`: the page as Markdown in `data.markdown`. Add `--version N` for an old version.
- `atl confluence page children REF [-r] [--max-depth N] --agent` (rows with type, depth and parent)
- `atl confluence page find WORDS [--space KEY] --agent` (title search)
- `atl confluence search "words" [--space KEY] [--type page] --agent` · `atl confluence search --cql 'CQL' --agent`
- `atl confluence page history REF --agent` · `atl confluence page comments REF [--inline] --agent`
- `atl confluence space list --agent` · `atl confluence space pages KEY --agent`
- `atl confluence blog list --space KEY --agent` · `atl confluence blog get ID --agent`

## Write

- `atl confluence page create --space KEY --title T --file notes.md [--parent REF] --agent`
- `atl confluence page update REF --file notes.md [--title T] --agent` (the version number goes up for you)
- `.md` files are converted from Markdown; `.html` and `.xml` files are sent as storage format.
- `atl confluence page move REF --parent REF | --before REF | --after REF --agent`
- `atl confluence page labels REF [--add L] [--remove L] --agent`
- `atl confluence page comment REF "Markdown" [--parent COMMENT_ID] --agent` · `atl confluence comment delete ID --agent --yes`
- `atl confluence page delete REF --agent --yes`
- `atl confluence blog create --space KEY --title T --body MD --agent` · `atl confluence space create KEY NAME --agent`

## Attachments

- `atl confluence page attachments REF [--pattern "*.png"] [--download --dest DIR] --agent`
- `atl confluence page attach REF FILE... [--replace] --agent`
- `atl confluence attachment download URL -o FILE --agent` · `atl confluence attachment delete ID --agent --yes`

## Docs as code

1. `atl confluence page export REF -r --dest DIR --agent` writes `<title>/index.md` for each page, with front matter (id, version). Attachments go in `_attachments/`.
2. Edit the files. New `.md` files become child pages of their folder's `index.md`.
3. `atl confluence page push DIR --dry-run --agent` shows the plan; `atl confluence page push DIR --agent` publishes. It refuses pages that changed on the server after the export unless you add `--force`.

`atl confluence page copy-tree REF --to REF [--suffix TEXT] --agent` copies a page tree. `atl convert FILE --from markdown --to storage|adf|wiki` converts offline.
