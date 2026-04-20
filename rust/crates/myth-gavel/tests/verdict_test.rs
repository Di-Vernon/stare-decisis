use myth_common::{Enforcement, Level};
use myth_gavel::{RuleMatch, Verdict};

fn fake_match(level: Level, rule_id: &str, item: &str) -> RuleMatch {
    RuleMatch {
        rule_id: rule_id.into(),
        item: item.into(),
        level,
        matched_span: (0, 5),
        matched_text: "hello".into(),
    }
}

#[test]
fn allow_emits_continue_true() {
    let v = Verdict::allow();
    let j = v.to_hook_json();
    assert_eq!(j["continue"], true);
    assert!(j.get("hookSpecificOutput").is_none());
}

#[test]
fn seal_emits_continue_false_with_stop_reason() {
    let v = Verdict::seal(fake_match(Level::Critical, "R1-A", "rm_rf_unsandboxed"));
    assert!(v.is_blocking());
    let j = v.to_hook_json();
    assert_eq!(j["continue"], false);
    assert!(j["stopReason"].as_str().unwrap().contains("R1-A"));
}

#[test]
fn advisory_emits_allow_permission_with_additional_context() {
    let m = fake_match(Level::Low, "S-1", "demo");
    let v = Verdict::with_enforcement(Enforcement::Advisory, m, None);
    let j = v.to_hook_json();
    assert_eq!(j["continue"], true);
    assert_eq!(j["hookSpecificOutput"]["hookEventName"], "PreToolUse");
    assert_eq!(j["hookSpecificOutput"]["permissionDecision"], "allow");
    assert!(j["hookSpecificOutput"]["additionalContext"]
        .as_str()
        .unwrap()
        .contains("S-1"));
}

#[test]
fn caution_uses_allow_permission_decision() {
    let m = fake_match(Level::Medium, "S-2", "demo");
    let v = Verdict::with_enforcement(Enforcement::Caution, m, None);
    let j = v.to_hook_json();
    assert_eq!(j["hookSpecificOutput"]["permissionDecision"], "allow");
}

#[test]
fn warn_uses_ask_permission_decision() {
    let m = fake_match(Level::High, "F-3", "demo");
    let v = Verdict::with_enforcement(Enforcement::Warn, m, None);
    let j = v.to_hook_json();
    assert_eq!(j["continue"], true);
    assert_eq!(j["hookSpecificOutput"]["permissionDecision"], "ask");
}

#[test]
fn strike_emits_continue_false() {
    let m = fake_match(Level::Critical, "F-9", "demo");
    let v = Verdict::with_enforcement(Enforcement::Strike, m, None);
    assert!(v.is_blocking());
    let j = v.to_hook_json();
    assert_eq!(j["continue"], false);
}
