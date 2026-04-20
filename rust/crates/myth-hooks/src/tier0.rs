//! Tier 0 deterministic failure classifier.
//!
//! Decision 3's fast path: a small set of regex/substring rules that
//! can resolve common failures without invoking an LLM. A hit here
//! short-circuits the Variant B template path entirely — the caller
//! records the classification straight into caselog + lesson-state
//! and returns Allow.
//!
//! Day-1 coverage (per 05-myth-hooks.md §classify_deterministic):
//!
//! - transient_network — timeout / connection reset / DNS failure
//! - rate_limit — HTTP 429 or "rate limit" substring
//! - file_not_found — ENOENT / "No such file or directory" /
//!   FileNotFoundError
//!
//! Permission-denied, syntax errors, and git-conflict markers are
//! listed in the design as "…" future entries — Milestone A adds
//! them after shadow-mode data shows which patterns actually recur.

use std::sync::LazyLock;

use myth_common::{Category, Level};
use regex::Regex;

static TIMEOUT_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)(?:timeout|timed out|connection reset|connection refused|network unreachable|temporary failure in name resolution|dns resolution failed)",
    )
    .expect("timeout regex")
});

static FILE_NOT_FOUND_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)(?:no such file or directory|enoent|filenotfounderror|file not found)",
    )
    .expect("file-not-found regex")
});

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeterministicClassification {
    pub level: Level,
    pub category: Category,
    /// Machine-readable label; caselog.jsonl's `rationale` field.
    pub rationale: &'static str,
}

/// Classify `error` against the Tier 0 rule set. Returns `None` when
/// the error text doesn't match any known signature — caller then
/// falls through to Tier 1 (Variant B template).
pub fn classify(error: &str) -> Option<DeterministicClassification> {
    if TIMEOUT_RE.is_match(error) {
        return Some(DeterministicClassification {
            level: Level::Low,
            category: Category::Process,
            rationale: "transient_network",
        });
    }

    // Rate-limit detection: explicit HTTP 429 or phrasing. Case-
    // insensitive substring because the phrasing varies widely.
    if error.contains("429") {
        return Some(DeterministicClassification {
            level: Level::Low,
            category: Category::Process,
            rationale: "rate_limit",
        });
    }
    let lower = error.to_lowercase();
    if lower.contains("rate limit") || lower.contains("ratelimit") {
        return Some(DeterministicClassification {
            level: Level::Low,
            category: Category::Process,
            rationale: "rate_limit",
        });
    }

    if FILE_NOT_FOUND_RE.is_match(error) {
        return Some(DeterministicClassification {
            level: Level::Medium,
            category: Category::Correctness,
            rationale: "file_not_found",
        });
    }

    None
}
