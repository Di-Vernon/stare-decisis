//! Aggressive text normalisation.
//!
//! Strips incidental differences — timestamps, UUIDs, long numbers,
//! hex blobs, filesystem paths — so two failure messages that differ
//! only in "when" or "which file" collapse to the same normalised
//! form and thus the same tier-1 hash.
//!
//! Determinism is load-bearing: the same input MUST always produce the
//! same output. All regexes are compiled once (LazyLock), operate on
//! `&str` deterministically, and the final normalisation is a
//! lowercase + whitespace-collapse pass. No locale-dependent behaviour.

use std::sync::LazyLock;

use regex::Regex;

static TIMESTAMP_RE: LazyLock<Regex> = LazyLock::new(|| {
    // ISO 8601-ish: 2026-04-19T14:23:45 / 2026-04-19 14:23:45.123Z etc.
    Regex::new(
        r"\d{4}-\d{2}-\d{2}[T ]\d{2}:\d{2}:\d{2}(?:\.\d+)?(?:Z|[+\-]\d{2}:?\d{2})?",
    )
    .expect("timestamp regex compiles")
});

static UUID_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}",
    )
    .expect("uuid regex compiles")
});

// Matches absolute POSIX paths beginning with /. Must run AFTER
// timestamp/UUID so their digits don't get absorbed into a path.
static PATH_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?:/[A-Za-z0-9._\-]+)+").expect("path regex compiles"));

static HEX_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\b[0-9a-fA-F]{6,}\b").expect("hex regex compiles"));

static NUM_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\b\d{3,}\b").expect("num regex compiles"));

pub fn normalize_aggressive(text: &str) -> String {
    // Ordering matters:
    //   1. timestamp — includes colons that PATH_RE wouldn't match, but
    //      digits that would fall into NUM_RE if left.
    //   2. UUID — specific pattern before HEX_RE which would also match.
    //   3. PATH — before HEX/NUM so the digits inside a path don't
    //      become <NUM> / <HEX> separately.
    //   4. HEX — before NUM (hex blobs look like long numbers too).
    //   5. NUM.
    //   6. lowercase + whitespace collapse.
    let s = TIMESTAMP_RE.replace_all(text, "<TS>");
    let s = UUID_RE.replace_all(&s, "<UUID>");
    let s = PATH_RE.replace_all(&s, "<PATH>");
    let s = HEX_RE.replace_all(&s, "<HEX>");
    let s = NUM_RE.replace_all(&s, "<NUM>");
    let lowered = s.to_lowercase();
    lowered.split_whitespace().collect::<Vec<_>>().join(" ")
}
