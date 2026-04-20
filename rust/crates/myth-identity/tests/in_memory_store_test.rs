//! Gate test for Task 2.3 single checkpoint — VectorStore trait +
//! InMemoryStore + vectors.bin round-trip.

use std::collections::HashMap;

use myth_common::LessonId;
use myth_identity::{Embedding, InMemoryStore, VectorStore, EMBEDDING_DIM};
use tempfile::tempdir;

/// Build a normalised vector that points "mostly" along axis `i`, with
/// a little noise on other axes. Useful for predictable KNN ordering.
fn unit_vector(i: usize) -> Embedding {
    assert!(i < EMBEDDING_DIM);
    let mut v = [0.0f32; EMBEDDING_DIM];
    v[i] = 1.0;
    v
}

fn perturbed(i: usize, perturb_axis: usize, amount: f32) -> Embedding {
    let mut v = unit_vector(i);
    v[perturb_axis] += amount;
    // Renormalise so integrity_check stays happy.
    let n: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
    for x in v.iter_mut() {
        *x /= n;
    }
    v
}

#[test]
fn create_then_open_roundtrips_generation() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("vectors.bin");

    let store = InMemoryStore::create(&path).unwrap();
    assert!(store.is_empty());
    assert_eq!(store.generation(), 0);

    let id = LessonId::new();
    store.upsert(id, &unit_vector(5)).unwrap();
    assert_eq!(store.len(), 1);
    assert_eq!(store.generation(), 1);

    // Reopen from disk — vectors and generation survive.
    let reopened = InMemoryStore::open(&path).unwrap();
    assert_eq!(reopened.len(), 1);
    assert_eq!(reopened.generation(), 1);
}

#[test]
fn upsert_and_count() {
    let dir = tempdir().unwrap();
    let store = InMemoryStore::create(dir.path().join("v.bin")).unwrap();
    for i in 0..10 {
        store.upsert(LessonId::new(), &unit_vector(i)).unwrap();
    }
    assert_eq!(store.len(), 10);
    assert_eq!(store.generation(), 10);
}

#[test]
fn knn_returns_known_neighbour_first() {
    let dir = tempdir().unwrap();
    let store = InMemoryStore::create(dir.path().join("v.bin")).unwrap();

    let id_target = LessonId::new();
    let id_near = LessonId::new();
    let id_far = LessonId::new();

    store.upsert(id_target, &unit_vector(0)).unwrap();
    store.upsert(id_near, &perturbed(0, 1, 0.05)).unwrap();
    store.upsert(id_far, &unit_vector(100)).unwrap();

    // Query with the same vector as id_target; it should rank first.
    let results = store.knn(&unit_vector(0), 3).unwrap();
    assert_eq!(results.len(), 3);
    assert_eq!(results[0].0, id_target, "exact match must be nearest");
    assert_eq!(results[1].0, id_near, "slightly perturbed must be second");
    assert_eq!(results[2].0, id_far, "orthogonal must be last");

    // Distances are monotonic increasing.
    assert!(results[0].1 <= results[1].1);
    assert!(results[1].1 <= results[2].1);
}

#[test]
fn update_same_id_keeps_row() {
    let dir = tempdir().unwrap();
    let store = InMemoryStore::create(dir.path().join("v.bin")).unwrap();
    let id = LessonId::new();

    store.upsert(id, &unit_vector(0)).unwrap();
    store.upsert(id, &unit_vector(1)).unwrap();

    assert_eq!(store.len(), 1, "same id must not grow count");
    // After updating, nearest neighbour to unit_vector(1) should be the
    // same id with distance ~0.
    let results = store.knn(&unit_vector(1), 1).unwrap();
    assert_eq!(results[0].0, id);
    assert!(results[0].1 < 0.01, "updated vector should match query");
}

#[test]
fn delete_removes_from_knn() {
    let dir = tempdir().unwrap();
    let store = InMemoryStore::create(dir.path().join("v.bin")).unwrap();
    let keep = LessonId::new();
    let drop_id = LessonId::new();

    store.upsert(keep, &unit_vector(0)).unwrap();
    store.upsert(drop_id, &unit_vector(10)).unwrap();
    assert_eq!(store.len(), 2);

    store.delete(drop_id).unwrap();
    assert_eq!(store.len(), 1);

    let results = store.knn(&unit_vector(10), 5).unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].0, keep);
}

#[test]
fn integrity_check_healthy_on_fresh_store() {
    let dir = tempdir().unwrap();
    let store = InMemoryStore::create(dir.path().join("v.bin")).unwrap();
    for i in 0..5 {
        store.upsert(LessonId::new(), &unit_vector(i)).unwrap();
    }
    let report = store.integrity_check().unwrap();
    assert_eq!(report.total_vectors, 5);
    assert!(report.index_consistent);
    assert!(report.generation_match);
    assert_eq!(report.norm_anomalies, 0);
    assert!(report.is_healthy());
}

#[test]
fn integrity_check_detects_generation_drift() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("v.bin");
    let store = InMemoryStore::create(&path).unwrap();
    store.upsert(LessonId::new(), &unit_vector(0)).unwrap();

    // Overwrite the on-disk file with a mismatched generation so the
    // in-memory struct and the persisted file disagree.
    let bytes = std::fs::read(&path).unwrap();
    let mut tampered = bytes.clone();
    // Generation lives at offset 0x0C (12..20).
    tampered[12] = 0xFF;
    tampered[13] = 0xFF;
    std::fs::write(&path, &tampered).unwrap();

    let report = store.integrity_check().unwrap();
    assert!(
        !report.generation_match,
        "integrity_check must flag generation drift"
    );
    assert!(!report.is_healthy());
}

#[test]
fn load_index_sets_id_mapping() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("v.bin");

    // Write a couple vectors in one store, then reopen and rehydrate
    // the index externally.
    let id_a = LessonId::new();
    let id_b = LessonId::new();
    {
        let s = InMemoryStore::create(&path).unwrap();
        s.upsert(id_a, &unit_vector(3)).unwrap();
        s.upsert(id_b, &unit_vector(7)).unwrap();
    }

    let reopened = InMemoryStore::open(&path).unwrap();
    assert_eq!(reopened.len(), 2);
    // Without the index, knn finds nothing (all rows unmapped).
    assert!(reopened.knn(&unit_vector(3), 1).unwrap().is_empty());

    let mut map = HashMap::new();
    map.insert(id_a, 0);
    map.insert(id_b, 1);
    reopened.load_index(map).unwrap();

    let results = reopened.knn(&unit_vector(3), 1).unwrap();
    assert_eq!(results[0].0, id_a);
}
