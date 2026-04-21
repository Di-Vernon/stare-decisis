//! `myth watch` — ratatui 대시보드.
//!
//! 드리프트 6 대응: `run_dashboard`가 Wave 4에서 `session_short` 파라미터를 받도록
//! 확장됨. docs/10은 0 arg 지시. CLI가 기본 SessionId를 생성해서 전달.

use anyhow::Result;
use myth_common::SessionId;
use std::process::ExitCode;

use crate::args::WatchArgs;

pub async fn run(_args: WatchArgs) -> Result<ExitCode> {
    let session = SessionId::new();
    myth_ui::run_dashboard(session.short()).await?;
    Ok(ExitCode::SUCCESS)
}
