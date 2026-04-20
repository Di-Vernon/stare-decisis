# `myth-db` — 영속 저장소

## 역할

SQLite + JSONL 기반 영속 계층. 두 종류의 저장소를 **깨끗하게 분리**한다:

- **SQLite** (`state.db`): 인덱스·쿼리·트랜잭션이 필요한 메타데이터 (lessons, hook events, appeals)
- **JSONL** (`caselog.jsonl`, `lesson-state.jsonl`, `audit.jsonl`): append-only 이벤트 로그

**의존**: `myth-common`.
**의존받음**: `myth-gavel`, `myth-identity`, `myth-hooks`, `myth-observer` (Python).

## Cargo.toml

```toml
[package]
name = "myth-db"
version = "0.1.0"
edition = "2021"
license = "MIT OR Apache-2.0"

[dependencies]
myth-common = { path = "../myth-common" }

serde = { workspace = true }
serde_json = { workspace = true }
rusqlite = { workspace = true }
chrono = { workspace = true }
uuid = { workspace = true }
tracing = { workspace = true }
blake3 = { workspace = true }
anyhow = { workspace = true }
thiserror = { workspace = true }
```

## 모듈 구조

```
crates/myth-db/
├── src/
│   ├── lib.rs               # 재내보내기
│   ├── sqlite/
│   │   ├── mod.rs          # Database struct
│   │   ├── pool.rs         # connection 관리
│   │   ├── pragmas.rs      # WAL, synchronous, busy_timeout
│   │   └── migration.rs    # user_version forward-only
│   ├── lesson.rs            # LessonStore trait + SQLite 구현
│   ├── events.rs            # HookEvent 테이블
│   ├── appeal.rs            # AppealHistory 테이블
│   ├── jsonl.rs             # Append-only writer (caselog, lesson-state)
│   └── audit/
│       ├── mod.rs          # Merkle chain
│       ├── entry.rs        # AuditEntry
│       └── hasher.rs       # blake3
└── migrations/
    ├── 001_initial.sql
    ├── 002_appeals.sql
    └── 003_migration_readiness.sql
```

## SQLite 스키마 v1 (초기)

`migrations/001_initial.sql`:

```sql
PRAGMA user_version = 1;

CREATE TABLE lessons (
    id BLOB PRIMARY KEY,                    -- LessonId (UUID v4 bytes)
    identity_hash_tier1 BLOB NOT NULL,      -- SHA1 20 bytes
    level INTEGER NOT NULL,                 -- 1-5
    category TEXT NOT NULL,                 -- security/correctness/...
    recurrence_count REAL NOT NULL DEFAULT 0,
    first_seen INTEGER NOT NULL,            -- unix timestamp
    last_seen INTEGER NOT NULL,
    lapse_score REAL NOT NULL DEFAULT 0,
    appeals INTEGER NOT NULL DEFAULT 0,
    status TEXT NOT NULL DEFAULT 'active',  -- active/lapsed/archived
    description TEXT NOT NULL,
    rationale TEXT NOT NULL,
    meta_json TEXT                          -- 확장용
);

CREATE INDEX idx_lessons_identity ON lessons(identity_hash_tier1);
CREATE INDEX idx_lessons_level ON lessons(level);
CREATE INDEX idx_lessons_last_seen ON lessons(last_seen);
CREATE INDEX idx_lessons_status ON lessons(status);

CREATE TABLE hook_events (
    id BLOB PRIMARY KEY,                    -- UUID
    session_id BLOB NOT NULL,
    event_type TEXT NOT NULL,               -- pre_tool/post_tool/post_tool_failure
    tool_name TEXT,
    ts INTEGER NOT NULL,
    latency_ms REAL NOT NULL,
    verdict TEXT NOT NULL,                  -- allow/deny/ask
    lesson_id BLOB,                         -- 관련 lesson (nullable)
    FOREIGN KEY(lesson_id) REFERENCES lessons(id)
);

CREATE INDEX idx_events_session ON hook_events(session_id);
CREATE INDEX idx_events_ts ON hook_events(ts);

CREATE TABLE appeal_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    lesson_id BLOB NOT NULL,
    appeal_type TEXT NOT NULL,              -- appeal/retrial
    ts INTEGER NOT NULL,
    result TEXT NOT NULL,                   -- granted/denied/pending
    rationale TEXT,
    FOREIGN KEY(lesson_id) REFERENCES lessons(id)
);

CREATE INDEX idx_appeals_lesson ON appeal_history(lesson_id);

CREATE TABLE grid_overrides (
    level INTEGER NOT NULL,
    recurrence INTEGER NOT NULL,
    enforcement TEXT NOT NULL,
    source TEXT NOT NULL,                   -- 'default'/'admin'/'observer_suggested'
    approved_at INTEGER,
    PRIMARY KEY(level, recurrence)
);
```

각 `PRAGMA user_version`가 migration 번호.

## 연결 관리

```rust
pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn open(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)?;
        Self::apply_pragmas(&conn)?;
        Self::apply_migrations(&conn)?;
        Ok(Self { conn })
    }
    
    fn apply_pragmas(conn: &Connection) -> Result<()> {
        conn.execute_batch("
            PRAGMA journal_mode = WAL;
            PRAGMA synchronous = NORMAL;
            PRAGMA busy_timeout = 5000;
            PRAGMA foreign_keys = ON;
            PRAGMA mmap_size = 268435456;  -- 256MB
        ")?;
        Ok(())
    }
}
```

## Trait 정의

`myth-db`는 **trait을 export**하고, 다른 crate가 구현체를 받아 쓴다:

```rust
pub trait LessonStore {
    fn insert(&self, lesson: &Lesson) -> Result<LessonId>;
    fn get(&self, id: LessonId) -> Result<Option<Lesson>>;
    fn find_by_identity(&self, hash: &[u8; 20]) -> Result<Option<Lesson>>;
    fn update(&self, lesson: &Lesson) -> Result<()>;
    fn increment_recurrence(&self, id: LessonId) -> Result<f64>;
    fn list_active(&self) -> Result<Vec<Lesson>>;
    fn list_lapsed(&self) -> Result<Vec<Lesson>>;
    fn mark_status(&self, id: LessonId, status: LessonStatus) -> Result<()>;
}

pub struct SqliteLessonStore<'a> { /* db: &'a Database */ }
impl LessonStore for SqliteLessonStore<'_> { ... }
```

Day-1에서 `myth-gavel`, `myth-identity` 등은 **같은 프로세스 내 단일 스레드**에서 store를
소비한다 (binary-per-hook 모델). `Box<dyn LessonStore>` 또는 단순 참조로 전달.

> **v0.1 구현 중 변경** (Jeffrey 승인 2026-04-19)
>
> 초안의 `LessonStore: Send + Sync` bound를 **제거**했다. 근거:
> `rusqlite::Connection`은 내부 `StatementCache`가 `RefCell<LruCache<…>>`를
> 사용하므로 `!Sync`이고, `SqliteLessonStore<'a> { db: &'a Database }`
> 구조에서는 `&Database`가 `Send`이려면 `Database: Sync`가 요구되는데
> 이 역시 동일 제약으로 불가능하다. myth Day-1은 `binary-per-hook` 단일
> 프로세스·단일 스레드 모델이라 cross-thread 공유가 실제로 필요 없다.
>
> Milestone C(The Gavel daemon 전환)가 발동해 공유 프로세스 상태가
> 등장하면, 그 시점에 `Mutex<Connection>` 래퍼 레이어를 추가하고 필요한
> bound(`Send`, 필요 시 `Sync`)를 복원한다. Article 19 (Day-1 Bounded
> Responsibility)에 따라 지금은 최소 bound 유지.

## JSONL writer

append-only 로그. 세 파일 공용 helper:

```rust
pub struct JsonlWriter {
    path: PathBuf,
    // 매번 open+append (파일 잠금으로 동시성 처리)
}

impl JsonlWriter {
    pub fn new(path: impl Into<PathBuf>) -> Self { ... }
    
    pub fn append<T: Serialize>(&self, record: &T) -> Result<()> {
        let line = serde_json::to_string(record)?;
        // fcntl flock → write line + '\n' → unlock
    }
    
    pub fn iter<T: DeserializeOwned>(&self) -> impl Iterator<Item = Result<T>> {
        // 파일 read + 줄단위 파싱
    }
    
    pub fn count_lines(&self) -> Result<usize> { ... }
}
```

**파일 잠금**으로 동시 append 안전 (여러 hook 바이너리가 동시 실행 가능).

### 사용 패턴

```rust
// caselog
let caselog = JsonlWriter::new(myth_common::caselog_path());
caselog.append(&FailureRecord {
    ts: now(),
    session_id,
    tool: "Bash".into(),
    error: "...".into(),
    ...
})?;

// hook-latency (stats용, 기록 빈도 높음)
let latency_log = JsonlWriter::new(myth_common::hook_latency_path());
latency_log.append(&LatencyRecord { ... })?;
```

## Merkle Audit Chain

`audit.jsonl`은 tamper-evident. 각 엔트리가 이전 엔트리의 hash를 포함:

```rust
pub struct AuditEntry {
    pub ts: Timestamp,
    pub event: AuditEvent,  // enum: LessonCreated, AppealGranted, ...
    pub prev_hash: [u8; 32],  // blake3 이전 엔트리
    pub hash: [u8; 32],       // 현 엔트리의 blake3
}

impl AuditEntry {
    pub fn new(event: AuditEvent, prev_hash: [u8; 32]) -> Self {
        let payload = Self { ts: now(), event, prev_hash, hash: [0; 32] };
        let serialized = serde_json::to_vec(&payload).unwrap();
        let hash = blake3::hash(&serialized);
        Self { hash: *hash.as_bytes(), ..payload }
    }
}

pub struct AuditLog {
    writer: JsonlWriter,
    last_hash: Mutex<[u8; 32]>,
}

impl AuditLog {
    pub fn append(&self, event: AuditEvent) -> Result<()> {
        let mut last = self.last_hash.lock().unwrap();
        let entry = AuditEntry::new(event, *last);
        self.writer.append(&entry)?;
        *last = entry.hash;
        Ok(())
    }
    
    pub fn verify(&self) -> Result<bool> {
        // 전체 체인 검증: 각 prev_hash가 실제 이전 엔트리의 hash인지
    }
}
```

**검증 목적**: 악의적 수정이나 복구 실패 감지. `myth audit verify` CLI가 호출.

## 마이그레이션 전략

forward-only. 다운그레이드 미지원 (v1→v2 후 v1으로 못 감).

```rust
pub fn apply_migrations(conn: &Connection) -> Result<()> {
    let current_version: u32 = conn.query_row(
        "PRAGMA user_version", [], |r| r.get(0)
    )?;
    
    for (target, sql) in MIGRATIONS.iter() {
        if *target > current_version {
            conn.execute_batch(sql)?;
            // PRAGMA user_version = X 는 sql 안에 포함
            tracing::info!("applied migration to v{}", target);
        }
    }
    Ok(())
}

const MIGRATIONS: &[(u32, &str)] = &[
    (1, include_str!("../migrations/001_initial.sql")),
    // 미래: (2, include_str!("../migrations/002_...sql")),
];
```

## 테스트

```
tests/
├── sqlite_test.rs          # 연결, pragma, 기본 CRUD
├── migration_test.rs       # 빈 DB → v1 적용 검증
├── lesson_crud_test.rs     # LessonStore trait 구현 검증
├── jsonl_concurrent_test.rs  # 여러 프로세스 동시 append 안전성
└── audit_chain_test.rs     # Merkle 체인 tamper 감지
```

임시 디렉토리 (`tempfile` crate)에서 실제 SQLite 생성.

## 관련 결정

- Decision 1 (벡터 저장소): `myth-db`는 스칼라 저장만. 벡터는 `myth-identity`의 in-memory store.
- ARCHITECTURE Contract 5 (SQLite schema forward-only): 이 문서의 migration 전략이 구현.
