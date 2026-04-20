use myth_common::ReminderId;
use myth_hooks::templates::variant_b::{render, truncate_error, ERROR_MAX_BYTES};
use myth_hooks::PostToolUseFailureData;

fn sample_failure() -> PostToolUseFailureData {
    PostToolUseFailureData {
        tool_name: "Bash".into(),
        tool_input: serde_json::json!({
            "command": "cat /nonexistent",
            "description": "probe"
        }),
        tool_use_id: "toolu_01Nv3TzCkg1WZb9Vcaqvxxqy".into(),
        error: "Exit code 1\ncat: /nonexistent: No such file or directory".into(),
        is_interrupt: false,
    }
}

#[test]
fn render_includes_tool_name_and_reminder_id() {
    let rid = ReminderId::new();
    let out = render(&sample_failure(), rid);
    assert!(out.contains("Bash"));
    assert!(out.contains(&rid.to_string()));
    assert!(out.contains("assessor-trigger-id"));
}

#[test]
fn render_embeds_valid_compact_json_with_h5_fields() {
    let rid = ReminderId::new();
    let out = render(&sample_failure(), rid);

    // Extract the JSON literal from the prompt and re-parse it to
    // prove the H5 shape survives (tool_name, tool_input, tool_use_id,
    // error, is_interrupt) rather than the old tool_response form.
    let start = out.find("Failure payload: ").expect("found payload marker") + "Failure payload: ".len();
    let rest = &out[start..];
    let end = rest.find(". Return").expect("end marker");
    let json_str = &rest[..end];

    let v: serde_json::Value = serde_json::from_str(json_str)
        .expect("payload JSON parses");
    assert_eq!(v.get("tool_name").and_then(|x| x.as_str()), Some("Bash"));
    assert_eq!(
        v.get("tool_use_id").and_then(|x| x.as_str()),
        Some("toolu_01Nv3TzCkg1WZb9Vcaqvxxqy")
    );
    assert!(v.get("error").and_then(|x| x.as_str()).unwrap().contains("Exit code 1"));
    assert_eq!(v.get("is_interrupt").and_then(|x| x.as_bool()), Some(false));
    assert!(
        v.get("tool_response").is_none(),
        "post-H5 template must not emit the old tool_response field"
    );
}

#[test]
fn render_truncates_long_errors() {
    let mut data = sample_failure();
    // 6 000 bytes — exceeds ERROR_MAX_BYTES (4 000).
    data.error = "x".repeat(6_000);
    let rid = ReminderId::new();
    let out = render(&data, rid);

    // Locate the embedded JSON payload and read back the error string.
    let start = out.find("Failure payload: ").unwrap() + "Failure payload: ".len();
    let rest = &out[start..];
    let end = rest.find(". Return").unwrap();
    let v: serde_json::Value = serde_json::from_str(&rest[..end]).unwrap();
    let err = v["error"].as_str().unwrap();

    assert!(err.len() <= ERROR_MAX_BYTES, "got {} bytes", err.len());
    assert!(err.ends_with("[truncated by myth]"));
}

#[test]
fn truncate_error_respects_utf8_boundary() {
    // Multibyte — confirm boundary backoff on a forced cut.
    let s = "한글".repeat(3_000); // each unit is 6 bytes
    let out = truncate_error(&s);
    assert!(out.len() <= ERROR_MAX_BYTES);
    // Rust Strings are always valid UTF-8; reaching this line already
    // proves the boundary logic didn't panic.
    assert!(out.ends_with("[truncated by myth]") || out == s);
}

#[test]
fn short_error_is_passed_through() {
    let out = truncate_error("quick error");
    assert_eq!(out, "quick error");
}
