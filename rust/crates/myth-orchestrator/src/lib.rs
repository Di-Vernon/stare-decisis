//! `myth-orchestrator` — 병렬 Task 실행 & 라이프사이클.
//!
//! Wave-based plan을 받아 각 wave 안의 task들을 `max_concurrent` 이하로
//! 병렬 실행한다. state.db에 직접 연결하지 않는다 — 실패는 JSONL caselog로만.

pub mod executor;
pub mod plan;
pub mod report;
pub mod squad;
pub mod tmux;
pub mod watchdog;
pub mod worktree;

pub use plan::{load as load_plan, Plan, Task, Wave};
pub use report::{ExecutionReport, TaskOutcome, WaveResult};
pub use watchdog::{Watchdog, WatchdogResult};
pub use worktree::{MergeResult, Worktree};

use anyhow::Result;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;

#[derive(Debug, Clone)]
pub struct OrchestratorConfig {
    pub max_concurrent: usize,
    pub task_timeout: Duration,
    pub stale_threshold: Duration,
    pub worktree_base: PathBuf,
}

impl Default for OrchestratorConfig {
    fn default() -> Self {
        Self {
            max_concurrent: 3,
            task_timeout: Duration::from_secs(11 * 60),
            stale_threshold: Duration::from_secs(2 * 60),
            worktree_base: myth_common::myth_home().join("worktrees"),
        }
    }
}

#[derive(Clone)]
pub struct Orchestrator {
    config: Arc<OrchestratorConfig>,
}

impl Orchestrator {
    pub fn new(config: OrchestratorConfig) -> Self {
        Self {
            config: Arc::new(config),
        }
    }

    pub fn config(&self) -> &OrchestratorConfig {
        &self.config
    }

    /// plan.json 로드 후 각 wave를 순차 실행. 하나의 wave에서 critical
    /// failure가 있으면 이후 wave 중단.
    pub async fn execute_plan(
        &self,
        plan_path: &Path,
        base_repo: &Path,
    ) -> Result<ExecutionReport> {
        let plan = plan::load(plan_path)?;
        let mut report = ExecutionReport::new();

        for wave in &plan.waves {
            let wave_result = self.execute_wave(wave, base_repo).await;
            let stop = wave_result.has_critical_failure();
            report.waves.push(wave_result);
            if stop {
                tracing::warn!(
                    wave_id = wave.id.as_str(),
                    "critical failure in wave, stopping execution"
                );
                break;
            }
        }

        report.mark_ended();
        Ok(report)
    }

    pub async fn execute_wave(&self, wave: &Wave, base_repo: &Path) -> WaveResult {
        let limit = wave
            .max_concurrent
            .unwrap_or(self.config.max_concurrent)
            .max(1);
        let sem = Arc::new(Semaphore::new(limit));

        let mut joinset = tokio::task::JoinSet::new();
        for task in wave.tasks.clone() {
            let sem_c = sem.clone();
            let timeout = self.config.task_timeout;
            let stale = self.config.stale_threshold;
            let base = base_repo.to_path_buf();

            joinset.spawn(async move {
                let _permit = sem_c.acquire_owned().await.ok();
                run_single_task(&task, &base, timeout, stale).await
            });
        }

        let mut outcomes = Vec::new();
        while let Some(res) = joinset.join_next().await {
            match res {
                Ok(outcome) => outcomes.push(outcome),
                Err(e) => outcomes.push(TaskOutcome {
                    task_id: "unknown".into(),
                    succeeded: false,
                    exit_code: -1,
                    duration_ms: 0,
                    failure_reason: Some(format!("join error: {e}")),
                }),
            }
        }

        // task order로 정렬 (spawn 순서와 무관)
        outcomes.sort_by(|a, b| {
            let order_a = wave
                .tasks
                .iter()
                .position(|t| t.id == a.task_id)
                .unwrap_or(usize::MAX);
            let order_b = wave
                .tasks
                .iter()
                .position(|t| t.id == b.task_id)
                .unwrap_or(usize::MAX);
            order_a.cmp(&order_b)
        });

        WaveResult {
            wave_id: wave.id.clone(),
            tasks: outcomes,
        }
    }
}

/// Worktree 생성 → claude 실행 → worktree 정리. 임의 실패 시에도
/// TaskOutcome로 수렴.
async fn run_single_task(
    task: &Task,
    base_repo: &Path,
    task_timeout: Duration,
    stale_threshold: Duration,
) -> TaskOutcome {
    let worktree = match worktree::create(base_repo, &task.id).await {
        Ok(w) => w,
        Err(e) => {
            return TaskOutcome {
                task_id: task.id.clone(),
                succeeded: false,
                exit_code: -1,
                duration_ms: 0,
                failure_reason: Some(format!("worktree create failed: {e}")),
            };
        }
    };

    let outcome =
        executor::execute_task(task, &worktree.path, task_timeout, stale_threshold).await;

    if let Err(e) = worktree::remove(&worktree).await {
        tracing::warn!(
            task_id = task.id.as_str(),
            "worktree cleanup failed: {e}"
        );
    }

    outcome
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_values() {
        let c = OrchestratorConfig::default();
        assert_eq!(c.max_concurrent, 3);
        assert_eq!(c.task_timeout, Duration::from_secs(11 * 60));
        assert_eq!(c.stale_threshold, Duration::from_secs(2 * 60));
        assert!(c.worktree_base.ends_with("worktrees"));
    }

    #[test]
    fn orchestrator_new_stores_config() {
        let c = OrchestratorConfig::default();
        let o = Orchestrator::new(c);
        assert_eq!(o.config().max_concurrent, 3);
    }

    #[tokio::test]
    async fn execute_wave_with_zero_tasks_returns_empty() {
        let o = Orchestrator::new(OrchestratorConfig::default());
        let wave = Wave {
            id: "w0".into(),
            description: String::new(),
            tasks: vec![],
            max_concurrent: None,
        };
        let wr = o.execute_wave(&wave, Path::new(".")).await;
        assert_eq!(wr.wave_id, "w0");
        assert!(wr.tasks.is_empty());
    }
}
