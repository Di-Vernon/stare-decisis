use myth_identity::normalize_aggressive;

#[test]
fn timestamp_is_replaced() {
    let s = normalize_aggressive("error at 2026-04-19T14:23:45Z");
    assert!(s.contains("<ts>"));
    assert!(!s.contains("2026"));
}

#[test]
fn uuid_is_replaced() {
    let uuid = "550e8400-e29b-41d4-a716-446655440000";
    let s = normalize_aggressive(&format!("request {} failed", uuid));
    assert!(s.contains("<uuid>"));
    assert!(!s.contains("550e8400"));
}

#[test]
fn path_is_replaced() {
    let s = normalize_aggressive("cannot open /home/miirr/project/file.log");
    assert!(s.contains("<path>"));
    assert!(!s.contains("miirr"));
}

#[test]
fn long_hex_is_replaced() {
    let s = normalize_aggressive("commit abcdef1234 not found");
    assert!(s.contains("<hex>"));
    assert!(!s.contains("abcdef1234"));
}

#[test]
fn long_number_is_replaced() {
    let s = normalize_aggressive("ran for 12345 seconds");
    assert!(s.contains("<num>"));
    assert!(!s.contains("12345"));
}

#[test]
fn short_numbers_survive() {
    // 2-digit numbers are common in real error messages ("exit 1",
    // "line 42") and must not collapse — only 3+ digits count.
    let s = normalize_aggressive("exit 1 on line 42");
    assert!(s.contains("42"), "got: {:?}", s);
    assert!(s.contains("1"), "got: {:?}", s);
}

#[test]
fn lowercase_and_whitespace_collapse() {
    let s = normalize_aggressive("  Hello\t\tWORLD  ");
    assert_eq!(s, "hello world");
}

#[test]
fn identical_inputs_produce_identical_output() {
    // Determinism — load-bearing property.
    let a = normalize_aggressive("FileNotFoundError: /home/x/file-123.log");
    let b = normalize_aggressive("FileNotFoundError: /home/x/file-123.log");
    assert_eq!(a, b);
}

#[test]
fn constitution_example_normalises_as_documented() {
    // Example straight from docs/04-CRATES/04-myth-identity.md:
    //   raw:  "FileNotFoundError: /home/miirr/project/foo/tmp/abc-123.log not found"
    //   norm: "filenotfounderror <path> not found"
    let raw = "FileNotFoundError: /home/miirr/project/foo/tmp/abc-123.log not found";
    let norm = normalize_aggressive(raw);
    assert_eq!(norm, "filenotfounderror: <path> not found");
}
