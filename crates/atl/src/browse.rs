//! `atl browse` and the interactive `atl jira issue list`: a keyboard-driven,
//! full-screen browser for Jira issues, Confluence pages, and the API
//! catalog.
//!
//! The app is a stack of screens (Enter pushes, Esc pops). Network calls run
//! in background tasks and send their results back through a channel. The
//! event loop sleeps until a key, a result, or a timer (status expiry, the
//! preview debounce) arrives, and redraws only then.

use crate::Exit;
use anyhow::Result;
use atl_sdk::catalog::{self, Catalog, HttpMethod, ServiceDef, TagId};
use atl_sdk::confluence::{Confluence, Page, PageRef, PageSummary, Space};
use atl_sdk::jira::{Filter, IssueDetail, IssuePage, IssueQuery, IssueSummary, Jira, Transition};
use atl_sdk::{Client, Family, schema};
use base64::Engine;
use ratatui::Frame;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block, BorderType, Cell, Clear, Paragraph, Row as TableRow, Table, TableState, Wrap,
};
use std::collections::HashMap;
use std::future::Future;
use std::io::Write;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

const ACCENT: Color = Color::Cyan;
const MUTED: Color = Color::DarkGray;
const SELECTED: Style = Style::new()
    .bg(Color::Rgb(45, 50, 62))
    .add_modifier(Modifier::BOLD);

fn status_color(category: &str) -> Color {
    match category {
        "done" => Color::Green,
        "indeterminate" => Color::Yellow,
        _ => Color::Blue,
    }
}

fn method_color(method: HttpMethod) -> Color {
    match method {
        HttpMethod::Get | HttpMethod::Head | HttpMethod::Options => Color::Green,
        HttpMethod::Post => Color::Yellow,
        HttpMethod::Put => Color::Blue,
        HttpMethod::Patch => Color::Magenta,
        HttpMethod::Delete => Color::Red,
    }
}

// ---------------------------------------------------------------------------
// Lists

/// One row of a selectable list.
trait Row {
    fn cells(&self) -> Vec<Cell<'static>>;
    /// Text the `/` filter matches against.
    fn hay(&self) -> String;
}

struct Picker<T> {
    items: Vec<T>,
    selected: usize,
    filter: String,
    loading: bool,
    error: Option<String>,
    state: TableState,
}

impl<T: Row> Picker<T> {
    fn new(items: Vec<T>) -> Self {
        Self {
            items,
            selected: 0,
            filter: String::new(),
            loading: false,
            error: None,
            state: TableState::default(),
        }
    }

    fn loading() -> Self {
        Self {
            loading: true,
            ..Self::new(Vec::new())
        }
    }

    fn visible(&self) -> Vec<usize> {
        let filter = self.filter.to_lowercase();
        self.items
            .iter()
            .enumerate()
            .filter(|(_, item)| filter.is_empty() || item.hay().to_lowercase().contains(&filter))
            .map(|(index, _)| index)
            .collect()
    }

    fn current(&self) -> Option<&T> {
        self.visible()
            .get(self.selected)
            .map(|&index| &self.items[index])
    }

    fn step(&mut self, delta: isize) {
        let len = self.visible().len() as isize;
        self.selected = if len == 0 {
            0
        } else {
            (self.selected as isize + delta).clamp(0, len - 1) as usize
        };
    }

    fn set(&mut self, result: Result<Vec<T>, String>) {
        self.loading = false;
        match result {
            Ok(items) => {
                self.items = items;
                self.error = None;
            }
            Err(err) => self.error = Some(err),
        }
        self.step(0);
    }

    fn render(
        &mut self,
        frame: &mut Frame,
        area: Rect,
        title: &str,
        header: &[&str],
        widths: &[Constraint],
    ) {
        let visible = self.visible();
        let count = if self.filter.is_empty() {
            format!(" {} ", self.items.len())
        } else {
            format!(
                " {}/{} · /{} ",
                visible.len(),
                self.items.len(),
                self.filter
            )
        };
        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(Style::new().fg(MUTED))
            .title(Line::from(vec![
                Span::styled(format!(" {title} "), Style::new().fg(ACCENT).bold()),
                Span::styled(count, Style::new().fg(MUTED)),
            ]));
        if let Some(message) = self.error.as_ref().filter(|_| self.items.is_empty()) {
            frame.render_widget(
                Paragraph::new(message.as_str())
                    .fg(Color::Red)
                    .wrap(Wrap { trim: false })
                    .block(block),
                area,
            );
            return;
        }
        if self.items.is_empty() {
            let text = if self.loading {
                "Loading…"
            } else {
                "Nothing here."
            };
            frame.render_widget(Paragraph::new(text).fg(MUTED).block(block), area);
            return;
        }
        let rows: Vec<TableRow> = visible
            .iter()
            .map(|&index| TableRow::new(self.items[index].cells()))
            .collect();
        let table = Table::new(rows, widths.to_vec())
            .header(
                TableRow::new(header.iter().map(|h| Cell::from(h.to_string())))
                    .style(Style::new().fg(MUTED).add_modifier(Modifier::BOLD)),
            )
            .row_highlight_style(SELECTED)
            .highlight_symbol("▌ ")
            .block(block);
        self.state.select(Some(self.selected));
        frame.render_stateful_widget(table, area, &mut self.state);
    }
}

impl Row for IssueSummary {
    fn cells(&self) -> Vec<Cell<'static>> {
        vec![
            Cell::from(self.key.clone()).fg(ACCENT),
            Cell::from(self.status.clone()).fg(status_color(&self.status_category)),
            Cell::from(self.assignee.clone().unwrap_or_else(|| "—".into())),
            Cell::from(self.summary.clone()),
        ]
    }
    fn hay(&self) -> String {
        format!(
            "{} {} {} {} {}",
            self.key,
            self.summary,
            self.status,
            self.assignee.as_deref().unwrap_or(""),
            self.issue_type
        )
    }
}

impl Row for Space {
    fn cells(&self) -> Vec<Cell<'static>> {
        vec![
            Cell::from(self.key.clone()).fg(ACCENT),
            Cell::from(self.name.clone()),
            Cell::from(self.space_type.clone().unwrap_or_default()).fg(MUTED),
        ]
    }
    fn hay(&self) -> String {
        format!("{} {}", self.key, self.name)
    }
}

impl Row for PageSummary {
    fn cells(&self) -> Vec<Cell<'static>> {
        vec![
            Cell::from(self.title.clone()),
            Cell::from(self.id.clone()).fg(MUTED),
        ]
    }
    fn hay(&self) -> String {
        self.title.clone()
    }
}

impl Row for Transition {
    fn cells(&self) -> Vec<Cell<'static>> {
        vec![
            Cell::from(self.to.clone()).fg(ACCENT),
            Cell::from(self.name.clone()).fg(MUTED),
        ]
    }
    fn hay(&self) -> String {
        format!("{} {}", self.to, self.name)
    }
}

/// Home menu entries.
#[derive(Clone, Copy)]
enum HomeItem {
    MyIssues,
    Recent,
    Reported,
    Jql,
    Filters,
    Spaces,
    Explorer,
}

impl HomeItem {
    const ALL: [HomeItem; 7] = [
        Self::MyIssues,
        Self::Recent,
        Self::Reported,
        Self::Jql,
        Self::Filters,
        Self::Spaces,
        Self::Explorer,
    ];

    fn label(self) -> (&'static str, &'static str) {
        match self {
            Self::MyIssues => ("My open issues", "Unresolved Jira issues assigned to you"),
            Self::Recent => (
                "Recently updated",
                "Issues anyone updated in the last 7 days",
            ),
            Self::Reported => ("Reported by me", "Unresolved issues you reported"),
            Self::Jql => (
                "Search with JQL…",
                "Type any Jira query (Tab completes field names)",
            ),
            Self::Filters => ("Saved filters", "Run one of your saved Jira filters"),
            Self::Spaces => (
                "Confluence spaces",
                "Browse spaces, page trees, and pages as Markdown",
            ),
            Self::Explorer => (
                "API explorer",
                "Every REST operation of every service, with parameters and schemas",
            ),
        }
    }
}

impl Row for Filter {
    fn cells(&self) -> Vec<Cell<'static>> {
        vec![
            Cell::from(self.name.clone()).bold(),
            Cell::from(self.owner.clone().unwrap_or_default()).fg(MUTED),
            Cell::from(self.jql.clone()).fg(MUTED),
        ]
    }
    fn hay(&self) -> String {
        format!("{} {}", self.name, self.jql)
    }
}

impl Row for HomeItem {
    fn cells(&self) -> Vec<Cell<'static>> {
        let (label, detail) = self.label();
        vec![Cell::from(label).bold(), Cell::from(detail).fg(MUTED)]
    }
    fn hay(&self) -> String {
        let (label, detail) = self.label();
        format!("{label} {detail}")
    }
}

struct ServiceRow {
    def: &'static ServiceDef,
    catalog: Option<&'static Catalog>,
}

impl Row for ServiceRow {
    fn cells(&self) -> Vec<Cell<'static>> {
        let count = self
            .catalog
            .map(|c| c.ops.len().to_string())
            .unwrap_or_else(|| "n/a".into());
        vec![
            Cell::from(self.def.name).fg(ACCENT),
            Cell::from(count).fg(MUTED),
            Cell::from(self.def.title),
        ]
    }
    fn hay(&self) -> String {
        format!("{} {}", self.def.name, self.def.title)
    }
}

struct TagRow {
    catalog: &'static Catalog,
    tag: TagId,
    count: usize,
}

impl Row for TagRow {
    fn cells(&self) -> Vec<Cell<'static>> {
        let tag = self.catalog.tag(self.tag);
        vec![
            Cell::from(tag.name.to_string()).fg(ACCENT),
            Cell::from(self.count.to_string()).fg(MUTED),
            Cell::from(tag.title.to_string()),
        ]
    }
    fn hay(&self) -> String {
        let tag = self.catalog.tag(self.tag);
        format!("{} {}", tag.name, tag.title)
    }
}

struct OpRow {
    service: &'static ServiceDef,
    catalog: &'static Catalog,
    index: usize,
}

impl OpRow {
    fn op(&self) -> &'static atl_sdk::Operation {
        &self.catalog.ops[self.index]
    }
}

impl Row for OpRow {
    fn cells(&self) -> Vec<Cell<'static>> {
        let op = self.op();
        let summary = op.summary.as_deref().unwrap_or("").to_owned();
        vec![
            Cell::from(op.method.as_str())
                .fg(method_color(op.method))
                .bold(),
            Cell::from(op.name.to_string()),
            Cell::from(if op.deprecated {
                format!("(deprecated) {summary}")
            } else {
                summary
            })
            .fg(MUTED),
        ]
    }
    fn hay(&self) -> String {
        let op = self.op();
        format!(
            "{} {} {} {}",
            op.method,
            op.name,
            op.path,
            op.summary.as_deref().unwrap_or("")
        )
    }
}

// ---------------------------------------------------------------------------
// Screens and messages

struct IssuesView {
    title: String,
    jql: String,
    picker: Picker<IssueSummary>,
    next_token: Option<String>,
    details: HashMap<String, Result<IssueDetail, String>>,
}

// A handful of screens live at once; boxing variants would buy nothing.
#[allow(clippy::large_enum_variant)]
enum Screen {
    Home(Picker<HomeItem>),
    Issues(Box<IssuesView>),
    Issue {
        key: String,
        detail: Option<Result<IssueDetail, String>>,
        scroll: u16,
    },
    Spaces(Picker<Space>),
    Filters(Picker<Filter>),
    Pages {
        id: String,
        title: String,
        picker: Picker<PageSummary>,
    },
    Page {
        id: String,
        page: Option<Result<Page, String>>,
        scroll: u16,
    },
    Services(Picker<ServiceRow>),
    Resources {
        service: &'static ServiceDef,
        picker: Picker<TagRow>,
    },
    Operations {
        title: String,
        picker: Picker<OpRow>,
    },
    Operation {
        title: String,
        lines: Vec<Line<'static>>,
        scroll: u16,
    },
}

impl Screen {
    fn title(&self) -> String {
        match self {
            Screen::Home(_) => "Home".into(),
            Screen::Issues(view) => view.title.clone(),
            Screen::Issue { key, .. } => key.clone(),
            Screen::Spaces(_) => "Spaces".into(),
            Screen::Filters(_) => "Saved filters".into(),
            Screen::Pages { title, .. } => title.clone(),
            Screen::Page {
                page: Some(Ok(page)),
                ..
            } => page.title.clone(),
            Screen::Page { id, .. } => format!("Page {id}"),
            Screen::Services(_) => "API explorer".into(),
            Screen::Resources { service, .. } => service.name.into(),
            Screen::Operations { title, .. } | Screen::Operation { title, .. } => title.clone(),
        }
    }

    fn hints(&self) -> &'static str {
        match self {
            Screen::Home(_) => "↑↓ move · Enter open · / filter · ? help · q quit",
            Screen::Issues(_) => {
                "↑↓ move · Enter view · m move · a assign me · o open · c copy · n more · s JQL · r refresh · / filter · Esc back"
            }
            Screen::Issue { .. } => {
                "↑↓ scroll · m move · a assign me · o open · c copy · r refresh · Esc back"
            }
            Screen::Spaces(_) => "↑↓ move · Enter pages · o open · / filter · Esc back",
            Screen::Filters(_) => "↑↓ move · Enter run · / filter · Esc back",
            Screen::Pages { .. } => {
                "↑↓ move · Enter read · → children · o open · c copy · / filter · Esc back"
            }
            Screen::Page { .. } => "↑↓ scroll · PgUp/PgDn page · o open · c copy · Esc back",
            Screen::Services(_) | Screen::Resources { .. } | Screen::Operations { .. } => {
                "↑↓ move · Enter open · / filter · Esc back"
            }
            Screen::Operation { .. } => "↑↓ scroll · c copy command · Esc back",
        }
    }

    fn picker_filter(&mut self) -> Option<&mut String> {
        match self {
            Screen::Home(p) => Some(&mut p.filter),
            Screen::Issues(view) => Some(&mut view.picker.filter),
            Screen::Spaces(p) => Some(&mut p.filter),
            Screen::Filters(p) => Some(&mut p.filter),
            Screen::Pages { picker, .. } => Some(&mut picker.filter),
            Screen::Services(p) => Some(&mut p.filter),
            Screen::Resources { picker, .. } => Some(&mut picker.filter),
            Screen::Operations { picker, .. } => Some(&mut picker.filter),
            _ => None,
        }
    }
}

// One message per finished request; size is irrelevant.
#[allow(clippy::large_enum_variant)]
enum Msg {
    Issues {
        jql: String,
        result: Result<IssuePage, String>,
        /// The page token that was requested; `None` for the first page.
        token: Option<String>,
    },
    Detail {
        key: String,
        result: Result<IssueDetail, String>,
    },
    Transitions {
        key: String,
        result: Result<Vec<Transition>, String>,
    },
    Done {
        message: String,
        key: Option<String>,
    },
    Failed(String),
    Spaces(Result<Vec<Space>, String>),
    Filters(Result<Vec<Filter>, String>),
    JqlFields(Result<Vec<String>, String>),
    Pages {
        id: String,
        result: Result<Vec<PageSummary>, String>,
    },
    Page {
        id: String,
        result: Result<Page, String>,
    },
}

enum Popup {
    Help,
    Jql(String),
    Transitions {
        key: String,
        picker: Picker<Transition>,
    },
}

struct App {
    client: Client,
    stack: Vec<Screen>,
    popup: Option<Popup>,
    filtering: bool,
    status: Option<(String, bool, Instant)>,
    tx: mpsc::UnboundedSender<Msg>,
    quit: bool,
    /// JQL field and function names for Tab completion, loaded on first use.
    jql_words: Option<Vec<String>>,
    /// The issue preview to load once the selection rests on it.
    pending_detail: Option<(String, Instant)>,
}

/// How long a status message stays in the footer.
const STATUS_TIME: Duration = Duration::from_secs(6);
/// The selection must rest this long before its preview loads, so holding
/// an arrow key does not send one request per row.
const PREVIEW_DELAY: Duration = Duration::from_millis(150);

impl App {
    fn spawn(&self, task: impl Future<Output = Msg> + Send + 'static) {
        let tx = self.tx.clone();
        tokio::spawn(async move {
            let _ = tx.send(task.await);
        });
    }

    /// The next time the screen changes without input.
    fn deadline(&self) -> Option<Instant> {
        let status = self
            .status
            .as_ref()
            .map(|(_, _, at)| *at + STATUS_TIME)
            .filter(|at| *at > Instant::now());
        let preview = self.pending_detail.as_ref().map(|(_, at)| *at);
        status.into_iter().chain(preview).min()
    }

    /// Loads the debounced preview if the selection still rests on it.
    fn tick(&mut self) {
        let Some((key, at)) = self.pending_detail.take() else {
            return;
        };
        if at > Instant::now() {
            self.pending_detail = Some((key, at));
            return;
        }
        if let Some(Screen::Issues(view)) = self.stack.last()
            && view.picker.current().is_some_and(|i| i.key == key)
            && !view.details.contains_key(&key)
        {
            self.load_detail(key);
        }
    }

    fn notify(&mut self, message: impl Into<String>, error: bool) {
        self.status = Some((message.into(), error, Instant::now()));
    }

    fn load_issues(&self, jql: String, token: Option<String>) {
        let client = self.client.clone();
        self.spawn(async move {
            let result = Jira::new(&client)
                .search(&jql, 50, token.as_deref())
                .await
                .map_err(|e| e.to_string());
            Msg::Issues { jql, result, token }
        });
    }

    fn load_detail(&self, key: String) {
        let client = self.client.clone();
        self.spawn(async move {
            let result = Jira::new(&client)
                .issue(&key, 10, &[])
                .await
                .map_err(|e| e.to_string());
            Msg::Detail { key, result }
        });
    }

    fn open_issues(&mut self, title: &str, jql: String) {
        self.load_issues(jql.clone(), None);
        self.stack.push(Screen::Issues(Box::new(IssuesView {
            title: title.into(),
            jql,
            picker: Picker::loading(),
            next_token: None,
            details: HashMap::new(),
        })));
    }

    fn open_jql(&mut self, text: String) {
        if self.jql_words.is_none() {
            let client = self.client.clone();
            self.spawn(async move {
                Msg::JqlFields(
                    Jira::new(&client)
                        .jql_reference()
                        .await
                        .map(|r| {
                            r.fields
                                .into_iter()
                                .map(|f| f.name)
                                .chain(r.functions)
                                .collect()
                        })
                        .map_err(|e| e.to_string()),
                )
            });
        }
        self.popup = Some(Popup::Jql(text));
    }

    fn open_page(&mut self, id: String) {
        let client = self.client.clone();
        let page_id = id.clone();
        self.spawn(async move {
            let result = Confluence::new(&client)
                .page(&PageRef::Id(page_id.clone()))
                .await
                .map_err(|e| e.to_string());
            Msg::Page {
                id: page_id,
                result,
            }
        });
        self.stack.push(Screen::Page {
            id,
            page: None,
            scroll: 0,
        });
    }

    fn open_pages(&mut self, id: String, title: String, space: bool) {
        let client = self.client.clone();
        let target = id.clone();
        self.spawn(async move {
            let confluence = Confluence::new(&client);
            let result = if space {
                confluence.root_pages(&target, 200).await
            } else {
                confluence.children(&PageRef::Id(target.clone())).await
            };
            Msg::Pages {
                id: target,
                result: result.map_err(|e| e.to_string()),
            }
        });
        self.stack.push(Screen::Pages {
            id,
            title,
            picker: Picker::loading(),
        });
    }

    /// The issue the current screen is about.
    fn current_issue(&self) -> Option<(String, String)> {
        match self.stack.last()? {
            Screen::Issues(view) => view
                .picker
                .current()
                .map(|i| (i.key.clone(), i.url.clone())),
            Screen::Issue { key, .. } => {
                Some((key.clone(), Jira::new(&self.client).browse_url(key)))
            }
            _ => None,
        }
    }

    fn current_url(&self) -> Option<String> {
        if let Some((_, url)) = self.current_issue() {
            return Some(url);
        }
        match self.stack.last()? {
            Screen::Pages { picker, .. } => picker.current().map(|p| p.url.clone()),
            Screen::Page {
                page: Some(Ok(page)),
                ..
            } => Some(page.url.clone()),
            Screen::Spaces(picker) => picker.current().map(|s| s.url.clone()),
            _ => None,
        }
    }

    fn apply(&mut self, msg: Msg) {
        match msg {
            Msg::Issues { jql, result, token } => {
                let mut wanted = None;
                let mut failed = None;
                for screen in self.stack.iter_mut().rev() {
                    let Screen::Issues(view) = screen else {
                        continue;
                    };
                    if view.jql != jql {
                        continue;
                    }
                    match result {
                        Ok(page) => {
                            view.next_token = page.next_page_token;
                            let mut items = if token.is_some() {
                                std::mem::take(&mut view.picker.items)
                            } else {
                                Vec::new()
                            };
                            items.extend(page.issues);
                            view.picker.set(Ok(items));
                            wanted = view
                                .picker
                                .current()
                                .map(|i| i.key.clone())
                                .filter(|k| !view.details.contains_key(k));
                        }
                        // Keep the issues already shown and the token, so
                        // `n` can retry the page.
                        Err(err) if !view.picker.items.is_empty() => {
                            view.picker.loading = false;
                            view.next_token = token.or(view.next_token.take());
                            failed = Some(err);
                        }
                        Err(err) => view.picker.set(Err(err)),
                    }
                    break;
                }
                if let Some(err) = failed {
                    self.notify(err, true);
                }
                if let Some(key) = wanted {
                    self.load_detail(key);
                }
            }
            Msg::Detail { key, result } => {
                for screen in self.stack.iter_mut() {
                    match screen {
                        Screen::Issues(view) => {
                            view.details.insert(key.clone(), result.clone());
                        }
                        Screen::Issue { key: k, detail, .. } if *k == key => {
                            *detail = Some(result.clone())
                        }
                        _ => {}
                    }
                }
            }
            Msg::Transitions { key, result } => match result {
                Ok(options) if options.is_empty() => {
                    self.notify(format!("{key} has no available transitions"), true)
                }
                // Do not replace a popup the user opened meanwhile (a JQL draft).
                Ok(_) if self.popup.is_some() => {}
                Ok(options) => {
                    self.popup = Some(Popup::Transitions {
                        key,
                        picker: Picker::new(options),
                    })
                }
                Err(err) => self.notify(err, true),
            },
            Msg::Done { message, key } => {
                self.notify(message, false);
                if let Some(key) = key {
                    for screen in self.stack.iter_mut() {
                        if let Screen::Issues(view) = screen {
                            view.details.remove(&key);
                        }
                    }
                    self.load_detail(key);
                    if let Some(jql) = self.stack.iter().rev().find_map(|s| {
                        if let Screen::Issues(v) = s {
                            Some(v.jql.clone())
                        } else {
                            None
                        }
                    }) {
                        self.load_issues(jql, None);
                    }
                }
            }
            Msg::Failed(err) => self.notify(err, true),
            Msg::Filters(result) => {
                if let Some(Screen::Filters(picker)) = self
                    .stack
                    .iter_mut()
                    .rev()
                    .find(|s| matches!(s, Screen::Filters(_)))
                {
                    picker.set(result);
                }
            }
            Msg::JqlFields(result) => match result {
                Ok(words) => self.jql_words = Some(words),
                Err(err) => {
                    // An empty list stops "loading…" and the retry on every open.
                    self.jql_words = Some(Vec::new());
                    self.notify(format!("JQL completion unavailable: {err}"), true)
                }
            },
            Msg::Spaces(result) => {
                if let Some(Screen::Spaces(picker)) = self
                    .stack
                    .iter_mut()
                    .rev()
                    .find(|s| matches!(s, Screen::Spaces(_)))
                {
                    picker.set(result);
                }
            }
            Msg::Pages { id, result } => {
                for screen in self.stack.iter_mut().rev() {
                    if let Screen::Pages {
                        id: target, picker, ..
                    } = screen
                        && *target == id
                    {
                        picker.set(result);
                        break;
                    }
                }
            }
            Msg::Page { id, result } => {
                for screen in self.stack.iter_mut().rev() {
                    if let Screen::Page {
                        id: target, page, ..
                    } = screen
                        && *target == id
                    {
                        *page = Some(result);
                        break;
                    }
                }
            }
        }
    }

    /// After the selection moves, fetch the preview for the new issue.
    fn after_move(&mut self) {
        if let Some(Screen::Issues(view)) = self.stack.last() {
            let wanted = view
                .picker
                .current()
                .map(|i| i.key.clone())
                .filter(|k| !view.details.contains_key(k));
            let near_end = view.picker.selected + 5 >= view.picker.visible().len()
                && view.picker.filter.is_empty();
            let more = view
                .next_token
                .clone()
                .filter(|_| near_end && !view.picker.loading);
            let jql = view.jql.clone();
            if let Some(key) = wanted {
                self.pending_detail = Some((key, Instant::now() + PREVIEW_DELAY));
            }
            if let Some(token) = more {
                if let Some(Screen::Issues(view)) = self.stack.last_mut() {
                    view.picker.loading = true;
                    view.next_token = None;
                }
                self.load_issues(jql, Some(token));
            }
        }
    }

    fn key(&mut self, key: KeyEvent) {
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        if ctrl && key.code == KeyCode::Char('c') {
            self.quit = true;
            return;
        }
        if let Some(popup) = self.popup.take() {
            self.popup_key(popup, key);
            return;
        }
        if self.filtering {
            let Some(filter) = self.stack.last_mut().and_then(Screen::picker_filter) else {
                self.filtering = false;
                return;
            };
            match key.code {
                KeyCode::Esc => {
                    filter.clear();
                    self.filtering = false;
                }
                KeyCode::Enter | KeyCode::Down | KeyCode::Up => self.filtering = false,
                KeyCode::Backspace => {
                    filter.pop();
                }
                KeyCode::Char(c) => filter.push(c),
                _ => {}
            }
            self.with_picker(|step| step(0));
            return;
        }
        match key.code {
            KeyCode::Char('q') => self.quit = true,
            KeyCode::Char('?') => self.popup = Some(Popup::Help),
            KeyCode::Char('/')
                if self
                    .stack
                    .last_mut()
                    .and_then(Screen::picker_filter)
                    .is_some() =>
            {
                self.filtering = true
            }
            KeyCode::Esc | KeyCode::Backspace | KeyCode::Char('h') | KeyCode::Left => {
                let cleared = self
                    .stack
                    .last_mut()
                    .and_then(Screen::picker_filter)
                    .filter(|f| !f.is_empty())
                    .map(String::clear)
                    .is_some();
                if !cleared {
                    if self.stack.len() > 1 {
                        self.stack.pop();
                    } else if key.code == KeyCode::Esc {
                        self.quit = true;
                    }
                }
            }
            KeyCode::Char('o') => match self.current_url() {
                Some(url) => match open::that_detached(&url) {
                    Ok(()) => self.notify(format!("opened {url}"), false),
                    Err(err) => {
                        self.notify(format!("cannot open a browser ({err}); URL: {url}"), true)
                    }
                },
                None => self.notify("nothing to open here", true),
            },
            KeyCode::Char('c') => {
                let text = match self.stack.last() {
                    Some(Screen::Operation { lines, .. }) => lines.iter().find_map(|l| {
                        l.spans
                            .iter()
                            .find(|s| s.content.starts_with("atl "))
                            .map(|s| s.content.to_string())
                    }),
                    _ => self.current_url(),
                };
                match text {
                    Some(text) => {
                        copy(&text);
                        self.notify(format!("copied {text}"), false);
                    }
                    None => self.notify("nothing to copy here", true),
                }
            }
            KeyCode::Char('m') => match self.current_issue() {
                Some((key, _)) => {
                    let client = self.client.clone();
                    self.notify(format!("loading transitions for {key}…"), false);
                    self.spawn(async move {
                        let result = Jira::new(&client)
                            .transitions(&key)
                            .await
                            .map_err(|e| e.to_string());
                        Msg::Transitions { key, result }
                    });
                }
                None => self.notify("select an issue first", true),
            },
            KeyCode::Char('a') => {
                if let Some((key, _)) = self.current_issue() {
                    let client = self.client.clone();
                    self.notify(format!("assigning {key} to you…"), false);
                    self.spawn(async move {
                        match Jira::new(&client).assign(&key, "me").await {
                            Ok(()) => Msg::Done {
                                message: format!("assigned {key} to you"),
                                key: Some(key),
                            },
                            Err(err) => Msg::Failed(err.to_string()),
                        }
                    });
                }
            }
            KeyCode::Char('r') => self.refresh(),
            KeyCode::Char('n') => {
                if let Some(Screen::Issues(view)) = self.stack.last_mut() {
                    match view.next_token.take() {
                        Some(token) => {
                            view.picker.loading = true;
                            let jql = view.jql.clone();
                            self.load_issues(jql, Some(token));
                        }
                        None => self.notify("no more issues", false),
                    }
                }
            }
            KeyCode::Char('s') if matches!(self.stack.last(), Some(Screen::Issues(_))) => {
                let jql = if let Some(Screen::Issues(view)) = self.stack.last() {
                    view.jql.clone()
                } else {
                    String::new()
                };
                self.open_jql(jql);
            }
            KeyCode::Down | KeyCode::Char('j') => self.scroll_or_step(1),
            KeyCode::Up | KeyCode::Char('k') => self.scroll_or_step(-1),
            KeyCode::PageDown | KeyCode::Char('f') if key.code == KeyCode::PageDown || ctrl => {
                self.scroll_or_step(15)
            }
            KeyCode::PageUp | KeyCode::Char('b') if key.code == KeyCode::PageUp || ctrl => {
                self.scroll_or_step(-15)
            }
            KeyCode::Char('g') | KeyCode::Home => self.scroll_or_step(-100_000),
            KeyCode::Char('G') | KeyCode::End => self.scroll_or_step(100_000),
            KeyCode::Enter | KeyCode::Char('v') => self.enter(),
            KeyCode::Right | KeyCode::Char('l') => self.right(),
            _ => {}
        }
    }

    fn popup_key(&mut self, popup: Popup, key: KeyEvent) {
        match popup {
            Popup::Help => {}
            Popup::Jql(mut text) => match key.code {
                KeyCode::Esc => {}
                KeyCode::Enter if !text.trim().is_empty() => {
                    if let Some(Screen::Issues(_)) = self.stack.last() {
                        self.stack.pop();
                    }
                    self.open_issues("JQL", text);
                }
                KeyCode::Backspace => {
                    text.pop();
                    self.popup = Some(Popup::Jql(text));
                }
                KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    self.popup = Some(Popup::Jql(String::new()))
                }
                KeyCode::Tab => {
                    if let Some(word) = jql_matches(&text, self.jql_words.as_deref()).first() {
                        let start = text.len() - last_token(&text).len();
                        text.truncate(start);
                        text.push_str(word);
                        text.push(' ');
                    }
                    self.popup = Some(Popup::Jql(text));
                }
                KeyCode::Char(c) => {
                    text.push(c);
                    self.popup = Some(Popup::Jql(text));
                }
                _ => self.popup = Some(Popup::Jql(text)),
            },
            Popup::Transitions {
                key: issue,
                mut picker,
            } => match key.code {
                KeyCode::Esc | KeyCode::Char('q') => {}
                KeyCode::Down | KeyCode::Char('j') => {
                    picker.step(1);
                    self.popup = Some(Popup::Transitions { key: issue, picker });
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    picker.step(-1);
                    self.popup = Some(Popup::Transitions { key: issue, picker });
                }
                KeyCode::Enter => {
                    if let Some(target) = picker.current().map(|t| t.to.clone()) {
                        let client = self.client.clone();
                        self.notify(format!("moving {issue} to {target}…"), false);
                        self.spawn(async move {
                            match Jira::new(&client)
                                .transition(&issue, &target, None, &Default::default())
                                .await
                            {
                                Ok(t) => Msg::Done {
                                    message: format!("moved {issue} to {}", t.to),
                                    key: Some(issue),
                                },
                                Err(err) => Msg::Failed(err.to_string()),
                            }
                        });
                    }
                }
                _ => self.popup = Some(Popup::Transitions { key: issue, picker }),
            },
        }
    }

    fn with_picker(&mut self, f: impl FnOnce(&mut dyn FnMut(isize))) {
        match self.stack.last_mut() {
            Some(Screen::Home(p)) => f(&mut |d| p.step(d)),
            Some(Screen::Issues(v)) => f(&mut |d| v.picker.step(d)),
            Some(Screen::Spaces(p)) => f(&mut |d| p.step(d)),
            Some(Screen::Filters(p)) => f(&mut |d| p.step(d)),
            Some(Screen::Pages { picker, .. }) => f(&mut |d| picker.step(d)),
            Some(Screen::Services(p)) => f(&mut |d| p.step(d)),
            Some(Screen::Resources { picker, .. }) => f(&mut |d| picker.step(d)),
            Some(Screen::Operations { picker, .. }) => f(&mut |d| picker.step(d)),
            _ => {}
        }
    }

    fn scroll_or_step(&mut self, delta: isize) {
        match self.stack.last_mut() {
            Some(
                Screen::Issue { scroll, .. }
                | Screen::Page { scroll, .. }
                | Screen::Operation { scroll, .. },
            ) => {
                *scroll = (*scroll as isize + delta).clamp(0, u16::MAX as isize) as u16;
            }
            _ => {
                self.with_picker(|step| step(delta));
                self.after_move();
            }
        }
    }

    fn refresh(&mut self) {
        match self.stack.last_mut() {
            Some(Screen::Issues(view)) => {
                view.details.clear();
                view.picker.loading = true;
                let jql = view.jql.clone();
                self.load_issues(jql, None);
            }
            Some(Screen::Issue { key, .. }) => {
                let key = key.clone();
                self.load_detail(key);
            }
            Some(Screen::Page { id, .. }) => {
                let id = id.clone();
                self.stack.pop();
                self.open_page(id);
            }
            _ => {}
        }
    }

    fn enter(&mut self) {
        let Some(top) = self.stack.last() else { return };
        match top {
            Screen::Home(picker) => match picker.current().copied() {
                Some(HomeItem::MyIssues) => {
                    self.open_issues("My open issues", IssueQuery::default().to_jql())
                }
                Some(HomeItem::Recent) => self.open_issues(
                    "Recently updated",
                    "updated >= -7d ORDER BY updated DESC".into(),
                ),
                Some(HomeItem::Reported) => self.open_issues(
                    "Reported by me",
                    "reporter = currentUser() AND resolution = Unresolved ORDER BY updated DESC"
                        .into(),
                ),
                Some(HomeItem::Jql) => self.open_jql(String::new()),
                Some(HomeItem::Filters) => {
                    let client = self.client.clone();
                    self.spawn(async move {
                        Msg::Filters(
                            Jira::new(&client)
                                .filters(None)
                                .await
                                .map_err(|e| e.to_string()),
                        )
                    });
                    self.stack.push(Screen::Filters(Picker::loading()));
                }
                Some(HomeItem::Spaces) => {
                    let client = self.client.clone();
                    self.spawn(async move {
                        Msg::Spaces(
                            Confluence::new(&client)
                                .spaces(250)
                                .await
                                .map_err(|e| e.to_string()),
                        )
                    });
                    self.stack.push(Screen::Spaces(Picker::loading()));
                }
                Some(HomeItem::Explorer) => {
                    let rows = catalog::services()
                        .iter()
                        .map(|def| ServiceRow {
                            def,
                            catalog: self.client.catalog_for(def).ok(),
                        })
                        .collect();
                    self.stack.push(Screen::Services(Picker::new(rows)));
                }
                None => {}
            },
            Screen::Issues(view) => {
                if let Some(issue) = view.picker.current() {
                    let key = issue.key.clone();
                    let detail = view.details.get(&key).cloned();
                    if detail.is_none() {
                        self.load_detail(key.clone());
                    }
                    self.stack.push(Screen::Issue {
                        key,
                        detail,
                        scroll: 0,
                    });
                }
            }
            Screen::Filters(picker) => {
                if let Some(filter) = picker.current() {
                    let (name, jql) = (filter.name.clone(), filter.jql.clone());
                    self.open_issues(&name, jql);
                }
            }
            Screen::Spaces(picker) => {
                if let Some(space) = picker.current() {
                    let (key, title) =
                        (space.key.clone(), format!("{} · {}", space.key, space.name));
                    self.open_pages(key, title, true);
                }
            }
            Screen::Pages { picker, .. } => {
                if let Some(page) = picker.current() {
                    let id = page.id.clone();
                    self.open_page(id);
                }
            }
            Screen::Services(picker) => {
                if let Some(row) = picker.current() {
                    let service = row.def;
                    match row.catalog {
                        Some(catalog) => {
                            let rows = (0..catalog.tags.len())
                                .map(|i| TagId(i as u32))
                                .map(|tag| TagRow {
                                    catalog,
                                    tag,
                                    count: catalog.ops_in(tag).count(),
                                })
                                .collect();
                            self.stack.push(Screen::Resources {
                                service,
                                picker: Picker::new(rows),
                            });
                        }
                        None => self.notify(
                            format!("{} is not available on this deployment", service.name),
                            true,
                        ),
                    }
                }
            }
            Screen::Resources { service, picker } => {
                if let Some(row) = picker.current() {
                    let service: &'static ServiceDef = service;
                    let rows = row
                        .catalog
                        .ops
                        .iter()
                        .enumerate()
                        .filter(|(_, op)| op.tag == row.tag)
                        .map(|(index, _)| OpRow {
                            service,
                            catalog: row.catalog,
                            index,
                        })
                        .collect();
                    let title = format!("{} {}", service.name, row.catalog.tag(row.tag).name);
                    self.stack.push(Screen::Operations {
                        title,
                        picker: Picker::new(rows),
                    });
                }
            }
            Screen::Operations { picker, .. } => {
                if let Some(row) = picker.current() {
                    let title = format!("{} {}", row.service.name, row.op().id);
                    let lines = operation_lines(row.service, row.catalog, row.op());
                    self.stack.push(Screen::Operation {
                        title,
                        lines,
                        scroll: 0,
                    });
                }
            }
            _ => {}
        }
    }

    fn right(&mut self) {
        if let Some(Screen::Pages { picker, .. }) = self.stack.last()
            && let Some(page) = picker.current()
        {
            let (id, title) = (page.id.clone(), page.title.clone());
            self.open_pages(id, title, false);
        } else {
            self.enter();
        }
    }

    fn render(&mut self, frame: &mut Frame) {
        let [top, body, bottom] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .areas(frame.area());
        let crumbs: Vec<String> = self.stack.iter().map(Screen::title).collect();
        let profile = format!(
            "{} · jira {} · confluence {} ",
            self.client.profile(),
            self.client.deployment(Family::Jira),
            self.client.deployment(Family::Confluence)
        );
        let [left, right] = Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Length(profile.len() as u16),
        ])
        .areas(top);
        frame.render_widget(
            Line::from(vec![
                Span::styled(" atl ", Style::new().fg(Color::Black).bg(ACCENT).bold()),
                Span::raw("  "),
                Span::styled(crumbs.join(" › "), Style::new().bold()),
            ]),
            left,
        );
        frame.render_widget(Line::from(profile).fg(MUTED).right_aligned(), right);

        let hints = self.stack.last().map(Screen::hints).unwrap_or("");
        let footer = match &self.status {
            Some((message, error, at)) if at.elapsed() < STATUS_TIME => Line::from(Span::styled(
                format!(" {message}"),
                Style::new().fg(if *error { Color::Red } else { Color::Green }),
            )),
            _ if self.filtering => Line::from(vec![
                Span::styled(" filter /", Style::new().fg(ACCENT)),
                Span::raw(
                    self.stack
                        .last_mut()
                        .and_then(Screen::picker_filter)
                        .cloned()
                        .unwrap_or_default(),
                ),
                Span::raw("▏"),
            ]),
            _ => Line::from(Span::styled(format!(" {hints}"), Style::new().fg(MUTED))),
        };
        frame.render_widget(footer, bottom);

        let client = self.client.clone();
        match self.stack.last_mut() {
            Some(Screen::Home(picker)) => render_home(frame, body, picker, &client),
            Some(Screen::Issues(view)) => render_issues(frame, body, view),
            Some(Screen::Issue {
                key,
                detail,
                scroll,
            }) => {
                let lines = match detail {
                    None => vec![Line::from("Loading…").fg(MUTED)],
                    Some(Err(err)) => vec![Line::from(err.clone()).fg(Color::Red)],
                    Some(Ok(detail)) => issue_lines(detail),
                };
                render_scroll(frame, body, key, lines, scroll);
            }
            Some(Screen::Filters(picker)) => picker.render(
                frame,
                body,
                "Saved filters",
                &["NAME", "OWNER", "JQL"],
                &[
                    Constraint::Percentage(30),
                    Constraint::Length(18),
                    Constraint::Fill(1),
                ],
            ),
            Some(Screen::Spaces(picker)) => picker.render(
                frame,
                body,
                "Spaces",
                &["KEY", "NAME", "TYPE"],
                &[
                    Constraint::Length(14),
                    Constraint::Fill(1),
                    Constraint::Length(12),
                ],
            ),
            Some(Screen::Pages { title, picker, .. }) => picker.render(
                frame,
                body,
                title,
                &["TITLE", "ID"],
                &[Constraint::Fill(1), Constraint::Length(14)],
            ),
            Some(Screen::Page { id, page, scroll }) => {
                let (title, lines) = match page {
                    None => (format!("Page {id}"), vec![Line::from("Loading…").fg(MUTED)]),
                    Some(Err(err)) => (
                        format!("Page {id}"),
                        vec![Line::from(err.clone()).fg(Color::Red)],
                    ),
                    Some(Ok(page)) => {
                        let mut lines = vec![
                            Line::from(page.title.clone()).bold().fg(ACCENT),
                            Line::from(format!(
                                "{} · version {} · {}",
                                page.space.as_deref().unwrap_or(""),
                                page.version,
                                page.url
                            ))
                            .fg(MUTED),
                            Line::default(),
                        ];
                        lines.extend(markdown_lines(&page.markdown));
                        (page.title.clone(), lines)
                    }
                };
                render_scroll(frame, body, &title, lines, scroll);
            }
            Some(Screen::Services(picker)) => picker.render(
                frame,
                body,
                "Services",
                &["SERVICE", "OPS", "COVERS"],
                &[
                    Constraint::Length(15),
                    Constraint::Length(5),
                    Constraint::Fill(1),
                ],
            ),
            Some(Screen::Resources { service, picker }) => {
                let title = format!("{} resources", service.name);
                picker.render(
                    frame,
                    body,
                    &title,
                    &["RESOURCE", "OPS", "TITLE"],
                    &[
                        Constraint::Percentage(35),
                        Constraint::Length(5),
                        Constraint::Fill(1),
                    ],
                );
            }
            Some(Screen::Operations { title, picker }) => picker.render(
                frame,
                body,
                title,
                &["METHOD", "NAME", "SUMMARY"],
                &[
                    Constraint::Length(7),
                    Constraint::Percentage(40),
                    Constraint::Fill(1),
                ],
            ),
            Some(Screen::Operation {
                title,
                lines,
                scroll,
            }) => render_scroll(frame, body, title, lines.clone(), scroll),
            None => {}
        }

        match &mut self.popup {
            Some(Popup::Help) => render_help(frame),
            Some(Popup::Jql(text)) => {
                let area = frame
                    .area()
                    .centered(Constraint::Length(90), Constraint::Length(8));
                frame.render_widget(Clear, area);
                let block = Block::bordered()
                    .border_type(BorderType::Rounded)
                    .border_style(Style::new().fg(ACCENT))
                    .title(" JQL ")
                    .title_bottom(
                        Line::from(" Enter search · Tab complete · Ctrl-U clear · Esc cancel ")
                            .fg(MUTED),
                    );
                let matches = jql_matches(text, self.jql_words.as_deref());
                let hint = match (&self.jql_words, matches.is_empty()) {
                    (None, _) => Line::from("loading field names…").fg(MUTED),
                    (Some(_), true) => Line::default(),
                    (Some(_), false) => Line::from(
                        matches
                            .iter()
                            .take(8)
                            .cloned()
                            .collect::<Vec<_>>()
                            .join("  "),
                    )
                    .fg(MUTED),
                };
                let lines = vec![Line::from(format!("{text}▏")), Line::default(), hint];
                frame.render_widget(
                    Paragraph::new(lines)
                        .wrap(Wrap { trim: false })
                        .block(block),
                    area,
                );
            }
            Some(Popup::Transitions { key, picker }) => {
                let area = frame.area().centered(
                    Constraint::Length(50),
                    Constraint::Length((picker.items.len() as u16 + 4).min(20)),
                );
                frame.render_widget(Clear, area);
                picker.render(
                    frame,
                    area,
                    &format!("Move {key} to"),
                    &["STATUS", "TRANSITION"],
                    &[Constraint::Fill(1), Constraint::Fill(1)],
                );
            }
            None => {}
        }
    }
}

fn render_home(frame: &mut Frame, area: Rect, picker: &mut Picker<HomeItem>, client: &Client) {
    let [menu, info] =
        Layout::horizontal([Constraint::Percentage(60), Constraint::Fill(1)]).areas(area);
    picker.render(
        frame,
        menu,
        "Browse",
        &["", ""],
        &[Constraint::Length(22), Constraint::Fill(1)],
    );
    let mut lines = vec![Line::from("Signed in").bold().fg(ACCENT)];
    for family in [Family::Jira, Family::Confluence, Family::Bitbucket] {
        let text = match client.connection(family) {
            Some(c) => format!(
                "{family}: {} ({})",
                c.account.as_deref().unwrap_or("token"),
                c.base_url
            ),
            None => format!("{family}: not signed in"),
        };
        lines.push(Line::from(text).fg(if client.connection(family).is_some() {
            Color::Reset
        } else {
            MUTED
        }));
    }
    lines.extend([
        Line::default(),
        Line::from("Keys").bold().fg(ACCENT),
        Line::from("↑↓ / j k   move"),
        Line::from("Enter / v  open"),
        Line::from("/          filter the list"),
        Line::from("o / c      open in browser / copy URL"),
        Line::from("Esc        back · q quit · ? help"),
        Line::default(),
        Line::from("Agents: run any command with --agent").fg(MUTED),
    ]);
    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::new().fg(MUTED));
    frame.render_widget(
        Paragraph::new(lines)
            .wrap(Wrap { trim: false })
            .block(block),
        info,
    );
}

fn render_issues(frame: &mut Frame, area: Rect, view: &mut IssuesView) {
    let wide = area.width >= 110;
    let [list, preview] = if wide {
        Layout::horizontal([Constraint::Percentage(55), Constraint::Fill(1)]).areas(area)
    } else {
        [area, Rect::default()]
    };
    let title = if view.next_token.is_some() {
        format!("{} (more: n)", view.title)
    } else {
        view.title.clone()
    };
    view.picker.render(
        frame,
        list,
        &title,
        &["KEY", "STATUS", "ASSIGNEE", "SUMMARY"],
        &[
            Constraint::Length(11),
            Constraint::Length(14),
            Constraint::Length(16),
            Constraint::Fill(1),
        ],
    );
    if !wide {
        return;
    }
    let lines = match view.picker.current().map(|i| (i, view.details.get(&i.key))) {
        None => vec![Line::from(view.jql.clone()).fg(MUTED)],
        Some((issue, None)) => vec![
            Line::from(issue.key.clone()).fg(ACCENT).bold(),
            Line::from(issue.summary.clone()).bold(),
            Line::default(),
            Line::from("Loading…").fg(MUTED),
        ],
        Some((_, Some(Err(err)))) => vec![Line::from(err.clone()).fg(Color::Red)],
        Some((_, Some(Ok(detail)))) => issue_lines(detail),
    };
    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::new().fg(MUTED))
        .title(Line::from(" Preview ").fg(MUTED));
    frame.render_widget(
        Paragraph::new(lines)
            .wrap(Wrap { trim: false })
            .block(block),
        preview,
    );
}

fn render_scroll(
    frame: &mut Frame,
    area: Rect,
    title: &str,
    lines: Vec<Line<'static>>,
    scroll: &mut u16,
) {
    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::new().fg(MUTED))
        .title(Line::from(format!(" {title} ")).fg(ACCENT).bold());
    // Wrapped height, estimated from character counts.
    let inner = area.width.saturating_sub(2).max(1) as usize;
    let wrapped: usize = lines
        .iter()
        .map(|line| line.width().max(1).div_ceil(inner))
        .sum();
    let max = wrapped.saturating_sub(area.height.saturating_sub(2) as usize);
    let paragraph = Paragraph::new(lines).wrap(Wrap { trim: false });
    *scroll = (*scroll).min(max as u16);
    frame.render_widget(paragraph.block(block).scroll((*scroll, 0)), area);
}

fn render_help(frame: &mut Frame) {
    let rows = [
        ("↑↓ j k", "move or scroll"),
        ("g G", "first / last"),
        ("PgUp PgDn", "page"),
        ("Enter v →", "open"),
        ("Esc ← h", "back (Esc on Home quits)"),
        ("/", "filter the current list"),
        ("o", "open in browser"),
        ("c", "copy URL (or command)"),
        ("m", "move issue to another status"),
        ("a", "assign issue to me"),
        ("n", "load more issues"),
        ("s", "edit the JQL"),
        ("r", "refresh"),
        ("q", "quit"),
    ];
    let area = frame.area().centered(
        Constraint::Length(52),
        Constraint::Length(rows.len() as u16 + 4),
    );
    frame.render_widget(Clear, area);
    let lines: Vec<Line> = rows
        .iter()
        .map(|(k, v)| {
            Line::from(vec![
                Span::styled(format!("  {k:<12}"), Style::new().fg(ACCENT).bold()),
                Span::raw(*v),
            ])
        })
        .collect();
    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::new().fg(ACCENT))
        .title(" Keys ")
        .title_bottom(Line::from(" any key closes ").fg(MUTED));
    frame.render_widget(Paragraph::new(lines).block(block), area);
}

fn issue_lines(issue: &IssueDetail) -> Vec<Line<'static>> {
    let s = &issue.summary;
    let mut facts = vec![s.issue_type.clone()];
    facts.extend(s.priority.clone());
    facts.push(format!(
        "assignee {}",
        s.assignee.as_deref().unwrap_or("none")
    ));
    facts.extend(issue.reporter.as_ref().map(|r| format!("reporter {r}")));
    let mut lines = vec![
        Line::from(vec![
            Span::styled(s.key.clone(), Style::new().fg(ACCENT).bold()),
            Span::raw("  "),
            Span::styled(s.summary.clone(), Style::new().bold()),
        ]),
        Line::from(vec![
            Span::styled(
                format!(" {} ", s.status),
                Style::new()
                    .fg(Color::Black)
                    .bg(status_color(&s.status_category)),
            ),
            Span::raw("  "),
            Span::styled(facts.join(" · "), Style::new().fg(MUTED)),
        ]),
    ];
    if !issue.labels.is_empty() || issue.parent.is_some() {
        let mut more = Vec::new();
        more.extend(issue.parent.as_ref().map(|p| format!("parent {p}")));
        if !issue.labels.is_empty() {
            more.push(format!("labels {}", issue.labels.join(", ")));
        }
        lines.push(Line::from(more.join(" · ")).fg(MUTED));
    }
    lines.push(Line::from(s.url.clone()).fg(MUTED));
    lines.push(Line::default());
    if issue.description.trim().is_empty() {
        lines.push(Line::from("(no description)").fg(MUTED));
    } else {
        lines.extend(markdown_lines(&issue.description));
    }
    if !issue.comments.is_empty() {
        lines.push(Line::default());
        lines.push(
            Line::from(format!(
                "Comments ({} of {})",
                issue.comments.len(),
                issue.comment_count
            ))
            .bold()
            .fg(ACCENT),
        );
        for comment in &issue.comments {
            lines.push(Line::default());
            let when: String = comment
                .created
                .as_deref()
                .unwrap_or("")
                .chars()
                .take(10)
                .collect();
            lines.push(Line::from(vec![
                Span::styled(
                    comment.author.clone().unwrap_or_else(|| "unknown".into()),
                    Style::new().bold(),
                ),
                Span::styled(format!(" · {when}"), Style::new().fg(MUTED)),
            ]));
            lines.extend(markdown_lines(&comment.body));
        }
    }
    lines
}

/// Light Markdown styling: headings, code blocks, quotes, list bullets.
fn markdown_lines(text: &str) -> Vec<Line<'static>> {
    let mut in_code = false;
    text.lines()
        .map(|line| {
            if line.trim_start().starts_with("```") {
                in_code = !in_code;
                return Line::from(line.to_owned()).fg(MUTED);
            }
            if in_code {
                return Line::from(format!("  {line}")).fg(Color::Gray);
            }
            if line.starts_with('#') {
                return Line::from(line.trim_start_matches('#').trim().to_owned())
                    .bold()
                    .fg(ACCENT);
            }
            if line.starts_with('>') {
                return Line::from(line.to_owned()).italic().fg(Color::Gray);
            }
            Line::from(line.to_owned())
        })
        .collect()
}

fn operation_lines(
    service: &ServiceDef,
    catalog: &Catalog,
    op: &atl_sdk::Operation,
) -> Vec<Line<'static>> {
    let tag = catalog.tag(op.tag).name.to_string();
    let mut lines = vec![
        Line::from(vec![
            Span::styled(
                format!("{} ", op.method),
                Style::new().fg(method_color(op.method)).bold(),
            ),
            Span::styled(op.path.to_string(), Style::new().bold()),
        ]),
        Line::from(op.summary.clone().unwrap_or_default().to_string()).fg(ACCENT),
        Line::from(format!(
            "operationId {}{}",
            op.id,
            if op.deprecated { " · deprecated" } else { "" }
        ))
        .fg(MUTED),
        Line::default(),
        Line::from("Command").bold().fg(ACCENT),
        Line::from(Span::raw(format!(
            "atl api {} {} {}",
            service.name, tag, op.name
        ))),
        Line::default(),
    ];
    let params = catalog.params_of(op);
    if !params.is_empty() {
        lines.push(Line::from("Parameters").bold().fg(ACCENT));
        for param in params {
            let kind = param.schema_type().unwrap_or_else(|| "string".into());
            lines.push(Line::from(vec![
                Span::styled(format!("  --{}", param.flag), Style::new().bold()),
                Span::styled(
                    format!(
                        "  {} {}{}",
                        param.location.as_str(),
                        kind,
                        if param.required { " · required" } else { "" }
                    ),
                    Style::new().fg(MUTED),
                ),
            ]));
            if let Some(description) = &param.description {
                let first = description.lines().next().unwrap_or("").to_owned();
                lines.push(Line::from(format!("      {first}")).fg(Color::Gray));
            }
        }
        lines.push(Line::default());
    }
    if let Some(body) = &op.body {
        lines.push(
            Line::from(format!(
                "Request body ({}{})",
                body.content_type,
                if body.required { ", required" } else { "" }
            ))
            .bold()
            .fg(ACCENT),
        );
        let text = serde_json::to_string_pretty(&schema::expand(catalog, body.schema, 1))
            .unwrap_or_default();
        lines.extend(
            text.lines()
                .map(|l| Line::from(format!("  {l}")).fg(Color::Gray)),
        );
        lines.push(Line::default());
    }
    if let Some(description) = &op.description {
        lines.push(Line::from("Description").bold().fg(ACCENT));
        lines.extend(markdown_lines(description));
    }
    lines
}

/// Copies through the terminal (OSC 52), which also works over SSH.
fn copy(text: &str) {
    let encoded = base64::engine::general_purpose::STANDARD.encode(text);
    let mut stdout = std::io::stdout();
    let _ = write!(stdout, "\x1b]52;c;{encoded}\x07");
    let _ = stdout.flush();
}

async fn run(mut app: App, mut rx: mpsc::UnboundedReceiver<Msg>) -> Result<Exit> {
    let mut terminal = ratatui::init();
    // crossterm reads block, so a thread forwards terminal events.
    let (events_tx, mut events) = mpsc::unbounded_channel();
    std::thread::spawn(move || {
        while let Ok(event) = event::read() {
            if events_tx.send(event).is_err() {
                break;
            }
        }
    });
    let result: Result<()> = async {
        while !app.quit {
            terminal.draw(|frame| app.render(frame))?;
            let deadline = app.deadline();
            tokio::select! {
                Some(msg) = rx.recv() => app.apply(msg),
                Some(event) = events.recv() => match event {
                    Event::Key(key) if key.kind == KeyEventKind::Press => app.key(key),
                    Event::Key(_) => continue,
                    _ => {} // Resize and focus changes only need a redraw.
                },
                () = tokio::time::sleep_until(deadline.unwrap_or_else(Instant::now).into()), if deadline.is_some() => app.tick(),
            }
        }
        Ok(())
    }
    .await;
    ratatui::restore();
    result.map(|()| Exit::Ok)
}

fn new_app(client: &Client) -> (App, mpsc::UnboundedReceiver<Msg>) {
    let (tx, rx) = mpsc::unbounded_channel();
    (
        App {
            client: client.clone(),
            stack: Vec::new(),
            popup: None,
            filtering: false,
            status: None,
            tx,
            quit: false,
            jql_words: None,
            pending_detail: None,
        },
        rx,
    )
}

/// `atl browse`: the home screen.
pub async fn home(client: &Client) -> Result<Exit> {
    let (mut app, rx) = new_app(client);
    app.stack
        .push(Screen::Home(Picker::new(HomeItem::ALL.to_vec())));
    run(app, rx).await
}

/// Interactive `atl jira issue list`.
pub async fn issues(client: &Client, jql: String) -> Result<Exit> {
    let (mut app, rx) = new_app(client);
    app.open_issues("Issues", jql);
    run(app, rx).await
}

/// The word being typed at the end of a JQL query.
fn last_token(text: &str) -> &str {
    let start = text
        .rfind(|c: char| c.is_whitespace() || "(),=!<>~".contains(c))
        .map_or(0, |i| i + 1);
    &text[start..]
}

/// Field and function names that start with the word being typed.
fn jql_matches(text: &str, words: Option<&[String]>) -> Vec<String> {
    let token = last_token(text).to_lowercase();
    if token.is_empty() {
        return Vec::new();
    }
    words
        .unwrap_or_default()
        .iter()
        .filter(|w| w.to_lowercase().starts_with(&token) && w.len() > token.len())
        .cloned()
        .collect()
}
