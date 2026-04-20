//! Library-level tests for the DB path that the post-tool bin drives.
//! Subprocess round-trip (bin invocation → DB row) lands in Task 3.6.

use myth_common::SessionId;
use myth_db::events::{self, HookEvent, HookEventType, Verdict};
use myth_db::Database;
use tempfile::tempdir;
use uuid::Uuid;

fn sample_event() -> HookEvent {
    HookEvent {
        id: Uuid::new_v4(),
        session_id: SessionId::new(),
        event_type: HookEventType::PostTool,
        tool_name: Some("Bash".into()),
        ts: myth_common::now(),
        latency_ms: 0.0,
        verdict: Verdict::Allow,
        lesson_id: None,
    }
}

#[test]
fn post_tool_event_writes_one_row() {
    let dir = tempdir().unwrap();
    let db = Database::open(&dir.path().join("state.db")).unwrap();
    events::insert(&db, &sample_event()).unwrap();

    let n: i64 = db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM hook_events WHERE event_type = 'post_tool'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(n, 1);
}

#[test]
fn multiple_post_tool_events_accumulate() {
    let dir = tempdir().unwrap();
    let db = Database::open(&dir.path().join("state.db")).unwrap();

    for _ in 0..5 {
        events::insert(&db, &sample_event()).unwrap();
    }

    let n: i64 = db
        .conn
        .query_row("SELECT COUNT(*) FROM hook_events", [], |r| r.get(0))
        .unwrap();
    assert_eq!(n, 5);
}

#[test]
fn post_tool_event_carries_tool_name_and_verdict() {
    let dir = tempdir().unwrap();
    let db = Database::open(&dir.path().join("state.db")).unwrap();

    let e = HookEvent {
        tool_name: Some("Read".into()),
        ..sample_event()
    };
    events::insert(&db, &e).unwrap();

    let (tool_name, verdict, event_type): (String, String, String) = db
        .conn
        .query_row(
            "SELECT tool_name, verdict, event_type FROM hook_events LIMIT 1",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .unwrap();
    assert_eq!(tool_name, "Read");
    assert_eq!(verdict, "allow");
    assert_eq!(event_type, "post_tool");
}
