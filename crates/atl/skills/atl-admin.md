---
name: atl-admin
description: Manage an Atlassian Cloud organization through the `atl` CLI with an organization API key. Covers organizations, directories, domains, managed users (suspend, restore), groups and group members, and the audit log. Builds on the `atl` skill.
---

# Atlassian Admin with atl

Add `--agent` to every command (see the `atl` skill for output and errors). The organization comes from `--org ID|NAME`, then `[admin] org` in `.atl.toml`, then the only organization the API key can see.

Suspending a user or removing a group member can take away product access. Do it only when the user asked for it; these commands need `--yes`.

- `atl admin org list --agent` · `atl admin directory list --agent` · `atl admin domain list --agent`
- `atl admin user list [-s TEXT] --agent` · `atl admin group list [-s TEXT] --agent`
- `atl admin user suspend USER [--directory ID] --agent --yes` · `atl admin user restore USER --agent`. USER is an email or an account ID.
- `atl admin group add-member GROUP USER --agent` · `atl admin group remove-member GROUP USER --agent --yes`. GROUP is a name or an ID.
- `atl admin event list [-s TEXT] [--from DATE] [--to DATE] [--action A] --agent`: the audit log, newest first.
