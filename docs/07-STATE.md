# myth — 상태 저장소

## 역할

myth가 **장기 보존하는 모든 데이터**의 저장 구조를 박제한다. SQLite 스키마, JSONL 포맷, 벡터 바이너리 레이아웃, Merkle audit chain의 상세 스펙.

이 문서의 정의는 **`ARCHITECTURE.md` Contract 5 (SQLite forward-only)**의 구체 구현.

## 저장소 분류

myth는 **4종류**의 저장소를 사용한다. 각각 성질이 다르다.

| 저장소 | 파일 | 성질 | 용도 |
|---|---|---|---|
| **SQLite** | `state.db` | 트랜잭션, 쿼리, 인덱스 | 메타데이터, 쿼리 |
| **JSONL** | `caselog.jsonl`, `lesson-state.jsonl`, `audit.jsonl`, `hook-latency.ndjson` | append-only | 이벤트 로그 |
| **벡터 바이너리** | `vectors.bin` | mmap, 수시 재기록 | 임베딩 |
| **설정 YAML** | `*.yaml` (rules, grid, config) | 사람이 편집 | 규칙, 설정 |

## SQLite — `state.db`

### 물리 경로

```
~/.myth/state.db           # 주 DB
~/.myth/state.db-wal       # WAL (write-ahead log, SQLite 자동 생성)
~/.myth/state.db-shm       # Shared memory (SQLite 자동)
```

### PRAGMA 설정

```sql
PRAGMA journal_mode = WAL;         -- 동시 read + 단일 write
PRAGMA synchronous = NORMAL;       -- fsync 균형
PRAGMA busy_timeout = 5000;        -- 5초 대기
PRAGMA foreign_keys = ON;          -- FK 제약 활성
PRAGMA mmap_size = 268435456;      -- 256MB mmap
PRAGMA user_version = 1;           -- 스키마 버전
```

WAL 모드는 **여러 reader + 하나의 writer** 패턴에 최적. myth는 hook(read-heavy) + Observer(write-occasional) 구조에 정합.

### 스키마 v1 전체

`migrations/001_initial.sql`:

```sql
PRAGMA user_version = 1;

-- =============================
-- Lessons (핵심 테이블)
-- =============================
CREATE TABLE lessons (
    id BLOB PRIMARY KEY,                    -- UUID v4 bytes (16)
    
    -- Identity
    identity_hash_tier1 BLOB NOT NULL,      -- SHA1 20 bytes
    
    -- Classification
    level INTEGER NOT NULL CHECK(level BETWEEN 1 AND 5),
    category TEXT NOT NULL CHECK(category IN 
        ('security', 'correctness', 'process', 'data_safety', 'temporal')),
    
    -- Recurrence
    recurrence_count REAL NOT NULL DEFAULT 0,
    missed_hook_count INTEGER NOT NULL DEFAULT 0,
    
    -- Timestamps (unix seconds)
    first_seen INTEGER NOT NULL,
    last_seen INTEGER NOT NULL,
    
    -- Lapse
    lapse_score REAL NOT NULL DEFAULT 0,
    
    -- Appeals
    appeals INTEGER NOT NULL DEFAULT 0,
    
    -- Status
    status TEXT NOT NULL DEFAULT 'active' CHECK(status IN
        ('active', 'lapsed', 'archived', 'superseded')),
    
    -- Content
    description TEXT NOT NULL,
    rationale TEXT NOT NULL,
    
    -- Metadata (확장용 JSON)
    meta_json TEXT
);

CREATE INDEX idx_lessons_identity ON lessons(identity_hash_tier1);
CREATE INDEX idx_lessons_level ON lessons(level);
CREATE INDEX idx_lessons_last_seen ON lessons(last_seen DESC);
CREATE INDEX idx_lessons_status ON lessons(status);
CREATE INDEX idx_lessons_category ON lessons(category);

-- =============================
-- Vector metadata (vectors.bin 대응)
-- =============================
CREATE TABLE vector_metadata (
    lesson_id BLOB PRIMARY KEY,
    row_index INTEGER NOT NULL,               -- vectors.bin에서의 행 번호
    generation INTEGER NOT NULL,              -- vectors.bin의 current generation
    created_ts INTEGER NOT NULL,
    FOREIGN KEY(lesson_id) REFERENCES lessons(id) ON DELETE CASCADE
);

CREATE INDEX idx_vec_generation ON vector_metadata(generation);

-- Generation counter (단일 행)
CREATE TABLE vector_generation (
    id INTEGER PRIMARY KEY CHECK(id = 1),    -- 단일 행 강제
    current_generation INTEGER NOT NULL,
    last_updated INTEGER NOT NULL
);
INSERT INTO vector_generation (id, current_generation, last_updated) 
    VALUES (1, 0, strftime('%s', 'now'));

-- =============================
-- Hook events (latency + 실행 이력)
-- =============================
CREATE TABLE hook_events (
    id BLOB PRIMARY KEY,                      -- UUID v4 bytes
    session_id BLOB NOT NULL,
    event_type TEXT NOT NULL CHECK(event_type IN (
        'session_start', 'user_prompt', 
        'pre_tool', 'post_tool', 'post_tool_failure',
        'stop'
    )),
    tool_name TEXT,
    ts INTEGER NOT NULL,                      -- unix ms
    latency_ms REAL NOT NULL,
    verdict TEXT NOT NULL CHECK(verdict IN ('allow', 'deny', 'ask')),
    lesson_id BLOB,
    FOREIGN KEY(lesson_id) REFERENCES lessons(id)
);

CREATE INDEX idx_events_session ON hook_events(session_id);
CREATE INDEX idx_events_ts ON hook_events(ts DESC);
CREATE INDEX idx_events_type ON hook_events(event_type, ts DESC);

-- =============================
-- Appeals
-- =============================
CREATE TABLE appeal_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    lesson_id BLOB NOT NULL,
    appeal_type TEXT NOT NULL CHECK(appeal_type IN ('appeal', 'retrial')),
    ts INTEGER NOT NULL,
    result TEXT NOT NULL CHECK(result IN ('pending', 'granted', 'denied', 'withdrawn')),
    rationale TEXT,
    resolved_ts INTEGER,
    resolver TEXT,                            -- 'observer', 'admin'
    FOREIGN KEY(lesson_id) REFERENCES lessons(id)
);

CREATE INDEX idx_appeals_lesson ON appeal_history(lesson_id);
CREATE INDEX idx_appeals_status ON appeal_history(result);

-- =============================
-- Grid overrides
-- =============================
CREATE TABLE grid_overrides (
    level INTEGER NOT NULL CHECK(level BETWEEN 1 AND 5),
    recurrence INTEGER NOT NULL CHECK(recurrence BETWEEN 1 AND 6),
    enforcement TEXT NOT NULL CHECK(enforcement IN
        ('dismiss', 'note', 'advisory', 'caution', 'warn', 'strike', 'seal')),
    source TEXT NOT NULL CHECK(source IN ('default', 'admin', 'observer_suggested')),
    approved_ts INTEGER,
    rationale TEXT,
    PRIMARY KEY(level, recurrence)
);

-- =============================
-- Sessions (관찰 목적)
-- =============================
CREATE TABLE sessions (
    id BLOB PRIMARY KEY,
    started_ts INTEGER NOT NULL,
    ended_ts INTEGER,
    project_path TEXT,
    event_count INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX idx_sessions_started ON sessions(started_ts DESC);
```

### 추가 테이블 (Milestone 대비)

```sql
-- migrations/002_milestone_tier3.sql (Milestone A 활성 시 적용)
CREATE TABLE tier3_dispatches (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    reminder_id TEXT NOT NULL,
    ts INTEGER NOT NULL,
    model TEXT NOT NULL,
    tokens_in INTEGER NOT NULL,
    tokens_out INTEGER NOT NULL,
    cost_usd REAL NOT NULL,
    success INTEGER NOT NULL DEFAULT 1
);

-- migrations/003_milestone_c.sql (Gavel daemon 전환 시)
CREATE TABLE gavel_daemon_stats (
    ts INTEGER NOT NULL,
    pid INTEGER NOT NULL,
    p50_ms REAL NOT NULL,
    p99_ms REAL NOT NULL,
    requests INTEGER NOT NULL,
    PRIMARY KEY(ts, pid)
);
```

**forward-only**: 이전 스키마는 절대 변경하지 않고 새 테이블/컬럼만 추가.

### 주요 쿼리 예시

**활성 lesson 상위 10개 (recurrence 순)**:
```sql
SELECT id, level, category, recurrence_count, rationale
FROM lessons
WHERE status = 'active'
ORDER BY recurrence_count DESC
LIMIT 10;
```

**최근 7일 P99 hook latency (pre_tool)**:
```sql
SELECT latency_ms
FROM hook_events
WHERE event_type = 'pre_tool'
  AND ts > strftime('%s', 'now', '-7 days') * 1000
ORDER BY latency_ms;
-- percentile은 application layer에서 계산
```

**Appeal 대기 목록**:
```sql
SELECT a.*, l.rationale, l.level
FROM appeal_history a
JOIN lessons l ON l.id = a.lesson_id
WHERE a.result = 'pending'
ORDER BY a.ts;
```

### 백업

```bash
# 파일 레벨 백업 (DB 잠긴 상태에서도 안전)
sqlite3 ~/.myth/state.db ".backup '/tmp/state.db.backup-$(date +%Y%m%d)'"
```

`myth doctor`가 `--backup` 옵션 제공:

```rust
pub fn backup_state_db() -> Result<PathBuf> {
    let src = myth_common::state_db_path();
    let backup_dir = myth_common::myth_home().join("backups");
    std::fs::create_dir_all(&backup_dir)?;
    
    let dst = backup_dir.join(format!(
        "state-{}.db",
        chrono::Utc::now().format("%Y%m%d-%H%M%S")
    ));
    
    // SQLite 백업 API (온라인)
    let src_conn = rusqlite::Connection::open(&src)?;
    let dst_conn = rusqlite::Connection::open(&dst)?;
    let backup = rusqlite::backup::Backup::new(&src_conn, &dst_conn)?;
    backup.run_to_completion(5, Duration::from_millis(250), None)?;
    
    Ok(dst)
}
```

## JSONL — append-only 로그

4개 파일. 모두 UTF-8, 한 줄당 하나의 JSON 객체, 끝에 개행 `\n`.

### `~/.myth/caselog.jsonl`

**내용**: 모든 실패 이벤트의 원본.

```json
{"ts":"2026-04-19T14:23:45.123Z","session_id":"abc...","event":"post_tool_failure","tool_name":"Bash","tool_input":{"command":"..."},"error":"ENOENT","exit_code":1,"classified_level":3,"classified_category":"correctness","lesson_id":"L-...","reminder_id":null,"bedrock_match":null}
{"ts":"2026-04-19T14:24:12.456Z",...}
```

**write**: Rust `myth-db::JsonlWriter::append()` (fcntl flock).
**read**: Python Observer `analyzer.py`, Rust TUI `caselog.rs`.

### `~/.myth/lesson-state.jsonl`

**내용**: lesson의 상태 변화 시계열.

```json
{"ts":"2026-04-19T14:23:45Z","event":"pending_reflection","reminder_id":"rid-...","session_id":"...","turn_n":5,"tool_name":"Bash"}
{"ts":"2026-04-19T14:23:52Z","event":"compliant","reminder_id":"rid-..."}
{"ts":"2026-04-19T14:24:00Z","event":"lesson_created","lesson_id":"L-...","level":3,"category":"correctness"}
{"ts":"2026-04-19T14:30:00Z","event":"recurrence_increment","lesson_id":"L-...","new_count":2.5}
{"ts":"2026-04-25T09:00:00Z","event":"lapsed","lesson_id":"L-...","lapse_score":52.1}
```

**이벤트 타입**:
- `pending_reflection`: Tier 1 발동 기록
- `compliant` / `missed`: 다음 turn 준수 여부
- `lesson_created`
- `recurrence_increment`
- `appeal_filed` / `appeal_granted` / `appeal_denied`
- `split` / `merge`
- `lapsed` / `revived` / `archived`
- `superseded_by`: 다른 lesson으로 대체됨

**read**: Observer 분석, Appeal 검증, audit.

### `~/.myth/audit.jsonl`

**내용**: tamper-evident Merkle chain. 중요 결정 (Admin 명령, 규칙 변경, appeal granted 등).

```json
{"ts":"2026-04-19T00:00:00Z","event":{"type":"genesis"},"prev_hash":"0000...","hash":"abc123..."}
{"ts":"2026-04-19T12:00:00Z","event":{"type":"bedrock_rule_modified","rule_id":"R1-A","user":"jeffrey"},"prev_hash":"abc123...","hash":"def456..."}
{"ts":"2026-04-19T14:00:00Z","event":{"type":"appeal_granted","lesson_id":"L-...","resolver":"observer"},"prev_hash":"def456...","hash":"ghi789..."}
```

각 엔트리는:
- `ts`: 시간
- `event`: 이벤트 객체 (타입별 필드)
- `prev_hash`: 직전 엔트리의 `hash`
- `hash`: 이 엔트리의 blake3 해시 (자기 `hash` 필드 제외하고 계산)

**검증**: `myth audit verify` 명령.

### `~/.local/state/myth/hook-latency.ndjson`

**내용**: 모든 hook 호출의 latency.

```json
{"ts":"2026-04-19T14:23:45.123Z","event":"pre_tool","latency_ms":3.2,"result":"allow","session_id":"..."}
{"ts":"2026-04-19T14:23:46.789Z","event":"post_tool","latency_ms":0.8,"result":"logged","session_id":"..."}
```

**rotation**: logrotate (weekly, 4주 보관).
**주간 집계**: Observer가 P50/P99 계산해 brief.md에 반영.

## 벡터 바이너리 — `~/.myth/vectors.bin`

### 파일 레이아웃

```
Offset  Size   Field
------- -----  -----------------
0x00    4      Magic: 0x4D 0x59 0x45 0x56  ("MYEV")
0x04    2      Version: u16 LE = 1
0x06    2      Dimension: u16 LE = 384
0x08    4      Count: u32 LE
0x0C    8      Generation: u64 LE
0x14    12     Reserved (zero)
0x20    ...    Vector data: Count * 384 * 4 bytes (f32 LE, row-major)
```

### 무결성 검증

```rust
fn verify_vectors_file(path: &Path) -> Result<VectorHeader> {
    let file = File::open(path)?;
    let mmap = unsafe { Mmap::map(&file)? };
    
    if mmap.len() < 0x20 {
        return Err(anyhow!("file too small"));
    }
    
    let magic = &mmap[0..4];
    if magic != b"MYEV" {
        return Err(anyhow!("invalid magic"));
    }
    
    let version = u16::from_le_bytes([mmap[4], mmap[5]]);
    if version != 1 {
        return Err(anyhow!("unsupported version: {}", version));
    }
    
    let dim = u16::from_le_bytes([mmap[6], mmap[7]]);
    if dim != 384 {
        return Err(anyhow!("dimension mismatch: {}", dim));
    }
    
    let count = u32::from_le_bytes([mmap[8], mmap[9], mmap[10], mmap[11]]) as usize;
    let expected_size = 0x20 + count * 384 * 4;
    if mmap.len() != expected_size {
        return Err(anyhow!("size mismatch: {} vs {}", mmap.len(), expected_size));
    }
    
    let generation = u64::from_le_bytes(mmap[12..20].try_into().unwrap());
    
    // Generation 검증 (DB와 대조)
    let db_gen = query_generation_from_db()?;
    if generation != db_gen {
        return Err(anyhow!("generation mismatch: file {} vs db {}", generation, db_gen));
    }
    
    Ok(VectorHeader { count, generation })
}
```

### 재기록 전략

**atomic rename**:

```rust
fn rewrite_vectors(new_vectors: &[[f32; 384]]) -> Result<()> {
    let path = myth_common::vectors_bin_path();
    let tmp = path.with_extension("bin.tmp");
    
    let generation = bump_generation_in_db()?;
    
    // 1. 임시 파일에 작성
    let mut writer = BufWriter::new(File::create(&tmp)?);
    writer.write_all(b"MYEV")?;
    writer.write_all(&1u16.to_le_bytes())?;
    writer.write_all(&384u16.to_le_bytes())?;
    writer.write_all(&(new_vectors.len() as u32).to_le_bytes())?;
    writer.write_all(&generation.to_le_bytes())?;
    writer.write_all(&[0u8; 12])?;
    for v in new_vectors {
        for f in v.iter() {
            writer.write_all(&f.to_le_bytes())?;
        }
    }
    writer.flush()?;
    drop(writer);
    
    // 2. fsync
    let f = File::open(&tmp)?;
    f.sync_all()?;
    
    // 3. atomic rename
    std::fs::rename(&tmp, &path)?;
    
    Ok(())
}
```

**빈도**: lesson 생성/업데이트마다 재기록. 수십 KB~수 MB 크기라 허용 가능. Milestone B에서 sqlite-vec/usearch 전환 시 해결.

## 설정 YAML 파일

### `bedrock-rules.yaml` / `foundation-rules.yaml` / `surface-rules.yaml`

포맷:
```yaml
version: 1
item: rm_rf_unsandboxed
description: "Unsandboxed rm -rf on production/home path"
rules:
  - id: R1-A
    pattern: "(?x) ..."
    likelihood: HIGH
    source: "gitleaks v8.x (MIT)"
    level: 5
    tests:
      positive: ["rm -Rf ~", ...]
      negative: ["rm -rf /tmp/test", ...]
```

**로딩**: `myth-gavel::rules::bedrock.rs`가 serde_yaml 파싱.
**재로딩**: myth 설치 시 또는 config change 감지 (Day-1은 재시작 요구).

### `grid.yaml`

```yaml
version: 1
matrix:
  "1-I":   dismiss
  "1-II":  dismiss
  "1-III": note
  "1-IV":  note
  "1-V":   advisory
  "1-VI":  advisory
  "2-I":   note
  ...
  "5-VI":  strike
overrides:
  - level: 3
    recurrence: 3
    enforcement: warn
    source: "observer_suggested"
    approved_ts: "2026-04-21"
    rationale: "Repeated heredoc quoting issues escalate to warn at III"
```

### `~/.config/myth/config.yaml`

(이미 `03-DIRECTORY.md`에서 상세 예시)

## 저장소 간 일관성 유지

SQLite와 JSONL, vectors.bin이 **한 lesson에 대해 일관되어야** 한다.

### Lesson 생성 시 순서

```
1. vectors.bin 재기록 (atomic rename) + generation++
2. vector_metadata 행 삽입 (SQLite 트랜잭션, generation 포함)
3. lessons 행 삽입 (같은 트랜잭션)
4. lesson-state.jsonl: lesson_created 이벤트 append
5. audit.jsonl: 필요 시 append (level 4~5만)
```

**실패 복구**:
- step 1 후 step 2 실패 → vectors.bin에 "orphan 벡터" → 다음 Observer 실행 시 cleanup
- step 3 후 step 4 실패 → 로그에만 없음. 영향 없음 (lesson은 존재)

**무결성 검사** (`myth doctor --integrity`):
- lessons 전체 → vector_metadata에 해당 row 있는지
- vector_metadata의 generation과 vectors.bin의 generation 일치
- caselog.jsonl의 lesson_id가 lessons에 존재
- audit.jsonl hash chain 유효

## 복구 전략

### SQLite 손상

1. startup 시 `PRAGMA integrity_check`
2. 실패 감지 → `state.db.corrupted-{ts}`로 이동
3. 빈 DB 초기화 + 마이그레이션 적용
4. `caselog.jsonl` 전수 읽기 → lesson 재구성
5. `vectors.bin` 재생성 (Tier 2 임베딩은 재호출 필요)

**caselog는 primary source**, state.db는 derived.

### vectors.bin 손상

1. `verify_vectors_file` 실패 시
2. 파일 이동 → `vectors.bin.corrupted-{ts}`
3. `lessons`에서 모든 active lesson 읽기
4. 각 lesson을 `description`로 임베딩 재계산
5. 새 vectors.bin 작성, generation 1부터 재시작

비용: N 개 lesson × embed latency (~10ms/개). 수분 걸릴 수 있음.

### audit.jsonl 체인 단절

`myth audit verify`가 hash mismatch 발견 → stderr에 어느 엔트리부터 불일치인지 보고. **자동 복구 없음** — 사용자 판단 필요.

## 디스크 사용량 예상

| 저장소 | 1000 lesson 기준 | 1년 가정 |
|---|---|---|
| state.db (lessons + metadata) | ~2 MB | ~20 MB |
| vectors.bin (1000 × 384 × 4) | ~1.5 MB | ~15 MB |
| caselog.jsonl | ~50 MB | ~500 MB |
| lesson-state.jsonl | ~10 MB | ~100 MB |
| audit.jsonl | ~1 MB | ~12 MB |
| hook-latency.ndjson (rotate) | ~5 MB | ~50 MB (rotated) |
| **합계** | **~70 MB** | **~700 MB** |

1년 후 gigabyte 수준. 허용 가능. logrotate + archive 디렉토리 활용.

## 관련 결정

- ARCHITECTURE Contract 5: SQLite forward-only 마이그레이션
- ARCHITECTURE Contract 6: config 경로
- Decision 1 (벡터 저장소): `vector_metadata` 테이블과 vectors.bin 파일 포맷
- 카테고리 7 (문서 파일): caselog.jsonl, brief.md, audit.jsonl 이름 확정
- 카테고리 6 (Lapse): lesson.lapse_score 필드, lesson-state.jsonl의 lapsed 이벤트

---

## Wave 7 drift sync (Wave 8 Task 8.4)

### Grid sub-1b — templates/grid.yaml 런타임 미로드

`templates/grid.yaml`은 `install.rs`가 `~/.myth/grid.yaml`로 복사하지만
**런타임 reload 없음**. `Grid::load()` (rust/crates/myth-gavel/src/grid/
mod.rs)는 DB `grid_overrides` 테이블만 조회하고, 기본값은 `default.rs`의
`DEFAULT_MATRIX` 상수를 사용한다.

**결과**: 사용자가 `~/.myth/grid.yaml`을 수동 편집해도 실행 시 반영 안 됨.

**대응 (Day-1)**: `templates/grid.yaml` 헤더 주석에 경고 박제. 실제 셀
enforcement 변경은 `grid_overrides` 테이블에 INSERT 필요 (Admin 승인
경로). Observer 주간 리포트가 이 테이블 수정을 제안할 수 있음 (Article
7 Sovereignty — Admin 승인 후 Observer 반영).

**Milestone C**: Gavel daemon 전환 시 grid.yaml reload 로직 추가 검토.
daemon이 SIGHUP 수신 시 `Grid::load` 재호출로 yaml 반영 가능한 형태로
설계.

### Schema v1 — lesson split/merge relations (Option B)

Wave 8 Task 8.1에서 lesson split/merge 실구현 시 schema v1 유지 (migration
002 추가 없음). Parent ↔ Children 관계는 `lessons.meta_json` (arbitrary
JSON) 필드에 저장:

```json
// Parent (superseded)
{"split_to": ["<child_uuid_1>", "<child_uuid_2>"], "split_reason": "..."}

// Child (active)
{"split_from": "<parent_uuid>", "split_reason": "...", "split_part": 1}

// Merge source (superseded)
{"merged_into": "<new_uuid>", "merge_reason": "..."}

// Merge result (active)
{"merged_from": ["<src1_uuid>", "<src2_uuid>"], "merge_reason": "..."}
```

**인덱스 부재**: "find all children of X" 조회는 전체 meta_json 스캔 필요.
Day-1 split/merge 호출 빈도 낮음 (observer 주간 + 명시적 appeal 반영만)이라
JSON 파싱 비용 수용. Milestone C에서 `parent_lesson_id BLOB` 인덱스 필드
추가 여부 재검토 (Gavel daemon SQLite 액세스 패턴 일괄 재설계).

### Migration cold-start race (carry-forward 5)

`migration.rs`의 `user_version` 체크 + `CREATE TABLE`이 트랜잭션 밖. N
서브프로세스가 최초 `Database::open`을 동시 호출 시 CREATE 충돌 가능.
Wave 7 Task 7.6 warm-up 우회로 테스트 통과. 근본 수정은 Milestone C
연기 (Gavel daemon 전환 시 SQLite 접근 재설계와 묶음).
