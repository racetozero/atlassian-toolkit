//! `atl`: one CLI and MCP server for the Atlassian suite.

mod admin;
mod bitbucket;
mod browse;
mod cli;
mod confluence;
mod jira;
mod jsm;
mod login;
mod output;
mod prompt;
mod skill;
mod tasks;
mod trello;
mod update;

use serde_json::json;
use std::io::{IsTerminal, Write};
use std::process::ExitCode;

/// Process exit codes. Scripts can branch on them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Exit {
    Ok = 0,
    /// The server answered with an error status.
    Api = 1,
    /// No credential, or the server rejected it (401/403).
    Auth = 2,
    /// The input was invalid (bad flag, JSON, missing parameter).
    Invalid = 3,
    /// Unknown service, resource, or operation.
    Unknown = 4,
    /// Network, file system, or other failures.
    Internal = 5,
}

/// Runs on a thread with an 8 MiB stack: Windows gives the main thread
/// 1 MiB, and in debug builds clap's derived `jira issue` command tree
/// needs more than that.
fn main() -> ExitCode {
    std::thread::Builder::new()
        .name("atl".into())
        .stack_size(8 << 20)
        .spawn(|| {
            // `run` itself runs on this thread; one worker is enough for the
            // spawned tasks (the update check, the browser's loaders) and
            // starts much faster than one worker per core. A multi-thread
            // runtime keeps `block_in_place` working in the login form.
            tokio::runtime::Builder::new_multi_thread()
                .worker_threads(1)
                .enable_all()
                .build()
                .expect("cannot start the tokio runtime")
                .block_on(run())
        })
        .expect("cannot start the main thread")
        .join()
        .unwrap_or_else(|panic| std::panic::resume_unwind(panic))
}

async fn run() -> ExitCode {
    // Answers shell completion requests (COMPLETE=<shell>) and exits.
    clap_complete::CompleteEnv::with_factory(cli::completion_command).complete();
    let args: Vec<String> = std::env::args().collect();
    let update = update::start(&args);
    let result = cli::run(args).await;
    let code = report(result);
    update::finish(update).await;
    code
}

/// Prints an error (an envelope on stdout in agent mode, one line of text
/// on a terminal, JSON on stderr otherwise) and turns the result into the
/// documented exit code.
fn report(result: anyhow::Result<Exit>) -> ExitCode {
    match result {
        Ok(code) => ExitCode::from(code as u8),
        Err(err) => {
            let (code, kind) = classify(&err);
            let mut error = err
                .downcast_ref::<atl_sdk::Error>()
                .map_or_else(|| json!({}), atl_sdk::Error::to_json);
            error["code"] = kind.into();
            error["message"] = format!("{err:#}").into();
            if output::agent_mode() {
                // Agents read one JSON document from stdout, success or not.
                let envelope =
                    json!({ "ok": false, "command": output::command_name(), "error": error });
                output::print_line(&envelope.to_string());
            } else if std::io::stderr().is_terminal() {
                // People read one line, plus the values they can pass.
                let mut text = format!("error: {err:#}");
                if let Some(atl_sdk::Error::NeedsInput(need)) = err.downcast_ref() {
                    text.push_str(&format!("\nvalid values for {}:", need.arg));
                    for choice in &need.choices {
                        match choice.label == choice.value {
                            true => text.push_str(&format!("\n  {}", choice.value)),
                            false => {
                                text.push_str(&format!("\n  {}  ({})", choice.value, choice.label))
                            }
                        }
                    }
                }
                let _ = writeln!(std::io::stderr(), "{text}");
            } else {
                let text =
                    serde_json::to_string_pretty(&json!({ "error": error })).unwrap_or_default();
                let _ = writeln!(std::io::stderr(), "{text}");
            }
            ExitCode::from(code as u8)
        }
    }
}

fn classify(err: &anyhow::Error) -> (Exit, &'static str) {
    if err.downcast_ref::<tasks::NeedsConfirmation>().is_some() {
        return (Exit::Invalid, "confirmation_required");
    }
    match err.downcast_ref::<atl_sdk::Error>() {
        Some(sdk) => {
            let code = sdk.code();
            // Every code in atl_sdk::error::CODES.
            let exit = match code {
                "auth" => Exit::Auth,
                "api" => Exit::Api,
                "invalid" | "read_only" | "input_required" => Exit::Invalid,
                "unknown" => Exit::Unknown,
                // The CLI turns dry runs into output before they get here.
                "dry_run" | "internal" => Exit::Internal,
                other => unreachable!("atl_sdk::error::CODES has no `{other}`"),
            };
            (exit, code)
        }
        None => (Exit::Internal, "internal"),
    }
}
