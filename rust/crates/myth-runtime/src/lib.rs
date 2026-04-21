//! `myth-runtime` — Claude Code subprocess 관리.
//!
//! `ClaudeRuntime`가 `claude` CLI 바이너리를 찾고, 환경을 주입하고, 인터랙티브
//! 세션 혹은 watchdog 딸린 프로그래매틱 실행을 수행한다.
//!
//! state.db 접근은 하지 않는다 — orchestrator / cli 책임.

pub mod discovery;
pub mod env;
pub mod fallback;
pub mod io;
pub mod session;
pub mod version;

pub use fallback::{action_for, detect_quota_exhausted, QuotaAction, QuotaPolicy};
pub use session::Session;
pub use version::ClaudeVersion;

use anyhow::{Context, Result};
use myth_common::SessionId;
use std::path::{Path, PathBuf};
use std::process::{ExitCode, Stdio};
use std::time::{Duration, Instant};
use tokio::io::AsyncReadExt;
use tokio::process::Command;

pub struct ClaudeRuntime {
    claude_path: PathBuf,
    version: ClaudeVersion,
    worktree: PathBuf,
    session_id: SessionId,
}

impl ClaudeRuntime {
    pub fn new(worktree: &Path) -> Result<Self> {
        let claude_path = discovery::find_claude()?;
        let version = discovery::detect_version(&claude_path)?;
        version.validate_compatible()?;

        Ok(Self {
            claude_path,
            version,
            worktree: worktree.to_path_buf(),
            session_id: SessionId::new(),
        })
    }

    pub fn claude_path(&self) -> &Path {
        &self.claude_path
    }

    pub fn version(&self) -> ClaudeVersion {
        self.version
    }

    pub fn session_id(&self) -> SessionId {
        self.session_id
    }

    pub fn worktree(&self) -> &Path {
        &self.worktree
    }

    /// 인터랙티브 세션. foreground로 claude를 실행한다.
    pub async fn spawn_interactive(&self) -> Result<ExitCode> {
        let mut cmd = self.base_command();
        let status = cmd
            .status()
            .await
            .context("failed to spawn interactive claude session")?;
        let code = status
            .code()
            .and_then(|c| u8::try_from(c).ok())
            .unwrap_or(1);
        Ok(ExitCode::from(code))
    }

    /// 프로그래매틱 실행 — orchestrator가 호출.
    ///
    /// `hard_timeout` 경과 시 SIGKILL. `stale_threshold`는 현재 Claude Code 자체의
    /// `CLAUDE_STREAM_IDLE_TIMEOUT_MS` (env.rs에서 주입)에 위임된다. 파라미터는
    /// 향후 streaming activity 측정용으로 보존.
    pub async fn execute_with_watchdog(
        &self,
        prompt: &str,
        hard_timeout: Duration,
        _stale_threshold: Duration,
    ) -> TaskResult {
        let mut cmd = self.base_command();
        cmd.args(["-p", prompt])
            .args(["--max-turns", "10"])
            .arg("--no-session-persistence")
            .arg("--dangerously-skip-permissions")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let started = Instant::now();
        let mut child = match cmd.spawn() {
            Ok(c) => c,
            Err(e) => return TaskResult::crashed(format!("spawn failed: {e}"), Duration::ZERO),
        };

        let stdout_pipe = child.stdout.take();
        let stderr_pipe = child.stderr.take();

        let stdout_task = tokio::spawn(async move {
            let mut buf = Vec::new();
            if let Some(mut s) = stdout_pipe {
                let _ = s.read_to_end(&mut buf).await;
            }
            buf
        });
        let stderr_task = tokio::spawn(async move {
            let mut buf = Vec::new();
            if let Some(mut s) = stderr_pipe {
                let _ = s.read_to_end(&mut buf).await;
            }
            buf
        });

        let wait_result = tokio::select! {
            res = child.wait() => WatchdogResult::Exited(res),
            _ = tokio::time::sleep(hard_timeout) => WatchdogResult::HardTimeout,
        };

        let (status_opt, timed_out) = match wait_result {
            WatchdogResult::Exited(Ok(status)) => (Some(status), false),
            WatchdogResult::Exited(Err(e)) => {
                return TaskResult::crashed(format!("wait error: {e}"), started.elapsed());
            }
            WatchdogResult::HardTimeout => {
                let _ = child.kill().await;
                let _ = child.wait().await;
                (None, true)
            }
        };

        let stdout = stdout_task.await.unwrap_or_default();
        let stderr = stderr_task.await.unwrap_or_default();
        let stdout = String::from_utf8_lossy(&stdout).into_owned();
        let stderr = String::from_utf8_lossy(&stderr).into_owned();
        let duration = started.elapsed();

        if timed_out {
            return TaskResult {
                succeeded: false,
                exit_code: -1,
                stdout,
                stderr,
                duration,
                failure_reason: Some(format!(
                    "hard timeout ({}s) exceeded",
                    hard_timeout.as_secs()
                )),
            };
        }

        let status = status_opt.expect("status present when not timed out");
        let succeeded = status.success();
        TaskResult {
            succeeded,
            exit_code: status.code().unwrap_or(-1),
            failure_reason: if succeeded {
                None
            } else {
                Some(detect_failure_reason(&stderr))
            },
            stdout,
            stderr,
            duration,
        }
    }

    fn base_command(&self) -> Command {
        let mut cmd = Command::new(&self.claude_path);
        cmd.current_dir(&self.worktree);
        env::inject(&mut cmd, self.session_id);
        cmd
    }
}

enum WatchdogResult {
    Exited(std::io::Result<std::process::ExitStatus>),
    HardTimeout,
}

#[derive(Debug, Clone)]
pub struct TaskResult {
    pub succeeded: bool,
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub duration: Duration,
    pub failure_reason: Option<String>,
}

impl TaskResult {
    pub fn stderr_tail(&self, max_bytes: usize) -> String {
        if self.stderr.len() <= max_bytes {
            self.stderr.clone()
        } else {
            // byte index safe: UTF-8 aware trim to char boundary
            let start = self.stderr.len() - max_bytes;
            let mut idx = start;
            while idx < self.stderr.len() && !self.stderr.is_char_boundary(idx) {
                idx += 1;
            }
            self.stderr[idx..].to_string()
        }
    }

    pub fn crashed(reason: impl Into<String>, duration: Duration) -> Self {
        Self {
            succeeded: false,
            exit_code: -1,
            stdout: String::new(),
            stderr: String::new(),
            duration,
            failure_reason: Some(reason.into()),
        }
    }
}

fn detect_failure_reason(stderr: &str) -> String {
    if detect_quota_exhausted(stderr) {
        "quota exhausted".into()
    } else if stderr.contains("permission denied") || stderr.contains("Permission denied") {
        "permission denied".into()
    } else {
        "non-zero exit".into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn task_result_crashed() {
        let r = TaskResult::crashed("boom", Duration::from_millis(5));
        assert!(!r.succeeded);
        assert_eq!(r.exit_code, -1);
        assert_eq!(r.failure_reason.as_deref(), Some("boom"));
    }

    #[test]
    fn stderr_tail_short() {
        let r = TaskResult::crashed("x", Duration::ZERO);
        let mut r = r;
        r.stderr = "hello".into();
        assert_eq!(r.stderr_tail(100), "hello");
    }

    #[test]
    fn stderr_tail_long() {
        let mut r = TaskResult::crashed("x", Duration::ZERO);
        r.stderr = "0123456789".into();
        assert_eq!(r.stderr_tail(4), "6789");
    }

    #[test]
    fn failure_reason_quota() {
        assert_eq!(
            detect_failure_reason("Error: rate limit reached"),
            "quota exhausted"
        );
    }

    #[test]
    fn failure_reason_generic() {
        assert_eq!(
            detect_failure_reason("segmentation fault"),
            "non-zero exit"
        );
    }
}
