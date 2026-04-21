//! `myth observer run` — Python subprocess 호출.
//!
//! 전략 A: myth_py.observer.cli 모듈 실행. 모듈/Python 미설치 시 stderr 전파 +
//! ExitCode 1. Wave 6 전엔 `ModuleNotFoundError`가 자연스러움.

use anyhow::Result;
use myth_common::SessionId;
use std::process::ExitCode;
use tokio::process::Command;

use crate::args::{ObserverAction, ObserverArgs};

pub async fn run(args: ObserverArgs) -> Result<ExitCode> {
    match args.action {
        ObserverAction::Run { dry } => run_weekly(dry).await,
    }
}

async fn run_weekly(dry: bool) -> Result<ExitCode> {
    println!("Running Observer weekly analysis...");
    let mut cmd = Command::new("python3");
    cmd.args(["-m", "myth_py.observer.cli", "run"]);
    if dry {
        cmd.arg("--dry");
    }
    cmd.env("MYTH_SESSION_ID", SessionId::new().to_string());

    let output = cmd.output().await?;
    std::io::Write::write_all(&mut std::io::stdout(), &output.stdout).ok();
    std::io::Write::write_all(&mut std::io::stderr(), &output.stderr).ok();

    Ok(ExitCode::from(output.status.code().unwrap_or(1) as u8))
}
