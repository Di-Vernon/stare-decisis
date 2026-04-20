use myth_common::{Category, Level, LessonId};
use myth_db::{Database, Lesson, LessonStatus, LessonStore, SqliteLessonStore};
use tempfile::tempdir;

fn sample_lesson() -> Lesson {
    let now = myth_common::now();
    Lesson {
        id: LessonId::new(),
        identity_hash_tier1: [7u8; 20],
        level: Level::Medium,
        category: Category::Correctness,
        recurrence_count: 1.0,
        missed_hook_count: 0,
        first_seen: now,
        last_seen: now,
        lapse_score: 0.0,
        appeals: 0,
        status: LessonStatus::Active,
        description: "heredoc quoting issue".into(),
        rationale: "recurring in shell commands".into(),
        meta_json: None,
    }
}

#[test]
fn insert_and_get_roundtrip() {
    let dir = tempdir().unwrap();
    let db = Database::open(&dir.path().join("t.db")).unwrap();
    let store = SqliteLessonStore::new(db);

    let lesson = sample_lesson();
    let id = store.insert(&lesson).unwrap();

    let fetched = store.get(id).unwrap().expect("lesson must exist");
    assert_eq!(fetched.id, lesson.id);
    assert_eq!(fetched.identity_hash_tier1, lesson.identity_hash_tier1);
    assert_eq!(fetched.level, Level::Medium);
    assert_eq!(fetched.category, Category::Correctness);
    assert_eq!(fetched.description, lesson.description);
    assert_eq!(fetched.status, LessonStatus::Active);
}

#[test]
fn find_by_identity_returns_lesson() {
    let dir = tempdir().unwrap();
    let db = Database::open(&dir.path().join("t.db")).unwrap();
    let store = SqliteLessonStore::new(db);

    let lesson = sample_lesson();
    store.insert(&lesson).unwrap();
    let found = store
        .find_by_identity(&lesson.identity_hash_tier1)
        .unwrap()
        .unwrap();
    assert_eq!(found.id, lesson.id);
}

#[test]
fn increment_recurrence_persists() {
    let dir = tempdir().unwrap();
    let db = Database::open(&dir.path().join("t.db")).unwrap();
    let store = SqliteLessonStore::new(db);

    let lesson = sample_lesson();
    let id = store.insert(&lesson).unwrap();

    let new_count = store.increment_recurrence(id).unwrap();
    assert!((new_count - 2.0).abs() < f64::EPSILON);
    let fetched = store.get(id).unwrap().unwrap();
    assert!((fetched.recurrence_count - 2.0).abs() < f64::EPSILON);
}

#[test]
fn update_mutates_fields() {
    let dir = tempdir().unwrap();
    let db = Database::open(&dir.path().join("t.db")).unwrap();
    let store = SqliteLessonStore::new(db);

    let mut lesson = sample_lesson();
    let id = store.insert(&lesson).unwrap();
    lesson.description = "updated description".into();
    lesson.lapse_score = 75.0;
    store.update(&lesson).unwrap();

    let fetched = store.get(id).unwrap().unwrap();
    assert_eq!(fetched.description, "updated description");
    assert!((fetched.lapse_score - 75.0).abs() < f64::EPSILON);
}

#[test]
fn mark_status_moves_between_lists() {
    let dir = tempdir().unwrap();
    let db = Database::open(&dir.path().join("t.db")).unwrap();
    let store = SqliteLessonStore::new(db);

    let lesson = sample_lesson();
    let id = store.insert(&lesson).unwrap();
    assert_eq!(store.list_active().unwrap().len(), 1);
    assert_eq!(store.list_lapsed().unwrap().len(), 0);

    store.mark_status(id, LessonStatus::Lapsed).unwrap();
    assert_eq!(store.list_active().unwrap().len(), 0);
    assert_eq!(store.list_lapsed().unwrap().len(), 1);
}

#[test]
fn get_missing_returns_none() {
    let dir = tempdir().unwrap();
    let db = Database::open(&dir.path().join("t.db")).unwrap();
    let store = SqliteLessonStore::new(db);
    let random = LessonId::new();
    assert!(store.get(random).unwrap().is_none());
}
