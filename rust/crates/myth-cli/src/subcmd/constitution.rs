//! `myth constitution` — CONSTITUTION.md 열람 ($PAGER).

use anyhow::{anyhow, Result};
use std::process::ExitCode;
use tokio::process::Command;

use crate::args::ConstitutionArgs;

pub async fn run(_args: ConstitutionArgs) -> Result<ExitCode> {
    let path = myth_common::myth_home().join("CONSTITUTION.md");
    if !path.exists() {
        return Err(anyhow!(
            "CONSTITUTION.md not found at {}",
            path.display()
        ));
    }
    let pager = std::env::var("PAGER").unwrap_or_else(|_| "less".to_string());
    let status = Command::new(&pager).arg(&path).status().await?;
    Ok(ExitCode::from(status.code().unwrap_or(0) as u8))
}
