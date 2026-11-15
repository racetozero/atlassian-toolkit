//! Confluence pages as Markdown files ("docs as code"): front matter,
//! exporting a page tree to folders, pushing Markdown files back, and
//! copying a page tree.
//!
//! Export layout: one folder per page, `<name>/index.md`, attachments in
//! `<name>/_attachments/`, child pages as subfolders. Each `index.md` starts
//! with front matter that records the page:
//!
//! ```text
//! ---
//! title: "Handbook"
//! id: "12345"
//! space: "DOCS"
//! parent: "100"
//! version: 7
//! url: "https://acme.atlassian.net/wiki/..."
//! ---
//! ```

use crate::confluence::{Confluence, Page, PageRef};
use crate::error::{Error, Result};
use crate::markup;
use futures_util::{StreamExt, TryStreamExt};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

/// The `---` block at the top of a Markdown file, as YAML.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(default)]
pub struct FrontMatter {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Page IDs are numbers in the API, so `id: 5` is read as `"5"`.
    #[serde(
        skip_serializing_if = "Option::is_none",
        deserialize_with = "text_or_number"
    )]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub space: Option<String>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        deserialize_with = "text_or_number"
    )]
    pub parent: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// Other keys, kept as written.
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

fn text_or_number<'de, D: serde::Deserializer<'de>>(
    de: D,
) -> std::result::Result<Option<String>, D::Error> {
    Ok(match Option::<serde_json::Value>::deserialize(de)? {
        Some(serde_json::Value::String(text)) => Some(text),
        Some(serde_json::Value::Number(number)) => Some(number.to_string()),
        _ => None,
    })
}

/// The front matter and the body after it. Text without a `---` block (or
/// without its closing `---`) has none. `file` names the file in errors.
pub fn split_front_matter<'a>(text: &'a str, file: &str) -> Result<(FrontMatter, &'a str)> {
    let Some(rest) = text
        .strip_prefix("---\n")
        .or_else(|| text.strip_prefix("---\r\n"))
    else {
        return Ok((FrontMatter::default(), text));
    };
    let mut offset = 0;
    for line in rest.split_inclusive('\n') {
        if line.trim_end_matches(['\n', '\r']) == "---" {
            let yaml = &rest[..offset];
            let front = match yaml.trim().is_empty() {
                true => FrontMatter::default(),
                false => serde_yaml_ng::from_str(yaml).map_err(|err| {
                    Error::Invalid(format!(
                        "the front matter of {file} is not valid YAML ({err}); fix it, or remove the `---` block to publish the file as a new page"
                    ))
                })?,
            };
            return Ok((
                front,
                rest[offset + line.len()..].trim_start_matches(['\n', '\r']),
            ));
        }
        offset += line.len();
    }
    Ok((FrontMatter::default(), text))
}

/// The text with front matter on top.
pub(crate) fn with_front_matter(front: &FrontMatter, body: &str) -> String {
    let yaml =
        serde_yaml_ng::to_string(front).expect("front matter is strings, a number and JSON values");
    let mut out = format!("---\n{yaml}");
    if !out.ends_with('\n') {
        out.push('\n');
    }
    out.push_str("---\n");
    out.push_str(body);
    if !body.ends_with('\n') {
        out.push('\n');
    }
    out
}

/// The page as a Markdown document with front matter.
pub fn page_document(page: &Page) -> String {
    let front = FrontMatter {
        title: Some(page.title.clone()),
        id: Some(page.id.clone()),
        space: page.space.clone(),
        parent: page.parent.clone(),
        version: Some(page.version),
        url: Some(page.url.clone()),
        extra: serde_json::Map::new(),
    };
    with_front_matter(&front, &page.markdown)
}

/// Title patterns from `--exclude` and `--pattern`: `*` matches any text and
/// `?` one character, ignoring case.
pub struct TitleGlobs(Vec<wildmatch::WildMatch>);

impl TitleGlobs {
    pub fn new(patterns: &[String]) -> Self {
        Self(
            patterns
                .iter()
                .map(|p| wildmatch::WildMatch::new_case_insensitive(p))
                .collect(),
        )
    }

    pub fn is_match(&self, title: &str) -> bool {
        self.0.iter().any(|glob| glob.matches(title))
    }
}

/// A file or folder name for a title: characters that file systems reject
/// become `-`; an empty name becomes `fallback`.
pub fn file_name(title: &str, fallback: &str) -> String {
    let name: String = title
        .chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '-',
            c if c.is_control() => '-',
            c => c,
        })
        .collect();
    let name = name.trim().trim_end_matches('.').trim();
    match name.is_empty() {
        true => fallback.to_owned(),
        false => name.to_owned(),
    }
}

/// A folder name for a page under `parent`, unique among its siblings.
fn page_folder(
    used: &mut HashMap<PathBuf, HashSet<String>>,
    parent: &Path,
    title: &str,
    id: &str,
) -> PathBuf {
    let siblings = used.entry(parent.to_path_buf()).or_default();
    let mut name = file_name(title, id);
    if !siblings.insert(name.to_lowercase()) {
        name = format!("{name} ({id})");
        siblings.insert(name.to_lowercase());
    }
    parent.join(name)
}

fn local_files() -> Result<()> {
    #[cfg(not(feature = "native"))]
    return Err(Error::Invalid(
        "this reads and writes local files; use the native SDK or the CLI".into(),
    ));
    #[cfg(feature = "native")]
    Ok(())
}

fn write(path: &Path, bytes: &[u8]) -> Result<()> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)
            .map_err(|err| Error::io(format!("cannot create {}", dir.display()), err))?;
    }
    std::fs::write(path, bytes)
        .map_err(|err| Error::io(format!("cannot write {}", path.display()), err))
}

pub struct ExportOptions {
    /// Also export the pages below the root.
    pub recursive: bool,
    pub max_depth: Option<usize>,
    /// Title patterns (see [`TitleGlobs`]); a match skips the page and its subtree.
    pub exclude: Vec<String>,
    /// Download attachments into `_attachments/`.
    pub attachments: bool,
    /// Report the files without writing them.
    pub dry_run: bool,
}

#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct Exported {
    pub id: String,
    pub title: String,
    #[serde(rename = "type")]
    pub kind: String,
    /// The written `index.md` (or the folder, for a Cloud folder).
    pub path: String,
    pub attachments: usize,
}

/// Exports `root` (and with `recursive` its tree) to Markdown folders in `dest`.
pub async fn export(
    confluence: &Confluence<'_>,
    root: &PageRef,
    dest: &Path,
    options: &ExportOptions,
) -> Result<Vec<Exported>> {
    local_files()?;
    let root = confluence.page(root).await?;
    let mut used = HashMap::new();
    let mut folders: HashMap<String, PathBuf> = HashMap::new();
    let root_dir = page_folder(&mut used, dest, &root.title, &root.id);
    folders.insert(root.id.clone(), root_dir.clone());
    let mut rows = vec![export_page(confluence, &root, &root_dir, options).await?];
    if !options.recursive {
        return Ok(rows);
    }
    let exclude = TitleGlobs::new(&options.exclude);
    let mut skipped: HashSet<String> = HashSet::new();
    // Plan every folder in tree order first (names must be unique among
    // siblings), then fetch and write the pages several at a time.
    let mut planned = Vec::new();
    for node in confluence
        .tree(&PageRef::Id(root.id.clone()), options.max_depth)
        .await?
    {
        if skipped.contains(&node.parent) || exclude.is_match(&node.title) {
            skipped.insert(node.id);
            continue;
        }
        let Some(parent_dir) = folders.get(&node.parent).cloned() else {
            continue;
        };
        let dir = page_folder(&mut used, &parent_dir, &node.title, &node.id);
        folders.insert(node.id.clone(), dir.clone());
        planned.push((node, dir));
    }
    let exported: Vec<Option<Exported>> = futures_util::stream::iter(planned)
        .map(|(node, dir)| async move {
            match node.kind.as_str() {
                "page" => {
                    let page = confluence.page(&PageRef::Id(node.id)).await?;
                    export_page(confluence, &page, &dir, options)
                        .await
                        .map(Some)
                }
                "folder" => {
                    if !options.dry_run {
                        std::fs::create_dir_all(&dir).map_err(|err| {
                            Error::io(format!("cannot create {}", dir.display()), err)
                        })?;
                    }
                    Ok(Some(Exported {
                        id: node.id,
                        title: node.title,
                        kind: node.kind,
                        path: dir.display().to_string(),
                        attachments: 0,
                    }))
                }
                _ => Ok(None),
            }
        })
        .buffered(EXPORT_CONCURRENCY)
        .try_collect()
        .await?;
    rows.extend(exported.into_iter().flatten());
    Ok(rows)
}

/// Pages exported at the same time: fewer round trips in a row without
/// flooding the server.
const EXPORT_CONCURRENCY: usize = 8;

async fn export_page(
    confluence: &Confluence<'_>,
    page: &Page,
    dir: &Path,
    options: &ExportOptions,
) -> Result<Exported> {
    let mut page = page.clone();
    let mut saved = 0;
    if options.attachments {
        for attachment in confluence
            .attachments(&PageRef::Id(page.id.clone()))
            .await?
        {
            let name = file_name(&attachment.title, &attachment.id);
            let local = format!("](<_attachments/{name}>)");
            page.markdown = page
                .markdown
                .replace(&format!("](<{}>)", attachment.title), &local)
                .replace(&format!("]({})", attachment.title), &local);
            if !options.dry_run {
                let bytes = confluence.download(&attachment.download).await?;
                write(&dir.join("_attachments").join(&name), &bytes)?;
            }
            saved += 1;
        }
    }
    let path = dir.join("index.md");
    if !options.dry_run {
        write(&path, page_document(&page).as_bytes())?;
    }
    Ok(Exported {
        id: page.id,
        title: page.title,
        kind: "page".into(),
        path: path.display().to_string(),
        attachments: saved,
    })
}

pub struct PushOptions {
    /// Space for new pages without a `space` key or a parent.
    pub space: Option<String>,
    /// Parent for new pages without a `parent` key or an enclosing `index.md`.
    pub parent: Option<String>,
    /// Overwrite pages that changed on the server since the file was written.
    pub force: bool,
    pub dry_run: bool,
}

#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct Pushed {
    pub file: String,
    pub id: String,
    pub title: String,
    pub version: u64,
    /// created, updated, unchanged, or planned (dry run).
    pub action: String,
}

/// `.md` files under `paths` (directories recursively, without `_attachments`).
fn markdown_files(paths: &[PathBuf]) -> Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    let mut pending: Vec<PathBuf> = paths.to_vec();
    while let Some(path) = pending.pop() {
        if path.is_dir() {
            let entries = std::fs::read_dir(&path)
                .map_err(|err| Error::io(format!("cannot read {}", path.display()), err))?;
            for entry in entries.flatten() {
                let child = entry.path();
                if child.file_name().is_some_and(|n| n == "_attachments") {
                    continue;
                }
                if child.is_dir() || child.extension().is_some_and(|e| e == "md") {
                    pending.push(child);
                }
            }
        } else if path.is_file() {
            out.push(path);
        } else {
            return Err(Error::Invalid(format!(
                "{} does not exist; pass Markdown files or folders of them",
                path.display()
            )));
        }
    }
    // Parents first: a folder's index.md before the pages in and below it.
    let key = |path: &PathBuf| {
        let index = path.file_name().is_some_and(|n| n == "index.md");
        (
            path.components().count() - usize::from(index),
            !index,
            path.clone(),
        )
    };
    out.sort_by_key(key);
    out.dedup();
    Ok(out)
}

/// The `index.md` whose page is the parent of `file`.
fn parent_index(file: &Path) -> Option<PathBuf> {
    let dir = file.parent()?;
    let index = match file.file_name().is_some_and(|n| n == "index.md") {
        true => dir.parent()?.join("index.md"),
        false => dir.join("index.md"),
    };
    index.is_file().then_some(index)
}

/// Relative image paths in Markdown (`![alt](path)` or `![alt](<path>)`).
fn image_paths(markdown: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = markdown;
    while let Some(at) = rest.find("![") {
        rest = &rest[at + 2..];
        let Some(open) = rest.find("](") else { break };
        let after = &rest[open + 2..];
        let target = match after.strip_prefix('<') {
            Some(inner) => inner.split('>').next(),
            None => after.split([')', ' ']).next(),
        };
        if let Some(target) = target.filter(|t| !t.is_empty() && !t.contains("://")) {
            out.push(target.to_owned());
        }
    }
    out
}

/// Publishes Markdown files: updates the page in each file's front matter,
/// creates pages for files without an `id`, uploads referenced images, and
/// writes the new id and version back into the files.
pub async fn push(
    confluence: &Confluence<'_>,
    paths: &[PathBuf],
    options: &PushOptions,
) -> Result<Vec<Pushed>> {
    local_files()?;
    let mut rows = Vec::new();
    for file in markdown_files(paths)? {
        let shown = file.display().to_string();
        let text = std::fs::read_to_string(&file)
            .map_err(|err| Error::io(format!("cannot read {shown}"), err))?;
        let (mut front, body) = split_front_matter(&text, &shown)?;
        let mut body = body.to_owned();
        let title = match front.title.clone() {
            Some(title) => title,
            None => match body.trim_start().strip_prefix("# ") {
                Some(rest) => {
                    let heading = rest.lines().next().unwrap_or_default().trim().to_owned();
                    body = rest
                        .split_once('\n')
                        .map(|(_, b)| b.trim_start().to_owned())
                        .unwrap_or_default();
                    heading
                }
                None => {
                    let stem = match file.file_name().is_some_and(|n| n == "index.md") {
                        true => file.parent().and_then(Path::file_name),
                        false => file.file_stem(),
                    };
                    stem.map(|s| s.to_string_lossy().into_owned())
                        .unwrap_or_default()
                }
            },
        };
        let storage = markup::markdown_to_storage(&body);
        let (id, version, url, action) = match front.id.clone() {
            Some(id) => {
                let current = confluence.page(&PageRef::Id(id.clone())).await?;
                if let Some(local) = front.version
                    && current.version > local
                    && !options.force
                {
                    return Err(Error::Invalid(format!(
                        "{shown}: page {id} changed on the server (version {}, the file has {local}); export it again or pass --force",
                        current.version
                    )));
                }
                // Compare as Markdown: the server normalizes storage format.
                let unchanged =
                    current.markdown.trim() == markup::storage_to_markdown(&storage).trim();
                if current.title == title && unchanged {
                    (id, current.version, current.url, "unchanged")
                } else if options.dry_run {
                    (id, current.version, current.url, "planned")
                } else {
                    upload_images(confluence, &id, &file, &body).await?;
                    let saved = confluence
                        .update(&PageRef::Id(id.clone()), Some(&title), Some(storage))
                        .await?;
                    (id, current.version + 1, saved.url, "updated")
                }
            }
            None => {
                let parent = match &front.parent {
                    Some(parent) => Some(parent.clone()),
                    None => match parent_index(&file) {
                        Some(index) => {
                            let shown = index.display().to_string();
                            let text = std::fs::read_to_string(&index)
                                .map_err(|err| Error::io(format!("cannot read {shown}"), err))?;
                            split_front_matter(&text, &shown)?.0.id
                        }
                        None => None,
                    }
                    .or_else(|| options.parent.clone()),
                };
                let space = match front.space.clone().or_else(|| options.space.clone()) {
                    Some(space) => space,
                    None => match &parent {
                        Some(parent) => confluence
                            .page(&PageRef::parse(parent)?)
                            .await?
                            .space
                            .unwrap_or_default(),
                        None => String::new(),
                    },
                };
                if space.is_empty() {
                    return Err(Error::Invalid(format!(
                        "{shown}: no space for the new page; add `space: \"KEY\"` to its front matter, pass --space, or put it in a folder with an exported index.md"
                    )));
                }
                if options.dry_run {
                    rows.push(Pushed {
                        file: shown,
                        id: String::new(),
                        title,
                        version: 0,
                        action: "planned".into(),
                    });
                    continue;
                }
                let parent_ref = parent.as_deref().map(PageRef::parse).transpose()?;
                let created = confluence
                    .create(&space, &title, parent_ref.as_ref(), storage)
                    .await?;
                upload_images(confluence, &created.id, &file, &body).await?;
                front.space = Some(space);
                front.parent = parent;
                (created.id, 1, created.url, "created")
            }
        };
        if action == "created" || action == "updated" {
            front.title = Some(title.clone());
            front.id = Some(id.clone());
            front.version = Some(version);
            front.url = Some(url);
            write(&file, with_front_matter(&front, &body).as_bytes())?;
        }
        rows.push(Pushed {
            file: shown,
            id,
            title,
            version,
            action: action.into(),
        });
    }
    Ok(rows)
}

/// Uploads the local images the Markdown references (new versions when the
/// attachment exists).
async fn upload_images(
    confluence: &Confluence<'_>,
    id: &str,
    file: &Path,
    markdown: &str,
) -> Result<()> {
    let dir = file.parent().unwrap_or(Path::new("."));
    let files: Vec<PathBuf> = image_paths(markdown)
        .into_iter()
        .map(|p| dir.join(p))
        .filter(|p| p.is_file())
        .collect();
    if !files.is_empty() {
        confluence
            .attach_replace(&PageRef::Id(id.to_owned()), &files)
            .await?;
    }
    Ok(())
}

pub struct CopyOptions {
    /// Appended to each title (default: " (copy)" in the same space, else none).
    pub suffix: Option<String>,
    pub max_depth: Option<usize>,
    pub exclude: Vec<String>,
    pub dry_run: bool,
}

#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct Copied {
    pub source: String,
    /// The new page (empty in a dry run).
    pub id: String,
    pub title: String,
    pub parent: String,
    pub url: String,
}

/// Copies `source` and the pages below it under `target` (titles, bodies and
/// labels; not attachments or comments). Cloud folders are flattened into
/// their parent.
pub async fn copy_tree(
    confluence: &Confluence<'_>,
    source: &PageRef,
    target: &PageRef,
    options: &CopyOptions,
) -> Result<Vec<Copied>> {
    let source = confluence.page(source).await?;
    let target = confluence.page(target).await?;
    let space = target.space.clone().ok_or_else(|| {
        Error::Invalid(format!(
            "cannot find the space of the target page {}",
            target.id
        ))
    })?;
    let suffix = options
        .suffix
        .clone()
        .unwrap_or_else(|| match source.space == target.space {
            true => " (copy)".into(),
            false => String::new(),
        });
    let mut copies: HashMap<String, String> = HashMap::new();
    let mut rows = Vec::new();
    let root = copy_page(
        confluence,
        &source,
        &space,
        &target.id,
        &suffix,
        options.dry_run,
    )
    .await?;
    copies.insert(source.id.clone(), root.id.clone());
    rows.push(root);
    let exclude = TitleGlobs::new(&options.exclude);
    let mut skipped: HashSet<String> = HashSet::new();
    for node in confluence
        .tree(&PageRef::Id(source.id.clone()), options.max_depth)
        .await?
    {
        if skipped.contains(&node.parent) || exclude.is_match(&node.title) {
            skipped.insert(node.id);
            continue;
        }
        let Some(parent) = copies.get(&node.parent).cloned() else {
            continue;
        };
        match node.kind.as_str() {
            "page" => {
                let page = confluence.page(&PageRef::Id(node.id.clone())).await?;
                let copy =
                    copy_page(confluence, &page, &space, &parent, &suffix, options.dry_run).await?;
                copies.insert(node.id, copy.id.clone());
                rows.push(copy);
            }
            "folder" => {
                copies.insert(node.id, parent);
            }
            _ => {}
        }
    }
    Ok(rows)
}

async fn copy_page(
    confluence: &Confluence<'_>,
    page: &Page,
    space: &str,
    parent: &str,
    suffix: &str,
    dry_run: bool,
) -> Result<Copied> {
    let title = format!("{}{suffix}", page.title);
    if dry_run {
        // Children of a planned copy point at the source until it exists.
        return Ok(Copied {
            source: page.id.clone(),
            id: format!("(copy of {})", page.id),
            title,
            parent: parent.to_owned(),
            url: String::new(),
        });
    }
    let created = confluence
        .create(space, &title, Some(&PageRef::Id(parent.to_owned())), page.storage.clone())
        .await
        .map_err(|err| match err {
            Error::Http(http) if http.status == 400 && http.message.to_lowercase().contains("title") => {
                Error::Invalid(format!(
                    "Confluence refused the title `{title}` in space {space} ({}); pass --suffix with text that makes the titles unique",
                    http.message
                ))
            }
            other => other,
        })?;
    let labels = confluence.labels(&PageRef::Id(page.id.clone())).await?;
    if !labels.is_empty() {
        confluence
            .change_labels(&PageRef::Id(created.id.clone()), &labels, &[])
            .await?;
    }
    Ok(Copied {
        source: page.id.clone(),
        id: created.id,
        title,
        parent: parent.to_owned(),
        url: created.url,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn front_matter_round_trips_and_keeps_other_keys() {
        let mut extra = serde_json::Map::new();
        extra.insert("tags".into(), serde_json::json!(["a", "b"]));
        let front = FrontMatter {
            title: Some("Notes: \"draft\"".into()),
            id: Some("12".into()),
            version: Some(3),
            extra,
            ..Default::default()
        };
        let text = with_front_matter(&front, "# Body\n");
        assert!(
            text.starts_with("---\n") && text.contains("\n---\n# Body\n"),
            "{text}"
        );
        let (parsed, body) = split_front_matter(&text, "a.md").expect("valid");
        assert_eq!(parsed, front, "{text}");
        assert_eq!(body, "# Body\n");
        let plain = split_front_matter("no front matter", "a.md").expect("valid");
        assert_eq!(plain.1, "no front matter");
        let unclosed = split_front_matter("---\nunclosed: 1\n", "a.md").expect("valid");
        assert_eq!(unclosed.0, FrontMatter::default());
        // Hand-edited files: single quotes, numeric ids.
        let (old, _) =
            split_front_matter("---\ntitle: 'It''s'\nid: 5\nparent: \"7\"\n---\nx", "a.md")
                .expect("valid");
        assert_eq!(old.title.as_deref(), Some("It's"));
        assert_eq!(old.id.as_deref(), Some("5"));
        assert_eq!(old.parent.as_deref(), Some("7"));
        let broken =
            split_front_matter("---\ntitle: [unclosed\n---\nx", "docs/a.md").expect_err("invalid");
        assert!(broken.to_string().contains("docs/a.md"), "{broken}");
    }

    #[test]
    fn globs_match_titles_ignoring_case() {
        for (pattern, text, expected) in [
            ("*.png", "Diagram.PNG", true),
            ("*.png", "diagram.jpg", false),
            ("Draft*", "draft: plan", true),
            ("?b", "ab", true),
            ("?b", "abb", false),
            ("*", "", true),
            ("a*b*c", "aXXbYYc", true),
            ("*", "Team/Notes", true),
        ] {
            let globs = TitleGlobs::new(&[pattern.to_owned()]);
            assert_eq!(globs.is_match(text), expected, "{pattern} {text}");
        }
        assert!(
            !TitleGlobs::new(&[]).is_match("anything"),
            "no patterns match nothing"
        );
    }

    #[test]
    fn names_and_images_are_safe_for_files() {
        assert_eq!(file_name("a/b: c?", "1"), "a-b- c-");
        assert_eq!(file_name(" .. ", "7"), "7");
        assert_eq!(
            image_paths("![a](<_attachments/my d.png>) ![b](x.png) ![c](https://e/x.png)"),
            ["_attachments/my d.png", "x.png"]
        );
    }
}
