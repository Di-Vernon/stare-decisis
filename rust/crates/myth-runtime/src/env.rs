//! `claude` subprocess 환경변수 주입.

use myth_common::SessionId;
use tokio::process::Command;

/// 실행 전 Command에 myth 고유 + Claude Code 튜닝 env 주입.
///
/// 주입 목록:
/// - `MYTH_SESSION_ID`: 현재 세션 uuid.
/// - `MYTH_ACTIVE`: "1" — 자식 프로세스(hooks)가 myth 활성 여부 감지.
/// - `CLAUDE_STREAM_IDLE_TIMEOUT_MS`: "120000" (2분).
/// - `ENABLE_PROMPT_CACHING_1H`: "1".
pub fn inject(cmd: &mut Command, session_id: SessionId) {
    cmd.env("MYTH_SESSION_ID", session_id.to_string());
    cmd.env("MYTH_ACTIVE", "1");
    cmd.env("CLAUDE_STREAM_IDLE_TIMEOUT_MS", "120000");
    cmd.env("ENABLE_PROMPT_CACHING_1H", "1");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Stdio;

    #[tokio::test]
    async fn inject_sets_env_vars() {
        let session = SessionId::new();
        let mut cmd = Command::new("/usr/bin/env");
        cmd.stdout(Stdio::piped()).stderr(Stdio::null());
        inject(&mut cmd, session);

        let output = cmd.output().await.expect("env exec");
        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);

        assert!(
            stdout.contains(&format!("MYTH_SESSION_ID={}", session)),
            "stdout: {}",
            stdout
        );
        assert!(stdout.contains("MYTH_ACTIVE=1"));
        assert!(stdout.contains("CLAUDE_STREAM_IDLE_TIMEOUT_MS=120000"));
        assert!(stdout.contains("ENABLE_PROMPT_CACHING_1H=1"));
    }
}
