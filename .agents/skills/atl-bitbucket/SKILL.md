---
name: atl-bitbucket
description: Work with Bitbucket through the `atl` CLI. Covers repositories, branches, commits, pull requests (create, review, comments, build checks, merge, drafts, checkout) and Pipelines, on Bitbucket Cloud and Data Center. Builds on the `atl` skill.
---

# Bitbucket with atl

Add `--agent` to every command (see the `atl` skill for output and errors). The repository comes from `-R WORKSPACE/REPO` (`PROJECT/REPO` on Data Center), then `[bitbucket] repo` in `.atl.toml`, then the git `origin` remote.

## Repositories

- `atl bitbucket repo list [--owner WORKSPACE] --agent` · `atl bitbucket repo view --agent`
- `atl bitbucket branch list [FILTER] --agent` · `atl bitbucket branch create NAME [--from REF] --agent` · `atl bitbucket branch delete NAME --agent --yes`
- `atl bitbucket commit list [-b BRANCH] --agent`

## Pull requests

- `atl bitbucket pr list [-s open|merged|declined|all] --agent` · `atl bitbucket pr view ID --agent` · `atl bitbucket pr diff ID --agent`
- `atl bitbucket pr create -t TITLE [--from BRANCH] [--to main] [-b "Markdown"] [--draft] --agent`. The source is the current branch by default.
- `atl bitbucket pr edit ID [-t T] [-b MD] [--to BRANCH] --agent` · `atl bitbucket pr reviewers ID [--add USER] [--remove USER] --agent`
- `atl bitbucket pr ready ID [--undo] --agent` ends or restarts a draft.
- `atl bitbucket pr checkout ID --agent` runs `git fetch` and `git switch`; `--dry-run` only prints them.

## Review

- `atl bitbucket pr comments ID --agent`: general and inline comments with path, line and parent.
- `atl bitbucket pr checks ID --agent`: build statuses of the latest commit (`data.passed`, `data.pending`; exit 1 when a build failed).
- `atl bitbucket pr comment ID "Markdown" --agent`
- `atl bitbucket pr approve ID --agent` · `atl bitbucket pr unapprove ID --agent` · `atl bitbucket pr request-changes ID --agent`
- `atl bitbucket pr merge ID [--strategy squash] --agent --yes` · `atl bitbucket pr decline ID [--comment MD] --agent --yes`

## Pipelines (Cloud only)

- `atl bitbucket pipeline list --agent` · `atl bitbucket pipeline view N --agent` · `atl bitbucket pipeline logs N [--step NAME] --agent`
- `atl bitbucket pipeline run [--branch B] [--custom NAME] --agent` · `atl bitbucket pipeline stop N --agent`
- N is a build number or a UUID.
