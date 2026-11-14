---
name: atl-jsm
description: Work with Jira Service Management through the `atl` CLI. Covers service desks, request types, queues, customer requests, replies and internal notes, transitions, approvals, participants, SLAs and attachments, on Cloud and Data Center. Builds on the `atl` skill.
---

# Jira Service Management with atl

Add `--agent` to every command (see the `atl` skill for output and errors). JSM uses the Jira login. DESK is a service desk ID or project key; `[jsm] desk` in `.atl.toml` sets a default.

## Desks and queues

- `atl jsm desk list --agent` · `atl jsm desk types DESK --agent` (request types)
- `atl jsm queue list DESK --agent` · `atl jsm queue issues QUEUE --desk DESK --agent`

## Requests

- `atl jsm request list [--desk DESK] [-s open|closed|all] [--search WORDS] [--mine] --agent`
- `atl jsm request view KEY --agent`: description and comments.
- `atl jsm request create --desk DESK -t "Request type" -s "Summary" [-b TEXT] [--on-behalf-of EMAIL] --agent`
- `atl jsm request comment KEY "text" --agent` replies to the customer. Add `--internal` for a note that customers cannot see.
- `atl jsm request transitions KEY --agent` · `atl jsm request move KEY "Resolve" --agent`

## Approvals, participants, SLAs and files

- `atl jsm request approvals KEY --agent`
- `atl jsm request approve KEY [ID] --agent` · `atl jsm request decline KEY [ID] --agent`. Without ID, `atl` answers the one pending approval you can answer.
- `atl jsm request participants KEY [--add USER]... [--remove USER]... --agent`. USER is `me`, an email or a name.
- `atl jsm request sla KEY --agent`: state and time left for each SLA.
- `atl jsm request attach KEY FILE... [--internal] --agent` · `atl jsm request attachments KEY --agent`
