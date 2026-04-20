# myth — Rust Crates Overview

이 문서는 `~/myth/rust/` 워크스페이스에 포함된 **10개 crate의 관계도**와 **각 crate의 책임 요약**이다. 각 crate의 상세는 `04-CRATES/01~10.md`에 개별 파일로 있다.

## 1. 워크스페이스 구조

```
~/myth/rust/
├── Cargo.toml           # workspace = { members = ["crates/*"] }
├── Cargo.lock
├── .cargo/config.toml   # rustflags, linker
└── crates/
    ├── myth-common/      # 공통 타입·유틸
    ├── myth-db/          # SQLite + JSONL 추상
    ├── myth-gavel/       # PreToolUse 판정 (The Gavel)
    ├── myth-identity/    # 3-Tier identity + vector store
    ├── myth-hooks/       # hook 바이너리 (6개 bin target)
    ├── myth-embed/       # 임베딩 daemon (client/server 통합 bin)
    ├── myth-orchestrator/ # Claude Squad / tmux / worktree 병렬 실행
    ├── myth-runtime/     # Claude Code subprocess 래핑
    ├── myth-ui/          # ratatui TUI
    └── myth-cli/         # myth 주 CLI (bin target)
```

**workspace 단일 `Cargo.lock`**. 의존성 버전 통일.

## 2. 의존성 계보 (Dependency Layers)

crate들은 **레이어 구조**로 의존한다. 하위 레이어는 상위를 모른다. 순환 의존 없음.

```
Layer 5 (최상위): myth-cli
         ↓ 의존
Layer 4:          myth-ui ← myth-runtime ← myth-orchestrator
         ↓
Layer 3:          myth-hooks ← myth-embed
         ↓
Layer 2:          myth-gavel ← myth-identity
         ↓
Layer 1:          myth-db
         ↓
Layer 0 (기반):   myth-common
```

화살표는 "**왼쪽이 오른쪽에 의존**" 의미. 즉 `myth-cli`는 모든 crate를 사용할 수 있지만, `myth-common`은 다른 어느 crate도 의존하지 않는다.

### 주요 관계 요약

- `myth-common`: 모든 crate가 의존. 타입·에러·유틸.
- `myth-db`: 하위 2개 layer. state.db 추상.
- `myth-identity`: `myth-db` + `myth-common`. 임베딩 호출은 `myth-embed` 프로토콜을 통해 (직접 의존 아님).
- `myth-gavel`: `myth-db` + `myth-common`. 정규식 컴파일 + Grid lookup.
- `myth-hooks`: 위 모두 조합. hook 바이너리 6개 = 6 bin target.
- `myth-embed`: 독립적. `myth-common`만 의존. IPC 프로토콜로 다른 crate와 통신.
- `myth-orchestrator`: 기존 `harness-orchestrator/lib/` 포팅 + Rust 래핑.
- `myth-runtime`: Claude Code subprocess 관리.
- `myth-ui`: ratatui + syntect. `myth-runtime` 관찰.
- `myth-cli`: `myth` 바이너리. 모든 서브커맨드 dispatch.

## 3. 각 crate 요약

### Layer 0 — `myth-common`

**역할**: 모든 crate가 공유하는 기반.

- 타입: `LessonId`, `Level`, `Recurrence`, `Category`, `Enforcement`, `IdentityHash`
- 에러: `MythError` (thiserror)
- 시간: `chrono` wrapper
- UUID 생성
- 로깅 초기화 (`tracing`)
- 경로 helper (XDG 표준)

**상세**: `04-CRATES/01-myth-common.md`

### Layer 1 — `myth-db`

**역할**: 영속 저장소 추상.

- SQLite 연결 관리 (`rusqlite`, bundled)
- WAL 모드 설정
- PRAGMA user_version 마이그레이션
- JSONL append-only writer (caselog, lesson-state, audit)
- Merkle audit chain (blake3)
- Trait 정의: `LessonStore`, `AuditLog`

**상세**: `04-CRATES/02-myth-db.md`

### Layer 2 — `myth-gavel`

**역할**: The Gavel (PreToolUse 판정자).

- Bedrock/Foundation/Surface rule YAML 로더
- 정규식 컴파일 (`regex-automata` + `include_bytes!`)
- Grid lookup (Level × Recurrence → Enforcement)
- Fatigue tracker (session당 advisory/caution/warn 상한)
- 판정 결과 → Hook JSON 직렬화

**bin 없음** (라이브러리만). `myth-hooks`가 호출.

**상세**: `04-CRATES/03-myth-gavel.md`

### Layer 2 — `myth-identity`

**역할**: 3-Tier Identity 매칭.

- Tier 1: SHA1 normalize + hash
- Tier 2: Embedding → vector store knn
- Tier 3: LLM judge (Python dispatch)
- `trait VectorStore` + `InMemoryStore` 구현 (Day-1)
- `SqliteVecStore` 스텁 (Milestone B에서 활성)
- Aggressive text normalization (path, timestamp, UUID, hex 제거)

**상세**: `04-CRATES/04-myth-identity.md`

### Layer 3 — `myth-hooks`

**역할**: Claude Code hook 엔트리 6개.

bin targets:
- `myth-hook-pre-tool` (The Gavel 호출)
- `myth-hook-post-tool` (latency 기록만)
- `myth-hook-post-tool-failure` (Assessor trigger)
- `myth-hook-user-prompt` (pending reflection 감시)
- `myth-hook-stop` (Tier 2 block, 비활성 상태)
- `myth-hook-session-start` (brief.md 주입)

각 바이너리는 **최소 책임 원칙**. 공통 로직은 `myth-hooks::core` 모듈.

**상세**: `04-CRATES/05-myth-hooks.md`

### Layer 3 — `myth-embed`

**역할**: 임베딩 daemon.

**bin target** 1개: `myth-embed` (클라이언트/데몬 통합).

- 클라이언트 모드: Unix socket connect → bincode 요청/응답
- 데몬 모드 (`--daemon`): fastembed-rs 상주, socket listen
- 프로토콜: length-prefixed bincode v1
- 투명성 명령: status, stop, probe
- 15분 유휴 자가 종료

**다른 crate와 약한 결합**. `myth-common`만 직접 의존. 다른 crate는 IPC로만 소통 (의존성 없음).

**상세**: `04-CRATES/06-myth-embed.md`

### Layer 4 — `myth-orchestrator`

**역할**: 병렬 실행 orchestration.

기존 `harness-orchestrator/lib/execute.sh` 포팅 + Rust wrapper:

- tmux 세션 관리
- git worktree 생성/정리
- Claude Squad 연동
- 최대 N 동시 실행 (기본 3~4)
- 실패 수집 및 caselog 기록

Jeffrey의 기존 harness-orchestrator 작업을 그대로 재활용. **shell script + Rust 하이브리드**.

**상세**: `04-CRATES/07-myth-orchestrator.md`

### Layer 4 — `myth-runtime`

**역할**: Claude Code subprocess 관리.

- `claude` 바이너리 호출 (Option 4+ Hybrid Wrapper 핵심)
- stdin/stdout 파이프
- 세션 ID 할당 (MYTH_SESSION_ID)
- Claude Code 버전 감지
- Extra Usage fallback (Max 소진 시)

**상세**: `04-CRATES/08-myth-runtime.md`

### Layer 4 — `myth-ui`

**역할**: TUI 대시보드.

- ratatui + crossterm
- 패널: caselog live, lesson status, brief preview, Migration Readiness
- syntect 구문 강조
- pulldown-cmark 마크다운 렌더링
- 키 바인딩 (vim 스타일)

Claude 세션과 **분리된 뷰어**. `myth status` 또는 `myth watch` 명령으로 띄움.

**상세**: `04-CRATES/09-myth-ui.md`

### Layer 5 — `myth-cli`

**역할**: 사용자 진입점.

**bin target** 1개: `myth` (단일 바이너리, 모든 서브커맨드 dispatch).

서브커맨드:
- `myth init` (프로젝트 스캐폴딩)
- `myth install` / `myth uninstall`
- `myth run [COMMAND]` (Claude Code subprocess)
- `myth status` / `myth watch`
- `myth doctor` (health check, Migration Readiness)
- `myth lesson list/show/appeal/retrial/split/merge`
- `myth observer run [--dry]`
- `myth gavel status/stop` (Milestone C 이후)
- `myth embed status/stop/probe`
- `myth constitution [view]`
- `myth key set/show/clear` (Milestone A 이후)

`clap` 기반. 서브커맨드별 파일 분리.

**상세**: `04-CRATES/10-myth-cli.md`

## 4. 공통 의존성 버전

```toml
# ~/myth/rust/Cargo.toml workspace-level

[workspace.dependencies]
# 기반
serde = { version = "1", features = ["derive"] }
serde_json = "1"
serde_yaml = "0.9"
anyhow = "1"
thiserror = "1"
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1", features = ["v4", "serde"] }
tracing = "0.1"
tracing-subscriber = "0.3"

# DB
rusqlite = { version = "0.31", features = ["bundled", "modern_sqlite"] }

# 정규식
regex = "1"
regex-automata = "0.4"

# Hash
sha1 = "0.10"
blake3 = "1"

# IPC
bincode = "1"

# Runtime (daemon용)
tokio = { version = "1", default-features = false, features = ["rt", "io-util", "net", "time", "macros"] }

# Allocator
mimalloc = { version = "0.1", default-features = false }

# Vector 기반
memmap2 = "0.9"
simsimd = "4"

# 임베딩 (rustls TLS to avoid libssl-dev system dependency — see 06-myth-embed.md change box)
fastembed = { version = "5", default-features = false, features = [
    "hf-hub",
    "hf-hub-rustls-tls",
    "ort-download-binaries-rustls-tls",
] }
# ort: transitive via fastembed (no direct workspace entry — see 06-myth-embed.md change box)

# TUI
ratatui = "0.26"
crossterm = "0.27"
syntect = "5"
pulldown-cmark = "0.10"

# CLI
clap = { version = "4", features = ["derive"] }
```

각 crate는 `workspace = true` 로 상속:

```toml
# crates/myth-common/Cargo.toml
[dependencies]
serde = { workspace = true }
thiserror = { workspace = true }
```

## 5. bin targets 전체

| bin | 위치 | 용도 |
|---|---|---|
| `myth` | myth-cli | 주 CLI |
| `myth-hook-pre-tool` | myth-hooks | PreToolUse hook (The Gavel) |
| `myth-hook-post-tool` | myth-hooks | PostToolUse (latency 로깅) |
| `myth-hook-post-tool-failure` | myth-hooks | Assessor trigger |
| `myth-hook-user-prompt` | myth-hooks | UserPromptSubmit 감시 |
| `myth-hook-stop` | myth-hooks | Stop hook (Tier 2 대기) |
| `myth-hook-session-start` | myth-hooks | brief.md 주입 |
| `myth-embed` | myth-embed | 임베딩 daemon (클/서 통합) |

**총 8개 바이너리**. 각각 release 빌드 시 mimalloc + LTO fat + panic abort 적용.

install.sh가 `~/.local/bin/`에 심볼릭 링크 또는 copy.

## 6. 빌드·테스트

### 전체 빌드

```bash
cd ~/myth/rust
cargo build --release
```

`--release`는 `Cargo.toml`의 `[profile.release]` 사용.

### 전체 테스트

```bash
cargo test --workspace
cargo nextest run --workspace  # 빠른 대안
```

### crate별 빌드

```bash
cargo build --release -p myth-gavel
```

### PGO 빌드 (Milestone C 대비)

```bash
~/myth/scripts/pgo-build.sh
```

## 7. 개발 규칙

### 새 crate 추가

- `~/myth/rust/crates/myth-X/` 생성
- `cargo new --lib crates/myth-X` 또는 `--bin`
- workspace `Cargo.toml`의 members에 추가 (자동이면 skip)
- `04-CRATES/` 아래 문서 1개 추가

### 의존 레이어 규칙

- 레이어 상향 의존 금지 (myth-common이 myth-gavel을 의존할 수 없음)
- 같은 레이어 의존 최소화
- 순환 의존 금지 (`cargo tree` 검증)

### 라이센싱

- 각 crate의 `Cargo.toml`에 `license = "MIT OR Apache-2.0"`
- 외부 차용 코드는 `THIRD-PARTY.md` 명기

## 8. crate 매핑 — 어디서 뭘 구현하는가

자주 묻는 "이 기능 어느 crate에 있지?" 가이드:

| 기능 | 위치 |
|---|---|
| Level/Recurrence/Enforcement 타입 | `myth-common` |
| SQLite 스키마·마이그레이션 | `myth-db/migrations/` |
| 47개 정규식 컴파일 | `myth-gavel::rules` |
| Grid 매트릭스 lookup | `myth-gavel::grid` |
| SHA1 identity hash | `myth-identity::tier1` |
| 임베딩 유사도 매칭 | `myth-identity::tier2` |
| in-memory vector store | `myth-identity::store::in_memory` |
| PreToolUse hook 바이너리 | `myth-hooks/src/bin/pre_tool.rs` |
| Unix socket 서버 (embed) | `myth-embed::daemon` |
| fastembed 모델 로딩 | `myth-embed::model` |
| tmux + worktree 관리 | `myth-orchestrator::exec` |
| `claude` subprocess | `myth-runtime::claude` |
| TUI 레이아웃 | `myth-ui::panels` |
| 마크다운 렌더링 | `myth-ui::markdown` |
| CLI 서브커맨드 파싱 | `myth-cli/src/subcmd/*.rs` |

## 9. 변경 이력

| 날짜 | 버전 | 변경 |
|---|---|---|
| 2026-04-19 | v1.0 | 초기 작성. 10개 crate 확정, 레이어 구조 박제. |
