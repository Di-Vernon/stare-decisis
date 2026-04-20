//! Tier 1 — SHA1 of the aggressively-normalised text.

pub mod normalize;

pub use normalize::normalize_aggressive;

use sha1::{Digest, Sha1};

/// Compute the tier-1 20-byte SHA1 hash of `normalized`. Deterministic:
/// same input → same output.
pub fn tier1_hash(normalized: &str) -> [u8; 20] {
    let mut hasher = Sha1::new();
    hasher.update(normalized.as_bytes());
    hasher.finalize().into()
}
