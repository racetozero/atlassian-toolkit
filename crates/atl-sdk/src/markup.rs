//! Rich-text conversion between Markdown and Atlassian formats.
//!
//! - Atlassian Document Format (ADF) is the JSON tree Jira Cloud uses for
//!   descriptions and comments.
//! - Storage format is the XHTML dialect Confluence stores pages in.
//!
//! Conversions are best effort: unknown ADF nodes keep their text, unknown
//! Confluence macros keep their body text.

use pulldown_cmark::{CodeBlockKind, CowStr, Event, HeadingLevel, Options, Parser, Tag, TagEnd};
use serde_json::{Map, Value, json};

fn parser(markdown: &str) -> Parser<'_> {
    Parser::new_ext(
        markdown,
        Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH | Options::ENABLE_TASKLISTS,
    )
}

// ---------------------------------------------------------------------------
// Markdown -> ADF

struct AdfBuilder {
    /// Open nodes; index 0 is the document. `implicit` marks paragraphs the
    /// builder opened because ADF needs a paragraph where Markdown has bare
    /// text (tight list items, table cells).
    stack: Vec<(Map<String, Value>, bool)>,
    marks: Vec<Value>,
    in_table_head: bool,
}

fn node(kind: &str) -> Map<String, Value> {
    let mut map = Map::new();
    map.insert("type".into(), kind.into());
    map
}

impl AdfBuilder {
    fn kind(&self) -> &str {
        self.stack
            .last()
            .and_then(|(n, _)| n.get("type"))
            .and_then(Value::as_str)
            .unwrap_or("")
    }

    fn open(&mut self, map: Map<String, Value>) {
        self.close_implicit();
        self.stack.push((map, false));
    }

    fn close(&mut self) {
        self.close_implicit();
        self.pop_into_parent();
    }

    fn close_implicit(&mut self) {
        if self.stack.last().is_some_and(|(_, implicit)| *implicit) {
            self.pop_into_parent();
        }
    }

    fn pop_into_parent(&mut self) {
        if self.stack.len() < 2 {
            return;
        }
        let (finished, _) = self.stack.pop().expect("checked length");
        let (parent, _) = self.stack.last_mut().expect("document stays open");
        parent
            .entry("content")
            .or_insert_with(|| Value::Array(Vec::new()))
            .as_array_mut()
            .expect("content is an array")
            .push(Value::Object(finished));
    }

    /// Inline content needs a paragraph-like parent.
    fn ensure_inline_parent(&mut self) {
        if matches!(
            self.kind(),
            "doc" | "listItem" | "tableCell" | "tableHeader" | "blockquote"
        ) {
            self.stack.push((node("paragraph"), true));
        }
    }

    fn push_inline(&mut self, inline: Map<String, Value>) {
        self.ensure_inline_parent();
        let (parent, _) = self.stack.last_mut().expect("document stays open");
        parent
            .entry("content")
            .or_insert_with(|| Value::Array(Vec::new()))
            .as_array_mut()
            .expect("content is an array")
            .push(Value::Object(inline));
    }

    fn text(&mut self, text: &str, code: bool) {
        if text.is_empty() {
            return;
        }
        if self.kind() == "codeBlock" {
            let (block, _) = self.stack.last_mut().expect("code block open");
            block
                .entry("content")
                .or_insert_with(|| json!([]))
                .as_array_mut()
                .expect("array")
                .push(json!({ "type": "text", "text": text }));
            return;
        }
        let mut inline = node("text");
        inline.insert("text".into(), text.into());
        // ADF allows `code` only together with `link`.
        let marks: Vec<Value> = if code {
            std::iter::once(json!({ "type": "code" }))
                .chain(self.marks.iter().filter(|m| m["type"] == "link").cloned())
                .collect()
        } else {
            self.marks.clone()
        };
        if !marks.is_empty() {
            inline.insert("marks".into(), Value::Array(marks));
        }
        self.push_inline(inline);
    }
}

/// Converts Markdown (CommonMark plus tables, strikethrough and task lists)
/// to an ADF document.
pub fn markdown_to_adf(markdown: &str) -> Value {
    let mut doc = node("doc");
    doc.insert("version".into(), 1.into());
    doc.insert("content".into(), json!([]));
    let mut b = AdfBuilder {
        stack: vec![(doc, false)],
        marks: Vec::new(),
        in_table_head: false,
    };
    for event in parser(markdown) {
        match event {
            Event::Start(tag) => match tag {
                Tag::Paragraph => b.open(node("paragraph")),
                Tag::Heading { level, .. } => {
                    let mut heading = node("heading");
                    heading.insert("attrs".into(), json!({ "level": heading_level(level) }));
                    b.open(heading);
                }
                Tag::BlockQuote(_) => b.open(node("blockquote")),
                Tag::CodeBlock(kind) => {
                    let mut block = node("codeBlock");
                    if let CodeBlockKind::Fenced(lang) = kind
                        && let Some(lang) = lang.split_whitespace().next()
                    {
                        block.insert("attrs".into(), json!({ "language": lang }));
                    }
                    b.open(block);
                }
                Tag::List(Some(start)) => {
                    let mut list = node("orderedList");
                    list.insert("attrs".into(), json!({ "order": start }));
                    b.open(list);
                }
                Tag::List(None) => b.open(node("bulletList")),
                Tag::Item => b.open(node("listItem")),
                Tag::Table(_) => {
                    let mut table = node("table");
                    table.insert(
                        "attrs".into(),
                        json!({ "isNumberColumnEnabled": false, "layout": "default" }),
                    );
                    b.open(table);
                }
                Tag::TableHead => {
                    b.in_table_head = true;
                    b.open(node("tableRow"));
                }
                Tag::TableRow => b.open(node("tableRow")),
                Tag::TableCell => b.open(node(if b.in_table_head {
                    "tableHeader"
                } else {
                    "tableCell"
                })),
                Tag::Emphasis => b.marks.push(json!({ "type": "em" })),
                Tag::Strong => b.marks.push(json!({ "type": "strong" })),
                Tag::Strikethrough => b.marks.push(json!({ "type": "strike" })),
                Tag::Link { dest_url, .. } | Tag::Image { dest_url, .. } => {
                    b.marks
                        .push(json!({ "type": "link", "attrs": { "href": dest_url.to_string() } }));
                }
                _ => {}
            },
            Event::End(tag) => match tag {
                TagEnd::Paragraph
                | TagEnd::Heading(_)
                | TagEnd::BlockQuote(_)
                | TagEnd::CodeBlock
                | TagEnd::List(_)
                | TagEnd::Item
                | TagEnd::Table
                | TagEnd::TableRow
                | TagEnd::TableCell => {
                    if tag == TagEnd::CodeBlock {
                        trim_code_newline(&mut b);
                    }
                    b.close();
                }
                TagEnd::TableHead => {
                    b.close();
                    b.in_table_head = false;
                }
                TagEnd::Emphasis
                | TagEnd::Strong
                | TagEnd::Strikethrough
                | TagEnd::Link
                | TagEnd::Image => {
                    b.marks.pop();
                }
                _ => {}
            },
            Event::Text(text) => b.text(&text, false),
            Event::Code(text) => b.text(&text, true),
            Event::Html(text) | Event::InlineHtml(text) => b.text(&text, false),
            Event::SoftBreak => b.text(" ", false),
            Event::HardBreak => b.push_inline(node("hardBreak")),
            Event::Rule => {
                b.close_implicit();
                b.open(node("rule"));
                b.close();
            }
            Event::TaskListMarker(done) => b.text(if done { "[x] " } else { "[ ] " }, false),
            _ => {}
        }
    }
    while b.stack.len() > 1 {
        b.close();
    }
    Value::Object(b.stack.pop().expect("document").0)
}

fn heading_level(level: HeadingLevel) -> u8 {
    level as u8
}

/// pulldown-cmark ends code block text with a newline that ADF would show.
fn trim_code_newline(b: &mut AdfBuilder) {
    if let Some((block, _)) = b.stack.last_mut()
        && let Some(Value::Array(content)) = block.get_mut("content")
        && let Some(Value::String(text)) = content.last_mut().and_then(|t| t.get_mut("text"))
        && text.ends_with('\n')
    {
        text.pop();
    }
}

// ---------------------------------------------------------------------------
// ADF -> Markdown

/// Converts an ADF document (or any ADF node) to Markdown.
pub fn adf_to_markdown(adf: &Value) -> String {
    let mut out = block(adf);
    while out.ends_with('\n') {
        out.pop();
    }
    out
}

fn children(node: &Value) -> &[Value] {
    node.get("content")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default()
}

fn attr<'a>(node: &'a Value, name: &str) -> Option<&'a Value> {
    node.get("attrs").and_then(|attrs| attrs.get(name))
}

fn blocks(nodes: &[Value]) -> String {
    nodes
        .iter()
        .map(block)
        .filter(|text| !text.is_empty())
        .collect::<Vec<_>>()
        .join("\n\n")
}

fn indent_rest(text: &str, prefix: &str) -> String {
    let mut lines = text.lines();
    let mut out = lines.next().unwrap_or("").to_owned();
    for line in lines {
        out.push('\n');
        if !line.is_empty() {
            out.push_str(prefix);
        }
        out.push_str(line);
    }
    out
}

fn block(node: &Value) -> String {
    let kind = node.get("type").and_then(Value::as_str).unwrap_or("");
    match kind {
        "doc" => blocks(children(node)),
        "paragraph" => inlines(children(node)),
        "heading" => {
            let level = attr(node, "level")
                .and_then(Value::as_u64)
                .unwrap_or(1)
                .clamp(1, 6) as usize;
            format!("{} {}", "#".repeat(level), inlines(children(node)))
        }
        "bulletList" | "orderedList" | "taskList" | "decisionList" => {
            let start = attr(node, "order").and_then(Value::as_u64).unwrap_or(1);
            children(node)
                .iter()
                .enumerate()
                .map(|(i, item)| {
                    let marker = match (kind, item.get("type").and_then(Value::as_str)) {
                        ("orderedList", _) => format!("{}.", start + i as u64),
                        (_, Some("taskItem")) => {
                            if attr(item, "state").and_then(Value::as_str) == Some("DONE") {
                                "- [x]".into()
                            } else {
                                "- [ ]".into()
                            }
                        }
                        _ => "-".into(),
                    };
                    let body = match item.get("type").and_then(Value::as_str) {
                        // Task and decision items hold inline content directly.
                        Some("taskItem" | "decisionItem") => inlines(children(item)),
                        // Tight lists: one line per block inside an item.
                        _ => children(item)
                            .iter()
                            .map(block)
                            .filter(|t| !t.is_empty())
                            .collect::<Vec<_>>()
                            .join("\n"),
                    };
                    let pad = " ".repeat(marker.len() + 1);
                    format!("{marker} {}", indent_rest(&body, &pad))
                })
                .collect::<Vec<_>>()
                .join("\n")
        }
        "listItem" | "taskItem" | "decisionItem" => blocks(children(node)),
        "codeBlock" => {
            let lang = attr(node, "language").and_then(Value::as_str).unwrap_or("");
            let code: String = children(node)
                .iter()
                .filter_map(|t| t.get("text").and_then(Value::as_str))
                .collect();
            format!("```{lang}\n{code}\n```")
        }
        "blockquote" => quote(&blocks(children(node))),
        "panel" => {
            let label = attr(node, "panelType")
                .and_then(Value::as_str)
                .unwrap_or("info");
            let mut label = label.to_owned();
            if let Some(first) = label.get_mut(0..1) {
                first.make_ascii_uppercase();
            }
            quote(&format!("**{label}:** {}", blocks(children(node))))
        }
        "rule" => "---".into(),
        "expand" | "nestedExpand" => {
            let title = attr(node, "title")
                .and_then(Value::as_str)
                .unwrap_or("Details");
            format!("**{title}**\n\n{}", blocks(children(node)))
        }
        "table" => table(node),
        "mediaSingle" | "mediaGroup" => children(node)
            .iter()
            .map(block)
            .collect::<Vec<_>>()
            .join("\n"),
        "media" => {
            let name = attr(node, "alt")
                .or_else(|| attr(node, "id"))
                .and_then(Value::as_str)
                .unwrap_or("attachment");
            format!("[media: {name}]")
        }
        "text" | "hardBreak" | "mention" | "emoji" | "inlineCard" | "status" | "date" => {
            inlines(std::slice::from_ref(node))
        }
        "blockCard" | "embedCard" => attr(node, "url")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_owned(),
        _ if children(node)
            .iter()
            .any(|c| c.get("type").and_then(Value::as_str) == Some("text")) =>
        {
            inlines(children(node))
        }
        _ => blocks(children(node)),
    }
}

fn quote(text: &str) -> String {
    text.lines()
        .map(|line| {
            if line.is_empty() {
                ">".to_owned()
            } else {
                format!("> {line}")
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn table(node: &Value) -> String {
    let rows: Vec<Vec<String>> = children(node)
        .iter()
        .map(|row| {
            children(row)
                .iter()
                .map(|cell| {
                    blocks(children(cell))
                        .replace('\n', " ")
                        .replace('|', "\\|")
                })
                .collect()
        })
        .collect();
    let Some(width) = rows.iter().map(Vec::len).max() else {
        return String::new();
    };
    let line = |cells: &[String]| {
        let mut padded: Vec<&str> = cells.iter().map(String::as_str).collect();
        padded.resize(width, "");
        format!("| {} |", padded.join(" | "))
    };
    let mut out = vec![line(&rows[0]), format!("|{}", " --- |".repeat(width))];
    out.extend(rows[1..].iter().map(|row| line(row)));
    out.join("\n")
}

fn inlines(nodes: &[Value]) -> String {
    let mut out = String::new();
    for node in nodes {
        let kind = node.get("type").and_then(Value::as_str).unwrap_or("");
        match kind {
            "text" => {
                let mut text = node
                    .get("text")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_owned();
                let marks = node
                    .get("marks")
                    .and_then(Value::as_array)
                    .map(Vec::as_slice)
                    .unwrap_or_default();
                let has = |name: &str| marks.iter().any(|m| m["type"] == name);
                if has("code") {
                    text = format!("`{text}`");
                }
                if has("strike") {
                    text = format!("~~{text}~~");
                }
                if has("em") {
                    text = format!("*{text}*");
                }
                if has("strong") {
                    text = format!("**{text}**");
                }
                if let Some(href) = marks
                    .iter()
                    .find(|m| m["type"] == "link")
                    .and_then(|m| m.pointer("/attrs/href"))
                    .and_then(Value::as_str)
                {
                    text = format!("[{text}]({href})");
                }
                out.push_str(&text);
            }
            "hardBreak" => out.push('\n'),
            "mention" => {
                let text = attr(node, "text").and_then(Value::as_str).unwrap_or("");
                if text.starts_with('@') {
                    out.push_str(text);
                } else {
                    out.push('@');
                    out.push_str(if text.is_empty() {
                        attr(node, "id").and_then(Value::as_str).unwrap_or("user")
                    } else {
                        text
                    });
                }
            }
            "emoji" => out.push_str(
                attr(node, "text")
                    .or_else(|| attr(node, "shortName"))
                    .and_then(Value::as_str)
                    .unwrap_or(""),
            ),
            "inlineCard" => out.push_str(attr(node, "url").and_then(Value::as_str).unwrap_or("")),
            "status" => {
                out.push('[');
                out.push_str(attr(node, "text").and_then(Value::as_str).unwrap_or(""));
                out.push(']');
            }
            "date" => {
                let millis = attr(node, "timestamp").and_then(|t| {
                    t.as_str()
                        .and_then(|s| s.parse::<i64>().ok())
                        .or_else(|| t.as_i64())
                });
                out.push_str(
                    millis
                        .map(crate::time::utc)
                        .as_deref()
                        .map_or("", |t| &t[..10]),
                );
            }
            "placeholder" => out.push_str(attr(node, "text").and_then(Value::as_str).unwrap_or("")),
            _ => out.push_str(&inlines(children(node))),
        }
    }
    out
}

/// Removes fields that cost tokens and never help an agent: `self`,
/// `expand`, avatar and icon URLs, `_expandable`, `_links` other than
/// `next`/`base`/`webui`, and nulls. ADF documents become Markdown. The CLI's
/// `--agent` output and the MCP server's operation results use it.
pub fn lean(value: Value) -> Value {
    match value {
        Value::Object(map) => {
            if map.get("type").and_then(Value::as_str) == Some("doc")
                && map.contains_key("content")
                && map.contains_key("version")
            {
                return Value::String(adf_to_markdown(&Value::Object(map)));
            }
            Value::Object(
                map.into_iter()
                    .filter(|(key, value)| {
                        !value.is_null()
                            && !matches!(
                                key.as_str(),
                                "self"
                                    | "expand"
                                    | "avatarUrls"
                                    | "iconUrl"
                                    | "_expandable"
                                    | "avatarId"
                                    | "profilePicture"
                            )
                    })
                    .map(|(key, value)| {
                        let value = if key == "_links" {
                            match value {
                                Value::Object(links) => Value::Object(
                                    links
                                        .into_iter()
                                        .filter(|(k, _)| {
                                            matches!(k.as_str(), "next" | "base" | "webui")
                                        })
                                        .collect(),
                                ),
                                other => other,
                            }
                        } else {
                            lean(value)
                        };
                        (key, value)
                    })
                    .collect(),
            )
        }
        Value::Array(items) => Value::Array(items.into_iter().map(lean).collect()),
        other => other,
    }
}

// ---------------------------------------------------------------------------
// Markdown <-> Confluence storage

/// Converts Markdown to Confluence storage format. Fenced code becomes the
/// Confluence code macro; everything else is XHTML.
pub fn markdown_to_storage(markdown: &str) -> String {
    let mut events: Vec<Event> = Vec::new();
    let mut code: Option<(String, String)> = None;
    for event in parser(markdown) {
        match (&mut code, event) {
            (None, Event::Start(Tag::CodeBlock(kind))) => {
                let lang = match kind {
                    CodeBlockKind::Fenced(lang) => {
                        lang.split_whitespace().next().unwrap_or("").to_owned()
                    }
                    CodeBlockKind::Indented => String::new(),
                };
                code = Some((lang, String::new()));
            }
            (Some((_, body)), Event::Text(text)) => body.push_str(&text),
            (Some(_), Event::End(TagEnd::CodeBlock)) => {
                let (lang, body) = code.take().expect("inside code block");
                let language = if lang.is_empty() {
                    String::new()
                } else {
                    format!(
                        "<ac:parameter ac:name=\"language\">{}</ac:parameter>",
                        xml_escape(&lang)
                    )
                };
                let body = body
                    .strip_suffix('\n')
                    .unwrap_or(&body)
                    .replace("]]>", "]]]]><![CDATA[>");
                events.push(Event::Html(CowStr::from(format!(
                    "<ac:structured-macro ac:name=\"code\">{language}<ac:plain-text-body><![CDATA[{body}]]></ac:plain-text-body></ac:structured-macro>\n"
                ))));
            }
            (Some(_), _) => {}
            (None, event) => events.push(event),
        }
    }
    let mut html = String::new();
    pulldown_cmark::html::push_html(&mut html, events.into_iter());
    // Images: a relative path names an attachment of the page; a URL stays a URL.
    let html = replace_blocks(&html, "<img", "/>", |block| {
        let src = attribute(block, "src").unwrap_or_default();
        let alt = attribute(block, "alt").unwrap_or_default();
        let alt = match alt.is_empty() {
            true => String::new(),
            false => format!(" ac:alt=\"{}\"", xml_escape(&alt)),
        };
        if src.contains("://") {
            return Some(format!(
                "<ac:image{alt}><ri:url ri:value=\"{}\" /></ac:image>",
                xml_escape(&src)
            ));
        }
        let decoded = percent_encoding::percent_decode_str(&src).decode_utf8_lossy();
        let name = decoded.rsplit('/').next().unwrap_or_default();
        Some(format!(
            "<ac:image{alt}><ri:attachment ri:filename=\"{}\" /></ac:image>",
            xml_escape(name)
        ))
    });
    html.trim_end().to_owned()
}

fn xml_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Converts Confluence storage format to Markdown. Code macros become fenced
/// code, links to pages become their text, and other macros keep their body.
pub fn storage_to_markdown(storage: &str) -> String {
    let html = unwrap_cdata(storage);
    let html = replace_blocks(
        &html,
        "<ac:structured-macro",
        "</ac:structured-macro>",
        |block| {
            if !block.contains("ac:name=\"code\"") && !block.contains("ac:name=\"noformat\"") {
                return None;
            }
            let lang = between(
                block,
                "<ac:parameter ac:name=\"language\">",
                "</ac:parameter>",
            )
            .unwrap_or("");
            let body =
                between(block, "<ac:plain-text-body>", "</ac:plain-text-body>").unwrap_or("");
            Some(format!(
                "<pre><code class=\"language-{lang}\">{body}</code></pre>"
            ))
        },
    );
    let html = replace_blocks(&html, "<ac:link", "</ac:link>", |block| {
        let body = between(
            block,
            "<ac:plain-text-link-body>",
            "</ac:plain-text-link-body>",
        )
        .or_else(|| between(block, "<ac:link-body>", "</ac:link-body>"))
        .map(str::to_owned)
        .or_else(|| {
            attribute(block, "ri:content-title")
                .or_else(|| attribute(block, "ri:filename"))
                .or_else(|| attribute(block, "ri:userkey"))
        });
        Some(format!("<strong>{}</strong>", body.unwrap_or_default()))
    });
    let html = replace_blocks(&html, "<ac:parameter", "</ac:parameter>", |_| {
        Some(String::new())
    });
    // Images become Markdown images: `![name](<name>)` for an attachment.
    let html = replace_blocks(&html, "<ac:image", "</ac:image>", |block| {
        let src = attribute(block, "ri:filename")
            .or_else(|| attribute(block, "ri:value"))
            .unwrap_or_default();
        let alt = attribute(block, "ri:filename").unwrap_or_default();
        Some(format!(
            "<img src=\"{}\" alt=\"{}\">",
            xml_escape(&src),
            xml_escape(&alt)
        ))
    });
    let converter = htmd::HtmlToMarkdown::builder()
        .options(htmd::options::Options {
            hr_style: htmd::options::HrStyle::Dashes,
            bullet_list_marker: htmd::options::BulletListMarker::Dash,
            ul_bullet_spacing: 1,
            ol_number_spacing: 1,
            ..Default::default()
        })
        .build();
    converter.convert(&html).unwrap_or(html)
}

/// HTML parsers treat CDATA as a comment; turn it into escaped text first.
fn unwrap_cdata(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(start) = rest.find("<![CDATA[") {
        out.push_str(&rest[..start]);
        let after = &rest[start + 9..];
        let end = after.find("]]>").unwrap_or(after.len());
        out.push_str(&xml_escape(&after[..end]));
        rest = after.get(end + 3..).unwrap_or("");
    }
    out.push_str(rest);
    out
}

/// Replaces each `open...close` block (not nested) for which `f` returns a
/// replacement.
fn replace_blocks(
    text: &str,
    open: &str,
    close: &str,
    f: impl Fn(&str) -> Option<String>,
) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(start) = rest.find(open) {
        out.push_str(&rest[..start]);
        let tail = &rest[start..];
        // A self-closing element (`<ac:link ... />`) ends at its own `/>`.
        let tag_end = tail.find('>').map_or(tail.len(), |i| i + 1);
        let end = if tail[..tag_end].ends_with("/>") {
            tag_end
        } else {
            tail.find(close).map_or(tail.len(), |i| i + close.len())
        };
        let block = &tail[..end];
        match f(block) {
            Some(replacement) => out.push_str(&replacement),
            None => out.push_str(block),
        }
        rest = &tail[end..];
    }
    out.push_str(rest);
    out
}

fn between<'a>(text: &'a str, start: &str, end: &str) -> Option<&'a str> {
    let from = text.find(start)? + start.len();
    let to = text[from..].find(end)? + from;
    Some(&text[from..to])
}

fn attribute(text: &str, name: &str) -> Option<String> {
    let value = between(text, &format!("{name}=\""), "\"")?;
    Some(value.replace("&amp;", "&").replace("&quot;", "\""))
}

// ---------------------------------------------------------------------------
// Markdown <-> Jira wiki markup (Jira Data Center rich text)

/// Converts Markdown to Jira wiki markup, the rich-text format of Jira Data
/// Center descriptions and comments.
pub fn markdown_to_wiki(markdown: &str) -> String {
    let mut out = String::new();
    let mut lists: Vec<char> = Vec::new();
    let mut links: Vec<String> = Vec::new();
    let mut in_code = false;
    let mut table_head = false;
    let mut quote_depth = 0usize;
    let block_end = |out: &mut String, lists: &[char]| {
        let want = if lists.is_empty() { "\n\n" } else { "\n" };
        while out.ends_with(' ') {
            out.pop();
        }
        if !out.is_empty() && !out.ends_with(want) {
            out.truncate(out.trim_end_matches('\n').len());
            out.push_str(want);
        }
    };
    for event in parser(markdown) {
        match event {
            Event::Start(tag) => match tag {
                Tag::Heading { level, .. } => out.push_str(&format!("h{}. ", heading_level(level))),
                Tag::BlockQuote(_) => {
                    quote_depth += 1;
                    out.push_str("{quote}\n");
                }
                Tag::CodeBlock(kind) => {
                    in_code = true;
                    match kind {
                        CodeBlockKind::Fenced(lang) if !lang.trim().is_empty() => {
                            out.push_str(&format!(
                                "{{code:{}}}\n",
                                lang.split_whitespace().next().unwrap_or("")
                            ))
                        }
                        _ => out.push_str("{code}\n"),
                    }
                }
                Tag::List(start) => {
                    if !lists.is_empty() && !out.ends_with('\n') {
                        out.push('\n');
                    }
                    lists.push(if start.is_some() { '#' } else { '*' });
                }
                Tag::Item => {
                    if !out.is_empty() && !out.ends_with('\n') {
                        out.push('\n');
                    }
                    out.extend(lists.iter());
                    out.push(' ');
                }
                Tag::TableHead => table_head = true,
                Tag::TableRow => {}
                Tag::TableCell => out.push_str(if table_head { "||" } else { "|" }),
                Tag::Emphasis => out.push('_'),
                Tag::Strong => out.push('*'),
                Tag::Strikethrough => out.push('-'),
                Tag::Link { dest_url, .. } => {
                    out.push('[');
                    links.push(dest_url.to_string());
                }
                Tag::Image { dest_url, .. } => {
                    out.push('!');
                    out.push_str(&dest_url);
                    links.push(String::new());
                }
                _ => {}
            },
            Event::End(tag) => match tag {
                TagEnd::Paragraph | TagEnd::Heading(_) => block_end(&mut out, &lists),
                TagEnd::BlockQuote(_) => {
                    quote_depth = quote_depth.saturating_sub(1);
                    out.truncate(out.trim_end_matches('\n').len());
                    out.push_str("\n{quote}");
                    block_end(&mut out, &lists);
                }
                TagEnd::CodeBlock => {
                    in_code = false;
                    if !out.ends_with('\n') {
                        out.push('\n');
                    }
                    out.push_str("{code}");
                    block_end(&mut out, &lists);
                }
                TagEnd::List(_) => {
                    lists.pop();
                    block_end(&mut out, &lists);
                }
                TagEnd::Item => {}
                TagEnd::TableHead => {
                    out.push_str("||\n");
                    table_head = false;
                }
                TagEnd::TableRow => out.push_str("|\n"),
                TagEnd::Table => block_end(&mut out, &lists),
                TagEnd::Emphasis => out.push('_'),
                TagEnd::Strong => out.push('*'),
                TagEnd::Strikethrough => out.push('-'),
                TagEnd::Link => {
                    let url = links.pop().unwrap_or_default();
                    out.push('|');
                    out.push_str(&url);
                    out.push(']');
                }
                TagEnd::Image => {
                    links.pop();
                    out.push('!');
                }
                _ => {}
            },
            Event::Text(text) if in_code => out.push_str(&text),
            Event::Text(_) if links.last().is_some_and(String::is_empty) => {} // image alt text
            Event::Text(text) => out.push_str(&escape_wiki(&text)),
            Event::Code(text) => {
                out.push_str("{{");
                out.push_str(&text);
                out.push_str("}}");
            }
            Event::Html(text) | Event::InlineHtml(text) => out.push_str(&text),
            Event::SoftBreak => out.push(if quote_depth > 0 { '\n' } else { ' ' }),
            Event::HardBreak => out.push('\n'),
            Event::Rule => {
                out.push_str("----");
                block_end(&mut out, &lists);
            }
            Event::TaskListMarker(done) => out.push_str(if done { "(/) " } else { "(x) " }),
            _ => {}
        }
    }
    out.trim_end().to_owned()
}

fn escape_wiki(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        if matches!(c, '*' | '_' | '{' | '}' | '[' | ']' | '|' | '^' | '+' | '~') {
            out.push('\\');
        }
        out.push(c);
    }
    out
}

/// Converts Jira wiki markup to Markdown.
pub fn wiki_to_markdown(wiki: &str) -> String {
    let mut out: Vec<String> = Vec::new();
    let mut code: Option<Vec<String>> = None;
    let mut quote = false;
    let mut table_started = false;
    for raw in wiki.replace("\r\n", "\n").split('\n') {
        let line = raw.trim_end();
        if let Some(lines) = &mut code {
            if line.trim() == "{code}" || line.trim() == "{noformat}" {
                out.push(std::mem::take(lines).join("\n"));
                out.push("```".into());
                code = None;
            } else {
                lines.push(raw.to_owned());
            }
            continue;
        }
        let trimmed = line.trim_start();
        if trimmed.starts_with("{code") || trimmed.starts_with("{noformat") {
            let lang = trimmed
                .strip_prefix("{code:")
                .and_then(|rest| rest.split(['}', '|']).next())
                .filter(|l| !l.contains('='))
                .unwrap_or("");
            out.push(format!("```{lang}"));
            let after = trimmed.split_once('}').map(|(_, rest)| rest).unwrap_or("");
            // `{code}int x;{code}` opens and closes on one line.
            if let Some(end) = after.find("{code}").or_else(|| after.find("{noformat}")) {
                out.push(after[..end].to_owned());
                out.push("```".into());
                continue;
            }
            code = Some(if after.is_empty() {
                Vec::new()
            } else {
                vec![after.to_owned()]
            });
            continue;
        }
        if trimmed == "{quote}" {
            quote = !quote;
            continue;
        }
        let is_table = trimmed.starts_with('|');
        if !is_table {
            table_started = false;
        }
        let converted = if let Some(rest) = heading(trimmed) {
            rest
        } else if let Some(rest) = trimmed.strip_prefix("bq. ") {
            format!("> {}", wiki_inline(rest))
        } else if trimmed == "----" {
            "---".into()
        } else if is_table {
            let header = trimmed.starts_with("||");
            let cells: Vec<String> = trimmed
                .trim_matches('|')
                .split(if header { "||" } else { "|" })
                .map(|c| wiki_inline(c.trim()))
                .collect();
            let row = format!("| {} |", cells.join(" | "));
            let first = !table_started;
            table_started = true;
            if header || first {
                let separator = format!("|{}", " --- |".repeat(cells.len()));
                if header {
                    format!("{row}\n{separator}")
                } else {
                    format!(
                        "| {} |\n{separator}\n{row}",
                        vec![""; cells.len()].join(" | ")
                    )
                }
            } else {
                row
            }
        } else if let Some((marks, rest)) = list_item(trimmed) {
            let depth = marks.len();
            let bullet = if marks.ends_with('#') { "1." } else { "-" };
            format!("{}{bullet} {}", "  ".repeat(depth - 1), wiki_inline(rest))
        } else {
            wiki_inline(line)
        };
        out.push(if quote {
            format!("> {converted}")
        } else {
            converted
        });
    }
    if let Some(lines) = code {
        out.push(lines.join("\n"));
        out.push("```".into());
    }
    out.join("\n").trim_end().to_owned()
}

fn heading(line: &str) -> Option<String> {
    let bytes = line.as_bytes();
    if bytes.len() > 4
        && bytes[0] == b'h'
        && (b'1'..=b'6').contains(&bytes[1])
        && &line[2..4] == ". "
    {
        let level = (bytes[1] - b'0') as usize;
        return Some(format!("{} {}", "#".repeat(level), wiki_inline(&line[4..])));
    }
    None
}

fn list_item(line: &str) -> Option<(&str, &str)> {
    let marks = line.len() - line.trim_start_matches(['*', '#', '-']).len();
    if marks == 0 || !line[marks..].starts_with(' ') {
        return None;
    }
    // `----` is a rule and `-text-` is strikethrough, not a list.
    if line[..marks].contains('-') && marks > 1 {
        return None;
    }
    Some((&line[..marks], line[marks..].trim_start()))
}

/// Inline wiki markup: `*bold*`, `_italic_`, `-strike-`, `{{code}}`,
/// `[text|url]`, `[url]`, `!image!`, and backslash escapes.
fn wiki_inline(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len());
    let mut i = 0;
    let boundary = |i: usize| i == 0 || !chars[i - 1].is_alphanumeric();
    let closes = |i: usize, mark: char| -> Option<usize> {
        let end = (i + 2..chars.len()).find(|&j| chars[j] == mark && chars[j - 1] != ' ')?;
        let after_ok = end + 1 == chars.len() || !chars[end + 1].is_alphanumeric();
        (after_ok && chars[i + 1] != ' ').then_some(end)
    };
    while i < chars.len() {
        let c = chars[i];
        if c == '\\' && i + 1 < chars.len() {
            out.push(chars[i + 1]);
            i += 2;
            continue;
        }
        if c == '{' && chars.get(i + 1) == Some(&'{') {
            let rest: String = chars[i + 2..].iter().collect();
            if let Some(end) = rest.find("}}") {
                out.push('`');
                out.push_str(&rest[..end]);
                out.push('`');
                i += 2 + rest[..end].chars().count() + 2;
                continue;
            }
        }
        if c == '[' {
            let rest: String = chars[i + 1..].iter().collect();
            if let Some(end) = rest.find(']') {
                let inner = &rest[..end];
                match inner.split_once('|') {
                    Some((label, url)) => {
                        out.push_str(&format!("[{}]({})", wiki_inline(label), url.trim()))
                    }
                    None if inner.contains("://") => out.push_str(&format!("<{inner}>")),
                    None => out.push_str(&format!("[{inner}]")),
                }
                i += 1 + inner.chars().count() + 1;
                continue;
            }
        }
        if c == '!' {
            let rest: String = chars[i + 1..].iter().collect();
            if let Some(end) = rest
                .find('!')
                .filter(|&e| e > 0 && !rest[..e].contains(' '))
            {
                let target = rest[..end].split('|').next().unwrap_or("");
                out.push_str(&format!("![]({target})"));
                i += 1 + rest[..end].chars().count() + 1;
                continue;
            }
        }
        if matches!(c, '*' | '_' | '-')
            && boundary(i)
            && let Some(end) = closes(i, c)
        {
            let inner: String = chars[i + 1..end].iter().collect();
            let wrap = match c {
                '*' => "**",
                '_' => "*",
                _ => "~~",
            };
            out.push_str(wrap);
            out.push_str(&wiki_inline(&inner));
            out.push_str(wrap);
            i = end + 1;
            continue;
        }
        out.push(c);
        i += 1;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn images_round_trip_between_markdown_and_storage() {
        let storage = markdown_to_storage(
            "See ![my d.png](<_attachments/my d.png>) and ![](https://e.com/a.png)",
        );
        assert_eq!(
            storage,
            "<p>See <ac:image ac:alt=\"my d.png\"><ri:attachment ri:filename=\"my d.png\" /></ac:image> and <ac:image><ri:url ri:value=\"https://e.com/a.png\" /></ac:image></p>"
        );
        assert_eq!(
            storage_to_markdown(&storage),
            "See ![my d.png](<my d.png>) and ![](https://e.com/a.png)"
        );
    }

    #[test]
    fn markdown_becomes_valid_adf() {
        let adf = markdown_to_adf(
            "# Title\n\nSome **bold** and `code` with [link](https://x.io).\n\n- one\n- two\n  1. nested\n\n```rust\nfn main() {}\n```\n\n| a | b |\n|---|---|\n| 1 | 2 |\n",
        );
        let content = adf["content"].as_array().expect("blocks");
        let kinds: Vec<&str> = content
            .iter()
            .map(|n| n["type"].as_str().unwrap_or(""))
            .collect();
        assert_eq!(
            kinds,
            ["heading", "paragraph", "bulletList", "codeBlock", "table"]
        );
        assert_eq!(content[0]["attrs"]["level"], 1);
        let para = content[1]["content"].as_array().expect("inline");
        assert_eq!(
            para[1],
            json!({ "type": "text", "text": "bold", "marks": [{ "type": "strong" }] })
        );
        assert_eq!(
            para[3],
            json!({ "type": "text", "text": "code", "marks": [{ "type": "code" }] })
        );
        assert_eq!(para[5]["marks"][0]["attrs"]["href"], "https://x.io");
        // Tight list items get the paragraph ADF requires; nesting survives.
        let first = &content[2]["content"][0];
        assert_eq!(first["content"][0]["type"], "paragraph");
        let second = &content[2]["content"][1]["content"];
        assert_eq!(second[1]["type"], "orderedList");
        assert_eq!(
            content[3],
            json!({ "type": "codeBlock", "attrs": { "language": "rust" }, "content": [{ "type": "text", "text": "fn main() {}" }] })
        );
        assert_eq!(
            content[4]["content"][0]["content"][0]["type"],
            "tableHeader"
        );
        assert_eq!(
            content[4]["content"][1]["content"][0]["content"][0]["type"],
            "paragraph"
        );
    }

    #[test]
    fn adf_round_trips_to_the_same_markdown() {
        let markdown = "## Title\n\nSome **bold** and `code` with [link](https://x.io).\n\n- one\n- two\n  1. nested\n\n```rust\nfn main() {}\n```\n\n> quoted\n\n---\n\n| a | b |\n| --- | --- |\n| 1 | 2 |";
        assert_eq!(adf_to_markdown(&markdown_to_adf(markdown)), markdown);
    }

    #[test]
    fn adf_only_nodes_render_as_readable_markdown() {
        let adf = json!({ "type": "doc", "version": 1, "content": [
            { "type": "paragraph", "content": [
                { "type": "mention", "attrs": { "id": "abc", "text": "@Ada" } },
                { "type": "text", "text": " see " },
                { "type": "inlineCard", "attrs": { "url": "https://acme.atlassian.net/browse/A-1" } },
                { "type": "text", "text": " " },
                { "type": "status", "attrs": { "text": "BLOCKED" } },
                { "type": "text", "text": " by " },
                { "type": "date", "attrs": { "timestamp": "1767225600000" } }
            ]},
            { "type": "panel", "attrs": { "panelType": "warning" }, "content": [{ "type": "paragraph", "content": [{ "type": "text", "text": "careful" }] }] },
            { "type": "taskList", "content": [{ "type": "taskItem", "attrs": { "state": "DONE" }, "content": [{ "type": "text", "text": "ship" }] }] },
            { "type": "someFutureNode", "content": [{ "type": "paragraph", "content": [{ "type": "text", "text": "kept" }] }] }
        ]});
        assert_eq!(
            adf_to_markdown(&adf),
            "@Ada see https://acme.atlassian.net/browse/A-1 [BLOCKED] by 2026-01-01\n\n> **Warning:** careful\n\n- [x] ship\n\nkept"
        );
    }

    #[test]
    fn storage_conversion_keeps_code_and_links() {
        let storage = markdown_to_storage("Hi **there**\n\n```sh\necho ']]>'\n```\n");
        assert_eq!(
            storage,
            "<p>Hi <strong>there</strong></p>\n<ac:structured-macro ac:name=\"code\"><ac:parameter ac:name=\"language\">sh</ac:parameter><ac:plain-text-body><![CDATA[echo ']]]]><![CDATA[>']]></ac:plain-text-body></ac:structured-macro>"
        );
        let page = "<h2>Setup</h2><p>See <ac:link><ri:page ri:content-title=\"Install &amp; run\" /></ac:link>.</p><ac:structured-macro ac:name=\"code\"><ac:parameter ac:name=\"language\">sh</ac:parameter><ac:plain-text-body><![CDATA[make <all>]]></ac:plain-text-body></ac:structured-macro><ac:structured-macro ac:name=\"info\"><ac:parameter ac:name=\"title\">Note</ac:parameter><ac:rich-text-body><p>Be careful</p></ac:rich-text-body></ac:structured-macro>";
        assert_eq!(
            storage_to_markdown(page),
            "## Setup\n\nSee **Install & run**.\n\n```sh\nmake <all>\n```\n\nBe careful"
        );
    }

    #[test]
    fn markdown_and_jira_wiki_markup_convert_both_ways() {
        let markdown = "## Steps\n\nOpen **login** with *care*, run `make`, see [docs](https://x.io) and ~~old~~.\n\n- one\n- two\n  1. nested\n\n```sh\necho hi\n```\n\n---";
        let wiki = markdown_to_wiki(markdown);
        assert_eq!(
            wiki,
            "h2. Steps\n\nOpen *login* with _care_, run {{make}}, see [docs|https://x.io] and -old-.\n\n* one\n* two\n*# nested\n\n{code:sh}\necho hi\n{code}\n\n----"
        );
        assert_eq!(
            wiki_to_markdown(&wiki),
            "## Steps\n\nOpen **login** with *care*, run `make`, see [docs](https://x.io) and ~~old~~.\n\n- one\n- two\n  1. nested\n\n```sh\necho hi\n```\n\n---"
        );
        assert_eq!(
            markdown_to_wiki("a * b and file_name_x"),
            "a \\* b and file\\_name\\_x"
        );
        assert_eq!(
            wiki_to_markdown("||A||B||\n|1|2|\nbq. note"),
            "| A | B |\n| --- | --- |\n| 1 | 2 |\n> note"
        );
        assert_eq!(
            wiki_to_markdown("{code:java}int x;{code}\nafter"),
            "```java\nint x;\n```\nafter"
        );
    }
}
