//! `myth embed status/stop/probe` — myth-embed 바이너리 위임.

use anyhow::{anyhow, Result};
use std::process::ExitCode;
use tokio::process::Command;

use crate::args::{EmbedAction, EmbedArgs};

pub async fn run(args: EmbedArgs) -> Result<ExitCode> {
    let exe = locate_myth_embed()?;
    let mut cmd = Command::new(&exe);
    match &args.action {
        EmbedAction::Status => {
            cmd.arg("status");
        }
        EmbedAction::Stop => {
            cmd.arg("stop");
        }
        EmbedAction::Probe { text } => {
            cmd.args(["probe", text]);
        }
    }
    let status = cmd.status().await?;
    Ok(ExitCode::from(status.code().unwrap_or(1) as u8))
}

fn locate_myth_embed() -> Result<std::path::PathBuf> {
    // current_exe와 같은 디렉토리 우선 (install 후 ~/.local/bin 또는 target/release)
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            let candidate = parent.join("myth-embed");
            if candidate.exists() {
                return Ok(candidate);
            }
        }
    }
    // MYTH_REPO_ROOT fallback
    if let Ok(root) = std::env::var("MYTH_REPO_ROOT") {
        let p = std::path::PathBuf::from(root).join("rust/target/release/myth-embed");
        if p.exists() {
            return Ok(p);
        }
    }
    Err(anyhow!(
        "myth-embed binary not found. Run `myth install` or set MYTH_REPO_ROOT."
    ))
}
