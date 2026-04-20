//! Integration-style tests for `Gavel::judge`, using `from_parts` to
//! bypass filesystem and DB dependencies.

use myth_common::{Enforcement, SessionId};
use myth_gavel::{CompiledRules, Gavel, Grid, RuleSet, ToolInput};

fn empty_rules() -> CompiledRules {
    CompiledRules::from_yaml_str("version: 1\nitems: []\n", "test", None).unwrap()
}

fn bedrock_rm_rf() -> CompiledRules {
    let yaml = r#"
version: 1
items:
  - id: rm_rf_unsandboxed
    description: "Bedrock demo"
    rules:
      - id: R1-A
        pattern: "rm\\s+-[rR][fF]\\s+/"
        level: 5
        likelihood: HIGH
"#;
    CompiledRules::from_yaml_str(yaml, "bedrock", None).unwrap()
}

fn foundation_force_push() -> CompiledRules {
    // Level 4 rule — goes through the Grid path, not Seal.
    let yaml = r#"
version: 1
items:
  - id: main_force_push
    rules:
      - id: F1-A
        pattern: "git\\s+push.*--force"
        level: 4
"#;
    CompiledRules::from_yaml_str(yaml, "foundation", None).unwrap()
}

fn make_input(serialized: &str) -> ToolInput {
    ToolInput {
        tool_name: "Bash".into(),
        session_id: SessionId::new(),
        serialized: serialized.into(),
    }
}

#[test]
fn no_rules_means_allow() {
    let rules = RuleSet::from_parts(empty_rules(), empty_rules(), empty_rules());
    let gavel = Gavel::from_parts(rules, Grid::new());
    let verdict = gavel.judge(&make_input("rm -rf /"));
    assert!(!verdict.is_blocking());
    assert_eq!(verdict.enforcement, Enforcement::Dismiss);
}

#[test]
fn bedrock_match_produces_seal() {
    let rules = RuleSet::from_parts(bedrock_rm_rf(), empty_rules(), empty_rules());
    let gavel = Gavel::from_parts(rules, Grid::new());
    let verdict = gavel.judge(&make_input("rm -rf /tmp"));
    assert_eq!(verdict.enforcement, Enforcement::Seal);
    assert!(verdict.is_blocking());
    let rationale = verdict.rationale.clone();
    assert!(rationale.contains("R1-A"));
    let j = verdict.to_hook_json();
    assert_eq!(j["continue"], false);
    assert!(j["stopReason"].as_str().unwrap().contains("R1-A"));
}

#[test]
fn bedrock_precedes_foundation_and_surface() {
    // Load the same regex into all three buckets — Bedrock must win.
    let rules = RuleSet::from_parts(
        bedrock_rm_rf(),
        foundation_force_push(),
        empty_rules(),
    );
    let gavel = Gavel::from_parts(rules, Grid::new());
    let verdict = gavel.judge(&make_input("rm -rf /"));
    assert_eq!(verdict.enforcement, Enforcement::Seal);
}

#[test]
fn foundation_match_uses_grid_path_at_recurrence_one() {
    // Level 4 × Recurrence I → Caution (per default matrix).
    let rules = RuleSet::from_parts(empty_rules(), foundation_force_push(), empty_rules());
    let gavel = Gavel::from_parts(rules, Grid::new());
    let verdict = gavel.judge(&make_input("git push --force origin main"));
    assert_eq!(verdict.enforcement, Enforcement::Caution);
    let j = verdict.to_hook_json();
    assert_eq!(j["hookSpecificOutput"]["permissionDecision"], "allow");
}

#[test]
fn foundation_match_respects_fatigue_cap() {
    // Level 4 × Rec I = Caution. After 3 cautions in the session, the
    // 4th match downgrades to Note. Note emits "continue: true" with
    // no hookSpecificOutput (silent).
    let rules = RuleSet::from_parts(empty_rules(), foundation_force_push(), empty_rules());
    let gavel = Gavel::from_parts(rules, Grid::new());
    let sid = SessionId::new();
    for _ in 0..3 {
        let input = ToolInput {
            tool_name: "Bash".into(),
            session_id: sid,
            serialized: "git push --force".into(),
        };
        assert_eq!(gavel.judge(&input).enforcement, Enforcement::Caution);
    }
    let input = ToolInput {
        tool_name: "Bash".into(),
        session_id: sid,
        serialized: "git push --force".into(),
    };
    assert_eq!(gavel.judge(&input).enforcement, Enforcement::Note);
}

#[test]
fn no_match_outside_rule_scope_is_allow() {
    let rules = RuleSet::from_parts(bedrock_rm_rf(), foundation_force_push(), empty_rules());
    let gavel = Gavel::from_parts(rules, Grid::new());
    let verdict = gavel.judge(&make_input("ls -la /home"));
    assert!(!verdict.is_blocking());
    assert_eq!(verdict.enforcement, Enforcement::Dismiss);
}
