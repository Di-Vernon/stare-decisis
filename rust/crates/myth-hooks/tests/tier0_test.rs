use myth_common::{Category, Level};
use myth_hooks::{classify_tier0, DeterministicClassification};

#[test]
fn timeout_is_classified_as_transient_network() {
    let c = classify_tier0("Exit code 1\nrequest timed out after 30s").unwrap();
    assert_eq!(c.level, Level::Low);
    assert_eq!(c.category, Category::Process);
    assert_eq!(c.rationale, "transient_network");
}

#[test]
fn connection_reset_is_timeout_class() {
    let c = classify_tier0("connection reset by peer").unwrap();
    assert_eq!(c.rationale, "transient_network");
}

#[test]
fn dns_failure_is_timeout_class() {
    let c = classify_tier0("temporary failure in name resolution").unwrap();
    assert_eq!(c.rationale, "transient_network");
}

#[test]
fn http_429_is_rate_limit() {
    let c = classify_tier0("HTTP 429 Too Many Requests").unwrap();
    assert_eq!(c.level, Level::Low);
    assert_eq!(c.category, Category::Process);
    assert_eq!(c.rationale, "rate_limit");
}

#[test]
fn rate_limit_phrase_is_rate_limit() {
    let c = classify_tier0("API response: rate limit exceeded").unwrap();
    assert_eq!(c.rationale, "rate_limit");
}

#[test]
fn file_not_found_via_enoent() {
    let c =
        classify_tier0("Exit code 1\ncat: /nonexistent: No such file or directory").unwrap();
    assert_eq!(c.level, Level::Medium);
    assert_eq!(c.category, Category::Correctness);
    assert_eq!(c.rationale, "file_not_found");
}

#[test]
fn file_not_found_via_python_error_class() {
    let c = classify_tier0("FileNotFoundError: [Errno 2]").unwrap();
    assert_eq!(c.rationale, "file_not_found");
}

#[test]
fn ambiguous_error_returns_none() {
    // Does not match any Tier 0 rule — must fall through to Tier 1.
    assert!(
        classify_tier0("AssertionError: expected 42, got 41").is_none(),
        "assertion errors are Tier 1 territory on Day-1"
    );
    assert!(
        classify_tier0("Permission denied").is_none(),
        "permission_denied is listed as future Tier 0 — not shipped on Day-1"
    );
    assert!(
        classify_tier0("panicked at 'something went wrong'").is_none()
    );
}

#[test]
fn empty_error_string_returns_none() {
    assert!(classify_tier0("").is_none());
}

#[test]
fn classification_is_copy_cheap() {
    // The struct is small and PartialEq — cheap to clone in hot paths
    // or pass around. Lock the expected shape.
    let c: DeterministicClassification = classify_tier0("timeout").unwrap();
    let c2 = c.clone();
    assert_eq!(c, c2);
}
