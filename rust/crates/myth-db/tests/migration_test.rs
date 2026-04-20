use myth_db::Database;
use tempfile::tempdir;

#[test]
fn fresh_db_ends_at_user_version_1() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("fresh.db");
    let db = Database::open(&path).unwrap();
    let v: i64 = db
        .conn
        .query_row("PRAGMA user_version", [], |r| r.get(0))
        .unwrap();
    assert_eq!(v, 1);
}

#[test]
fn reopening_an_initialized_db_is_idempotent() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("reopen.db");

    {
        let _db = Database::open(&path).unwrap();
    }

    let db = Database::open(&path).unwrap();
    let v: i64 = db
        .conn
        .query_row("PRAGMA user_version", [], |r| r.get(0))
        .unwrap();
    assert_eq!(v, 1);

    // No duplicate tables created on reopen.
    let tables: i64 = db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'lessons'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(tables, 1);
}
