# `myth-identity` — 3-Tier Identity 매칭

## 역할

두 실패 이벤트가 **같은 것**인지 판정. Lesson의 `recurrence_count` 증가 여부, 신규 lesson 생성 여부가 여기서 결정된다. **학습 루프의 심장 부품**.

**의존**: `myth-common`, `myth-db`.
**의존받음**: `myth-hooks` (PostToolUseFailure 경로), Python Assessor (간접).
**약한 결합**: `myth-embed` (Unix socket으로만).

## Cargo.toml

```toml
[package]
name = "myth-identity"
version = "0.1.0"
edition = "2021"
license = "MIT OR Apache-2.0"

[dependencies]
myth-common = { path = "../myth-common" }
myth-db = { path = "../myth-db" }

sha1 = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
bincode = { workspace = true }
memmap2 = { workspace = true }
simsimd = { workspace = true }
regex = { workspace = true }
tracing = { workspace = true }
anyhow = { workspace = true }
```

## 모듈 구조

```
crates/myth-identity/
└── src/
    ├── lib.rs                 # Identity + Matcher 공개
    ├── tier1/
    │   ├── mod.rs            # SHA1 매칭
    │   └── normalize.rs      # Aggressive normalize
    ├── tier2/
    │   ├── mod.rs            # Embedding 유사도
    │   └── embed_client.rs   # myth-embed daemon Unix socket 클라
    ├── tier3/
    │   └── mod.rs            # LLM judge (Python subprocess)
    ├── store/
    │   ├── mod.rs            # trait VectorStore
    │   ├── in_memory.rs      # Day-1 기본 구현
    │   ├── sqlite_vec.rs     # Milestone B 스텁
    │   └── usearch.rs        # Milestone B 스텁
    └── matcher.rs             # 3-Tier 통합
```

## Aggressive Text Normalization

Tier 1 SHA1 매칭 전에 **의미 있는 차이만 남기고 지워내는** 정규화. 같은 실수의 우연한 차이(timestamp, UUID, 경로)를 제거.

```rust
pub fn normalize_aggressive(text: &str) -> String {
    let mut s = text.to_string();
    
    // 타임스탬프 → <TS>
    s = TIMESTAMP_RE.replace_all(&s, "<TS>").into();
    
    // UUID → <UUID>
    s = UUID_RE.replace_all(&s, "<UUID>").into();
    
    // Hex >= 6자 → <HEX>
    s = HEX_RE.replace_all(&s, "<HEX>").into();
    
    // 숫자 3자 이상 → <NUM>
    s = NUM_RE.replace_all(&s, "<NUM>").into();
    
    // 경로 /home/user/... → <PATH>
    s = PATH_RE.replace_all(&s, "<PATH>").into();
    
    // 파일명 .log, .tmp → <FILE>
    // 연속 공백 → 단일 공백
    // lowercase
    
    s.to_lowercase().split_whitespace().collect::<Vec<_>>().join(" ")
}
```

**예시**:
```
raw:  "FileNotFoundError: /home/miirr/project/foo/tmp/abc-123.log not found"
norm: "filenotfounderror <path> not found"
```

한 실패 패턴의 **본질적 특징만** 남김.

## Tier 1 — SHA1 매칭

```rust
pub fn tier1_hash(normalized: &str) -> [u8; 20] {
    let mut hasher = sha1::Sha1::new();
    hasher.update(normalized.as_bytes());
    hasher.finalize().into()
}

pub struct Tier1Matcher {
    store: Arc<dyn LessonStore>,
}

impl Tier1Matcher {
    pub fn find(&self, normalized: &str) -> Result<Option<Lesson>> {
        let hash = tier1_hash(normalized);
        self.store.find_by_identity(&hash)
    }
}
```

가장 빠름 (<0.01ms). 완전 일치만 감지.

## Tier 2 — Embedding 유사도

```rust
pub struct Tier2Matcher {
    embed_client: EmbedClient,
    vector_store: Arc<dyn VectorStore>,
}

impl Tier2Matcher {
    pub fn find(&self, normalized: &str) -> Result<Option<(LessonId, f32)>> {
        let embedding = self.embed_client.embed(normalized)?;
        let results = self.vector_store.knn(&embedding, 1)?;
        
        match results.first() {
            Some((id, distance)) => {
                // distance 변환: cosine distance → similarity
                let similarity = 1.0 - distance;
                if similarity >= 0.90 {
                    // auto-merge: 같은 lesson
                    Ok(Some((*id, similarity)))
                } else if similarity >= 0.75 {
                    // 애매: Tier 3 필요
                    Ok(Some((*id, similarity)))
                } else {
                    Ok(None)
                }
            }
            None => Ok(None),
        }
    }
}
```

## `trait VectorStore`

Decision 1 핵심. 여러 구현체를 교체 가능하게.

```rust
pub trait VectorStore: Send + Sync {
    fn upsert(&self, id: LessonId, vec: &[f32; 384]) -> Result<()>;
    fn knn(&self, query: &[f32; 384], k: usize) -> Result<Vec<(LessonId, f32)>>;
    fn delete(&self, id: LessonId) -> Result<()>;
    fn len(&self) -> usize;
    fn integrity_check(&self) -> Result<IntegrityReport>;
}
```

### Day-1 구현: `InMemoryStore`

```rust
pub struct InMemoryStore {
    // vectors.bin을 mmap으로 매핑
    mmap: Arc<RwLock<Mmap>>,
    // lesson_id ↔ row index 매핑은 SQLite의 별도 테이블
    index: Arc<RwLock<HashMap<LessonId, usize>>>,
    // generation 번호 (SQLite의 vectors_generation과 대조)
    generation: Arc<RwLock<u64>>,
}

impl InMemoryStore {
    pub fn open() -> Result<Self> {
        let path = myth_common::vectors_bin_path();
        let file = File::open(&path)?;
        let mmap = unsafe { Mmap::map(&file)? };
        
        // SQLite의 vector_metadata 테이블에서 index + generation 로드
        let (index, generation) = load_metadata()?;
        
        Ok(Self {
            mmap: Arc::new(RwLock::new(mmap)),
            index: Arc::new(RwLock::new(index)),
            generation: Arc::new(RwLock::new(generation)),
        })
    }
}

impl VectorStore for InMemoryStore {
    fn knn(&self, query: &[f32; 384], k: usize) -> Result<Vec<(LessonId, f32)>> {
        let mmap = self.mmap.read().unwrap();
        let vectors = mmap_to_vectors(&mmap);  // &[[f32; 384]]
        let index = self.index.read().unwrap();
        
        // SIMD cosine distance (정규화된 벡터 → 내적)
        let mut results: Vec<(LessonId, f32)> = vectors.iter().enumerate()
            .map(|(row_idx, v)| {
                let id = lookup_id_by_row(&index, row_idx);
                let distance = simsimd::cosine_distance_f32(query, v);
                (id, distance)
            })
            .collect();
        
        results.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        results.truncate(k);
        Ok(results)
    }
    
    fn upsert(&self, id: LessonId, vec: &[f32; 384]) -> Result<()> {
        // 1. 새 vectors.bin 작성 (기존 + 신규)
        // 2. atomic rename
        // 3. SQLite 트랜잭션: vector_metadata 업데이트, generation 증가
        // 4. mmap 재로드
        
        let mut index = self.index.write().unwrap();
        let mut generation = self.generation.write().unwrap();
        
        let new_gen = *generation + 1;
        let new_path = myth_common::vectors_bin_path().with_extension("bin.new");
        
        // 전체 재작성
        let mut new_vectors: Vec<[f32; 384]> = current_vectors(&self.mmap.read().unwrap())?.to_vec();
        if let Some(&existing_row) = index.get(&id) {
            new_vectors[existing_row] = *vec;
        } else {
            index.insert(id, new_vectors.len());
            new_vectors.push(*vec);
        }
        
        write_vectors_file(&new_path, new_gen, &new_vectors)?;
        std::fs::rename(&new_path, myth_common::vectors_bin_path())?;
        
        update_metadata(&index, new_gen)?;
        *generation = new_gen;
        
        // mmap 재로드
        drop(self.mmap.read().unwrap());  // 기존 lock 해제
        let file = File::open(myth_common::vectors_bin_path())?;
        let new_mmap = unsafe { Mmap::map(&file)? };
        *self.mmap.write().unwrap() = new_mmap;
        
        Ok(())
    }
}
```

**vectors.bin 파일 포맷**:

```
Magic:      4 bytes = 0x4D594556  ("MYEV")
Version:    2 bytes = 1
Dim:        2 bytes = 384
Count:      4 bytes
Generation: 8 bytes
Reserved:   8 bytes (padding to 32)
Vectors:    count * 384 * 4 bytes (float32, row-major)
```

헤더 검증 실패 → `integrity_check()` 반환 에러 → 사용자에게 "재임베딩 필요" 알림.

### `integrity_check`

```rust
fn integrity_check(&self) -> Result<IntegrityReport> {
    let mmap = self.mmap.read().unwrap();
    
    // 1. Magic 검증
    // 2. Version 검증 (1 기대)
    // 3. Dim == 384
    // 4. Count와 index.len() 일치
    // 5. Generation과 SQLite vector_metadata.generation 일치
    // 6. 모든 벡터의 norm이 0.95~1.05 범위 (정규화 확인)
    
    Ok(IntegrityReport {
        total_vectors: count,
        matches_index: ...,
        matches_db: ...,
        norm_anomalies: ...,
    })
}
```

Milestone B까지 in-memory가 유효. Milestone B 발동 시 `sqlite_vec.rs` 또는 `usearch.rs` 활성.

## Tier 3 — LLM judge

```rust
pub struct Tier3Matcher {
    // Python subprocess 호출 (Anthropic SDK 간접)
}

impl Tier3Matcher {
    pub fn judge(&self, text_a: &str, text_b: &str) -> Result<bool> {
        // Milestone A 이후 활성
        // 현재는 no-op 또는 "false" 반환
        if !is_tier3_enabled() {
            return Ok(false);
        }
        
        // Python 호출
        let output = std::process::Command::new("python3")
            .args(["-m", "myth_py.identity.judge"])
            .stdin(...)
            .output()?;
        
        parse_judge_response(&output.stdout)
    }
}
```

## 통합 `Matcher`

```rust
pub struct IdentityMatcher {
    tier1: Tier1Matcher,
    tier2: Tier2Matcher,
    tier3: Tier3Matcher,
    store: Arc<dyn LessonStore>,
}

impl IdentityMatcher {
    pub fn find_or_create(&self, raw_text: &str) -> Result<Lesson> {
        let normalized = normalize_aggressive(raw_text);
        
        // Tier 1: SHA1
        if let Some(lesson) = self.tier1.find(&normalized)? {
            self.store.increment_recurrence(lesson.id)?;
            return Ok(lesson);
        }
        
        // Tier 2: Embedding
        match self.tier2.find(&normalized)? {
            Some((id, sim)) if sim >= 0.90 => {
                // auto-merge
                let lesson = self.store.get(id)?.unwrap();
                self.store.increment_recurrence(id)?;
                return Ok(lesson);
            }
            Some((id, sim)) if sim >= 0.75 => {
                // Tier 3 judge
                let candidate = self.store.get(id)?.unwrap();
                if self.tier3.judge(&normalized, &candidate.description)? {
                    self.store.increment_recurrence(id)?;
                    return Ok(candidate);
                }
                // 새 lesson 생성
            }
            _ => {}
        }
        
        // 신규 lesson 생성
        let lesson = Lesson::new(&normalized, ...);
        self.store.insert(&lesson)?;
        Ok(lesson)
    }
}
```

## Lapse 계산

Decision: Quiescence → Lapse 재명명 반영.

```rust
pub fn compute_lapse_score(lesson: &Lesson, now: Timestamp) -> f64 {
    let idle_days = (now - lesson.last_seen).num_days() as f64;
    let missed_hooks = lesson.missed_hook_count as f64;  // 관련 도구가 사용됐지만 매칭 안 된 횟수
    
    missed_hooks * 1.0 + idle_days * 10.0
}

pub fn lapse_threshold(level: Level) -> Option<f64> {
    match level {
        Level::Info | Level::Low => Some(50.0),
        Level::Medium | Level::High => Some(200.0),
        Level::Critical => None,  // Bedrock 면제
    }
}
```

Observer가 주간 실행 시 모든 active lesson의 `lapse_score`를 재계산, 임계 초과 시 `status = 'lapsed'`로 전환.

## 성능 예산

| 작업 | 예상 |
|---|---|
| `normalize_aggressive` (1KB 텍스트) | ~50μs |
| `tier1_hash` + DB 조회 | ~500μs |
| Tier 2 embedding (daemon 경유, 50K 벡터 KNN) | ~15ms + 34ms = ~50ms |
| Tier 3 (Python subprocess, LLM) | ~1000ms+ |

**Hook 임계 경로에는 Tier 1만 사용**. Tier 2/3는 PostToolUseFailure 비동기 경로에서만.

## 테스트

```
tests/
├── normalize_test.rs       # 다양한 입력 정규화 예시
├── tier1_test.rs           # SHA1 매칭
├── in_memory_store_test.rs # 100개 벡터 CRUD + KNN
├── integrity_test.rs       # 손상된 vectors.bin 감지
└── lapse_test.rs           # Lapse score 계산
```

실제 `fastembed-rs` 모델 호출은 integration test에서만 (느림).

## 관련 결정

- Decision 1: in-memory store + trait VectorStore
- Decision 2: multilingual-e5-small (384-dim 고정)
- Decision 6: myth-embed daemon 프로토콜 (`embed_client`가 클라이언트 역할)
- Decision 3: Tier 3 활성 플래그 (`enable_tier3`)
- 카테고리 6: Quiescence → Lapse (여기 `compute_lapse_score` 구현)
