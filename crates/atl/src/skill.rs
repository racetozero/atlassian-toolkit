//! `atl skill`: the agent skills, so an agent can drive `atl` from a shell
//! instead of loading MCP tool schemas.
//!
//! The skills are written in `.agents/skills/<name>/SKILL.md`, where
//! `npx skills add racetozero/atlassian-toolkit` finds them. `cargo publish`
//! packages only files inside the crate, so `crates/atl/skills/<name>.md`
//! holds copies; a test keeps them equal.

use crate::Exit;
use crate::output::{Out, col, flex};
use anyhow::{Context, Result};
use serde_json::{Value, json};
use std::path::PathBuf;

/// Every skill as (name, SKILL.md text). The first one is the core skill.
pub const SKILLS: [(&str, &str); 7] = [
    ("atl", include_str!("../skills/atl.md")),
    ("atl-admin", include_str!("../skills/atl-admin.md")),
    ("atl-bitbucket", include_str!("../skills/atl-bitbucket.md")),
    (
        "atl-confluence",
        include_str!("../skills/atl-confluence.md"),
    ),
    ("atl-jira", include_str!("../skills/atl-jira.md")),
    ("atl-jsm", include_str!("../skills/atl-jsm.md")),
    ("atl-trello", include_str!("../skills/atl-trello.md")),
];

fn description(text: &str) -> &str {
    text.lines()
        .find_map(|line| line.strip_prefix("description: "))
        .unwrap_or_default()
}

pub fn run(out: &Out, name: Option<String>, list: bool, install: Option<PathBuf>) -> Result<Exit> {
    if list {
        let rows: Vec<Value> = SKILLS
            .iter()
            .map(|(name, text)| json!({ "name": name, "description": description(text) }))
            .collect();
        return out.list(
            rows,
            &[col("NAME", "name"), flex("DESCRIPTION", "description")],
            Value::Null,
        );
    }
    if let Some(dir) = install {
        let mut installed = Vec::new();
        for (name, text) in SKILLS {
            let folder = dir.join(name);
            std::fs::create_dir_all(&folder)
                .with_context(|| format!("cannot create {}", folder.display()))?;
            let path = folder.join("SKILL.md");
            std::fs::write(&path, text)
                .with_context(|| format!("cannot write {}", path.display()))?;
            installed.push(path.display().to_string());
        }
        return out.item(json!({ "installed": installed }), Value::Null, |_| {
            format!("Installed {} skills in {}", SKILLS.len(), dir.display())
        });
    }
    let name = name.as_deref().unwrap_or("atl");
    let Some((_, text)) = SKILLS.iter().find(|(skill, _)| *skill == name) else {
        let names: Vec<&str> = SKILLS.iter().map(|(name, _)| *name).collect();
        anyhow::bail!(atl_sdk::Error::Invalid(format!(
            "there is no skill named `{name}`; use one of: {}",
            names.join(", ")
        )));
    };
    if out.agent {
        return out.item(json!({ "name": name, "skill": text }), Value::Null, |_| {
            String::new()
        });
    }
    // Plain Markdown, so `atl skill atl-jira > SKILL.md` works.
    crate::output::print_line(text.trim_end());
    Ok(Exit::Ok)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The embedded copies equal `.agents/skills/<name>/SKILL.md`, and every
    /// skill there is embedded. `ATL_UPDATE_SKILLS=1` rewrites the copies.
    #[test]
    fn embedded_skills_match_agents_skills() {
        let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");
        let mut on_disk: Vec<String> = std::fs::read_dir(format!("{root}/.agents/skills"))
            .expect(".agents/skills")
            .map(|entry| {
                entry
                    .expect("entry")
                    .file_name()
                    .to_string_lossy()
                    .into_owned()
            })
            .collect();
        on_disk.sort();
        let embedded: Vec<&str> = SKILLS.iter().map(|(name, _)| *name).collect();
        assert_eq!(
            on_disk, embedded,
            "add each skill in .agents/skills to SKILLS"
        );
        for (name, text) in SKILLS {
            let source = std::fs::read_to_string(format!("{root}/.agents/skills/{name}/SKILL.md"))
                .expect("SKILL.md");
            if std::env::var_os("ATL_UPDATE_SKILLS").is_some() {
                std::fs::write(format!("{root}/crates/atl/skills/{name}.md"), &source)
                    .expect("write");
                continue;
            }
            assert!(
                source == text,
                "crates/atl/skills/{name}.md differs from .agents/skills/{name}/SKILL.md; run `ATL_UPDATE_SKILLS=1 cargo test -p atl --bin atl skills`"
            );
            assert!(
                text.starts_with(&format!("---\nname: {name}\n")),
                "{name}: front matter names the skill"
            );
            assert!(
                !description(text).is_empty(),
                "{name}: front matter has a description"
            );
            // YAML reads `: ` inside a plain value as a nested mapping, and the
            // skills CLI then skips the skill.
            assert!(
                !description(text).contains(": "),
                "{name}: the description must not contain `: `"
            );
        }
    }
}
