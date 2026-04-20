use myth_identity::{normalize_aggressive, tier1_hash};

#[test]
fn same_text_produces_same_hash() {
    let a = tier1_hash(&normalize_aggressive("error: missing file"));
    let b = tier1_hash(&normalize_aggressive("error: missing file"));
    assert_eq!(a, b);
}

#[test]
fn different_text_produces_different_hash() {
    let a = tier1_hash(&normalize_aggressive("error: missing file"));
    let b = tier1_hash(&normalize_aggressive("error: bad syntax"));
    assert_ne!(a, b);
}

#[test]
fn timestamp_difference_collapses_to_same_hash() {
    // Two messages that differ only in timestamp should hash
    // identically after aggressive normalisation — that's the whole
    // point of tier 1.
    let a = tier1_hash(&normalize_aggressive(
        "FileNotFound at 2026-04-19T14:23:45Z in /home/x/foo.log",
    ));
    let b = tier1_hash(&normalize_aggressive(
        "FileNotFound at 2026-04-20T09:11:02Z in /home/x/foo.log",
    ));
    assert_eq!(a, b, "timestamps must normalise to the same hash");
}

#[test]
fn hash_is_twenty_bytes() {
    let h = tier1_hash("anything");
    assert_eq!(h.len(), 20);
}
