//! tmux 세션 래퍼. 실제 작업은 `tmux` 바이너리 호출로 위임.
//!
//! 단위 테스트는 tmux 의존 없이 순수 argv 조립 검증에 집중한다. 실제
//! tmux 호출이 필요한 경로는 integration test (Wave 8)에서 다룬다.

use anyhow::{anyhow, Result};
use std::path::Path;
use tokio::process::Command;

#[derive(Debug, Clone)]
pub struct TmuxSession {
    pub name: String,
}

pub async fn create_session(name: &str, cwd: &Path) -> Result<TmuxSession> {
    let output = Command::new("tmux")
        .args(["new-session", "-d", "-s", name, "-c"])
        .arg(cwd)
        .output()
        .await?;

    if !output.status.success() {
        return Err(anyhow!(
            "tmux new-session failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Ok(TmuxSession {
        name: name.to_string(),
    })
}

pub async fn send_keys(session: &str, cmd: &str) -> Result<()> {
    let status = Command::new("tmux")
        .args(["send-keys", "-t", session])
        .arg(cmd)
        .arg("Enter")
        .status()
        .await?;
    if !status.success() {
        return Err(anyhow!("tmux send-keys failed"));
    }
    Ok(())
}

pub async fn capture_pane(session: &str) -> Result<String> {
    let output = Command::new("tmux")
        .args(["capture-pane", "-t", session, "-p"])
        .output()
        .await?;
    if !output.status.success() {
        return Err(anyhow!(
            "tmux capture-pane failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

pub async fn kill_session(session: &str) -> Result<()> {
    Command::new("tmux")
        .args(["kill-session", "-t", session])
        .status()
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn kill_nonexistent_does_not_panic() {
        // tmux returns non-zero for missing session; our function only
        // awaits status and returns Ok regardless by design (best-effort cleanup).
        let _ = kill_session("myth-test-nonexistent-xyz-9999").await;
    }
}
