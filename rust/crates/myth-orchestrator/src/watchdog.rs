//! 타임아웃 + 스테일(무활동) 감지기.
//!
//! `monitor`는 tmux 세션의 pane 출력을 주기적으로 읽어 길이 증가 여부로
//! 활동을 판단한다. 완료 신호 감지(`DONE|` 또는 Claude 종료 문구) 포함.

use std::time::{Duration, Instant};

use crate::tmux;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WatchdogResult {
    Completed,
    HardTimeout,
    Stale,
}

pub struct Watchdog {
    pub hard_timeout: Duration,
    pub stale_threshold: Duration,
    /// 한 tick 간격. 테스트에서 짧게 조정.
    pub tick: Duration,
}

impl Watchdog {
    pub fn new(hard_timeout: Duration, stale_threshold: Duration) -> Self {
        Self {
            hard_timeout,
            stale_threshold,
            tick: Duration::from_secs(5),
        }
    }

    pub async fn monitor(&self, session_name: &str, start: Instant) -> WatchdogResult {
        let mut last_output_len = 0usize;
        let mut last_change = Instant::now();
        let mut interval = tokio::time::interval(self.tick);

        loop {
            interval.tick().await;

            if start.elapsed() >= self.hard_timeout {
                return WatchdogResult::HardTimeout;
            }

            let output = tmux::capture_pane(session_name).await.unwrap_or_default();

            if is_completion_signal(&output) {
                return WatchdogResult::Completed;
            }

            if output.len() > last_output_len {
                last_output_len = output.len();
                last_change = Instant::now();
            } else if last_change.elapsed() >= self.stale_threshold {
                return WatchdogResult::Stale;
            }
        }
    }
}

pub fn is_completion_signal(pane: &str) -> bool {
    if pane.contains("DONE|") {
        return true;
    }
    // Claude Code가 세션 종료 시 보이는 프롬프트 패턴 (approximation)
    let tail: String = pane.chars().rev().take(200).collect();
    let tail: String = tail.chars().rev().collect();
    tail.contains("$ ") || tail.contains("# ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_done_marker() {
        assert!(is_completion_signal("hello\nDONE|/path/to/artifact"));
    }

    #[test]
    fn detects_shell_prompt() {
        assert!(is_completion_signal(
            "doing work\n...\nexit\nuser@host:~/project$ "
        ));
    }

    #[test]
    fn no_signal_in_running_output() {
        assert!(!is_completion_signal("working..."));
        assert!(!is_completion_signal(""));
    }

    #[test]
    fn watchdog_new_fields() {
        let w = Watchdog::new(Duration::from_secs(10), Duration::from_secs(2));
        assert_eq!(w.hard_timeout, Duration::from_secs(10));
        assert_eq!(w.stale_threshold, Duration::from_secs(2));
        assert_eq!(w.tick, Duration::from_secs(5));
    }
}
