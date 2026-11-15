//! How results reach stdout: agent envelopes, JSON, NDJSON, CSV, tables, or
//! human text.
//!
//! Agent mode (`--agent` or `ATL_AGENT=1`) prints exactly one compact JSON
//! document per command: `{"ok":true,"command":...,"data":...,"meta":...}`
//! on success and `{"ok":false,"command":...,"error":{...}}` on failure
//! (also on stdout), so an agent parses one stream.

use crate::Exit;
use anyhow::Result;
use serde::Serialize;
use serde_json::{Map, Value, json};
use std::borrow::Cow;
use std::io::{IsTerminal, Write};
use std::sync::OnceLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum Format {
    /// Human text and tables in a terminal; JSON when piped.
    Auto,
    Json,
    Ndjson,
    Table,
    Csv,
    /// Tab-separated values (tabs and newlines in cells become spaces).
    Tsv,
}

static AGENT: OnceLock<bool> = OnceLock::new();
static COMMAND: OnceLock<String> = OnceLock::new();

/// Decided from argv and ATL_AGENT before anything can fail, so every
/// error in agent mode becomes the JSON envelope.
pub fn set_agent(agent: bool) {
    let _ = AGENT.set(agent);
}

/// The subcommand path (`issue list`), known once clap has parsed.
pub fn set_command(command: String) {
    let _ = COMMAND.set(command);
}

pub fn agent_mode() -> bool {
    AGENT.get().copied().unwrap_or(false)
}

pub fn command_name() -> String {
    COMMAND.get().cloned().unwrap_or_default()
}

/// A table or CSV column: header and the record key it shows.
pub struct Col {
    pub header: Cow<'static, str>,
    pub key: Cow<'static, str>,
    /// Takes the remaining terminal width (one per table).
    pub flex: bool,
}

pub const fn col(header: &'static str, key: &'static str) -> Col {
    Col {
        header: Cow::Borrowed(header),
        key: Cow::Borrowed(key),
        flex: false,
    }
}

pub const fn flex(header: &'static str, key: &'static str) -> Col {
    Col {
        header: Cow::Borrowed(header),
        key: Cow::Borrowed(key),
        flex: true,
    }
}

pub struct Out {
    pub agent: bool,
    pub format: Format,
    pub select: Vec<String>,
    pub raw: bool,
    /// stdout is a terminal.
    pub tty: bool,
    /// Omit the header row of tables, CSV and TSV.
    pub no_headers: bool,
    /// Do not shorten table cells.
    pub no_truncate: bool,
    /// A jq filter applied to the data before printing (`--jq`).
    pub jq: Option<String>,
    /// Page long human text through `$ATL_PAGER`/`$PAGER` (off with `--no-pager`).
    pub pager: bool,
}

impl Out {
    pub fn new(agent: bool, format: Format, select: Vec<String>, raw: bool) -> Self {
        Self {
            agent,
            format,
            select,
            raw,
            tty: std::io::stdout().is_terminal(),
            no_headers: false,
            no_truncate: false,
            jq: None,
            pager: !agent,
        }
    }

    /// True when a full-screen browser may take over the terminal.
    pub fn interactive(&self) -> bool {
        !self.agent && self.format == Format::Auto && self.tty && std::io::stdin().is_terminal()
    }

    /// True when human text (not JSON) should be printed.
    pub fn human(&self) -> bool {
        !self.agent && (self.format == Format::Table || (self.format == Format::Auto && self.tty))
    }

    /// True when a command should print its own text: human output with no
    /// `--select` or `--jq` reshaping the data.
    pub fn wants_text(&self) -> bool {
        self.human() && self.select.is_empty() && self.jq.is_none()
    }

    fn envelope(&self, data: Value, meta: Value) -> Result<Exit> {
        let mut out = json!({ "ok": true, "command": command_name(), "data": data });
        if !meta.is_null() && meta.as_object().is_none_or(|m| !m.is_empty()) {
            out["meta"] = meta;
        }
        print_line(&serde_json::to_string(&out)?);
        Ok(Exit::Ok)
    }

    /// With `--jq`: prints the filter's results (one per line, strings raw;
    /// in agent mode the envelope's data is the result or an array of them).
    fn filtered(&self, filter: &str, data: &Value, meta: Value) -> Result<Exit> {
        let mut results = jq(filter, data)?;
        if self.agent {
            let data = match results.len() {
                1 => results.remove(0),
                _ => Value::Array(results),
            };
            return self.envelope(data, meta);
        }
        for result in results {
            match result {
                Value::String(text) => print_line(&text),
                other => print_line(&serde_json::to_string(&other)?),
            }
        }
        Ok(Exit::Ok)
    }

    fn machine(&self, data: &Value) -> Result<Exit> {
        match (self.format, data) {
            (Format::Ndjson, Value::Array(items)) => {
                for item in items {
                    print_line(&serde_json::to_string(item)?);
                }
            }
            (Format::Ndjson, other) => print_line(&serde_json::to_string(other)?),
            _ => print_line(&serde_json::to_string_pretty(data)?),
        }
        Ok(Exit::Ok)
    }

    /// A list of records (issues, pages, spaces, ...).
    pub fn list(&self, rows: impl Serialize, columns: &[Col], meta: Value) -> Result<Exit> {
        let rows = match serde_json::to_value(rows).expect("rows serialize to JSON") {
            Value::Array(rows) => rows,
            other => vec![other],
        };
        let rows: Vec<Value> = rows
            .into_iter()
            .map(|row| select(row, &self.select))
            .collect();
        if let Some(filter) = &self.jq {
            return self.filtered(filter, &Value::Array(rows), meta);
        }
        if self.agent {
            let mut meta = meta;
            if meta.is_null() {
                meta = json!({});
            }
            meta["count"] = rows.len().into();
            return self.envelope(Value::Array(rows), meta);
        }
        let selected: Vec<Col>;
        let columns = if self.select.is_empty() {
            columns
        } else {
            selected = self
                .select
                .iter()
                .map(|key| Col {
                    header: key.to_ascii_uppercase().into(),
                    key: key.clone().into(),
                    flex: false,
                })
                .collect();
            &selected
        };
        match self.format {
            Format::Csv | Format::Tsv => {
                // TSV has no quoting: tabs and line breaks in a cell become spaces.
                let tsv = self.format == Format::Tsv;
                let mut writer = csv::WriterBuilder::new()
                    .delimiter(if tsv { b'\t' } else { b',' })
                    .quote_style(if tsv {
                        csv::QuoteStyle::Never
                    } else {
                        csv::QuoteStyle::Necessary
                    })
                    .terminator(csv::Terminator::Any(b'\n'))
                    .from_writer(Vec::new());
                let clean = |text: String| match tsv {
                    true => text.replace(['\t', '\n', '\r'], " "),
                    false => text,
                };
                if !self.no_headers {
                    writer.write_record(columns.iter().map(|c| clean(c.header.to_string())))?;
                }
                for row in &rows {
                    writer.write_record(columns.iter().map(|c| clean(cell(row, &c.key))))?;
                }
                let text = String::from_utf8(writer.into_inner()?)?;
                print_line(text.trim_end_matches('\n'));
                Ok(Exit::Ok)
            }
            _ if self.human() => {
                print_table(&rows, columns, self.tty, self.no_headers, self.no_truncate);
                if let Some(hint) = meta.get("next").and_then(Value::as_str) {
                    eprintln!("{}", dim(&format!("more results: {hint}"), self.tty));
                }
                Ok(Exit::Ok)
            }
            _ => self.machine(&Value::Array(rows)),
        }
    }

    /// One record; `human` renders it as text for terminals.
    pub fn item(
        &self,
        data: Value,
        meta: Value,
        human: impl FnOnce(&Value) -> String,
    ) -> Result<Exit> {
        let data = select(data, &self.select);
        if let Some(filter) = &self.jq {
            return self.filtered(filter, &data, meta);
        }
        if self.agent {
            return self.envelope(data, meta);
        }
        if self.wants_text() {
            let text = human(&data);
            let text = text.trim_end();
            if !(self.pager && self.tty && std::io::stdin().is_terminal() && page(text)) {
                print_line(text);
            }
            return Ok(Exit::Ok);
        }
        self.machine(&data)
    }

    /// A raw API response from a generated command.
    pub fn api(&self, data: Value, meta: Value) -> Result<Exit> {
        let data = if self.agent && !self.raw {
            atl_sdk::markup::lean(data)
        } else {
            data
        };
        let data = select(data, &self.select);
        if let Some(filter) = &self.jq {
            return self.filtered(filter, &data, meta);
        }
        if self.agent {
            return self.envelope(data, meta);
        }
        self.machine(&data)
    }
}

/// Runs a jq filter on `input` and returns every result.
pub fn jq(filter: &str, input: &Value) -> Result<Vec<Value>, atl_sdk::Error> {
    use jaq_core::load::{Arena, File, Loader};
    use jaq_core::{Compiler, Ctx, Vars, data, unwrap_valr};
    let invalid = |reason: String| {
        atl_sdk::Error::Invalid(format!(
            "--jq `{filter}` {reason}; use jq syntax such as '.[] | .key' or '.issues | length'"
        ))
    };
    let text = serde_json::to_string(input).unwrap_or_default();
    let input = jaq_json::read::parse_single(text.as_bytes())
        .map_err(|err| invalid(format!("cannot read the data ({err})")))?;
    let defs = jaq_core::defs()
        .chain(jaq_std::defs())
        .chain(jaq_json::defs());
    let funs = jaq_core::funs()
        .chain(jaq_std::funs())
        .chain(jaq_json::funs());
    let loader = Loader::new(defs);
    let arena = Arena::default();
    let modules = loader
        .load(
            &arena,
            File {
                code: filter,
                path: (),
            },
        )
        .map_err(|errs| {
            let kinds: Vec<String> = errs.iter().map(|(_, err)| format!("{err:?}")).collect();
            invalid(format!("does not parse ({})", kinds.join("; ")))
        })?;
    let compiled = Compiler::default()
        .with_funs(funs)
        .compile(modules)
        .map_err(|errs| {
            let kinds: Vec<String> = errs.iter().map(|(_, err)| format!("{err:?}")).collect();
            invalid(format!("does not compile ({})", kinds.join("; ")))
        })?;
    let ctx = Ctx::<data::JustLut<jaq_json::Val>>::new(&compiled.lut, Vars::new([]));
    compiled
        .id
        .run((ctx, input))
        .map(unwrap_valr)
        .map(|result| {
            let value = result.map_err(|err| invalid(format!("failed: {err}")))?;
            serde_json::from_str(&value.to_string())
                .map_err(|err| invalid(format!("produced a value that is not JSON ({err})")))
        })
        .collect()
}

/// Shows `text` in the pager when it is longer than the terminal; false
/// when no pager ran (short text, no pager configured, or it failed).
fn page(text: &str) -> bool {
    let rows = terminal_size().map_or(usize::MAX, |(_, h)| h as usize);
    if text.lines().count() < rows {
        return false;
    }
    let configured = std::env::var("ATL_PAGER")
        .or_else(|_| std::env::var("PAGER"))
        .ok();
    let command = match configured {
        Some(command) => command,
        None if cfg!(unix) => "less -FRX".to_owned(),
        None => return false,
    };
    let command = command.trim().to_owned();
    if command.is_empty() || command == "cat" {
        return false;
    }
    #[cfg(unix)]
    let mut child = std::process::Command::new("sh");
    #[cfg(unix)]
    child.arg("-c").arg(&command);
    #[cfg(not(unix))]
    let mut child = {
        let mut parts = command.split_whitespace();
        let mut child = std::process::Command::new(parts.next().unwrap_or("more"));
        child.args(parts);
        child
    };
    let Ok(mut child) = child.stdin(std::process::Stdio::piped()).spawn() else {
        return false;
    };
    if let Some(mut stdin) = child.stdin.take() {
        // The reader may quit early (`q` in less); that is not an error.
        let _ = writeln!(stdin, "{text}");
    }
    let _ = child.wait();
    true
}

/// Styles Markdown for a terminal: headings, code, bold, links and quotes.
/// The text stays Markdown; without a terminal it is returned unchanged.
pub fn markdown(text: &str, tty: bool) -> String {
    if !styled(tty) {
        return text.to_owned();
    }
    let mut out = Vec::new();
    let mut fenced = false;
    for line in text.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            fenced.toggle();
            out.push(dim(line, true));
        } else if fenced {
            out.push(format!("\x1b[36m{line}\x1b[0m"));
        } else if trimmed.starts_with('#') {
            out.push(format!("\x1b[1;35m{line}\x1b[0m"));
        } else if trimmed.starts_with('>') {
            out.push(dim(line, true));
        } else {
            out.push(inline_markdown(line));
        }
    }
    out.join("\n")
}

fn inline_markdown(line: &str) -> String {
    let mut out = String::new();
    let mut rest = line;
    while let Some((at, mark)) = ["**", "`", "["]
        .iter()
        .filter_map(|mark| rest.find(mark).map(|at| (at, *mark)))
        .min_by_key(|(at, _)| *at)
    {
        out.push_str(&rest[..at]);
        let after = &rest[at + mark.len()..];
        let styled = match mark {
            "**" => after
                .find("**")
                .map(|end| (bold(&after[..end], true), end + 2)),
            "`" => after
                .find('`')
                .map(|end| (format!("\x1b[36m{}\x1b[0m", &after[..end]), end + 1)),
            _ => after.find("](").and_then(|close| {
                let url_end = after[close + 2..].find(')')?;
                let (label, url) = (&after[..close], &after[close + 2..close + 2 + url_end]);
                Some((
                    format!("\x1b[4m{label}\x1b[0m {}", dim(&format!("({url})"), true)),
                    close + 2 + url_end + 1,
                ))
            }),
        };
        match styled {
            Some((text, used)) => {
                out.push_str(&text);
                rest = &after[used..];
            }
            None => {
                out.push_str(mark);
                rest = after;
            }
        }
    }
    out.push_str(rest);
    out
}

pub fn print_line(text: &str) {
    let mut out = std::io::stdout().lock();
    // A closed pipe (`| head`) is not an error worth reporting.
    let _ = writeln!(out, "{text}");
}

/// Keeps only the requested keys (dot paths such as `fields.summary` reach
/// into objects). Arrays are projected item by item.
pub fn select(value: Value, keys: &[String]) -> Value {
    if keys.is_empty() {
        return value;
    }
    match value {
        Value::Array(items) => {
            Value::Array(items.into_iter().map(|item| select(item, keys)).collect())
        }
        Value::Object(map) => {
            let source = Value::Object(map);
            let mut out = Map::new();
            for key in keys {
                let pointer = format!("/{}", key.replace('.', "/"));
                out.insert(
                    key.clone(),
                    source.pointer(&pointer).cloned().unwrap_or(Value::Null),
                );
            }
            Value::Object(out)
        }
        other => other,
    }
}

fn cell(row: &Value, key: &str) -> String {
    match row.pointer(&format!("/{}", key.replace('.', "/"))) {
        None | Some(Value::Null) => String::new(),
        Some(Value::String(text)) => text.clone(),
        Some(Value::Array(items)) => items
            .iter()
            .map(|i| {
                i.as_str()
                    .map(str::to_owned)
                    .unwrap_or_else(|| i.to_string())
            })
            .collect::<Vec<_>>()
            .join(", "),
        Some(other) => other.to_string(),
    }
}

/// Prints rows as an aligned table: one line per row, cells cut with `…`
/// at 40 columns (the flex column takes the rest of the terminal), unless
/// `--no-truncate`.
fn print_table(rows: &[Value], columns: &[Col], tty: bool, no_headers: bool, no_truncate: bool) {
    if rows.is_empty() {
        eprintln!("{}", dim("no results", tty));
        return;
    }
    let width = terminal_size().map_or(120, |(width, _)| width);
    for (index, line) in table_lines(rows, columns, width, no_headers, no_truncate)
        .iter()
        .enumerate()
    {
        match index == 0 && !no_headers {
            true => print_line(&bold(line, tty)),
            false => print_line(line),
        }
    }
}

/// The table as lines without trailing spaces, `width` columns wide.
fn table_lines(
    rows: &[Value],
    columns: &[Col],
    width: u16,
    no_headers: bool,
    no_truncate: bool,
) -> Vec<String> {
    use comfy_table::{CellAlignment, ColumnConstraint, ContentArrangement, Row, Table, Width};
    let mut table = Table::new();
    table
        .load_style(comfy_table::presets::NOTHING)
        .set_truncation_indicator("…")
        .set_content_arrangement(match no_truncate {
            true => ContentArrangement::Disabled,
            false => ContentArrangement::Dynamic,
        })
        .set_width(width);
    if !no_headers {
        table.set_header(columns.iter().map(|c| c.header.as_ref()));
    }
    for row in rows {
        let mut cells = Row::from(columns.iter().map(|c| cell(row, &c.key).replace('\n', " ")));
        cells.max_height(1);
        table.add_row(cells);
    }
    // Columns keep their natural width up to 40; only the flex column
    // shrinks to fit the terminal, and not below 20.
    let natural = table.column_max_content_widths();
    let last = columns.len().saturating_sub(1);
    for (index, column) in table.column_iter_mut().enumerate() {
        let padding = if index == last { 0 } else { 2 };
        column.set_padding((0, padding));
        column.set_cell_alignment(CellAlignment::Left);
        if no_truncate {
            continue;
        }
        column.set_constraint(match columns[index].flex {
            true => ColumnConstraint::LowerBoundary(Width::Fixed(20 + padding)),
            false => {
                let width = Width::Fixed(natural[index].min(40) + padding);
                ColumnConstraint::Boundaries {
                    lower: width,
                    upper: width,
                }
            }
        });
    }
    table
        .lines()
        .map(|line| line.trim_end().to_owned())
        .collect()
}

/// The terminal's columns and rows. Without a terminal on stdout or stderr
/// there is none: crossterm would then run `tput` twice (about 3 ms).
fn terminal_size() -> Option<(u16, u16)> {
    let attached = std::io::stdout().is_terminal() || std::io::stderr().is_terminal();
    attached
        .then(|| ratatui::crossterm::terminal::size().ok())
        .flatten()
}

/// True when output may carry ANSI styles: a terminal, and neither
/// `NO_COLOR` (<https://no-color.org>) nor `TERM=dumb` is set.
fn styled(tty: bool) -> bool {
    static PLAIN: OnceLock<bool> = OnceLock::new();
    tty && !*PLAIN.get_or_init(|| {
        std::env::var_os("NO_COLOR").is_some_and(|value| !value.is_empty())
            || std::env::var_os("TERM").is_some_and(|term| term == "dumb")
    })
}

fn paint(code: &str, text: &str, tty: bool) -> String {
    match styled(tty) {
        true => format!("\x1b[{code}m{text}\x1b[0m"),
        false => text.to_owned(),
    }
}

pub fn bold(text: &str, tty: bool) -> String {
    paint("1", text, tty)
}

pub fn dim(text: &str, tty: bool) -> String {
    paint("2", text, tty)
}

/// Colors a Jira status by its category (`new`, `indeterminate`, `done`).
pub fn status(text: &str, category: &str, tty: bool) -> String {
    if !tty {
        return format!("[{text}]");
    }
    let code = match category {
        "done" => "32",
        "indeterminate" => "33",
        _ => "34",
    };
    paint(code, &format!("[{text}]"), tty)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn widths(lines: &[String]) -> Vec<usize> {
        use unicode_width::UnicodeWidthStr;
        lines.iter().map(|line| line.width()).collect()
    }

    #[test]
    fn tables_align_wide_characters_and_shrink_only_the_flex_column() {
        let rows = vec![
            json!({ "key": "AB-1", "owner": "张伟", "summary": "日本語のタイトルと絵文字 🎉 がある長い説明文です。".repeat(3) }),
            json!({ "key": "AB-22", "owner": "Ada", "summary": "short" }),
        ];
        let columns = [
            col("KEY", "key"),
            col("OWNER", "owner"),
            flex("SUMMARY", "summary"),
        ];
        let lines = table_lines(&rows, &columns, 60, false, false);
        assert_eq!(lines.len(), 3, "{lines:#?}");
        assert!(widths(&lines).iter().all(|w| *w <= 60), "{lines:#?}");
        // The fixed columns keep their width, so the SUMMARY column starts at
        // the same display column on every line.
        use unicode_width::UnicodeWidthStr;
        let start = |line: &str, needle: &str| line[..line.find(needle).expect(needle)].width();
        assert_eq!(start(&lines[0], "SUMMARY"), start(&lines[1], "日本語"));
        assert_eq!(start(&lines[0], "SUMMARY"), start(&lines[2], "short"));
        assert!(lines[1].ends_with('…'), "{}", lines[1]);
        let full = table_lines(&rows, &columns, 60, true, true);
        assert_eq!(full.len(), 2, "no header with --no-headers");
        assert!(
            full[0].contains(&"日本語のタイトルと絵文字 🎉 がある長い説明文です。".repeat(3)),
            "--no-truncate keeps whole cells"
        );
    }

    #[test]
    fn non_flex_columns_stop_at_40_columns() {
        let rows = vec![json!({ "a": "x".repeat(70), "b": "y" })];
        let lines = table_lines(&rows, &[col("A", "a"), col("B", "b")], 200, false, false);
        assert!(
            lines[1].starts_with(&format!("{}…", "x".repeat(39))),
            "{}",
            lines[1]
        );
    }

    #[test]
    fn lean_removes_noise_and_renders_adf() {
        let raw = json!({
            "self": "https://x/rest/api/3/issue/1", "expand": "names", "key": "A-1",
            "fields": {
                "customfield_10001": null,
                "assignee": { "displayName": "Ada", "avatarUrls": { "48x48": "https://x/a.png" } },
                "description": { "type": "doc", "version": 1, "content": [{ "type": "paragraph", "content": [{ "type": "text", "text": "hi" }] }] }
            },
            "_links": { "self": "x", "next": "/n", "webui": "/w" }
        });
        assert_eq!(
            atl_sdk::markup::lean(raw),
            json!({ "key": "A-1", "fields": { "assignee": { "displayName": "Ada" }, "description": "hi" }, "_links": { "next": "/n", "webui": "/w" } })
        );
    }

    #[test]
    fn markdown_styles_headings_code_and_links() {
        let text = "# H\n```\nx\n```\nSee **b**, `c` and [t](u).";
        assert_eq!(markdown(text, false), text);
        let styled = markdown(text, true);
        assert!(
            styled.starts_with("\x1b[1;35m# H\x1b[0m\n\x1b[2m```\x1b[0m\n\x1b[36mx\x1b[0m"),
            "{styled:?}"
        );
        assert!(
            styled.ends_with(
                "See \x1b[1mb\x1b[0m, \x1b[36mc\x1b[0m and \x1b[4mt\x1b[0m \x1b[2m(u)\x1b[0m."
            ),
            "{styled:?}"
        );
        assert_eq!(inline_markdown("a ** b [x"), "a ** b [x");
    }

    #[test]
    fn jq_filters_values_and_reports_bad_filters() {
        assert_eq!(
            jq(".a[] | select(. > 1)", &json!({ "a": [1, 2, 3] })).expect("filter"),
            [json!(2), json!(3)]
        );
        assert_eq!(
            jq(".[0].key", &json!([{ "key": "A-1" }])).expect("key"),
            [json!("A-1")]
        );
        let err = jq(".a[", &json!({})).expect_err("syntax");
        assert!(
            err.to_string().starts_with("--jq `.a[` does not parse"),
            "{err}"
        );
    }

    #[test]
    fn select_projects_records_with_dot_paths() {
        let rows = json!([{ "key": "A-1", "fields": { "summary": "S" }, "x": 1 }]);
        assert_eq!(
            select(
                rows,
                &["key".into(), "fields.summary".into(), "missing".into()]
            ),
            json!([{ "key": "A-1", "fields.summary": "S", "missing": null }])
        );
    }
}
