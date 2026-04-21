//! `claude` 바이너리 탐색 + 버전 감지.

use anyhow::{anyhow, Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::version::ClaudeVersion;

/// 세 단계 탐색: `MYTH_CLAUDE_BIN` env → `which claude` → 표준 후보.
pub fn find_claude() -> Result<PathBuf> {
    if let Ok(raw) = std::env::var("MYTH_CLAUDE_BIN") {
        let p = PathBuf::from(&raw);
        if p.exists() {
            return Ok(p);
        }
        return Err(anyhow!(
            "MYTH_CLAUDE_BIN={:?} but file does not exist",
            raw
        ));
    }

    if let Ok(p) = which::which("claude") {
        return Ok(p);
    }

    for p in standard_candidates() {
        if p.exists() {
            return Ok(p);
        }
    }

    Err(anyhow!(
        "claude binary not found. Install Claude Code or set MYTH_CLAUDE_BIN"
    ))
}

fn standard_candidates() -> Vec<PathBuf> {
    let mut out = Vec::new();
    if let Some(home) = dirs::home_dir() {
        out.push(home.join(".claude/local/claude"));
    }
    out.push(PathBuf::from("/usr/local/bin/claude"));
    out.push(PathBuf::from("/opt/claude/bin/claude"));
    out
}

/// `claude --version` 호출해 버전 파싱.
pub fn detect_version(claude_path: &Path) -> Result<ClaudeVersion> {
    let output = Command::new(claude_path)
        .arg("--version")
        .output()
        .with_context(|| format!("failed to spawn {:?} --version", claude_path))?;

    if !output.status.success() {
        return Err(anyhow!(
            "{:?} --version exited with status {:?}",
            claude_path,
            output.status.code()
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    ClaudeVersion::parse(&stdout)
        .with_context(|| format!("parsing version output: {:?}", stdout))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_env_var_bin_errors() {
        // 의도적으로 존재하지 않는 경로
        std::env::set_var("MYTH_CLAUDE_BIN", "/nonexistent/claude-bin-xyz");
        let err = find_claude().unwrap_err();
        assert!(err.to_string().contains("does not exist"));
        std::env::remove_var("MYTH_CLAUDE_BIN");
    }

    #[test]
    fn standard_candidates_nonempty() {
        assert!(!standard_candidates().is_empty());
    }
}
