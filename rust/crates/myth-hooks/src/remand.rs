//! Remand orchestration scaffold (RESERVED — dead code in v0.2).
//!
//! Activated on Milestone A (Tier 3 Assessor + subtleness classifier).
//!
//! Architecture (when activated):
//! - 3-cap retry counter keyed by `(session_id, task_fingerprint)`
//! - Mode-collapse detection: similarity ≥ 0.95 between consecutive
//!   responses → escalate to Strike (Reflexion failure mode)
//! - Subtleness gate: only fire when `Verdict.subtleness_score >= θ`
//!   (default θ = 0.7, tuned from production caselog at Milestone A)
//!
//! See `experiment/remand-prototype/results/FINAL_REPORT.md` Section 7
//! and `design/CONSTITUTION-v2.4-remand-draft.md` Part VII Section 2.

#![allow(dead_code)]

use myth_gavel::Verdict;

/// Default subtleness threshold for selective Remand trigger.
/// Tuned at Milestone A from production caselog (≥50 cases).
pub const DEFAULT_SUBTLENESS_THRESHOLD: f32 = 0.7;

/// Reflexion 3-cap. Hard limit on retries within one task.
pub const MAX_REMAND_ATTEMPTS: u32 = 3;

/// Mode collapse similarity threshold (validated in Phase 2.4 with
/// `difflib.SequenceMatcher.ratio()`; Rust port at activation will
/// likely use `strsim::normalized_levenshtein` with the same cutoff).
pub const MODE_COLLAPSE_SIMILARITY: f32 = 0.95;

/// Per-(session, task) retry counter. Reset on successful pass or Strike.
pub struct RemandCounter {
    pub attempts: u32,
    pub previous_response_hash: Option<u64>,
}

impl RemandCounter {
    pub fn new() -> Self {
        Self {
            attempts: 0,
            previous_response_hash: None,
        }
    }

    pub fn budget_remaining(&self) -> bool {
        self.attempts < MAX_REMAND_ATTEMPTS
    }
}

impl Default for RemandCounter {
    fn default() -> Self {
        Self::new()
    }
}

/// Activation gate. Currently always returns `false` — Remand is dead
/// code in v0.2. Milestone A flips this to subtleness-score-conditional.
pub fn should_remand(verdict: &Verdict, threshold: f32) -> bool {
    let _ = (verdict, threshold);
    false
}
