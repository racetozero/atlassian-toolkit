//! Confluence tasks.

use super::*;
use crate::client::Client;
use crate::confluence::{Confluence, PageRef, text_cql};
use crate::error::{Error, Result};
use crate::markup;
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::{Value, json};

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct PageArgs {
    /// Page ID, page URL, or SPACE:Title.
    page: PageRef,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct PageChildrenArgs {
    /// Page ID, page URL, or SPACE:Title.
    page: PageRef,
    /// The whole tree below the page, not only direct children.
    #[serde(default)]
    recursive: bool,
    /// With `recursive`, stop at this depth (1 = direct children).
    max_depth: Option<usize>,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct PageFindArgs {
    /// Words the title contains.
    title: String,
    space: Option<String>,
    #[serde(default = "default_limit")]
    limit: usize,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct PageListArgs {
    /// Page ID, page URL, or SPACE:Title.
    page: PageRef,
    #[serde(default = "default_limit")]
    limit: usize,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct PageCommentArgs {
    page: PageRef,
    /// Comment in Markdown.
    body: String,
    /// Reply to this comment ID.
    parent: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct PageCommentsArgs {
    page: PageRef,
    #[serde(default = "default_limit")]
    limit: usize,
    /// Inline comments instead of footer comments.
    #[serde(default)]
    inline: bool,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct IdArgs {
    /// Content ID.
    id: String,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct PageAttachmentsArgs {
    page: PageRef,
    /// Only titles matching this glob, for example "*.png".
    pattern: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct PageExportArgs {
    /// Page ID, page URL, or SPACE:Title.
    page: PageRef,
    /// Folder to write into.
    dest: std::path::PathBuf,
    /// Also export the pages below.
    #[serde(default)]
    recursive: bool,
    max_depth: Option<usize>,
    /// Title globs to skip (with their subtrees).
    #[serde(default)]
    exclude: Vec<String>,
    /// Download attachments into _attachments/ (default true).
    #[serde(default = "default_true")]
    attachments: bool,
    /// List the files without writing.
    #[serde(default)]
    dry_run: bool,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct PagePushArgs {
    /// Markdown files or folders of them.
    paths: Vec<std::path::PathBuf>,
    /// Space for new pages without one in their front matter.
    space: Option<String>,
    /// Parent page for new pages without one.
    parent: Option<String>,
    /// Overwrite pages that changed on the server since the file was written.
    #[serde(default)]
    force: bool,
    #[serde(default)]
    dry_run: bool,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct PageCopyTreeArgs {
    /// The page to copy with the pages below it.
    page: PageRef,
    /// The new parent page.
    target: PageRef,
    /// Appended to each title (default " (copy)" in the same space).
    suffix: Option<String>,
    max_depth: Option<usize>,
    #[serde(default)]
    exclude: Vec<String>,
    #[serde(default)]
    dry_run: bool,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct PageLabelArgs {
    page: PageRef,
    #[serde(default)]
    add: Vec<String>,
    #[serde(default)]
    remove: Vec<String>,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct PageMoveArgs {
    page: PageRef,
    /// The page to move next to or under.
    target: PageRef,
    /// append (last child of target), before or after (sibling; Cloud only).
    #[serde(default = "default_position")]
    position: String,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct PageCreateArgs {
    /// Space key.
    space: String,
    title: String,
    /// Parent page: ID, URL, or SPACE:Title.
    parent: Option<PageRef>,
    /// Body in Markdown.
    markdown: Option<String>,
    /// Body in storage format (instead of markdown).
    storage: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct PageUpdateArgs {
    page: PageRef,
    title: Option<String>,
    markdown: Option<String>,
    storage: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct SpaceListArgs {
    #[serde(default = "default_limit")]
    limit: usize,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct SpacePagesArgs {
    space: String,
    #[serde(default = "default_limit")]
    limit: usize,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct SearchArgs {
    /// Plain words (ignored when cql is given).
    query: Option<String>,
    /// Raw CQL.
    cql: Option<String>,
    space: Option<String>,
    /// page, blogpost, attachment, comment.
    #[serde(rename = "type")]
    content_type: Option<String>,
    #[serde(default = "default_limit")]
    limit: usize,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct PageAttachArgs {
    /// Upload new versions of attachments with the same name.
    #[serde(default)]
    replace: bool,
    /// Page ID, page URL, or SPACE:Title.
    page: PageRef,
    /// Local file paths to upload.
    files: Vec<std::path::PathBuf>,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct BlogListArgs {
    space: String,
    #[serde(default = "default_limit")]
    limit: usize,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct BlogGetArgs {
    /// Blog post ID.
    id: String,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct BlogCreateArgs {
    space: String,
    title: String,
    /// Body in Markdown.
    markdown: Option<String>,
    /// Body in storage format (instead of markdown).
    storage: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct SpaceCreateArgs {
    key: String,
    name: String,
    /// Plain text.
    description: Option<String>,
}

fn default_position() -> String {
    "append".into()
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct PageGetArgs {
    /// Page ID, page URL, or SPACE:Title.
    page: PageRef,
    /// Also return `document`: the Markdown with front matter (id, title, space, parent, version), ready for confluence_page_push.
    #[serde(default)]
    front_matter: bool,
    /// Also return the storage-format XHTML.
    #[serde(default)]
    storage: bool,
    /// An older version number (see confluence_page_history).
    version: Option<u64>,
}

pub(super) static TASKS: &[TaskDef] = &[
    task!(
        "confluence_page_get",
        true,
        PageGetArgs,
        "A Confluence page as Markdown (with title, space, version, url)."
    ),
    task!(
        "confluence_page_create",
        false,
        PageCreateArgs,
        "Create a Confluence page from Markdown (or storage format)."
    ),
    task!(
        "confluence_blog_list",
        true,
        BlogListArgs,
        "Blog posts of a Confluence space, newest first."
    ),
    task!(
        "confluence_blog_get",
        true,
        BlogGetArgs,
        "One Confluence blog post with its body as Markdown."
    ),
    task!(
        "confluence_blog_create",
        false,
        BlogCreateArgs,
        "Publish a Confluence blog post from Markdown."
    ),
    task!(
        "confluence_space_create",
        false,
        SpaceCreateArgs,
        "Create a Confluence space with a key, a name and an optional plain-text description."
    ),
    task!(
        "confluence_page_update",
        false,
        PageUpdateArgs,
        "Replace a Confluence page's title and/or body; the version is bumped automatically."
    ),
    task!(
        "confluence_page_delete",
        false,
        PageArgs,
        "Delete a Confluence page.",
        destructive
    ),
    task!(
        "confluence_page_children",
        true,
        PageChildrenArgs,
        "Children of a Confluence page (Cloud: also folders), or with `recursive` the whole tree, depth first, each row with type, depth and parent."
    ),
    task!(
        "confluence_page_find",
        true,
        PageFindArgs,
        "Confluence pages whose title contains the words, optionally in one space."
    ),
    task!(
        "confluence_page_attach",
        false,
        PageAttachArgs,
        "Upload local files to a Confluence page."
    ),
    task!(
        "confluence_page_attachments",
        true,
        PageAttachmentsArgs,
        "Attachments of a Confluence page with download URLs."
    ),
    task!(
        "confluence_page_comments",
        true,
        PageCommentsArgs,
        "Footer (or with `inline`, inline) comments of a Confluence page, oldest first, as Markdown, with the replied-to `parent`."
    ),
    task!(
        "confluence_page_comment",
        false,
        PageCommentArgs,
        "Add a footer comment (Markdown) to a Confluence page, or a reply with `parent`."
    ),
    task!(
        "confluence_comment_delete",
        false,
        IdArgs,
        "Delete a Confluence comment by ID.",
        destructive
    ),
    task!(
        "confluence_attachment_delete",
        false,
        IdArgs,
        "Delete a Confluence attachment by ID.",
        destructive
    ),
    task!(
        "confluence_page_export",
        true,
        PageExportArgs,
        "Export a Confluence page (and with `recursive` its tree) to local Markdown folders: <title>/index.md with front matter and _attachments/. Changes nothing on the server."
    ),
    task!(
        "confluence_page_push",
        false,
        PagePushArgs,
        "Publish local Markdown files: update the page in each file's front matter (refusing pages changed on the server unless `force`, skipping unchanged ones), create pages for new files under their folder's index.md, upload referenced images, and write ids and versions back."
    ),
    task!(
        "confluence_page_copy_tree",
        false,
        PageCopyTreeArgs,
        "Copy a Confluence page and the pages below it under another page (titles, bodies, labels)."
    ),
    task!(
        "confluence_page_labels",
        true,
        PageArgs,
        "Labels of a Confluence page."
    ),
    task!(
        "confluence_page_label",
        false,
        PageLabelArgs,
        "Add and remove labels on a Confluence page; returns the resulting labels."
    ),
    task!(
        "confluence_page_move",
        false,
        PageMoveArgs,
        "Move a Confluence page under another page (position append) or next to it (before/after, Cloud only)."
    ),
    task!(
        "confluence_page_history",
        true,
        PageListArgs,
        "Versions of a Confluence page, newest first: number, author, date, message. Read one with confluence_page_get and `version`."
    ),
    task!(
        "confluence_space_list",
        true,
        SpaceListArgs,
        "Confluence spaces."
    ),
    task!(
        "confluence_space_pages",
        true,
        SpacePagesArgs,
        "Top-level pages of a Confluence space."
    ),
    task!(
        "confluence_search",
        true,
        SearchArgs,
        "Search Confluence by words or CQL; returns id, type, title, space, excerpt, url."
    ),
];

pub(super) async fn run(client: &Client, name: &str, args: Value) -> Result<Value> {
    let confluence = Confluence::new(client);
    match name {
        "confluence_page_get" => {
            let a: PageGetArgs = parse(name, args)?;
            let page = confluence.page_at(&a.page, a.version).await?;
            let mut value = to_json(&page)?;
            if a.front_matter {
                value["document"] = crate::docs::page_document(&page).into();
            }
            if a.storage {
                value["storage"] = page.storage.into();
            }
            Ok(value)
        }
        "confluence_page_comments" => {
            let a: PageCommentsArgs = parse(name, args)?;
            to_json(confluence.comments(&a.page, a.limit, a.inline).await?)
        }
        "confluence_page_comment" => {
            let a: PageCommentArgs = parse(name, args)?;
            to_json(
                confluence
                    .comment(&a.page, &a.body, a.parent.as_deref())
                    .await?,
            )
        }
        "confluence_comment_delete" => {
            let a: IdArgs = parse(name, args)?;
            confluence.delete_comment(&a.id).await?;
            Ok(json!({ "id": a.id, "deleted": true }))
        }
        "confluence_attachment_delete" => {
            let a: IdArgs = parse(name, args)?;
            confluence.delete_attachment(&a.id).await?;
            Ok(json!({ "id": a.id, "deleted": true }))
        }
        "confluence_page_export" => {
            let a: PageExportArgs = parse(name, args)?;
            let options = crate::docs::ExportOptions {
                recursive: a.recursive,
                max_depth: a.max_depth,
                exclude: a.exclude,
                attachments: a.attachments,
                dry_run: a.dry_run,
            };
            to_json(crate::docs::export(&confluence, &a.page, &a.dest, &options).await?)
        }
        "confluence_page_push" => {
            let a: PagePushArgs = parse(name, args)?;
            let options = crate::docs::PushOptions {
                space: a.space,
                parent: a.parent,
                force: a.force,
                dry_run: a.dry_run,
            };
            to_json(crate::docs::push(&confluence, &a.paths, &options).await?)
        }
        "confluence_page_copy_tree" => {
            let a: PageCopyTreeArgs = parse(name, args)?;
            let options = crate::docs::CopyOptions {
                suffix: a.suffix,
                max_depth: a.max_depth,
                exclude: a.exclude,
                dry_run: a.dry_run,
            };
            to_json(crate::docs::copy_tree(&confluence, &a.page, &a.target, &options).await?)
        }
        "confluence_page_labels" => {
            let a: PageArgs = parse(name, args)?;
            to_json(confluence.labels(&a.page).await?)
        }
        "confluence_page_label" => {
            let a: PageLabelArgs = parse(name, args)?;
            to_json(confluence.change_labels(&a.page, &a.add, &a.remove).await?)
        }
        "confluence_page_move" => {
            let a: PageMoveArgs = parse(name, args)?;
            to_json(
                confluence
                    .move_page(&a.page, a.position.parse()?, &a.target)
                    .await?,
            )
        }
        "confluence_page_history" => {
            let a: PageListArgs = parse(name, args)?;
            to_json(confluence.versions(&a.page, a.limit).await?)
        }
        "confluence_page_create" => {
            let a: PageCreateArgs = parse(name, args)?;
            let storage = body(a.markdown, a.storage)?
                .ok_or_else(|| Error::Invalid("pass `markdown` or `storage`".into()))?;
            to_json(
                confluence
                    .create(&a.space, &a.title, a.parent.as_ref(), storage)
                    .await?,
            )
        }
        "confluence_page_update" => {
            let a: PageUpdateArgs = parse(name, args)?;
            let storage = body(a.markdown, a.storage)?;
            to_json(
                confluence
                    .update(&a.page, a.title.as_deref(), storage)
                    .await?,
            )
        }
        "confluence_page_delete" => {
            let a: PageArgs = parse(name, args)?;
            let id = confluence.delete(&a.page).await?;
            Ok(json!({ "id": id, "deleted": true }))
        }
        "confluence_page_children" => {
            let a: PageChildrenArgs = parse(name, args)?;
            let depth = if a.recursive { a.max_depth } else { Some(1) };
            to_json(confluence.tree(&a.page, depth).await?)
        }
        "confluence_page_find" => {
            let a: PageFindArgs = parse(name, args)?;
            to_json(
                confluence
                    .find(&a.title, a.space.as_deref(), a.limit)
                    .await?,
            )
        }
        "confluence_page_attach" => {
            let a: PageAttachArgs = parse(name, args)?;
            let page = a.page;
            to_json(match a.replace {
                true => confluence.attach_replace(&page, &a.files).await?,
                false => confluence.attach(&page, &a.files).await?,
            })
        }
        "confluence_page_attachments" => {
            let a: PageAttachmentsArgs = parse(name, args)?;
            let mut found = confluence.attachments(&a.page).await?;
            if let Some(pattern) = &a.pattern {
                let glob = crate::docs::TitleGlobs::new(std::slice::from_ref(pattern));
                found.retain(|att| glob.is_match(&att.title));
            }
            to_json(found)
        }
        "confluence_space_list" => {
            let a: SpaceListArgs = parse(name, args)?;
            to_json(confluence.spaces(a.limit).await?)
        }
        "confluence_space_pages" => {
            let a: SpacePagesArgs = parse(name, args)?;
            to_json(confluence.root_pages(&a.space, a.limit).await?)
        }
        "confluence_search" => {
            let a: SearchArgs = parse(name, args)?;
            let cql = match (a.cql, a.query) {
                (Some(cql), _) => cql,
                (None, Some(words)) => {
                    text_cql(&words, a.space.as_deref(), a.content_type.as_deref())
                }
                (None, None) => return Err(Error::Invalid("pass `query` (words) or `cql`".into())),
            };
            Ok(json!({ "cql": cql, "results": to_json(confluence.search(&cql, a.limit).await?)? }))
        }
        "confluence_blog_list" => {
            let a: BlogListArgs = parse(name, args)?;
            to_json(confluence.blog_posts(&a.space, a.limit).await?)
        }
        "confluence_blog_get" => {
            let a: BlogGetArgs = parse(name, args)?;
            to_json(confluence.blog_post(&a.id).await?)
        }
        "confluence_blog_create" => {
            let a: BlogCreateArgs = parse(name, args)?;
            let storage = body(a.markdown, a.storage)?
                .ok_or_else(|| Error::Invalid("pass `markdown` or `storage`".into()))?;
            to_json(
                confluence
                    .create_blog_post(&a.space, &a.title, storage)
                    .await?,
            )
        }
        "confluence_space_create" => {
            let a: SpaceCreateArgs = parse(name, args)?;
            to_json(
                confluence
                    .create_space(&a.key, &a.name, a.description.as_deref())
                    .await?,
            )
        }
        _ => Err(unknown(name)),
    }
}

fn body(markdown: Option<String>, storage: Option<String>) -> Result<Option<String>> {
    match (markdown, storage) {
        (Some(_), Some(_)) => Err(Error::Invalid(
            "pass either `markdown` or `storage`, not both".into(),
        )),
        (Some(markdown), None) => Ok(Some(markup::markdown_to_storage(&markdown))),
        (None, storage) => Ok(storage),
    }
}
