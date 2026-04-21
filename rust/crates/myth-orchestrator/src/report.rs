//! 실행 리포트 — summary(CLI용) + markdown(brief.md 삽입용).

use myth_common::{now, Timestamp};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskOutcome {
    pub task_id: String,
    pub succeeded: bool,
    pub exit_code: i32,
    pub duration_ms: u64,
    pub failure_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaveResult {
    pub wave_id: String,
    pub tasks: Vec<TaskOutcome>,
}

impl WaveResult {
    pub fn has_critical_failure(&self) -> bool {
        self.tasks.iter().any(|t| !t.succeeded)
    }

    pub fn summary(&self) -> String {
        let ok = self.tasks.iter().filter(|t| t.succeeded).count();
        format!("  [{}] {}/{}", self.wave_id, ok, self.tasks.len())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionReport {
    pub started: Timestamp,
    pub ended: Option<Timestamp>,
    pub waves: Vec<WaveResult>,
}

impl ExecutionReport {
    pub fn new() -> Self {
        Self {
            started: now(),
            ended: None,
            waves: Vec::new(),
        }
    }

    pub fn mark_ended(&mut self) {
        self.ended = Some(now());
    }

    pub fn total_tasks(&self) -> usize {
        self.waves.iter().map(|w| w.tasks.len()).sum()
    }

    pub fn succeeded(&self) -> usize {
        self.waves
            .iter()
            .flat_map(|w| &w.tasks)
            .filter(|t| t.succeeded)
            .count()
    }

    pub fn failed(&self) -> usize {
        self.waves
            .iter()
            .flat_map(|w| &w.tasks)
            .filter(|t| !t.succeeded)
            .count()
    }

    pub fn timed_out(&self) -> usize {
        self.waves
            .iter()
            .flat_map(|w| &w.tasks)
            .filter(|t| {
                t.failure_reason
                    .as_deref()
                    .is_some_and(|r| r.contains("hard timeout") || r.contains("Stale"))
            })
            .count()
    }

    pub fn elapsed(&self) -> Duration {
        match self.ended {
            Some(end) => (end - self.started)
                .to_std()
                .unwrap_or(Duration::ZERO),
            None => Duration::ZERO,
        }
    }

    pub fn to_summary(&self) -> String {
        let total = self.total_tasks();
        let ok = self.succeeded();
        let waves_fmt: Vec<String> = self.waves.iter().map(|w| w.summary()).collect();
        format!(
            "Execution: {} waves, {}/{} succeeded\n\
             Elapsed:   {}s\n\
             Failed:    {}\n\
             Timed out: {}\n\
             \n\
             {}",
            self.waves.len(),
            ok,
            total,
            self.elapsed().as_secs(),
            self.failed(),
            self.timed_out(),
            waves_fmt.join("\n"),
        )
    }

    pub fn to_markdown(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!(
            "**{} waves, {}/{} succeeded in {}s**\n\n",
            self.waves.len(),
            self.succeeded(),
            self.total_tasks(),
            self.elapsed().as_secs()
        ));
        for wave in &self.waves {
            out.push_str(&format!("### Wave `{}`\n\n", wave.wave_id));
            for t in &wave.tasks {
                let mark = if t.succeeded { "ok" } else { "fail" };
                let reason = t.failure_reason.as_deref().unwrap_or("");
                out.push_str(&format!(
                    "- `{}` — **{}** ({}ms) {}\n",
                    t.task_id, mark, t.duration_ms, reason
                ));
            }
            out.push('\n');
        }
        out
    }
}

impl Default for ExecutionReport {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mk_outcome(id: &str, ok: bool, reason: Option<&str>) -> TaskOutcome {
        TaskOutcome {
            task_id: id.into(),
            succeeded: ok,
            exit_code: if ok { 0 } else { 1 },
            duration_ms: 100,
            failure_reason: reason.map(|s| s.to_string()),
        }
    }

    #[test]
    fn counts_aggregation() {
        let mut r = ExecutionReport::new();
        r.waves.push(WaveResult {
            wave_id: "w1".into(),
            tasks: vec![
                mk_outcome("a", true, None),
                mk_outcome("b", false, Some("hard timeout (11s) exceeded")),
            ],
        });
        r.waves.push(WaveResult {
            wave_id: "w2".into(),
            tasks: vec![mk_outcome("c", true, None)],
        });

        assert_eq!(r.total_tasks(), 3);
        assert_eq!(r.succeeded(), 2);
        assert_eq!(r.failed(), 1);
        assert_eq!(r.timed_out(), 1);
    }

    #[test]
    fn has_critical_failure_detects() {
        let w = WaveResult {
            wave_id: "w".into(),
            tasks: vec![mk_outcome("a", true, None), mk_outcome("b", false, Some("x"))],
        };
        assert!(w.has_critical_failure());
    }

    #[test]
    fn summary_contains_counts() {
        let mut r = ExecutionReport::new();
        r.waves.push(WaveResult {
            wave_id: "w1".into(),
            tasks: vec![mk_outcome("a", true, None)],
        });
        let s = r.to_summary();
        assert!(s.contains("1 waves"));
        assert!(s.contains("1/1 succeeded"));
    }

    #[test]
    fn markdown_lists_tasks() {
        let mut r = ExecutionReport::new();
        r.waves.push(WaveResult {
            wave_id: "w1".into(),
            tasks: vec![mk_outcome("a", false, Some("boom"))],
        });
        let md = r.to_markdown();
        assert!(md.contains("### Wave `w1`"));
        assert!(md.contains("`a`"));
        assert!(md.contains("boom"));
    }
}
