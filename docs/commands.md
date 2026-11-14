# Commands

Every command has `--help`, and `atl man --dir ~/.local/share/man/man1` installs man pages (`man atl-jira-issue-list`). `list` commands also answer to `ls`.

Rich text is Markdown in and out. `atl` converts it to ADF on Jira Cloud, to wiki markup on Jira Data Center, and to storage format on Confluence. `atl convert` runs the same converters offline:

```sh
atl convert notes.md --to adf
atl convert page.xml --from storage --to markdown
```

When you leave out an issue key, `atl` takes it from the git branch name (`feature/abc-123-fix` is `ABC-123`). A command that asks first (deletes, merges, suspends) skips the question with `--yes`.

## Jira

### Finding issues

```sh
atl jira issue list [-p KEY] [-s STATUS]... [-a me|x|USER] [-r USER] [-t TYPE] [-l LABEL]
                    [-y PRIORITY] [-C COMPONENT] [-R RESOLUTION] [-P PARENT] [-w] [--history]
                    [--sprint current|next|closed|ID] [--created -7d] [--created-before DATE]
                    [--updated week] [--updated-before DATE] [--text WORDS] [-q JQL]
                    [--order-by FIELD] [--reverse] [--limit N] [--page-token T]
atl jira issue count [same filters]
```

With no filters you get your unresolved issues. A value that starts with `~` is excluded (`-s ~Done`). `-a x` means unassigned. `-w` lists issues you watch, and `--history` lists issues you viewed recently. `-q` adds raw JQL to the filters.

`--browse` (`-i`) opens the results in the full-screen browser. `--web` opens the same search on your Jira site. Both flags also work on `sprint issues`, `epic issues` and `filter run`.

```sh
atl jira sprint issues [ID | --current | --prev | --next] [--board ID] [filters]
atl jira epic issues EPIC [filters]
atl jira filter list [NAME]
atl jira filter run ID|NAME
atl jira jql check 'JQL'           # Jira's own parser; exit 3 with its errors
atl jira jql fields
atl jira jql suggest FIELD [PREFIX]
```

On Cloud, `filter list` shows every filter you can see. On Data Center it shows your favourites.

### Reading an issue

```sh
atl jira issue view [KEY] [--comments N] [--field NAME]... [--web]
atl jira issue history [KEY]        # field changes, newest first
atl jira issue worklogs [KEY]
atl jira issue transitions [KEY]
atl jira issue watchers [KEY]
atl jira issue attachments [KEY]
atl jira attachment download ID [-o FILE]
```

`view` prints the description, links and comments as Markdown, and the fields you name with `--field`.

### Creating and changing issues

```sh
atl jira issue create -p KEY -t TYPE -s SUMMARY [-b MD | --body-file F] [-a USER] [-r USER]
                      [-l LABEL] [-y PRIORITY] [-C COMPONENT] [--fix-version V]
                      [--affects-version V] [--estimate 2h] [--parent KEY]
                      [--field NAME=VALUE]... [--web]
atl jira issue edit [KEY] [-s SUMMARY] [-b MD] [-y PRIORITY] [-a USER] [-r USER] [--parent KEY]
                    [--estimate T] [-l LABEL | -l -LABEL] [-C COMP | -C -COMP]
                    [--fix-version V | -V] [--affects-version V | -V]
                    [--field NAME=VALUE]... [--skip-notify] [--editor] [--web]
atl jira issue clone [KEY] [-s SUMMARY] [-y PRIORITY] [-a USER] [-l LABEL] [-H FIND:REPLACE]
atl jira issue delete KEY [--subtasks]
```

In a terminal, `create` asks for any value you leave out. In `edit`, a leading `-` removes a label, component or version. `--editor`, or `edit` with no flags at a terminal, opens the description in `$VISUAL` or `$EDITOR`.

`--field` takes a field name or ID: `--field "Story Points=3"`. The text is converted for the field type. Numbers, options, users, comma-separated lists and Markdown text areas all work. A JSON value is sent unchanged: `--field 'Team={"id":"10"}'`. `atl jira field list [FILTER] [--custom]` lists the fields with their IDs and types.

### Workflow

```sh
atl jira issue move [KEY...] [STATUS] [-R RESOLUTION] [-a USER] [--comment MD]
atl jira issue assign [KEY...] [me | x | USER]
atl jira issue watch [KEY] [-u USER]
atl jira issue unwatch [KEY] [-u USER]
```

`move` takes a target status or a transition name. Without one it lists the choices. `assign` without a user lists who can be assigned. With several keys, each key gets one result row, and the exit code is 1 if one of them failed.

### Comments, links and time

```sh
atl jira issue comment [KEY] [TEXT] [--internal]
atl jira comment list [KEY]
atl jira comment edit KEY ID [TEXT | --body-file F]
atl jira comment delete KEY ID
atl jira issue link A-1 A-2 Blocks
atl jira issue unlink A-1 A-2 [--type T]
atl jira issue remote-link KEY URL [--title T]
atl jira issue link-types
atl jira issue worklog [KEY] "2h 30m"
atl jira issue attach [KEY] FILE...
```

Piped text becomes the comment: `git log -1 --format=%B | atl jira issue comment ABC-1`. With no text at a terminal, `comment` opens your editor. `--internal` adds a JSM internal note. `comment edit` with no text opens the current comment in your editor.

`link A-1 A-2 Blocks` means "A-1 blocks A-2". Inward phrases such as "is blocked by" swap the two issues.

### Projects and planning

```sh
atl jira project list
atl jira component list [-p KEY]
atl jira release list [-p KEY] [--state unreleased]
atl jira board list
atl jira sprint list [--board ID] [--state active,future]
atl jira sprint add SPRINT KEY...
atl jira sprint create NAME [--board ID] [--start DATE] [--end DATE] [--goal TEXT]
atl jira sprint start ID [--end DATE]
atl jira sprint close ID
atl jira epic list | add | remove
atl jira me
```

Dates are `YYYY-MM-DD` or ISO 8601. Jira needs an end date before it can start a sprint.

## Confluence

A page reference (REF) is a page ID, a page URL, or `SPACE:Title`.

### Pages

```sh
atl confluence page get REF [--storage] [--version N] [--front-matter] [--web]
atl confluence page create --space KEY --title T [--parent REF] (--body MD | --file F)
atl confluence page update REF [--title T] [--body MD | --file F]
atl confluence page edit REF
atl confluence page delete REF
atl confluence page move REF --parent REF | --before REF | --after REF
atl confluence page history REF [--limit N]
atl confluence page children REF [-r] [--max-depth N]
atl confluence page find WORDS [--space KEY]
```

`update` bumps the version number. `edit` opens the page in your editor. `.md` files are converted, and `.html` or `.xml` files are sent as storage format. `--before` and `--after` (Cloud only) put a page next to a sibling. Use `page get --version N` to read an old version.

### Docs as code

```sh
atl confluence page export REF [-r] [--dest DIR] [--max-depth N] [--exclude GLOB]... [--skip-attachments]
atl confluence page push PATH... [--space KEY] [--parent REF] [--force] [--dry-run]
atl confluence page copy-tree REF --to REF [--suffix TEXT] [--max-depth N] [--exclude GLOB]
```

`export` writes each page to `<title>/index.md`. The file has front matter, the page's attachments go in `_attachments/`, and child pages become subfolders.

`push` sends the files back:

- It updates each page named in a file's front matter. It skips pages that did not change, and refuses pages that changed on the server after the export, unless you add `--force`.
- New files become new pages under their folder's `index.md`.
- Referenced images are uploaded.
- The new IDs and versions are written back to the files.

`--dry-run` shows the plan without changing anything. `copy-tree` copies titles, bodies and labels.

### Comments, labels and attachments

```sh
atl confluence page comments REF [--inline]
atl confluence page comment REF [TEXT | --body-file F] [--parent ID]
atl confluence comment delete ID
atl confluence page labels REF [--add LABEL]... [--remove LABEL]...
atl confluence page attach REF FILE... [--replace]
atl confluence page attachments REF [--pattern GLOB] [--download] [--dest DIR]
atl confluence attachment download URL [-o FILE]
atl confluence attachment delete ID
```

### Spaces, blogs and search

```sh
atl confluence space list
atl confluence space pages [KEY]
atl confluence space create KEY NAME [--description TEXT]
atl confluence blog list [--space KEY]
atl confluence blog get ID [--web]
atl confluence blog create [--space KEY] --title T [--body MD | --file F]
atl confluence search WORDS [--space KEY] [--type page]
atl confluence search --cql 'CQL'
```

## Bitbucket

`atl` finds the repository in this order: `-R WORKSPACE/REPO` (`PROJECT/REPO` on Data Center), `[bitbucket] repo` in `.atl.toml`, then the git `origin` remote.

```sh
atl bitbucket repo list [--owner WORKSPACE]
atl bitbucket repo view [--web]
atl bitbucket branch list [FILTER]
atl bitbucket branch create NAME [--from REF]
atl bitbucket branch delete NAME
atl bitbucket commit list [-b BRANCH]
```

### Pull requests

```sh
atl bitbucket pr list [-s open|merged|declined|all]
atl bitbucket pr view ID [--web]
atl bitbucket pr diff ID
atl bitbucket pr create -t TITLE [--from BRANCH] [--to BRANCH] [-b MD] [--draft]
atl bitbucket pr ready ID [--undo]
atl bitbucket pr edit ID [-t TITLE] [-b MD] [--to BRANCH]
atl bitbucket pr reviewers ID [--add USER]... [--remove USER]...
atl bitbucket pr comments ID
atl bitbucket pr comment ID TEXT
atl bitbucket pr checks ID
atl bitbucket pr approve ID
atl bitbucket pr unapprove ID
atl bitbucket pr request-changes ID
atl bitbucket pr decline ID [--comment MD]
atl bitbucket pr merge ID [--strategy squash]
atl bitbucket pr checkout ID
```

By default `create` opens a pull request from the current branch to the main branch. `checks` prints the build statuses of the latest commit and exits with 1 if a build failed. `comments` shows general and inline comments with their file and line. `checkout` runs `git fetch` and `git switch` to the source branch (`pr/ID` for forks). Bitbucket Cloud cannot reopen a declined pull request, so `decline` asks first.

### Pipelines (Cloud only)

```sh
atl bitbucket pipeline list
atl bitbucket pipeline view N
atl bitbucket pipeline logs N [--step NAME]
atl bitbucket pipeline run [--branch B] [--custom NAME]
atl bitbucket pipeline stop N
```

N is a build number or a UUID.

## Jira Service Management

JSM commands use your Jira login, on Cloud and on Data Center. DESK is an ID or a project key.

```sh
atl jsm desk list
atl jsm desk types [DESK]
atl jsm queue list [DESK]
atl jsm queue issues QUEUE [--desk DESK]
atl jsm request list [--desk DESK] [-s open|closed|all] [--search WORDS] [--mine]
atl jsm request view KEY
atl jsm request create --desk DESK -t TYPE -s SUMMARY [-b TEXT] [--on-behalf-of EMAIL]
atl jsm request comment KEY TEXT [--internal]
atl jsm request transitions KEY
atl jsm request move KEY TRANSITION
atl jsm request approvals KEY
atl jsm request approve KEY [ID]
atl jsm request decline KEY [ID]
atl jsm request participants KEY [--add USER]... [--remove USER]...
atl jsm request sla KEY
atl jsm request attach KEY FILE... [--internal]
atl jsm request attachments KEY
```

Without an approval ID, `approve` and `decline` answer the one pending approval that you can answer. A participant is `me`, an email or a name. `sla` shows the state of each SLA and the time left.

## Trello

A board is a name, URL, short link or ID. A card is a URL, short link or ID.

```sh
atl trello board list
atl trello list list [-b BOARD]
atl trello card list [-b BOARD] [-l LIST] [--mine]
atl trello card view CARD
atl trello card create [-b BOARD] -l LIST -n NAME [-d MD] [--due DATE]
atl trello card edit CARD [-n NAME] [-d MD] [--due DATE | --no-due] [--done | --undone]
                          [--add-label L] [--remove-label L] [--add-member U] [--remove-member U]
atl trello card move CARD -l LIST
atl trello card comment CARD TEXT
atl trello card archive CARD
atl trello checklist list CARD
atl trello checklist create CARD NAME [--item TEXT]...
atl trello checklist check CARD ITEM
atl trello checklist uncheck CARD ITEM
```

A label is a name or a color. A member is a username, a full name, or `me`. ITEM is a checklist item name or ID.

## Atlassian Admin

Admin commands use an organization API key. `atl` finds the organization from `--org ID|NAME`, then `[admin] org` in `.atl.toml`, then the only organization the key can see.

```sh
atl admin org list
atl admin directory list
atl admin domain list
atl admin user list [-s TEXT]
atl admin user suspend USER [--directory ID]
atl admin user restore USER
atl admin group list [-s TEXT]
atl admin group add-member GROUP USER
atl admin group remove-member GROUP USER
atl admin event list [-s TEXT] [--from DATE] [--to DATE] [--action A]
```

USER is an email or an account ID. GROUP is a name or an ID. `suspend` and `remove-member` ask first.

## Everything else

```sh
atl browse                                  # full-screen browser
atl open [KEY | REF | URL]                  # default: the issue in the branch name
atl alias set mine "jira issue list -a me -s ~Done"
atl alias list
atl alias delete NAME
atl completions bash|zsh|fish|elvish|powershell
atl update
```

Extra words after an alias are added to its command: `atl mine --limit 5`. Aliases are saved in the config file under `[aliases]`, and an alias cannot have the name of a built-in command.

Completion covers the `atl api` commands too: `source <(atl completions zsh)`.

### Updates

In a terminal, `atl` checks GitHub for a new release at most once a day. The check runs in the background, and `atl` prints one line after the command when a new release is available. `atl update` installs it. If you installed with Homebrew, run `brew upgrade racetozero/tap/atl` instead. For other package managers, set `ATL_SELF_UPDATE_COMMAND`, for example `cargo install atl --force`.

The check does not run with `--agent`, in CI, when stderr is not a terminal, or when `ATL_NO_UPDATE_NOTIFIER=1` is set.

### The browser

`atl browse` opens a home screen. It shows your open issues, recent issues, issues you reported, your saved filters, Confluence spaces, a JQL prompt (`Tab` completes field and function names), and an explorer for every API operation.

| Key          | Action                                      |
| ------------ | ------------------------------------------- |
| `↑ ↓`, `j k` | move                                        |
| `g G`        | first, last                                 |
| `Enter`, `v` | open                                        |
| `Esc`        | back                                        |
| `/`          | filter the list                             |
| `m`          | change the status                           |
| `a`          | assign to me                                |
| `o`          | open in the web browser                     |
| `c`          | copy the URL (in the explorer: the command) |
| `n`          | load more                                   |
| `s`          | edit the JQL                                |
| `r`          | refresh                                     |
| `?`          | help                                        |
| `q`          | quit                                        |

In a space, `Enter` opens a page as Markdown and `→` shows its children. For each operation, the API explorer shows the `atl` command, the parameters and the request body schema.

### Output

In a terminal you get text and tables. Long text such as `issue view`, `page get` and `pr view` opens in `$ATL_PAGER`, then `$PAGER`, then `less -FRX`. Use `--no-pager` to turn the pager off.

Set `NO_COLOR=1` (or `TERM=dumb`) to turn off colors and bold text.

When the output is piped, you get JSON. `--format json|ndjson|table|csv|tsv` sets the format. `--select key,summary,fields.status.name` keeps only the fields you name, and `--jq EXPR` filters the output with jq.
