use myth_db::{AuditEvent, AuditLog};
use std::io::Write;
use tempfile::tempdir;

#[test]
fn ten_entry_chain_verifies_green() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("audit.jsonl");
    let log = AuditLog::open(&path).unwrap();

    log.append(AuditEvent::Genesis).unwrap();
    for i in 0..9 {
        log.append(AuditEvent::BedrockRuleModified {
            rule_id: format!("R{}", i),
            user: "jeffrey".into(),
        })
        .unwrap();
    }

    assert!(log.verify().unwrap(), "a freshly-built chain must verify");
}

#[test]
fn reopened_chain_continues_correctly() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("audit.jsonl");

    {
        let log = AuditLog::open(&path).unwrap();
        log.append(AuditEvent::Genesis).unwrap();
        log.append(AuditEvent::BedrockRuleModified {
            rule_id: "R1".into(),
            user: "jeffrey".into(),
        })
        .unwrap();
    }

    let log = AuditLog::open(&path).unwrap();
    log.append(AuditEvent::AppealGranted {
        lesson_id: "L-abc".into(),
        resolver: "observer".into(),
    })
    .unwrap();

    assert!(log.verify().unwrap(), "chain survives reopening");
}

#[test]
fn tampered_entry_fails_verification() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("audit.jsonl");
    let log = AuditLog::open(&path).unwrap();

    log.append(AuditEvent::Genesis).unwrap();
    for i in 0..9 {
        log.append(AuditEvent::BedrockRuleModified {
            rule_id: format!("R{}", i),
            user: "jeffrey".into(),
        })
        .unwrap();
    }

    // Tamper: rewrite line 5's payload but leave hashes intact → the
    // stored `hash` should no longer match a fresh recompute.
    let content = std::fs::read_to_string(&path).unwrap();
    let mut lines: Vec<String> = content.lines().map(String::from).collect();
    lines[5] = lines[5].replace("\"R4\"", "\"R9999\"");

    let mut f = std::fs::File::create(&path).unwrap();
    for line in &lines {
        writeln!(f, "{}", line).unwrap();
    }

    let log2 = AuditLog::open(&path).unwrap();
    assert!(
        !log2.verify().unwrap(),
        "tampered chain must fail verification"
    );
}
