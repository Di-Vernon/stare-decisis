//! Sanity: verify templates/bedrock-rules.yaml + foundation-rules.yaml
//! compile correctly. This gate catches regex errors early (Task 7.1).
//!
//! Intentionally scoped to parse + compile; fixture match correctness is
//! Task 7.4's harness.

use myth_gavel::CompiledRules;

fn repo_root() -> std::path::PathBuf {
    // CARGO_MANIFEST_DIR = rust/crates/myth-gavel → ancestors[3] = repo root
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(3)
        .expect("repo root")
        .to_path_buf()
}

#[test]
fn bedrock_template_compiles() {
    let path = repo_root().join("templates/bedrock-rules.yaml");
    let content = std::fs::read_to_string(&path).expect("read bedrock");
    let rules = CompiledRules::from_yaml_str(&content, "bedrock", Some(&path))
        .expect("bedrock template must compile");
    assert!(rules.len() >= 15, "expected 15 bedrock rules, got {}", rules.len());
}

#[test]
fn foundation_template_compiles() {
    let path = repo_root().join("templates/foundation-rules.yaml");
    let content = std::fs::read_to_string(&path).expect("read foundation");
    let rules = CompiledRules::from_yaml_str(&content, "foundation", Some(&path))
        .expect("foundation template must compile");
    assert!(rules.len() >= 5, "expected 5 foundation rules, got {}", rules.len());
}
