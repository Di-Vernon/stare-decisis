use myth_db::Database;
use tempfile::tempdir;

#[test]
fn open_creates_db_and_applies_pragmas() {
    let dir = tempdir().unwrap();
    let db = Database::open(&dir.path().join("test.db")).unwrap();

    let journal_mode: String = db
        .conn
        .query_row("PRAGMA journal_mode", [], |r| r.get(0))
        .unwrap();
    assert_eq!(journal_mode.to_lowercase(), "wal");

    let fk: i64 = db
        .conn
        .query_row("PRAGMA foreign_keys", [], |r| r.get(0))
        .unwrap();
    assert_eq!(fk, 1);

    let user_version: i64 = db
        .conn
        .query_row("PRAGMA user_version", [], |r| r.get(0))
        .unwrap();
    assert_eq!(user_version, 1);
}

#[test]
fn schema_tables_exist() {
    let dir = tempdir().unwrap();
    let db = Database::open(&dir.path().join("test.db")).unwrap();

    for table in [
        "lessons",
        "vector_metadata",
        "vector_generation",
        "hook_events",
        "appeal_history",
        "grid_overrides",
        "sessions",
    ] {
        let count: i64 = db
            .conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = ?1",
                [table],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 1, "table {} is missing", table);
    }
}

#[test]
fn vector_generation_seed_row_inserted() {
    let dir = tempdir().unwrap();
    let db = Database::open(&dir.path().join("test.db")).unwrap();

    let (id, gen): (i64, i64) = db
        .conn
        .query_row(
            "SELECT id, current_generation FROM vector_generation",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    assert_eq!(id, 1);
    assert_eq!(gen, 0);
}
