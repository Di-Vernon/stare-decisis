# `myth-common` — 기반 타입과 유틸

## 역할

모든 crate가 공유하는 **최하층 기반**. 타입 정의, 에러, 시간, UUID, 로깅, XDG 경로 해석.

**의존**: 외부 crate만. 다른 myth crate 의존 없음.
**의존받음**: 모든 다른 crate (Layer 0).

## Cargo.toml

```toml
[package]
name = "myth-common"
version = "0.1.0"
edition = "2021"
license = "MIT OR Apache-2.0"

[dependencies]
serde = { workspace = true }
serde_json = { workspace = true }
thiserror = { workspace = true }
anyhow = { workspace = true }
chrono = { workspace = true }
uuid = { workspace = true }
tracing = { workspace = true }
tracing-subscriber = { workspace = true }
```

## 모듈 구조

```
crates/myth-common/
└── src/
    ├── lib.rs           # pub use 모든 모듈
    ├── types.rs         # Level, Recurrence, Enforcement, Category, IdentityHash
    ├── error.rs         # MythError
    ├── ids.rs           # LessonId, SessionId, ReminderId (UUID wrapper)
    ├── time.rs          # Timestamp (chrono wrapper)
    ├── paths.rs         # XDG 경로 해석
    └── logging.rs       # tracing 초기화
```

## 주요 타입

### `Level`

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[repr(u8)]
pub enum Level {
    Info = 1,
    Low = 2,
    Medium = 3,
    High = 4,
    Critical = 5,
}

impl Level {
    pub fn label(&self) -> &'static str {
        match self {
            Level::Info => "INFO",
            Level::Low => "LOW",
            Level::Medium => "MEDIUM",
            Level::High => "HIGH",
            Level::Critical => "CRITICAL",
        }
    }
}
```

### `Recurrence`

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[repr(u8)]
pub enum Recurrence {
    I = 1,
    II = 2,
    III = 3,
    IV = 4,
    V = 5,
    VI = 6,
}

impl Recurrence {
    pub fn from_count(count: f64) -> Self {
        match count {
            c if c < 1.0 => Self::I,
            c if c < 2.0 => Self::II,
            c if c < 4.0 => Self::III,
            c if c < 7.0 => Self::IV,
            c if c < 12.0 => Self::V,
            _ => Self::VI,
        }
    }
}
```

### `Enforcement`

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Enforcement {
    Dismiss,
    Note,
    Advisory,
    Caution,
    Warn,
    Strike,
    Seal,
}

impl Enforcement {
    pub fn is_blocking(&self) -> bool {
        matches!(self, Self::Warn | Self::Strike | Self::Seal)
    }
    
    pub fn label_ko(&self) -> &'static str {
        match self {
            Self::Dismiss => "기각",
            Self::Note => "기록",
            Self::Advisory => "권고",
            Self::Caution => "주의",
            Self::Warn => "경고",
            Self::Strike => "차단",
            Self::Seal => "봉인",
        }
    }
}
```

### `Category`

실패·lesson의 분류. 헌법 Part VI.2 정의 5개:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Category {
    Security,       // 보안, 권한, 시크릿
    Correctness,    // 기능 결함, 로직 오류
    Process,        // CI/CD, 워크플로
    DataSafety,     // 데이터 무결성, 손실
    Temporal,       // 시간·버전 불일치 (폐기된 API 등)
}
```

### `IdentityHash`

```rust
pub struct IdentityHash {
    pub tier1_sha1: [u8; 20],        // SHA1 normalize hash
    pub tier2_embedding: Option<[f32; 384]>,  // multilingual-e5-small
    pub tier3_resolved: Option<bool>, // LLM judge 결과
}

impl IdentityHash {
    pub fn tier1_hex(&self) -> String { /* hex 인코딩 */ }
}
```

### `LessonId`, `SessionId`, `ReminderId`

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LessonId(pub Uuid);

impl LessonId {
    pub fn new() -> Self { Self(Uuid::new_v4()) }
    pub fn short(&self) -> String { /* 앞 8자 */ }
}

// SessionId, ReminderId 동일 패턴
```

모두 `Uuid` newtype. 타입 혼동 방지.

### `Timestamp`

```rust
pub type Timestamp = DateTime<Utc>;

pub fn now() -> Timestamp { Utc::now() }
pub fn format_iso(ts: &Timestamp) -> String { /* 2026-04-19T14:23:45Z */ }
```

## 에러

```rust
#[derive(thiserror::Error, Debug)]
pub enum MythError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),
    
    #[error("YAML parse error: {0}")]
    Yaml(#[from] serde_yaml::Error),
    
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    
    #[error("config not found at {path}")]
    ConfigMissing { path: String },
    
    #[error("rule file parse error in {file}: {message}")]
    RuleParse { file: String, message: String },
    
    #[error("hook timeout after {ms}ms")]
    HookTimeout { ms: u64 },
    
    #[error("daemon unavailable: {reason}")]
    DaemonUnavailable { reason: String },
    
    #[error("{0}")]
    Other(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, MythError>;
```

`anyhow::Error`를 감싸서 `?` 연산자 편의성 유지.

## XDG 경로

```rust
pub fn myth_home() -> PathBuf {
    dirs::home_dir()
        .expect("no home dir")
        .join(".myth")
}

pub fn myth_config() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| dirs::home_dir().unwrap().join(".config"))
        .join("myth")
}

pub fn myth_state() -> PathBuf {
    dirs::state_dir()
        .unwrap_or_else(|| dirs::home_dir().unwrap().join(".local").join("state"))
        .join("myth")
}

pub fn myth_runtime() -> PathBuf {
    std::env::var("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let uid = nix::unistd::getuid().as_raw();
            PathBuf::from(format!("/tmp/myth-{}", uid))
        })
        .join("myth")
}

/// 자주 쓰는 특정 경로들
pub fn bedrock_rules_path() -> PathBuf { myth_home().join("bedrock-rules.yaml") }
pub fn foundation_rules_path() -> PathBuf { myth_home().join("foundation-rules.yaml") }
pub fn surface_rules_path() -> PathBuf { myth_home().join("surface-rules.yaml") }
pub fn state_db_path() -> PathBuf { myth_home().join("state.db") }
pub fn vectors_bin_path() -> PathBuf { myth_home().join("vectors.bin") }
pub fn caselog_path() -> PathBuf { myth_home().join("caselog.jsonl") }
pub fn brief_path() -> PathBuf { myth_home().join("brief.md") }
pub fn hook_latency_path() -> PathBuf { myth_state().join("hook-latency.ndjson") }
pub fn embed_socket_path() -> PathBuf { myth_runtime().join("embed.sock") }
```

## 로깅

```rust
pub fn init_logging(binary_name: &str) {
    let filter = std::env::var("MYTH_LOG")
        .unwrap_or_else(|_| "myth=info,warn".to_string());
    
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .with_target(false)
        .compact()
        .init();
    
    tracing::info!(binary = binary_name, "myth starting");
}
```

모든 myth 바이너리는 `main()` 시작 시 호출. stderr 출력 (stdout은 hook protocol용).

## 테스트

```
tests/
├── types_test.rs        # Level/Recurrence/Enforcement 변환 검증
├── paths_test.rs        # XDG 해석
└── error_test.rs
```

의존성 없이 단위 테스트.

## 관련 결정

- Decision 5 (네이밍): Level, Recurrence, Enforcement, Category 모두 확정 이름
- Decision 6 (embed socket): `myth_runtime()` 경로 정의
