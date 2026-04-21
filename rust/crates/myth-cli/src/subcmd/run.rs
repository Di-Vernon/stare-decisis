//! `myth run [--plan ...]` — Claude Code 실행 (인터랙티브 또는 병렬).
//!
//! Wave 4 API 앵커 #1 (ClaudeRuntime::new) + #2 (Orchestrator::execute_plan) 사용.

use anyhow::Result;
use myth_orchestrator::{Orchestrator, OrchestratorConfig};
use myth_runtime::ClaudeRuntime;
use std::process::ExitCode;

use crate::args::RunArgs;

pub async fn run(args: RunArgs) -> Result<ExitCode> {
    if let Some(plan_path) = args.plan {
        // 병렬 실행 — Wave 4 앵커 #2
        let config = OrchestratorConfig {
            max_concurrent: args.max_concurrent,
            ..OrchestratorConfig::default()
        };
        let orch = Orchestrator::new(config);
        let base_repo = std::env::current_dir()?;

        let report = orch.execute_plan(&plan_path, &base_repo).await?;
        println!("{}", report.to_summary());
        if report.failed() == 0 {
            Ok(ExitCode::SUCCESS)
        } else {
            Ok(ExitCode::from(1))
        }
    } else {
        // 인터랙티브 — Wave 4 앵커 #1
        let worktree = std::env::current_dir()?;
        let runtime = ClaudeRuntime::new(&worktree)?;
        tracing::info!(session = %runtime.session_id(), "starting claude session");
        runtime.spawn_interactive().await
    }
}
