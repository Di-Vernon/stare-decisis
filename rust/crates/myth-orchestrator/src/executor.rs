//! 단일 Task 실행 조율: worktree 생성 → claude 실행 → 결과 정리.
//!
//! Day-1은 tmux 동반 없이 ClaudeRuntime::execute_with_watchdog 직접 호출.
//! tmux + Squad 통합 경로는 Wave 8에서 확장.

use std::path::Path;
use std::time::Duration;

use myth_db::JsonlWriter;
use myth_runtime::{ClaudeRuntime, TaskResult};
use serde::Serialize;

use crate::plan::Task;
use crate::report::TaskOutcome;

#[derive(Debug, Serialize)]
struct OrchestratorFailure {
    ts: String,
    source: &'static str,
    task_id: String,
    reason: Option<String>,
    stderr_excerpt: String,
    duration_ms: u64,
}

/// Day-1 경로: worktree 지정(호출측이 준비), claude 바로 실행 → outcome 반환.
///
/// `base_repo`는 실제 worktree 루트 경로(이미 생성됨). 호출자가
/// [`crate::worktree::create`]를 먼저 호출해 Worktree를 준비하고
/// `base_repo = worktree.path`를 넘긴다.
pub async fn execute_task(
    task: &Task,
    base_repo: &Path,
    task_timeout: Duration,
    stale_threshold: Duration,
) -> TaskOutcome {
    let runtime = match ClaudeRuntime::new(base_repo) {
        Ok(r) => r,
        Err(e) => {
            return TaskOutcome {
                task_id: task.id.clone(),
                succeeded: false,
                exit_code: -1,
                duration_ms: 0,
                failure_reason: Some(format!("runtime init failed: {e}")),
            };
        }
    };

    let effective_timeout = task
        .timeout_seconds
        .map(Duration::from_secs)
        .unwrap_or(task_timeout);

    let result = runtime
        .execute_with_watchdog(&task.prompt, effective_timeout, stale_threshold)
        .await;

    if !result.succeeded {
        let _ = record_failure(&task.id, &result);
    }

    TaskOutcome {
        task_id: task.id.clone(),
        succeeded: result.succeeded,
        exit_code: result.exit_code,
        duration_ms: result.duration.as_millis() as u64,
        failure_reason: result.failure_reason.clone(),
    }
}

fn record_failure(task_id: &str, result: &TaskResult) -> anyhow::Result<()> {
    let record = OrchestratorFailure {
        ts: myth_common::format_iso(&myth_common::now()),
        source: "orchestrator",
        task_id: task_id.to_string(),
        reason: result.failure_reason.clone(),
        stderr_excerpt: result.stderr_tail(1000),
        duration_ms: result.duration.as_millis() as u64,
    };
    JsonlWriter::new(myth_common::caselog_path()).append(&record)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn record_failure_serializes() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("caselog.jsonl");
        let record = OrchestratorFailure {
            ts: "2026-04-21T12:00:00Z".into(),
            source: "orchestrator",
            task_id: "T1".into(),
            reason: Some("boom".into()),
            stderr_excerpt: "stderr".into(),
            duration_ms: 42,
        };
        JsonlWriter::new(&path).append(&record).unwrap();
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("\"task_id\":\"T1\""));
        assert!(content.contains("\"source\":\"orchestrator\""));
        assert!(content.contains("\"duration_ms\":42"));
    }
}
