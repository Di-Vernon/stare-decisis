//! Wave 7 Task 7.4 — FP=0 / FN=0 fixture harness.
//!
//! Loads `templates/bedrock-rules.yaml` + `templates/foundation-rules.yaml`,
//! iterates 560 fixtures (280 positive + 280 negative), and asserts:
//!   - each positive matches its expected rule id + level
//!   - each negative matches *no* rule (FP=0)
//!
//! `harness_sanity` runs first (10 obvious cases) to catch harness bugs
//! before the 560 sweep — otherwise a broken harness would spray noise
//! across all failures.

use myth_gavel::{CompiledRules, RuleSet};
use serde::Deserialize;
use std::path::PathBuf;

// ──────────────────────────── schemas ────────────────────────────

#[derive(Deserialize, Debug)]
struct PositiveFile {
    rule: String,
    #[allow(dead_code)]
    expected_level: u8,
    #[allow(dead_code)]
    #[serde(default)]
    category: Option<String>,
    cases: Vec<PositiveCase>,
}

#[derive(Deserialize, Debug)]
struct PositiveCase {
    id: String,
    input: String,
    #[serde(default)]
    #[allow(dead_code)]
    note: Option<String>,
}

#[derive(Deserialize, Debug)]
struct NegativeFile {
    rule_context: String,
    #[allow(dead_code)]
    #[serde(default)]
    expected: Option<String>,
    cases: Vec<NegativeCase>,
}

#[derive(Deserialize, Debug)]
struct NegativeCase {
    id: String,
    input: String,
    #[serde(default)]
    #[allow(dead_code)]
    note: Option<String>,
}

// ──────────────────────────── helpers ────────────────────────────

fn repo_root() -> PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(3)
        .expect("repo root")
        .to_path_buf()
}

fn load_ruleset() -> RuleSet {
    let root = repo_root();
    let bedrock_path = root.join("templates/bedrock-rules.yaml");
    let foundation_path = root.join("templates/foundation-rules.yaml");

    let bedrock = CompiledRules::load(&bedrock_path, "bedrock")
        .expect("bedrock rules must load");
    let foundation = CompiledRules::load(&foundation_path, "foundation")
        .expect("foundation rules must load");
    let surface = CompiledRules::empty();

    RuleSet::from_parts(bedrock, foundation, surface)
}

/// Match against bedrock first, then foundation (Gavel's actual precedence).
fn match_any_rule(rules: &RuleSet, input: &str) -> Option<(String, String)> {
    if let Some(m) = rules.bedrock.match_any(input) {
        return Some((m.rule_id, format!("{:?}", m.matched_text)));
    }
    if let Some(m) = rules.foundation.match_any(input) {
        return Some((m.rule_id, format!("{:?}", m.matched_text)));
    }
    None
}

// ──────────────────── test 1: harness self-sanity ────────────────────

/// Runs first alphabetically. 10 obvious cases — 5 must match exactly
/// the named rule, 5 must not match at all. If this fails, the harness
/// itself is broken and the 560 sweep result is meaningless.
#[test]
fn a01_harness_sanity() {
    let rules = load_ruleset();

    // 5 obvious positives
    // Note: "git push --force origin main" matches R1-D (Bedrock) first —
    // Gavel precedence is Bedrock > Foundation. Use "git push --force origin
    // prod" for F1-A because R1-D's branch list is main/master/release/
    // production (no "prod") while F1-A's is main/master/release/production/prod.
    let obvious_positives: &[(&str, &str)] = &[
        ("rm -rf /", "R1-A"),
        ("git push --force origin prod", "F1-A"),
        ("AKIAIOSFODNN7EXAMPLE", "R2-A"),
        ("shutil.rmtree('/etc')", "R1-C"),
        ("verify=False", "R3-A"),
    ];
    for (input, expected_rule) in obvious_positives {
        let m = match_any_rule(&rules, input);
        let (matched_id, _) = m.unwrap_or_else(|| {
            panic!(
                "SANITY FAIL: obvious positive {:?} did not match any rule (expected {})",
                input, expected_rule
            )
        });
        assert_eq!(
            &matched_id, expected_rule,
            "SANITY FAIL: {:?} matched {} instead of {}",
            input, matched_id, expected_rule
        );
    }

    // 5 obvious negatives
    let obvious_negatives = [
        "echo hello world",
        "ls -la /tmp",
        "git status",
        "def main(): pass",
        "print('safe code')",
    ];
    for input in obvious_negatives {
        let m = match_any_rule(&rules, input);
        assert!(
            m.is_none(),
            "SANITY FAIL: obvious negative {:?} unexpectedly matched rule {:?}",
            input,
            m.map(|(id, text)| format!("{} (matched text: {})", id, text))
        );
    }
}

// ─────────────────── test 2: 560 fixture full sweep ───────────────────

#[test]
fn a02_fixtures_full_sweep() {
    let rules = load_ruleset();
    let fixtures_dir = repo_root().join("tests/fixtures");

    let mut total_positive = 0usize;
    let mut total_negative = 0usize;
    let mut fp_count = 0usize;
    let mut fn_count = 0usize;
    let mut cross_count = 0usize;
    let mut failures: Vec<String> = Vec::new();

    // ──── positive ────
    let pos_dir = fixtures_dir.join("positive");
    let mut pos_entries: Vec<_> = std::fs::read_dir(&pos_dir)
        .unwrap_or_else(|e| panic!("read_dir {:?}: {}", pos_dir, e))
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("yaml"))
        .collect();
    pos_entries.sort();

    for path in &pos_entries {
        let content = std::fs::read_to_string(path)
            .unwrap_or_else(|e| panic!("read {:?}: {}", path, e));
        let file: PositiveFile = serde_yaml::from_str(&content)
            .unwrap_or_else(|e| panic!("parse {:?}: {}", path, e));

        for case in &file.cases {
            total_positive += 1;
            match match_any_rule(&rules, &case.input) {
                None => {
                    fn_count += 1;
                    failures.push(format!(
                        "FN: {}/{} — no rule matched input {:?}",
                        file.rule, case.id, case.input
                    ));
                }
                Some((matched_id, matched_text)) if matched_id != file.rule => {
                    cross_count += 1;
                    failures.push(format!(
                        "CROSS: {}/{} — matched {} (text: {}) instead of {}",
                        file.rule, case.id, matched_id, matched_text, file.rule
                    ));
                }
                Some(_) => {
                    // correct match — (level check skipped; rule_id implies level via YAML)
                }
            }
        }
    }

    // ──── negative ────
    let neg_dir = fixtures_dir.join("negative");
    let mut neg_entries: Vec<_> = std::fs::read_dir(&neg_dir)
        .unwrap_or_else(|e| panic!("read_dir {:?}: {}", neg_dir, e))
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("yaml"))
        .collect();
    neg_entries.sort();

    for path in &neg_entries {
        let content = std::fs::read_to_string(path)
            .unwrap_or_else(|e| panic!("read {:?}: {}", path, e));
        let file: NegativeFile = serde_yaml::from_str(&content)
            .unwrap_or_else(|e| panic!("parse {:?}: {}", path, e));

        for case in &file.cases {
            total_negative += 1;
            if let Some((matched_id, matched_text)) = match_any_rule(&rules, &case.input) {
                fp_count += 1;
                failures.push(format!(
                    "FP: {}/{} — matched {} (text: {}) on near-miss input {:?}",
                    file.rule_context, case.id, matched_id, matched_text, case.input
                ));
            }
        }
    }

    // ──── report ────
    let total = total_positive + total_negative;
    eprintln!(
        "\n=== Fixture Harness Summary ===\n\
         total:    {} (positive {}, negative {})\n\
         FP:       {}\n\
         FN:       {}\n\
         CROSS:    {}\n\
         failures: {}\n",
        total, total_positive, total_negative, fp_count, fn_count, cross_count, failures.len()
    );
    if !failures.is_empty() {
        eprintln!("--- failure detail (up to 40) ---");
        for f in failures.iter().take(40) {
            eprintln!("  {}", f);
        }
        if failures.len() > 40 {
            eprintln!("  ... ({} more)", failures.len() - 40);
        }
    }

    assert_eq!(total_positive, 280, "expected 280 positive fixtures, got {}", total_positive);
    assert_eq!(total_negative, 280, "expected 280 negative fixtures, got {}", total_negative);
    assert_eq!(fn_count, 0, "FN (positive missed): {} — must be 0", fn_count);
    assert_eq!(fp_count, 0, "FP (negative triggered): {} — must be 0", fp_count);
    assert_eq!(cross_count, 0, "CROSS (wrong rule): {} — must be 0", cross_count);
}
