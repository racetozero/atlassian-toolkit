# Signing in and configuration

## The login form

`atl auth login` opens a form in the terminal. "Verify and save" asks the server who you are. If the server rejects the credential, the form shows the server's message and stays open. A `*.atlassian.net` URL is Cloud, and any other URL is Data Center.

## Auth methods

| Product           | Cloud                                                             | Data Center                                                                     |
| ----------------- | ----------------------------------------------------------------- | ------------------------------------------------------------------------------- |
| Jira, Confluence  | email + API token, email + scoped API token, OAuth 2.0, anonymous | personal access token, username + password, session cookie, `.netrc`, anonymous |
| Bitbucket         | email + API token, access token, OAuth 2.0                        | personal access token, username + password, session cookie, `.netrc`, anonymous |
| Trello            | API key + token                                                   | —                                                                               |
| Admin, Admin SCIM | organization API key                                              | —                                                                               |

Scoped API tokens go through `api.atlassian.com/ex/<product>/<cloudId>`. `atl` finds the cloud ID for you.

## Scripts and CI

Use flags instead of the form:

```sh
printf '%s' "$TOKEN" | atl auth login --product jira --url https://jira.corp.example --token-stdin
printf '%s' "$TOKEN" | atl auth login --product cloud-site --url https://acme.atlassian.net --user ada@acme.com --token-stdin
atl auth status            # profiles and masked secrets; --verify asks each server
atl auth default work      # make another profile the default
atl auth logout --product jira
atl auth token jira        # print the token: curl -H "Authorization: Bearer $(atl auth token jira)" ...
```

Or set environment variables. They take priority over the config file:

| Variable                  | Meaning                                                                                                                    |
| ------------------------- | -------------------------------------------------------------------------------------------------------------------------- |
| `ATL_<FAMILY>_URL`        | base URL                                                                                                                   |
| `ATL_<FAMILY>_USER`       | user name or email. With a user, the token is sent as Basic auth; without one, as Bearer. For Trello, this is the API key. |
| `ATL_<FAMILY>_TOKEN`      | token or password. Without one, `atl` uses the `~/.netrc` entry for the host when there is one.                            |
| `ATL_<FAMILY>_DEPLOYMENT` | `cloud` or `data-center`                                                                                                   |
| `ATL_PROFILE`             | the profile to use                                                                                                         |
| `ATL_READ_ONLY=1`         | refuse writes                                                                                                              |

FAMILY is `JIRA`, `CONFLUENCE`, `BITBUCKET`, `TRELLO`, `ADMIN` or `ADMIN_SCIM`. Jira, Jira Software and JSM share the Jira login.

## Read-only logins

`atl auth login --read-only` saves a login that refuses every write except POST searches. You can also set the "Access" field in the form, or set `ATL_READ_ONLY=1`. The check is in the SDK, so the CLI, the MCP server and the SDKs all obey it. A refused write fails with `error.code = "read_only"`.

## OAuth 2.0

Use OAuth if your organization disables API tokens or requires SSO. First, register an OAuth app:

- **Atlassian Cloud:** go to developer.atlassian.com → Console → Create → OAuth 2.0 integration. Set the callback URL to `http://localhost:8976/callback`, add the scopes listed below, and copy the client ID and secret.
- **Bitbucket Cloud:** go to Workspace settings → OAuth consumers → Add consumer. Use the same callback URL.

Then sign in:

```sh
printf '%s' "$CLIENT_SECRET" | atl auth login --method oauth --product cloud-site --user "$CLIENT_ID" --token-stdin
printf '%s' "$KEY_SECRET" | atl auth login --method oauth --product bitbucket --user "$CONSUMER_KEY" --token-stdin
```

`atl` opens the sign-in page, waits for the callback on 127.0.0.1, and saves the tokens. If you have more than one site, add `--url`. `atl` refreshes tokens before they expire and after a 401.

The default scopes are:

- **Jira:** `read:jira-work write:jira-work read:jira-user read:servicedesk-request write:servicedesk-request read:board-scope:jira-software read:sprint:jira-software write:sprint:jira-software read:issue:jira-software write:issue:jira-software`
- **Confluence:** `read:confluence-content.all write:confluence-content read:confluence-space.summary search:confluence read:confluence-user read:page:confluence write:page:confluence read:space:confluence read:comment:confluence write:comment:confluence read:label:confluence write:label:confluence read:attachment:confluence write:attachment:confluence`
- Always: `offline_access`.

`--scopes "..."` replaces the default scopes. `--callback-port` changes the port, and it must match the callback URL of the app. `ATL_OAUTH_CLIENT_SECRET` can hold the client secret instead of stdin.

## Where secrets live

`atl auth login` saves the secret in the config file, `~/.config/atl/config.toml` (or `$XDG_CONFIG_HOME/atl/config.toml`, or `$ATL_CONFIG_DIR/config.toml`). The path is the same on macOS, Linux and Windows.

On macOS and Linux, only your user can read the file (mode 0600). On Windows, the file is in your user profile, which other standard users cannot read.

`atl` does not use the OS keychain. Nothing asks for permission, an agent or a script never waits for a dialog, and a server, a container or an SSH session works the same as a laptop.

To keep a secret out of the file, use the `ATL_<FAMILY>_TOKEN` environment variables or `--method netrc`.

## Data Center behind a gateway

- **Client certificates and private CAs:** `atl auth login ... --client-cert me.pem --client-key me.key --ca-cert corp-ca.pem`. The config file saves only the file paths, and `atl` reads the files on each run, so you can replace a certificate without signing in again. The environment variables are `ATL_<FAMILY>_CLIENT_CERT`, `_CLIENT_KEY` and `_CA_CERT`. The certificate file can also hold the key. `--ca-cert` adds to the system's trusted roots.
- **SSO session cookie:** copy the `Cookie` request header from your browser's developer tools, then run `printf '%s' "$COOKIE" | atl auth login --product confluence --url https://wiki.corp.example --method cookie --token-stdin`. To send a gateway cookie with a token login, set `ATL_<FAMILY>_COOKIE='name=value'`.
- **`.netrc`:** `atl auth login --product jira --url https://jira.corp.example --method netrc` saves no secret. On each run, `atl` reads `machine HOST login USER password TOKEN` from `$NETRC` or `~/.netrc`.
- **Public sites:** `--method none` sends no credential.

## The config file

`atl` uses the first of these that is set: `$ATL_CONFIG_DIR/config.toml`, `$XDG_CONFIG_HOME/atl/config.toml`, `~/.config/atl/config.toml`. It writes the file atomically with mode 0600.

```toml
default_profile = "default"

[profiles.default.jira]
deployment = "cloud"
base_url = "https://acme.atlassian.net"
account = "Ada Lovelace"
auth = { kind = "basic", username = "ada@acme.com", secret = "ATATT3x..." }

[profiles.onprem.confluence]
deployment = "data-center"
base_url = "https://wiki.corp.example"
auth = { kind = "bearer", token = "NjM4..." }

[aliases]
mine = "jira issue list -a me -s ~Done"
```

Each product family has its own login, because Data Center products are separate servers with separate tokens.

## Repository defaults

`atl` reads `.atl.toml` from the current directory or the nearest parent that has one:

```toml
profile = "work"
[jira]
project = "ABC"
[confluence]
space = "DOCS"
[bitbucket]
repo = "acme/website"
[jsm]
desk = "HELP"
[trello]
board = "Roadmap"
[admin]
org = "Acme"
```
