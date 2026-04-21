//! Tier 3 subprocess dispatch — structural wiring for Milestone A.
//!
//! Wave 8 Task 8.3 (carry-forward #3): the call path from Rust hook
//! to Python Assessor is built here but the **gate is hard-wired
//! false on Day-1**. Milestone A flips the gate once shadow-mode
//! data shows Tier 1 compliance below Decision 3's 70% threshold.
//!
//! Design notes:
//!   - The gate returns `false` unconditionally today. When the
//!     Milestone A upgrade lands it reads shadow metrics and
//!     evaluates a rolling compliance window; the signature stays
//!     the same.
//!   - `maybe_tier3_dispatch` writes the failure envelope to
//!     `$TMPDIR/myth-tier3-<session>.json`, spawns
//!     `python3 -m myth_py.assessor.cli classify --input <path>`,
//!     and cleans up the temp file. Failure is observability-class
//!     — never propagated to the hook result.
//!   - The Python endpoint returns `{"status":"not_enabled", ...}`
//!     on Day-1 (see `myth_py.assessor.cli::classify`). When
//!     Milestone A lands, the Python side switches to real Anthropic
//!     SDK dispatch (Decision 4) without needing a Rust-side change.

use anyhow::{Context, Result};
use serde_json::json;

use crate::PostToolUseFailureData;

/// Gate evaluator. Day-1: always false. Milestone A: compliance rate
/// threshold check (Decision 3 §Milestone A — Tier 1 <70% triggers).
pub fn tier3_gate_active() -> bool {
    false
}

/// Subprocess dispatch to the Python Assessor. Observability-class —
/// caller should ignore any error so the hook result is never
/// affected. On Day-1 this is unreachable because the gate is false;
/// kept here so the wiring is ready for the Milestone A flip.
pub fn maybe_tier3_dispatch(
    session_id: &str,
    data: &PostToolUseFailureData,
) -> Result<()> {
    let tmpdir = std::env::temp_dir();
    let envelope_path = tmpdir.join(format!("myth-tier3-{}.json", session_id));

    let envelope = json!({
        "session_id": session_id,
        "tool_name": data.tool_name,
        "tool_input": data.tool_input,
        "tool_use_id": data.tool_use_id,
        "error": data.error,
    });

    std::fs::write(
        &envelope_path,
        serde_json::to_string(&envelope).context("serialising tier3 envelope")?,
    )
    .with_context(|| format!("writing tier3 envelope to {}", envelope_path.display()))?;

    let status = std::process::Command::new("python3")
        .args(["-m", "myth_py.assessor.cli", "classify", "--input"])
        .arg(&envelope_path)
        .status();

    // Best-effort cleanup — don't let a remove failure eclipse the
    // real dispatch outcome we want to bubble up.
    let _ = std::fs::remove_file(&envelope_path);

    match status {
        Ok(s) if s.success() => Ok(()),
        Ok(s) => Err(anyhow::anyhow!(
            "tier3 classifier exited with status {}",
            s.code().unwrap_or(-1)
        )),
        Err(e) => Err(anyhow::Error::from(e).context("spawning python3 for tier3 dispatch")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gate_is_inactive_on_day_1() {
        assert!(
            !tier3_gate_active(),
            "Tier 3 gate must be false on Day-1 (Milestone A gate)"
        );
    }
}
