//! Shared parts of the product commands (`atl jira`, `atl confluence`,
//! `atl bitbucket`, `atl jsm`): per-repository defaults, the command context,
//! git lookups, confirmation, the editor, and opening URLs.
//!
//! Product commands run tasks from `atl_sdk::tasks`, so the CLI,
//! the MCP server and the language SDKs return the same results.

use crate::Exit;
use crate::output::{Col, Out};
use anyhow::{Context, Result};
use atl_sdk::confluence::{Confluence, PageRef};
use atl_sdk::jira::{Jira, find_issue_key, is_issue_key};
use atl_sdk::{Client, Error};
use serde::Deserialize;
use serde_json::{Value, json};
use std::io::Read;
use std::path::{Path, PathBuf};

/// Per-repository defaults from `.atl.toml` in the current directory or a
/// parent:
///
/// ```toml
/// profile = "work"
/// [jira]
/// project = "ABC"
/// [confluence]
/// space = "DOCS"
/// [bitbucket]
/// repo = "acme/website"
/// [jsm]
/// desk = "HELP"
/// [trello]
/// board = "Roadmap"
/// [admin]
/// org = "Acme"
/// ```
#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RepoConfig {
    pub profile: Option<String>,
    #[serde(default)]
    jira: Table<Project>,
    #[serde(default)]
    confluence: Table<Space>,
    #[serde(default)]
    bitbucket: Table<Repo>,
    #[serde(default)]
    jsm: Table<Desk>,
    #[serde(default)]
    trello: Table<Board>,
    #[serde(default)]
    admin: Table<Org>,
}

/// One `.atl.toml` table with its single key, so a key in the wrong table
/// (`[jira] desk`) is an error instead of being ignored.
#[derive(Debug, Default, Deserialize)]
#[serde(transparent)]
struct Table<T>(T);

macro_rules! table_key {
    ($($name:ident => $key:ident),*) => {$(
        #[derive(Debug, Default, Deserialize)]
        #[serde(deny_unknown_fields)]
        struct $name {
            $key: Option<String>,
        }
    )*};
}
table_key!(Project => project, Space => space, Repo => repo, Desk => desk, Board => board, Org => org);

/// Each default applies when the command line leaves the value out.
impl RepoConfig {
    pub fn project(&self, flag: Option<String>) -> Option<String> {
        flag.or_else(|| self.jira.0.project.clone())
    }
    pub fn space(&self, flag: Option<String>) -> Option<String> {
        flag.or_else(|| self.confluence.0.space.clone())
    }
    pub fn repo(&self, flag: Option<String>) -> Option<String> {
        flag.or_else(|| self.bitbucket.0.repo.clone())
    }
    pub fn desk(&self, flag: Option<String>) -> Option<String> {
        flag.or_else(|| self.jsm.0.desk.clone())
    }
    pub fn board(&self, flag: Option<String>) -> Option<String> {
        flag.or_else(|| self.trello.0.board.clone())
    }
    pub fn org(&self, flag: Option<String>) -> Option<String> {
        flag.or_else(|| self.admin.0.org.clone())
    }
    pub fn require_space(&self, flag: Option<String>) -> Result<String, Error> {
        self.space(flag).ok_or_else(|| {
            Error::Invalid("pass a space key (or set [confluence] space in .atl.toml)".into())
        })
    }
    pub fn require_desk(&self, flag: Option<String>) -> Result<String, Error> {
        self.desk(flag).ok_or_else(|| {
            Error::Invalid(
                "pass a service desk ID or project key (or set [jsm] desk in .atl.toml); list them with `atl jsm desk list`".into(),
            )
        })
    }

    pub fn discover() -> Result<RepoConfig, Error> {
        let Ok(cwd) = std::env::current_dir() else {
            return Ok(RepoConfig::default());
        };
        let Some(path) = cwd
            .ancestors()
            .map(|dir| dir.join(".atl.toml"))
            .find(|p| p.is_file())
        else {
            return Ok(RepoConfig::default());
        };
        let text = std::fs::read_to_string(&path)
            .map_err(|err| Error::Invalid(format!("cannot read {}: {err}", path.display())))?;
        toml::from_str(&text).map_err(|err| {
            Error::Invalid(format!(
                "{} is invalid ({err}); allowed keys: profile, [jira] project, [confluence] space, [bitbucket] repo, [jsm] desk, [trello] board, [admin] org",
                path.display()
            ))
        })
    }
}

pub struct Ctx<'a> {
    pub client: &'a Client,
    pub out: &'a Out,
    pub repo: &'a RepoConfig,
    pub yes: bool,
    pub dry_run: bool,
    /// Global `-o/--output`.
    pub output: Option<&'a Path>,
}

impl Ctx<'_> {
    /// Runs one SDK task. When the task names a missing or ambiguous
    /// argument with choices and a person can be asked, asks and runs the
    /// task again with the answer.
    pub async fn task(&self, name: &str, args: Value) -> Result<Value> {
        run_task(self.client, name, args).await
    }

    /// Runs a task that returns rows and prints them as a table.
    pub async fn list(&self, name: &str, args: Value, columns: &[Col]) -> Result<Exit> {
        let rows = self.task(name, args).await?;
        self.out.list(rows, columns, Value::Null)
    }

    /// Runs a task that returns one object; `human` is the terminal text.
    pub async fn item(
        &self,
        name: &str,
        args: Value,
        human: impl FnOnce(&Value) -> String,
    ) -> Result<Exit> {
        let value = self.task(name, args).await?;
        self.out.item(value, Value::Null, human)
    }
}

/// Runs one SDK task, asking for a missing argument (see [`Ctx::task`]).
pub async fn run_task(client: &atl_sdk::Client, name: &str, mut args: Value) -> Result<Value> {
    for _ in 0..3 {
        match atl_sdk::tasks::run(client, name, args.clone()).await {
            Err(Error::NeedsInput(need))
                if crate::prompt::allowed()
                    && !need.choices.is_empty()
                    && args.get(&need.arg).is_some() =>
            {
                // The first clause ("pass a status for AB-1"); the list follows.
                let label = need.message.split(';').next().unwrap_or(&need.message);
                args[&need.arg] = crate::prompt::select(label, &need.choices)?.into();
            }
            other => return Ok(other?),
        }
    }
    Ok(atl_sdk::tasks::run(client, name, args).await?)
}

/// The string at `key`, or "" (for terminal text).
pub fn s<'a>(value: &'a Value, key: &str) -> &'a str {
    value.get(key).and_then(Value::as_str).unwrap_or("")
}

/// Inline text, a file, or `-` for stdin.
pub fn read_text(inline: Option<String>, file: Option<&Path>) -> Result<Option<String>, Error> {
    match (inline, file) {
        (Some(_), Some(_)) => Err(Error::Invalid(
            "pass either the text or a file, not both".into(),
        )),
        (Some(text), None) => Ok(Some(text)),
        (None, Some(path)) if path == Path::new("-") => {
            let mut text = String::new();
            std::io::stdin()
                .read_to_string(&mut text)
                .map_err(|err| Error::Invalid(format!("cannot read stdin: {err}")))?;
            Ok(Some(text))
        }
        (None, Some(path)) => std::fs::read_to_string(path).map(Some).map_err(|err| {
            Error::Invalid(format!(
                "cannot read {}: {err}; pass an existing file or - for stdin",
                path.display()
            ))
        }),
        (None, None) => Ok(None),
    }
}

/// True when stdin is a pipe (`cmd | atl ...`) or a redirected file
/// (`atl ... < file`), outside agent mode. Terminals, `/dev/null` and the
/// sockets some agent harnesses pass are never read, so a command cannot
/// wait on a stdin that nobody closes; agents pass `--body-file -`.
pub fn stdin_piped() -> bool {
    if crate::output::agent_mode() {
        return false;
    }
    #[cfg(unix)]
    {
        use std::os::fd::AsFd;
        use std::os::unix::fs::FileTypeExt;
        std::io::stdin()
            .as_fd()
            .try_clone_to_owned()
            .map(std::fs::File::from)
            .and_then(|file| file.metadata())
            .is_ok_and(|meta| meta.file_type().is_fifo() || meta.file_type().is_file())
    }
    #[cfg(not(unix))]
    {
        !std::io::IsTerminal::is_terminal(&std::io::stdin())
    }
}

/// [`read_text`], else piped stdin (see [`stdin_piped`]); an empty pipe is
/// no body.
pub fn read_body(inline: Option<String>, file: Option<&Path>) -> Result<Option<String>, Error> {
    if inline.is_some() || file.is_some() || !stdin_piped() {
        return read_text(inline, file);
    }
    Ok(read_text(None, Some(Path::new("-")))?.filter(|text| !text.trim().is_empty()))
}

/// Output of a git command in the current directory, if it succeeds.
pub fn git(args: &[&str]) -> Option<String> {
    std::process::Command::new("git")
        .args(args)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_owned())
        .filter(|text| !text.is_empty())
}

/// The current git branch (`branch --show-current` also works before the
/// first commit).
pub fn git_branch() -> Option<String> {
    git(&["branch", "--show-current"])
}

/// The issue key from the current git branch (`feature/ABC-12-x` -> `ABC-12`).
pub fn branch_key() -> Result<String, Error> {
    match git_branch() {
        Some(branch) => find_issue_key(&branch).ok_or_else(|| {
            Error::Invalid(format!("pass an issue key; the git branch `{branch}` contains none (name branches like ABC-123-short-title)"))
        }),
        None => Err(Error::Invalid("pass an issue key (not in a git repository, so it cannot come from the branch name)".into())),
    }
}

pub fn key_or_branch(key: Option<String>) -> Result<String, Error> {
    match key {
        Some(key) => Ok(key.to_ascii_uppercase()),
        None => branch_key(),
    }
}

/// `[KEY] VALUE` positionals: two values are KEY and VALUE, one is VALUE.
pub fn key_and_value(mut args: Vec<String>) -> Result<(String, String), Error> {
    let value = args.pop().expect("clap requires one value");
    Ok((key_or_branch(args.pop())?, value))
}

/// `[KEY] [TEXT]` for comments: TEXT, `--body-file`, piped stdin (see
/// [`stdin_piped`]), or the editor at a terminal. One positional is KEY
/// when `is_key` says so, else TEXT; `key_or_default` resolves the key.
pub fn key_and_text(
    mut args: Vec<String>,
    body_file: Option<&Path>,
    is_key: impl Fn(&str) -> bool,
    key_or_default: impl Fn(Option<String>) -> Result<String, Error>,
) -> Result<(String, String)> {
    let text = match args.len() {
        2 => args.pop(),
        1 if !is_key(&args[0]) => args.pop(),
        _ => None,
    };
    if args.len() > 1 || (text.is_some() && body_file.is_some()) {
        return Err(Error::Invalid(
            "pass `[KEY] TEXT`, `[KEY] --body-file <path>`, or pipe the text".into(),
        )
        .into());
    }
    let key = key_or_default(args.pop())?;
    let text = match text {
        Some(text) => text,
        None => match read_body(None, body_file)? {
            Some(text) => text,
            None if crate::prompt::allowed() => edit_in_editor("")?.unwrap_or_default(),
            None => String::new(),
        },
    };
    if text.trim().is_empty() {
        return Err(Error::Invalid(
            "the comment is empty; pass the text, --body-file <path> (- for stdin), or pipe it"
                .into(),
        )
        .into());
    }
    Ok((key, text))
}

/// A destructive action ran without `--yes` where nobody can be asked.
#[derive(Debug)]
pub struct NeedsConfirmation(pub String);

impl std::fmt::Display for NeedsConfirmation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} needs confirmation; pass --yes to proceed", self.0)
    }
}

impl std::error::Error for NeedsConfirmation {}

/// Asks before destructive actions. Agents, scripts and `--no-input` must
/// pass --yes.
pub fn confirm(yes: bool, what: &str) -> Result<()> {
    if yes {
        return Ok(());
    }
    if !crate::prompt::allowed() {
        return Err(NeedsConfirmation(what.to_owned()).into());
    }
    match crate::prompt::confirm(&format!("{what}?"), false)? {
        true => Ok(()),
        false => Err(Error::Invalid(format!("{what}: cancelled")).into()),
    }
}

/// Opens a URL in the default browser. Agents get the URL back instead.
pub fn open_url(out: &Out, url: &str) -> Result<Exit> {
    if out.agent || !out.tty {
        return out.item(json!({ "url": url }), Value::Null, |_| url.to_owned());
    }
    open::that_detached(url).with_context(|| format!("cannot open a browser; visit {url}"))?;
    eprintln!("opened {url}");
    Ok(Exit::Ok)
}

/// Opens `initial` in `$VISUAL`/`$EDITOR` (default `vi`) and returns the
/// edited text, or `None` when nothing changed.
pub fn edit_in_editor(initial: &str) -> Result<Option<String>> {
    if !crate::prompt::allowed() {
        return Err(Error::Invalid("the editor needs a terminal (and no --agent or --no-input); pass the text with --body, --body-file or --file instead".into()).into());
    }
    // `edit` runs $VISUAL or $EDITOR (with arguments such as `code --wait`),
    // or the first common editor it finds, on a temporary `.md` file.
    let edited = edit::edit_with_builder(initial, tempfile::Builder::new().prefix("atl-").suffix(".md"))
        .map_err(|err| {
            Error::Invalid(format!(
                "the editor failed ({err}); nothing was saved. Set $EDITOR (for example `export EDITOR=vim`), or pass the text with --body or --body-file"
            ))
        })?;
    Ok((edited.trim_end() != initial.trim_end()).then_some(edited))
}

/// Writes downloaded bytes to `-o PATH` or `name` and reports it.
pub fn save_download(ctx: &Ctx<'_>, name: &str, bytes: &[u8]) -> Result<Exit> {
    let path = ctx
        .output
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from(name));
    std::fs::write(&path, bytes).with_context(|| format!("cannot write {}", path.display()))?;
    ctx.out.item(
        json!({ "saved": path.display().to_string(), "bytes": bytes.len() }),
        Value::Null,
        |_| format!("Saved {} ({} bytes)", path.display(), bytes.len()),
    )
}

/// `atl open [TARGET]`: an issue key, a page reference, or a URL.
pub async fn run_open(ctx: &Ctx<'_>, target: Option<String>) -> Result<Exit> {
    let target = match target {
        Some(target) => target,
        None => branch_key()?,
    };
    if target.starts_with("http://") || target.starts_with("https://") {
        return open_url(ctx.out, &target);
    }
    if is_issue_key(&target) {
        return open_url(
            ctx.out,
            &Jira::new(ctx.client).browse_url(&target.to_ascii_uppercase()),
        );
    }
    let page = Confluence::new(ctx.client)
        .page(&PageRef::parse(&target)?)
        .await?;
    open_url(ctx.out, &page.url)
}
