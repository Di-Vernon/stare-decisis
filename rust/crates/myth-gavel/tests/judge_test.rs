//! Integration-style tests for `Gavel::judge`, using `from_parts` to
//! bypass filesystem and (mostly) DB dependencies. A throw-away SQLite
//! lives in a tempdir so the `Box<dyn LessonStore>` invariant holds.

use myth_common::{Category, Enforcement, Level, LessonId, SessionId};
use myth_db::{
    Database, Lesson, LessonStatus, LessonStore, SqliteLessonStore,
};
use myth_gavel::{CompiledRules, Gavel, Grid, RuleSet, ToolInput};
use tempfile::TempDir;

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

/// Spin up a fresh Gavel over an empty on-disk SQLite store. Returns
/// the tempdir so it lives as long as the test — dropping it cleans up.
fn make_gavel_with_empty_store(rules: RuleSet, grid: Grid) -> (Gavel, TempDir) {
    let dir = tempfile::tempdir().unwrap();
    let db = Database::open(&dir.path().join("state.db")).unwrap();
    let store = Box::new(SqliteLessonStore::new(db));
    (Gavel::from_parts(rules, grid, store), dir)
}

/// Spin up a Gavel whose lesson store already has one lesson pre-
/// inserted for the given (rule_id, matched_text) identity. Useful
/// for exercising the grid_path recurrence lookup.
fn make_gavel_with_seeded_lesson(
    rules: RuleSet,
    grid: Grid,
    rule_id: &str,
    matched_text: &str,
    recurrence_count: f64,
    level: Level,
) -> (Gavel, TempDir) {
    let dir = tempfile::tempdir().unwrap();
    let db = Database::open(&dir.path().join("state.db")).unwrap();
    let store = SqliteLessonStore::new(db);

    let hash = Gavel::compute_identity(rule_id, matched_text);
    let now = myth_common::now();
    let lesson = Lesson {
        id: LessonId::new(),
        identity_hash_tier1: hash,
        level,
        category: Category::Correctness,
        recurrence_count,
        missed_hook_count: 0,
        first_seen: now,
        last_seen: now,
        lapse_score: 0.0,
        appeals: 0,
        status: LessonStatus::Active,
        description: "pre-seeded for test".into(),
        rationale: "test".into(),
        meta_json: None,
    };
    store.insert(&lesson).unwrap();

    let gavel = Gavel::from_parts(rules, grid, Box::new(store));
    (gavel, dir)
}

#[test]
fn no_rules_means_allow() {
    let rules = RuleSet::from_parts(empty_rules(), empty_rules(), empty_rules());
    let (gavel, _dir) = make_gavel_with_empty_store(rules, Grid::new());
    let verdict = gavel.judge(&make_input("rm -rf /"));
    assert!(!verdict.is_blocking());
    assert_eq!(verdict.enforcement, Enforcement::Dismiss);
}

#[test]
fn bedrock_match_produces_seal() {
    let rules = RuleSet::from_parts(bedrock_rm_rf(), empty_rules(), empty_rules());
    let (gavel, _dir) = make_gavel_with_empty_store(rules, Grid::new());
    let verdict = gavel.judge(&make_input("rm -rf /tmp"));
    assert_eq!(verdict.enforcement, Enforcement::Seal);
    assert!(verdict.is_blocking());
    assert!(verdict.rationale.contains("R1-A"));
    let j = verdict.to_hook_json();
    assert_eq!(j["continue"], false);
    assert!(j["stopReason"].as_str().unwrap().contains("R1-A"));
}

#[test]
fn bedrock_precedes_foundation_and_surface() {
    let rules = RuleSet::from_parts(bedrock_rm_rf(), foundation_force_push(), empty_rules());
    let (gavel, _dir) = make_gavel_with_empty_store(rules, Grid::new());
    let verdict = gavel.judge(&make_input("rm -rf /"));
    assert_eq!(verdict.enforcement, Enforcement::Seal);
}

#[test]
fn foundation_match_uses_grid_path_at_recurrence_one() {
    // Level 4 × Recurrence I → Caution (per default matrix).
    let rules = RuleSet::from_parts(empty_rules(), foundation_force_push(), empty_rules());
    let (gavel, _dir) = make_gavel_with_empty_store(rules, Grid::new());
    let verdict = gavel.judge(&make_input("git push --force origin main"));
    assert_eq!(verdict.enforcement, Enforcement::Caution);
    let j = verdict.to_hook_json();
    assert_eq!(j["hookSpecificOutput"]["permissionDecision"], "allow");
}

#[test]
fn foundation_match_respects_fatigue_cap() {
    let rules = RuleSet::from_parts(empty_rules(), foundation_force_push(), empty_rules());
    let (gavel, _dir) = make_gavel_with_empty_store(rules, Grid::new());
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
    let (gavel, _dir) = make_gavel_with_empty_store(rules, Grid::new());
    let verdict = gavel.judge(&make_input("ls -la /home"));
    assert!(!verdict.is_blocking());
    assert_eq!(verdict.enforcement, Enforcement::Dismiss);
}

#[test]
fn grid_path_lifts_recurrence_from_seeded_lesson() {
    // Pre-seed a lesson with recurrence_count = 5.0 → Recurrence::IV
    // (per Recurrence::from_count: >=4 and <7). Level 4 × Rec IV in
    // the default matrix is Strike — blocking.
    let rules = RuleSet::from_parts(empty_rules(), foundation_force_push(), empty_rules());

    // `matched_text` must be what the F1-A regex actually captures. The
    // regex `git\s+push.*--force` greedily matches from `git push` to
    // `--force` inclusive.
    let matched_text = "git push --force";
    let (gavel, _dir) = make_gavel_with_seeded_lesson(
        rules,
        Grid::new(),
        "F1-A",
        matched_text,
        5.0,
        Level::High,
    );

    let input = make_input("git push --force origin main");
    let verdict = gavel.judge(&input);
    assert_eq!(
        verdict.enforcement,
        Enforcement::Strike,
        "Level 4 × Recurrence IV must be Strike via grid_path lookup, got {:?}",
        verdict.enforcement
    );
    assert!(verdict.lesson_id.is_some(), "verdict must carry lesson id");
}

#[test]
fn grid_path_falls_back_to_recurrence_one_when_no_lesson() {
    // Same rule but no seeded lesson — Recurrence::I, so Level 4 × I =
    // Caution (default matrix).
    let rules = RuleSet::from_parts(empty_rules(), foundation_force_push(), empty_rules());
    let (gavel, _dir) = make_gavel_with_empty_store(rules, Grid::new());
    let verdict = gavel.judge(&make_input("git push --force"));
    assert_eq!(verdict.enforcement, Enforcement::Caution);
    assert!(verdict.lesson_id.is_none());
}
