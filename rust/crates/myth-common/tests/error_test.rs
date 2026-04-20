use anyhow::anyhow;
use myth_common::error::{MythError, Result};

#[test]
fn io_error_converts_via_from() {
    fn raises() -> Result<()> {
        let err = std::io::Error::new(std::io::ErrorKind::NotFound, "nope");
        Err(err.into())
    }
    let e = raises().unwrap_err();
    assert!(matches!(e, MythError::Io(_)));
}

#[test]
fn anyhow_converts_into_other() {
    fn raises() -> Result<()> {
        Err(anyhow!("external").into())
    }
    let e = raises().unwrap_err();
    assert!(matches!(e, MythError::Other(_)));
}

#[test]
fn json_error_converts_via_question_mark() {
    fn raises() -> Result<i32> {
        let n: i32 = serde_json::from_str("not json")?;
        Ok(n)
    }
    let e = raises().unwrap_err();
    assert!(matches!(e, MythError::Json(_)));
}

#[test]
fn config_missing_display_format() {
    let e = MythError::ConfigMissing {
        path: "/etc/myth/config".into(),
    };
    assert_eq!(e.to_string(), "config not found at /etc/myth/config");
}

#[test]
fn rule_parse_display_format() {
    let e = MythError::RuleParse {
        file: "bedrock-rules.yaml".into(),
        message: "bad anchor".into(),
    };
    assert_eq!(
        e.to_string(),
        "rule file parse error in bedrock-rules.yaml: bad anchor"
    );
}
