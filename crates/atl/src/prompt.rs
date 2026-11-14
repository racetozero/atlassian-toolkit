//! Questions for a person at a terminal: choose one value, type a line,
//! or answer yes/no. Prompts write to stderr, so stdout stays clean for
//! pipes, and never run for agents, `--no-input`, `ATL_NO_INPUT`, or
//! without a terminal; commands then fail with the missing value instead.

use crate::output::agent_mode;
use anyhow::Result;
use atl_sdk::Error;
use atl_sdk::error::Choice;
use dialoguer::console::Term;
use dialoguer::theme::ColorfulTheme;
use dialoguer::{Confirm, Input, Select};
use std::io::IsTerminal;
use std::sync::OnceLock;

static DISABLED: OnceLock<bool> = OnceLock::new();

/// From `--no-input` / `ATL_NO_INPUT`, once clap has parsed.
pub fn set_disabled(disabled: bool) {
    let _ = DISABLED.set(disabled);
}

/// True when a person can be asked.
pub fn allowed() -> bool {
    !agent_mode()
        && !DISABLED.get().copied().unwrap_or(false)
        && std::io::stdin().is_terminal()
        && std::io::stderr().is_terminal()
}

fn cancelled() -> anyhow::Error {
    Error::Invalid("cancelled; nothing was changed".into()).into()
}

fn failed(err: dialoguer::Error) -> anyhow::Error {
    Error::Invalid(format!("cannot read the answer: {err}")).into()
}

/// Shows the choices and returns the chosen value.
pub fn select(label: &str, choices: &[Choice]) -> Result<String> {
    let labels: Vec<&str> = choices.iter().map(|c| c.label.as_str()).collect();
    let index = Select::with_theme(&ColorfulTheme::default())
        .with_prompt(label)
        .items(&labels)
        .default(0)
        .max_length(15)
        .interact_on_opt(&Term::stderr())
        .map_err(failed)?
        .ok_or_else(cancelled)?;
    Ok(choices[index].value.clone())
}

/// One non-empty line of text.
pub fn text(label: &str) -> Result<String> {
    let answer: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt(label)
        .validate_with(|text: &String| match text.trim().is_empty() {
            true => Err("enter a value (Ctrl-C cancels)"),
            false => Ok(()),
        })
        .interact_text_on(&Term::stderr())
        .map_err(failed)?;
    Ok(answer.trim().to_owned())
}

pub fn confirm(label: &str, default: bool) -> Result<bool> {
    Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt(label)
        .default(default)
        .interact_on_opt(&Term::stderr())
        .map_err(failed)?
        .ok_or_else(cancelled)
}
