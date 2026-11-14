---
name: atl-jira
description: Work with Jira and Jira Software through the `atl` CLI. Covers searching, viewing, creating, editing, moving, assigning and commenting on issues, custom fields, links, worklogs, attachments, JQL, filters, boards, sprints, epics, components and releases, on Cloud and Data Center. Builds on the `atl` skill.
---

# Jira with atl

Add `--agent` to every command (see the `atl` skill for output and errors). When you leave out KEY, `atl` uses the issue key in the current git branch name.

## Find issues

- `atl jira issue list --agent [-p KEY] [-s STATUS]... [-a me|x|USER] [-r USER] [-t TYPE] [-l LABEL] [-y PRIORITY] [-C COMPONENT] [-R RESOLUTION] [-P PARENT] [-w] [--history] [--sprint current|next|closed|ID] [--created -7d] [--updated week] [--text WORDS] [-q 'JQL'] [--limit 50]`
  - With no filters: your unresolved issues. `~` excludes (`-s ~Done`). `-a x` is unassigned. `-w` is issues you watch.
- `atl jira issue count --agent [same filters]`
- `atl jira sprint issues [ID | --current | --prev | --next] [--board ID] [filters] --agent`
- `atl jira epic issues EPIC [filters] --agent`
- `atl jira filter list --agent` · `atl jira filter run "My bugs" --agent`
- JQL: `atl jira jql check 'JQL' --agent` (exit 3 with Jira's errors) · `atl jira jql fields --agent` · `atl jira jql suggest status In --agent`

## Read

- `atl jira issue view KEY --agent [--comments 5] [--field "Story Points"]`: description, links and comments as Markdown.
- `atl jira issue history KEY --agent` · `atl jira issue worklogs KEY --agent` · `atl jira issue transitions KEY --agent` · `atl jira issue watchers KEY --agent`
- `atl jira comment list KEY --agent` (with comment IDs)

## Create and change

- `atl jira issue create --agent -p KEY -t Task -s "Summary" [-b "Markdown" | --body-file F] [-a me] [-r USER] [-l LABEL] [-C COMPONENT] [--fix-version V] [--affects-version V] [--estimate 2h] [--parent KEY] [--field "NAME=VALUE"]`
- `atl jira issue edit KEY --agent [-s SUMMARY] [-b MD] [-y PRIORITY] [-a USER] [--parent KEY] [-l add] [-l -remove] [-C COMP | -C -COMP] [--fix-version V | -V] [--field "Story Points=5"] [--skip-notify]`
- Custom fields: `atl jira field list [FILTER] --agent` gives names, IDs and types. `--field` text is converted for the field type, and a JSON value is sent unchanged.
- `atl jira issue move KEY "In Progress" --agent [-R Fixed] [-a me] [--comment "..."]`
- `atl jira issue assign KEY me|x|USER --agent`
- Several issues: `atl jira issue move A-1 A-2 Done --agent` or `atl jira issue assign A-1 A-2 me --agent` (one row per key with `ok` and `error`; exit 1 when one failed).
- `atl jira issue comment KEY "Markdown" --agent` · `atl jira comment edit KEY ID "Markdown" --agent` · `atl jira comment delete KEY ID --agent --yes`
- `atl jira issue clone KEY [-s S] [-H "find:replace"] --agent` · `atl jira issue delete KEY [--subtasks] --agent --yes`
- `atl jira issue watch|unwatch KEY [-u USER] --agent`

## Links, time and files

- `atl jira issue link A-1 A-2 Blocks --agent` means "A-1 blocks A-2". `atl jira issue unlink A-1 A-2 --agent` · `atl jira issue link-types --agent`
- `atl jira issue remote-link KEY URL --title T --agent`
- `atl jira issue worklog KEY "2h 30m" --agent`
- `atl jira issue attach KEY FILE... --agent` · `atl jira issue attachments KEY --agent` · `atl jira attachment download ID -o FILE --agent`

## Planning

- `atl jira project list --agent` · `atl jira component list -p KEY --agent` · `atl jira release list -p KEY [--state unreleased] --agent`
- `atl jira board list --agent` · `atl jira sprint list [--board ID] --agent` · `atl jira sprint add SPRINT KEY... --agent`
- `atl jira sprint create NAME [--board ID] [--start DATE] --end DATE --agent` · `atl jira sprint start ID [--end DATE] --agent` · `atl jira sprint close ID --agent`
- `atl jira epic list --agent` · `atl jira epic add EPIC KEY... --agent` · `atl jira epic remove KEY... --agent`
- `atl jira me --agent`
