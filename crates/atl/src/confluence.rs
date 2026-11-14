//! `atl confluence`: pages as Markdown, spaces, search, attachments.

use crate::Exit;
use crate::output::{Col, bold, col, dim, flex};
use crate::prompt;
use crate::tasks::{
    Ctx, confirm, edit_in_editor, key_and_text, open_url, read_body, read_text, s, save_download,
};
use anyhow::Result;
use atl_sdk::Error;
use atl_sdk::confluence::{Confluence, PageRef, text_cql};
use atl_sdk::error::Choice;
use clap::{Args, Subcommand};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

#[derive(Subcommand, Debug)]
pub enum ConfluenceCommand {
    /// Pages as Markdown: get, create, update, edit, delete, move, children, comments, labels, history, attachments.
    #[command(subcommand)]
    Page(PageCommand),
    /// Spaces: list, pages, create.
    #[command(subcommand)]
    Space(SpaceCommand),
    /// Blog posts: list, get, create.
    #[command(subcommand)]
    Blog(BlogCommand),
    /// Search (plain words or --cql).
    Search(SearchArgs),
    /// Download an attachment by its download URL (see `page attachments`; -o chooses the file), or delete one.
    #[command(subcommand)]
    Attachment(AttachmentCommand),
    /// Delete a comment.
    #[command(subcommand)]
    Comment(CommentCommand),
}

#[derive(Subcommand, Debug)]
pub enum CommentCommand {
    /// Delete a comment by ID (asks first; --yes skips the question).
    Delete {
        /// Comment ID.
        id: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum PageCommand {
    /// Print a page as Markdown. PAGE is an ID, a page URL, or SPACE:Title.
    Get {
        /// Page ID, page URL, or SPACE:Title.
        page: String,
        /// Print the storage-format XHTML instead of Markdown.
        #[arg(long)]
        storage: bool,
        /// Open the page in the browser instead.
        #[arg(long, short = 'w')]
        web: bool,
        /// Print this older version (see `page history`).
        #[arg(long)]
        version: Option<u64>,
        /// Start with front matter (id, title, space, parent, version) for `page push`.
        #[arg(long)]
        front_matter: bool,
    },
    /// Export a page (and with -r its tree) to Markdown folders: NAME/index.md with front matter and _attachments/.
    Export {
        /// Page ID, page URL, or SPACE:Title.
        page: String,
        /// Folder to write into.
        #[arg(long, default_value = ".")]
        dest: PathBuf,
        /// Also export the pages below.
        #[arg(short = 'r', long)]
        recursive: bool,
        /// With -r, stop at this depth.
        #[arg(long, value_name = "N")]
        max_depth: Option<usize>,
        /// Skip pages whose title matches this glob, with their subtrees; repeatable.
        #[arg(long, value_name = "GLOB")]
        exclude: Vec<String>,
        /// Do not download attachments.
        #[arg(long)]
        skip_attachments: bool,
    },
    /// Publish Markdown files or folders (from `page export` or new): update, create, upload images.
    Push {
        /// Markdown files or folders (from `page export`, or new).
        #[arg(required = true)]
        paths: Vec<PathBuf>,
        /// Space for new pages (default: front matter, then [confluence] space in .atl.toml).
        #[arg(long)]
        space: Option<String>,
        /// Parent page for new pages outside an exported folder.
        #[arg(long)]
        parent: Option<String>,
        /// Overwrite pages that changed on the server since the file was written.
        #[arg(long)]
        force: bool,
    },
    /// Copy a page and the pages below it under another page.
    CopyTree {
        /// Page ID, page URL, or SPACE:Title.
        page: String,
        /// The new parent page.
        #[arg(long)]
        to: String,
        /// Appended to each title (default " (copy)" in the same space).
        #[arg(long)]
        suffix: Option<String>,
        /// How many levels of child pages to copy (default: all).
        #[arg(long, value_name = "N")]
        max_depth: Option<usize>,
        /// Skip pages whose title matches this glob; repeatable.
        #[arg(long, value_name = "GLOB")]
        exclude: Vec<String>,
    },
    /// Create a page from Markdown.
    Create {
        /// Space key (default: [confluence] space in .atl.toml).
        #[arg(long)]
        space: Option<String>,
        /// Page title (asked in a terminal when missing).
        #[arg(long)]
        title: Option<String>,
        /// Parent page: ID, URL, or SPACE:Title.
        #[arg(long)]
        parent: Option<String>,
        #[command(flatten)]
        body: Body,
    },
    /// Replace a page's title and/or body; the version is bumped for you.
    Update {
        /// Page ID, page URL, or SPACE:Title.
        page: String,
        /// New title.
        #[arg(long)]
        title: Option<String>,
        #[command(flatten)]
        body: Body,
    },
    /// Delete a page (asks first; --yes skips the question).
    Delete {
        /// Page ID, page URL, or SPACE:Title.
        page: String,
    },
    /// List a page's children (Cloud: also folders); --recursive shows the tree.
    Children {
        /// Page ID, page URL, or SPACE:Title.
        page: String,
        /// The whole tree below the page.
        #[arg(short = 'r', long)]
        recursive: bool,
        /// With --recursive, stop at this depth (1 = direct children).
        #[arg(long, value_name = "N")]
        max_depth: Option<usize>,
    },
    /// Find pages whose title contains the words.
    Find {
        /// Words in the page title.
        title: String,
        /// Space key (default: [confluence] space in .atl.toml).
        #[arg(long)]
        space: Option<String>,
        /// Maximum number of results.
        #[arg(long, default_value_t = 25)]
        limit: usize,
    },
    /// Edit a page as Markdown in $VISUAL or $EDITOR; saves a new version when you change it.
    Edit {
        /// Page ID, page URL, or SPACE:Title.
        page: String,
    },
    /// Upload files to a page.
    Attach {
        /// Page ID, page URL, or SPACE:Title.
        page: String,
        /// Files to attach.
        #[arg(required = true)]
        files: Vec<PathBuf>,
        /// Upload new versions of attachments with the same name.
        #[arg(long)]
        replace: bool,
    },
    /// List a page's attachments with download URLs; --download saves them.
    Attachments {
        /// Page ID, page URL, or SPACE:Title.
        page: String,
        /// Only titles matching this glob, for example "*.png".
        #[arg(long, value_name = "GLOB")]
        pattern: Option<String>,
        /// Download the attachments into --dest.
        #[arg(long)]
        download: bool,
        /// Folder for --download (default: the current folder).
        #[arg(long, default_value = ".")]
        dest: PathBuf,
    },
    /// List a page's footer (or --inline) comments as Markdown, oldest first.
    Comments {
        /// Page ID, page URL, or SPACE:Title.
        page: String,
        /// Maximum number of results.
        #[arg(long, default_value_t = 50)]
        limit: usize,
        /// Inline comments instead of footer comments.
        #[arg(long)]
        inline: bool,
    },
    /// Add a comment: `atl confluence page comment PAGE "text"` or --body-file.
    Comment {
        /// `PAGE [TEXT]`.
        #[arg(required = true, num_args = 1..=2, value_names = ["PAGE", "TEXT"])]
        args: Vec<String>,
        /// Read the comment (Markdown) from a file, or - for stdin.
        #[arg(long)]
        body_file: Option<PathBuf>,
        /// Reply to this comment ID.
        #[arg(long)]
        parent: Option<String>,
    },
    /// List labels; with --add or --remove, change them and list the result.
    Labels {
        /// Page ID, page URL, or SPACE:Title.
        page: String,
        /// Label to add; repeatable.
        #[arg(long)]
        add: Vec<String>,
        /// Label to remove; repeatable.
        #[arg(long)]
        remove: Vec<String>,
    },
    /// Move a page under another page (--parent) or next to one (--before/--after, Cloud only).
    #[command(group = clap::ArgGroup::new("target").required(true))]
    Move {
        /// Page ID, page URL, or SPACE:Title.
        page: String,
        /// New parent page: ID, URL, or SPACE:Title.
        #[arg(long, group = "target")]
        parent: Option<String>,
        /// Put the page just before this sibling (Cloud).
        #[arg(long, group = "target")]
        before: Option<String>,
        /// Put the page just after this sibling (Cloud).
        #[arg(long, group = "target")]
        after: Option<String>,
    },
    /// List a page's versions, newest first.
    History {
        /// Page ID, page URL, or SPACE:Title.
        page: String,
        /// Maximum number of results.
        #[arg(long, default_value_t = 25)]
        limit: usize,
    },
}

#[derive(Args, Debug)]
pub struct Body {
    /// Body in Markdown.
    #[arg(long)]
    body: Option<String>,
    /// Read the body from a file (.md, or .html/.xml for storage format), or - for stdin.
    #[arg(long)]
    file: Option<PathBuf>,
}

impl Body {
    /// `{"markdown": ...}` or `{"storage": ...}` for the page tasks; with
    /// `piped`, a body piped on stdin counts too.
    fn into_args(self, mut args: Value, piped: bool) -> Result<Value, Error> {
        let storage = self
            .file
            .as_deref()
            .and_then(Path::extension)
            .is_some_and(|ext| ext == "html" || ext == "xml");
        let text = match piped {
            true => read_body(self.body, self.file.as_deref())?,
            false => read_text(self.body, self.file.as_deref())?,
        };
        if let Some(text) = text {
            args[if storage { "storage" } else { "markdown" }] = text.into();
        }
        Ok(args)
    }
}

#[derive(Subcommand, Debug)]
pub enum SpaceCommand {
    /// List spaces.
    #[command(visible_alias = "ls")]
    List {
        /// Maximum number of results.
        #[arg(long, default_value_t = 50)]
        limit: usize,
    },
    /// List the top-level pages of a space.
    Pages {
        /// Space key (default: [confluence] space in .atl.toml).
        space: Option<String>,
        /// Maximum number of results.
        #[arg(long, default_value_t = 100)]
        limit: usize,
    },
    /// Create a space: `atl confluence space create ENG "Engineering"`.
    Create {
        /// Space key, for example DOCS.
        key: String,
        /// Space name.
        name: String,
        /// Plain-text description.
        #[arg(long)]
        description: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
pub enum BlogCommand {
    /// Blog posts of a space, newest first.
    #[command(visible_alias = "ls")]
    List {
        /// Space key (default: [confluence] space in .atl.toml).
        #[arg(long)]
        space: Option<String>,
        /// Maximum number of results.
        #[arg(long, default_value_t = 25)]
        limit: usize,
    },
    /// Print a blog post as Markdown.
    Get {
        /// Blog post ID.
        id: String,
        /// Open it in the browser instead.
        #[arg(long, short = 'w')]
        web: bool,
    },
    /// Publish a blog post from Markdown.
    Create {
        /// Space key (default: [confluence] space in .atl.toml).
        #[arg(long)]
        space: Option<String>,
        /// Blog post title.
        #[arg(long)]
        title: String,
        #[command(flatten)]
        body: Body,
    },
}

#[derive(Args, Debug)]
pub struct SearchArgs {
    /// Words to find, or a CQL query with --cql.
    pub query: String,
    /// Treat QUERY as raw CQL.
    #[arg(long)]
    pub cql: bool,
    /// Limit to a space (default: [confluence] space in .atl.toml when set).
    #[arg(long)]
    pub space: Option<String>,
    /// Content type: page, blogpost, attachment, comment.
    #[arg(long = "type")]
    pub content_type: Option<String>,
    /// Maximum number of results.
    #[arg(long, default_value_t = 50)]
    pub limit: usize,
}

#[derive(Subcommand, Debug)]
pub enum AttachmentCommand {
    /// Download an attachment by its download URL.
    Download {
        /// Download URL of the attachment.
        url: String,
    },
    /// Delete an attachment by ID (asks first; --yes skips the question).
    Delete {
        /// Attachment ID.
        id: String,
    },
}

const ATTACHMENT_COLUMNS: [Col; 4] = [
    col("ID", "id"),
    col("SIZE", "size"),
    col("TYPE", "media_type"),
    flex("TITLE", "title"),
];

fn saved(p: &Value) -> String {
    format!("Saved page {} {}", s(p, "id"), s(p, "url"))
}

pub async fn run(ctx: &Ctx<'_>, command: ConfluenceCommand) -> Result<Exit> {
    match command {
        ConfluenceCommand::Page(command) => run_page(ctx, command).await,
        ConfluenceCommand::Space(SpaceCommand::List { limit }) => {
            ctx.list(
                "confluence_space_list",
                json!({ "limit": limit }),
                &[col("KEY", "key"), col("TYPE", "type"), flex("NAME", "name")],
            )
            .await
        }
        ConfluenceCommand::Space(SpaceCommand::Pages { space, limit }) => {
            ctx.list(
                "confluence_space_pages",
                json!({ "space": ctx.repo.require_space(space)?, "limit": limit }),
                &[col("ID", "id"), flex("TITLE", "title")],
            )
            .await
        }
        ConfluenceCommand::Space(SpaceCommand::Create {
            key,
            name,
            description,
        }) => {
            ctx.item(
                "confluence_space_create",
                json!({ "key": key, "name": name, "description": description }),
                |sp| format!("Created space {} {}", s(sp, "key"), s(sp, "url")),
            )
            .await
        }
        ConfluenceCommand::Blog(BlogCommand::List { space, limit }) => {
            ctx.list(
                "confluence_blog_list",
                json!({ "space": ctx.repo.require_space(space)?, "limit": limit }),
                &[col("ID", "id"), flex("TITLE", "title")],
            )
            .await
        }
        ConfluenceCommand::Blog(BlogCommand::Get { id, web }) => {
            let value = ctx.task("confluence_blog_get", json!({ "id": id })).await?;
            if web {
                return open_url(ctx.out, s(&value, "url"));
            }
            let tty = ctx.out.tty;
            ctx.out.item(value, Value::Null, |p| {
                format!(
                    "{}\n{}\n\n{}",
                    bold(s(p, "title"), tty),
                    dim(s(p, "url"), tty),
                    crate::output::markdown(s(p, "markdown"), tty)
                )
            })
        }
        ConfluenceCommand::Blog(BlogCommand::Create { space, title, body }) => {
            let args = body.into_args(
                json!({ "space": ctx.repo.require_space(space)?, "title": title }),
                true,
            )?;
            if args.get("markdown").is_none() && args.get("storage").is_none() {
                return Err(Error::Invalid(
                    "the blog post has no body; pass --body MARKDOWN, --file FILE (- for stdin), or pipe it".into(),
                )
                .into());
            }
            ctx.item("confluence_blog_create", args, |b| {
                format!("Published {} {}", s(b, "title"), s(b, "url"))
            })
            .await
        }
        ConfluenceCommand::Search(args) => {
            let cql = if args.cql {
                args.query
            } else {
                let space = ctx.repo.space(args.space);
                text_cql(&args.query, space.as_deref(), args.content_type.as_deref())
            };
            if ctx.dry_run {
                return ctx.out.item(json!({ "cql": cql }), Value::Null, |d| {
                    s(d, "cql").to_owned()
                });
            }
            let mut found = ctx
                .task(
                    "confluence_search",
                    json!({ "cql": cql, "limit": args.limit }),
                )
                .await?;
            ctx.out.list(
                found["results"].take(),
                &[
                    col("ID", "id"),
                    col("TYPE", "type"),
                    col("SPACE", "space"),
                    flex("TITLE", "title"),
                ],
                json!({ "cql": cql }),
            )
        }
        ConfluenceCommand::Attachment(AttachmentCommand::Delete { id }) => {
            confirm(ctx.yes, &format!("delete Confluence attachment {id}"))?;
            ctx.item("confluence_attachment_delete", json!({ "id": id }), |_| {
                format!("Deleted attachment {id}")
            })
            .await
        }
        ConfluenceCommand::Comment(CommentCommand::Delete { id }) => {
            confirm(ctx.yes, &format!("delete Confluence comment {id}"))?;
            ctx.item("confluence_comment_delete", json!({ "id": id }), |_| {
                format!("Deleted comment {id}")
            })
            .await
        }
        ConfluenceCommand::Attachment(AttachmentCommand::Download { url }) => {
            let bytes = Confluence::new(ctx.client).download(&url).await?;
            let last = url::Url::parse(&url)
                .ok()
                .and_then(|u| u.path_segments()?.next_back().map(str::to_owned))
                .unwrap_or_default();
            let last = percent_encoding::percent_decode_str(&last).decode_utf8_lossy();
            save_download(ctx, &atl_sdk::docs::file_name(&last, "attachment"), &bytes)
        }
    }
}

async fn run_page(ctx: &Ctx<'_>, command: PageCommand) -> Result<Exit> {
    match command {
        PageCommand::Get {
            page,
            storage,
            web,
            version,
            front_matter,
        } => {
            let value = ctx
                .task(
                    "confluence_page_get",
                    json!({ "page": page, "storage": storage, "version": version, "front_matter": front_matter }),
                )
                .await?;
            if web {
                return open_url(ctx.out, s(&value, "url"));
            }
            let tty = ctx.out.tty;
            ctx.out.item(value, Value::Null, |p| {
                if storage {
                    return s(p, "storage").to_owned();
                }
                if front_matter {
                    return s(p, "document").to_owned();
                }
                let header = match tty {
                    true => format!(
                        "{}\n{}\n\n",
                        bold(s(p, "title"), tty),
                        dim(s(p, "url"), tty)
                    ),
                    false => String::new(),
                };
                format!("{header}{}", crate::output::markdown(s(p, "markdown"), tty))
            })
        }
        PageCommand::Create {
            space,
            title,
            parent,
            body,
        } => {
            let ask = prompt::allowed();
            let space = match ctx.repo.space(space) {
                Some(space) => space,
                None if ask => {
                    let spaces = ctx
                        .task("confluence_space_list", json!({ "limit": 250 }))
                        .await?;
                    let choices: Vec<Choice> = spaces
                        .as_array()
                        .into_iter()
                        .flatten()
                        .map(|p| {
                            Choice::new(s(p, "key"), format!("{}  {}", s(p, "key"), s(p, "name")))
                        })
                        .collect();
                    prompt::select("Space", &choices)?
                }
                None => ctx.repo.require_space(None)?,
            };
            let title = match title {
                Some(title) => title,
                None if ask => prompt::text("Title")?,
                None => {
                    return Err(Error::Invalid("pass --title with the page title".into()).into());
                }
            };
            let mut args = body.into_args(
                json!({ "space": space, "title": title, "parent": parent }),
                true,
            )?;
            if args.get("markdown").is_none() && args.get("storage").is_none() {
                let written = if ask { edit_in_editor("")? } else { None };
                match written {
                    Some(markdown) => args["markdown"] = markdown.into(),
                    None => {
                        return Err(Error::Invalid(
                            "pass --body '<markdown>', --file <path>, or pipe the Markdown".into(),
                        )
                        .into());
                    }
                }
            }
            ctx.item("confluence_page_create", args, saved).await
        }
        PageCommand::Update { page, title, body } => {
            let args = body.into_args(json!({ "page": page, "title": title }), false)?;
            ctx.item("confluence_page_update", args, saved).await
        }
        PageCommand::Delete { page } => {
            PageRef::parse(&page)?;
            confirm(ctx.yes, &format!("delete Confluence page {page}"))?;
            ctx.item("confluence_page_delete", json!({ "page": page }), |d| {
                format!("Deleted page {}", s(d, "id"))
            })
            .await
        }
        PageCommand::Edit { page } => {
            let current = ctx
                .task("confluence_page_get", json!({ "page": page }))
                .await?;
            let Some(markdown) = edit_in_editor(s(&current, "markdown"))? else {
                return ctx.out.item(
                    json!({ "id": current["id"], "updated": false }),
                    Value::Null,
                    |_| format!("{}: no changes", s(&current, "title")),
                );
            };
            ctx.item(
                "confluence_page_update",
                json!({ "page": current["id"], "markdown": markdown }),
                saved,
            )
            .await
        }
        PageCommand::Children {
            page,
            recursive,
            max_depth,
        } => {
            let mut rows = ctx
                .task(
                    "confluence_page_children",
                    json!({ "page": page, "recursive": recursive || max_depth.is_some(), "max_depth": max_depth }),
                )
                .await?;
            if ctx.out.wants_text() {
                // An indented tree for people; JSON keeps `depth` and `parent`.
                for row in rows.as_array_mut().into_iter().flatten() {
                    let depth = row["depth"].as_u64().unwrap_or(1) as usize;
                    row["title"] = format!(
                        "{}{}",
                        "  ".repeat(depth.saturating_sub(1)),
                        s(row, "title")
                    )
                    .into();
                }
            }
            ctx.out.list(
                rows,
                &[col("ID", "id"), col("TYPE", "type"), flex("TITLE", "title")],
                Value::Null,
            )
        }
        PageCommand::Find {
            title,
            space,
            limit,
        } => {
            let space = ctx.repo.space(space);
            ctx.list(
                "confluence_page_find",
                json!({ "title": title, "space": space, "limit": limit }),
                &[
                    col("ID", "id"),
                    col("SPACE", "space"),
                    flex("TITLE", "title"),
                ],
            )
            .await
        }
        PageCommand::Attach {
            page,
            files,
            replace,
        } => {
            ctx.list(
                "confluence_page_attach",
                json!({ "page": page, "files": files, "replace": replace }),
                &ATTACHMENT_COLUMNS,
            )
            .await
        }
        PageCommand::Attachments {
            page,
            pattern,
            download,
            dest,
        } => {
            let mut found = ctx
                .task(
                    "confluence_page_attachments",
                    json!({ "page": page, "pattern": pattern }),
                )
                .await?;
            if !download {
                return ctx.out.list(found, &ATTACHMENT_COLUMNS, Value::Null);
            }
            let confluence = Confluence::new(ctx.client);
            if !ctx.dry_run {
                std::fs::create_dir_all(&dest).map_err(|err| {
                    Error::Invalid(format!(
                        "cannot create {}: {err}; pass a writable --dest",
                        dest.display()
                    ))
                })?;
            }
            for attachment in found.as_array_mut().into_iter().flatten() {
                let path = dest.join(atl_sdk::docs::file_name(
                    s(attachment, "title"),
                    "attachment",
                ));
                if !ctx.dry_run {
                    let bytes = confluence.download(s(attachment, "download")).await?;
                    std::fs::write(&path, bytes).map_err(|err| {
                        Error::Invalid(format!(
                            "cannot write {}: {err}; pass a writable --dest",
                            path.display()
                        ))
                    })?;
                }
                attachment["saved"] = path.display().to_string().into();
            }
            ctx.out.list(
                found,
                &[col("ID", "id"), col("SIZE", "size"), flex("SAVED", "saved")],
                Value::Null,
            )
        }
        PageCommand::Export {
            page,
            dest,
            recursive,
            max_depth,
            exclude,
            skip_attachments,
        } => {
            ctx.list(
                "confluence_page_export",
                json!({
                    "page": page, "dest": dest, "recursive": recursive || max_depth.is_some(),
                    "max_depth": max_depth, "exclude": exclude, "attachments": !skip_attachments,
                    "dry_run": ctx.dry_run,
                }),
                &[
                    col("TYPE", "type"),
                    col("ATTACHMENTS", "attachments"),
                    flex("PATH", "path"),
                ],
            )
            .await
        }
        PageCommand::Push {
            paths,
            space,
            parent,
            force,
        } => {
            ctx.list(
                "confluence_page_push",
                json!({
                    "paths": paths, "space": ctx.repo.space(space),
                    "parent": parent, "force": force, "dry_run": ctx.dry_run,
                }),
                &[
                    col("ACTION", "action"),
                    col("VERSION", "version"),
                    col("ID", "id"),
                    flex("FILE", "file"),
                ],
            )
            .await
        }
        PageCommand::CopyTree {
            page,
            to,
            suffix,
            max_depth,
            exclude,
        } => {
            ctx.list(
                "confluence_page_copy_tree",
                json!({
                    "page": page, "target": to, "suffix": suffix, "max_depth": max_depth,
                    "exclude": exclude, "dry_run": ctx.dry_run,
                }),
                &[
                    col("SOURCE", "source"),
                    col("NEW ID", "id"),
                    flex("TITLE", "title"),
                ],
            )
            .await
        }
        PageCommand::Comments {
            page,
            limit,
            inline,
        } => {
            let comments = ctx
                .task(
                    "confluence_page_comments",
                    json!({ "page": page, "limit": limit, "inline": inline }),
                )
                .await?;
            let tty = ctx.out.tty;
            ctx.out.item(comments, Value::Null, |list| {
                let blocks: Vec<String> = list
                    .as_array()
                    .into_iter()
                    .flatten()
                    .map(|c| {
                        format!(
                            "{} · {}\n{}",
                            bold(c["author"].as_str().unwrap_or("unknown"), tty),
                            s(c, "created").chars().take(10).collect::<String>(),
                            s(c, "body")
                        )
                    })
                    .collect();
                if blocks.is_empty() {
                    "(no comments)".into()
                } else {
                    blocks.join("\n\n")
                }
            })
        }
        PageCommand::Comment {
            args,
            body_file,
            parent,
        } => {
            let (page, text) = key_and_text(
                args,
                body_file.as_deref(),
                |_| true,
                |page| {
                    page.ok_or_else(|| {
                        Error::Invalid("pass `PAGE TEXT` or `PAGE --body-file <path>`".into())
                    })
                },
            )?;
            ctx.item(
                "confluence_page_comment",
                json!({ "page": page, "body": text, "parent": parent }),
                |_| format!("Commented on {page}"),
            )
            .await
        }
        PageCommand::Labels { page, add, remove } => {
            let (name, args) = if add.is_empty() && remove.is_empty() {
                ("confluence_page_labels", json!({ "page": page }))
            } else {
                (
                    "confluence_page_label",
                    json!({ "page": page, "add": add, "remove": remove }),
                )
            };
            ctx.item(name, args, |labels| {
                let names: Vec<&str> = labels
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter_map(Value::as_str)
                    .collect();
                if names.is_empty() {
                    "(no labels)".into()
                } else {
                    names.join(", ")
                }
            })
            .await
        }
        PageCommand::Move {
            page,
            parent,
            before,
            after,
        } => {
            let (position, target) = match (parent, before, after) {
                (Some(target), _, _) => ("append", target),
                (_, Some(target), _) => ("before", target),
                (_, _, Some(target)) => ("after", target),
                _ => unreachable!("clap requires one of --parent, --before, --after"),
            };
            ctx.item(
                "confluence_page_move",
                json!({ "page": page, "target": target, "position": position }),
                |_| format!("Moved {page} ({position} {target})"),
            )
            .await
        }
        PageCommand::History { page, limit } => {
            ctx.list(
                "confluence_page_history",
                json!({ "page": page, "limit": limit }),
                &[
                    col("VERSION", "number"),
                    col("CREATED", "created"),
                    col("AUTHOR", "author"),
                    flex("MESSAGE", "message"),
                ],
            )
            .await
        }
    }
}
