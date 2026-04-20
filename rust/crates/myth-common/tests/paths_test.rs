use myth_common::paths::{
    audit_path, bedrock_rules_path, caselog_path, myth_config, myth_home, state_db_path,
};

#[test]
fn myth_home_ends_with_dot_myth() {
    let p = myth_home();
    assert_eq!(p.file_name().and_then(|s| s.to_str()), Some(".myth"));
}

#[test]
fn myth_config_respects_xdg_config_home() {
    let orig = std::env::var_os("XDG_CONFIG_HOME");
    let tmp = std::env::temp_dir().join("myth-paths-test-xdg");
    std::env::set_var("XDG_CONFIG_HOME", &tmp);

    let p = myth_config();

    // Restore before the assertion fires.
    match orig {
        Some(v) => std::env::set_var("XDG_CONFIG_HOME", v),
        None => std::env::remove_var("XDG_CONFIG_HOME"),
    }

    assert!(
        p.starts_with(&tmp),
        "expected {:?} to start with {:?}",
        p,
        tmp
    );
    assert_eq!(p.file_name().and_then(|s| s.to_str()), Some("myth"));
}

#[test]
fn specific_paths_live_under_myth_home() {
    let home = myth_home();
    assert!(state_db_path().starts_with(&home));
    assert!(bedrock_rules_path().starts_with(&home));
    assert!(caselog_path().starts_with(&home));
    assert!(audit_path().starts_with(&home));
}
