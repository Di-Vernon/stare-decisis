use myth_gavel::CompiledRules;

const EMPTY_YAML: &str = "version: 1\nitems: []\n";

const SIMPLE_YAML: &str = r#"
version: 1
items:
  - id: rm_rf_unsandboxed
    description: "demo"
    rules:
      - id: R1-A
        pattern: "rm\\s+-[rR][fF]\\s+/"
        level: 5
        likelihood: HIGH
        source: "demo"
"#;

const INVALID_REGEX_YAML: &str = r#"
version: 1
items:
  - id: broken
    rules:
      - id: X1-A
        pattern: "(unclosed"
        level: 3
"#;

#[test]
fn empty_items_loads_to_empty_rule_set() {
    let rules = CompiledRules::from_yaml_str(EMPTY_YAML, "test", None).unwrap();
    assert!(rules.is_empty());
    assert_eq!(rules.len(), 0);
    assert!(rules.match_any("rm -rf /").is_none());
}

#[test]
fn simple_rule_compiles_and_matches() {
    let rules = CompiledRules::from_yaml_str(SIMPLE_YAML, "bedrock", None).unwrap();
    assert_eq!(rules.len(), 1);

    let m = rules.match_any("sudo rm -rf /").expect("match expected");
    assert_eq!(m.rule_id, "R1-A");
    assert_eq!(m.item, "rm_rf_unsandboxed");
}

#[test]
fn non_matching_text_returns_none() {
    let rules = CompiledRules::from_yaml_str(SIMPLE_YAML, "bedrock", None).unwrap();
    assert!(rules.match_any("ls -la /tmp").is_none());
}

#[test]
fn invalid_regex_pattern_errors_out() {
    let res = CompiledRules::from_yaml_str(INVALID_REGEX_YAML, "test", None);
    assert!(
        res.is_err(),
        "broken regex must fail to compile (deny by default upstream)"
    );
}

#[test]
fn missing_file_yields_empty_rules() {
    // A path that cannot exist — load() should log + return empty, not error.
    let nowhere = std::path::Path::new("/nonexistent/myth/bedrock-rules.yaml");
    let rules = CompiledRules::load(nowhere, "bedrock").unwrap();
    assert!(rules.is_empty());
}
