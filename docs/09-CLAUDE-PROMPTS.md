# myth — Claude Code Execution Playbook

**이 문서는 Claude Code에게 던지는 "작업 지시서"다.**

Jeffrey는 이 문서의 Wave 0부터 순서대로 Claude Code에 지시하면 된다. Claude Code는 myth-orchestrator의 wave-based `plan.json`을 생성해 자기 자신을 병렬 실행할 수 있지만, Day-0 상황에서는 **myth-orchestrator가 아직 구현되지 않았으므로** Claude Code 수동 또는 Jeffrey의 기존 `harness-orchestrator`를 활용한다.

## 0. 기본 원칙

Claude Code가 이 문서에 따라 작업할 때 **반드시 지켜야 할 것**:

### 0.1 참조 문서

모든 구현은 다음 문서들을 **정확한 원천**으로 참조:

- `~/myth/docs/02-CONCEPTS.md` — 용어 정의
- `~/myth/DECISIONS.md` — 왜 이런 선택인지 (헷갈릴 때 여기)
- `~/myth/ARCHITECTURE.md` — 실행 모델, API 계약
- `~/myth/docs/04-CRATES/*.md` — 각 crate 상세 스펙
- `~/myth/docs/05-PYTHON.md` — Python 레이어
- `~/myth/docs/06-HOOKS.md` — hook JSON schema
- `~/myth/docs/07-STATE.md` — SQLite 스키마
- `~/myth/PROTOCOL.md` — bincode wire protocol

**설계 문서와 실제 구현이 다르면 설계 문서가 정답이다**. 편의를 위한 일탈 금지.

### 0.2 언어

- **코드**: 영어 (변수명, 함수명, 파일명, 주석)
- **Git commit message**: 영어
- **테스트 fixture 이름**: 영어
- **에러 메시지 (내부 로그)**: 영어
- **사용자 가시 메시지 (CLI 출력)**: 영어가 기본, 한국어는 별도 i18n 레이어 (Day-1 이후)

### 0.3 의존성 정책

- `Cargo.toml`에 새 의존성 추가 시 workspace-level에 먼저 추가
- 라이선스 허용 범위: MIT, Apache-2.0, BSD-3, MPL-2.0, Unlicense, CC0
- **금지**: GPL, LGPL, AGPL
- 추가 시 `THIRD-PARTY.md`에 반영

### 0.4 커밋 단위

각 Wave 단위로 Git commit. 메시지 규약:

```
wave-N: <subject>

<optional body>

Refs: ~/myth/docs/...
```

### 0.5 실패 시 행동

- 컴파일 에러 → 설계 문서 재확인, 작은 단위로 분할 재시도
- 테스트 실패 → 실패 원인 분석 후 **코드 수정** (테스트 완화 금지)
- 설계 문서와 충돌 → **작업 중단 + Jeffrey에게 질문**

### 0.6 myth 자체 사용 금지 (Day-0 상태)

Claude Code가 myth를 빌드하는 도중에는 myth가 **아직 설치 안 됨**. 즉:
- `myth run`, `myth init` 사용 불가
- 프로젝트 .claude/settings.json에 myth hook 등록하지 말 것
- Claude Code 자체 기본 동작 (myth 없이) 사용

Day-1 완료 후에야 myth가 자기 자신의 hook을 사용할 수 있다.

---

## Wave 0 — 프로젝트 스캐폴딩

**목표**: `~/myth/` 디렉토리 안에 전체 프로젝트 구조 생성.

### Task 0.1 — Rust Workspace 생성

```bash
cd ~/myth
mkdir -p rust/crates
cd rust
```

`rust/Cargo.toml` 작성:

```toml
[workspace]
resolver = "2"
members = [
    "crates/myth-common",
    "crates/myth-db",
    "crates/myth-gavel",
    "crates/myth-identity",
    "crates/myth-hooks",
    "crates/myth-embed",
    "crates/myth-orchestrator",
    "crates/myth-runtime",
    "crates/myth-ui",
    "crates/myth-cli",
]

[workspace.package]
version = "0.1.0"
edition = "2021"
license = "MIT OR Apache-2.0"
repository = "https://github.com/Di-Vernon/myth"  # placeholder
authors = ["Jeffrey"]

[workspace.dependencies]
# (04-CRATES/00-overview.md §4에 정의된 전체 목록)
serde = { version = "1", features = ["derive"] }
serde_json = "1"
serde_yaml = "0.9"
anyhow = "1"
thiserror = "1"
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1", features = ["v4", "serde"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
rusqlite = { version = "0.31", features = ["bundled", "modern_sqlite"] }
regex = "1"
regex-automata = "0.4"
sha1 = "0.10"
blake3 = "1"
bincode = "1"
tokio = { version = "1", default-features = false, features = ["rt", "io-util", "net", "time", "macros", "sync", "process", "fs"] }
mimalloc = { version = "0.1", default-features = false }
memmap2 = "0.9"
simsimd = "4"
fastembed = "5"
ort = "2.0-rc"
ratatui = "0.26"
crossterm = "0.27"
syntect = "5"
pulldown-cmark = "0.10"
clap = { version = "4", features = ["derive"] }

[profile.release]
opt-level = 3
lto = "fat"
codegen-units = 1
strip = "symbols"
panic = "abort"
debug = 0
overflow-checks = false
incremental = false
```

`rust/.cargo/config.toml` 작성:

```toml
[build]
rustflags = ["-C", "target-cpu=x86-64-v3"]

[target.x86_64-unknown-linux-gnu]
linker = "clang"
rustflags = [
    "-C", "link-arg=-fuse-ld=mold",
    "-C", "target-cpu=x86-64-v3",
]
```

### Task 0.2 — 10개 crate 생성

각 crate는 빈 skeleton:

```bash
cd ~/myth/rust
for crate in myth-common myth-db myth-gavel myth-identity \
             myth-orchestrator myth-runtime myth-ui; do
    cargo new --lib crates/$crate --vcs none
done

for crate in myth-hooks myth-embed myth-cli; do
    cargo new --lib crates/$crate --vcs none  # lib + multiple bins 추후 추가
done
```

각 crate의 `Cargo.toml`을 `04-CRATES/XX-NAME.md`의 스펙으로 교체.

### Task 0.3 — Python 패키지

```bash
cd ~/myth
mkdir -p python/myth_py/assessor python/myth_py/observer python/tests
cd python
```

`python/pyproject.toml`은 `05-PYTHON.md` §pyproject.toml 섹션 참조.

각 서브패키지에 `__init__.py` 빈 파일:

```bash
touch python/myth_py/__init__.py
touch python/myth_py/assessor/__init__.py
touch python/myth_py/observer/__init__.py
touch python/tests/__init__.py
```

### Task 0.4 — Templates 디렉토리

```bash
cd ~/myth
mkdir -p templates/.claude/agents
mkdir -p templates/.claude/hooks
mkdir -p templates/commons
```

파일:
- `templates/.claude/agents/assessor.md` — `05-PYTHON.md` §Assessor subagent 정의 복사
- `templates/.claude/agents/observer.md` — Sonnet 기반, 주간 실행
- `templates/.claude/settings.json.template` — hook 등록 JSON (프로젝트 경로는 $HOME 치환)
- `templates/commons/seed-lessons.yaml` — 초기 예시 lesson (빈 파일도 OK)
- `templates/bedrock-rules.yaml` — 47개 정규식 원본 (Wave 7에 채움)
- `templates/foundation-rules.yaml`
- `templates/grid.yaml` — 기본 5×6 매트릭스

### Task 0.5 — scripts 디렉토리

```bash
cd ~/myth
mkdir -p scripts
```

파일:
- `scripts/install.sh` — Wave 5 구현 시 상세 작성
- `scripts/uninstall.sh`
- `scripts/pgo-build.sh` — 스켈레톤만
- `scripts/license-audit.sh` — `cargo license` + `pip-licenses` 호출

### Task 0.6 — tests/fixtures 디렉토리

```bash
cd ~/myth
mkdir -p tests/fixtures/positive tests/fixtures/negative
mkdir -p tests/integration tests/shadow
```

Wave 7에서 560개 fixture 작성.

### Task 0.7 — harness-orchestrator 쉘 복사

Jeffrey의 기존 쉘 스크립트 재활용:

```bash
mkdir -p ~/myth/rust/crates/myth-orchestrator/scripts
cp ~/project/harness-orchestrator/lib/*.sh ~/myth/rust/crates/myth-orchestrator/scripts/
```

### Task 0.8 — Git 초기화

```bash
cd ~/myth
git init
git add .
git commit -m "wave-0: initial scaffolding

- 10 Rust crates (workspace)
- Python myth_py package
- templates directory
- scripts skeleton
- harness-orchestrator shell scripts copied

Refs: ~/myth/docs/08-BUILD-SCOPE.md §5
"
```

**Wave 0 Exit criteria**:
- `cd ~/myth/rust && cargo check --workspace` (빈 lib.rs지만 컴파일 성공)
- 디렉토리 구조가 `03-DIRECTORY.md` §1과 일치
- Git log 1개

---

## Wave 1 — Layer 0~1 (기반)

### Task 1.1 — myth-common 구현

**참조**: `04-CRATES/01-myth-common.md`

구현 순서:
1. `src/types.rs`: Level, Recurrence, Enforcement, Category, IdentityHash
2. `src/error.rs`: MythError
3. `src/ids.rs`: LessonId, SessionId, ReminderId
4. `src/time.rs`: Timestamp helpers
5. `src/paths.rs`: XDG 경로 함수들
6. `src/logging.rs`: tracing 초기화
7. `src/lib.rs`: pub use 전체

**Cargo.toml**: `04-CRATES/01-myth-common.md` §Cargo.toml 그대로.

**필수 의존성 추가**: `dirs = "5"` (워크스페이스에 없으면 추가).

**테스트** (`tests/` 디렉토리):
- `types_test.rs`: Recurrence::from_count 경계값
- `paths_test.rs`: XDG_CONFIG_HOME 환경변수 처리
- `error_test.rs`: From 변환

### Task 1.2 — myth-db 구현

**참조**: `04-CRATES/02-myth-db.md`, `07-STATE.md`

구현 순서:
1. `migrations/001_initial.sql`: `07-STATE.md` §SQLite 스키마 v1 전체 복사
2. `src/sqlite/pool.rs`, `pragmas.rs`, `migration.rs`
3. `src/jsonl.rs`: fcntl flock 기반 append
4. `src/audit/`: AuditEntry (blake3), AuditLog
5. `src/lesson.rs`: LessonStore trait + SqliteLessonStore
6. `src/events.rs`: hook_events 테이블 접근
7. `src/appeal.rs`: appeal_history

**주의사항**:
- PRAGMA는 매 connection 열 때마다 적용 (WAL은 persistent지만 busy_timeout은 connection-local)
- JSONL writer는 파일 닫고 열기 반복 — 동시 write 안전
- `migrations/001_initial.sql`의 `PRAGMA user_version = 1`은 SQL의 마지막에 위치 (자동 증가 방지)

**테스트** (`tests/`):
- `sqlite_test.rs`: open + pragma + schema 검증
- `migration_test.rs`: 빈 DB → v1 마이그레이션
- `lesson_crud_test.rs`: insert, get, find_by_identity, update
- `jsonl_concurrent_test.rs`: 2개 프로세스 동시 append (fork)
- `audit_chain_test.rs`: 10개 엔트리 체인 → 검증 → 한 줄 위조 → 검증 실패

### Task 1.3 — Wave 1 통합 테스트

```bash
cd ~/myth/rust
cargo test -p myth-common
cargo test -p myth-db
cargo clippy -p myth-common -p myth-db
```

All green → Git commit:

```bash
git add rust/crates/myth-common rust/crates/myth-db
git commit -m "wave-1: implement Layer 0-1 (common, db)

myth-common: core types (Level, Recurrence, Enforcement, etc.), XDG paths
myth-db: SQLite schema v1, JSONL writer, Merkle audit chain

Refs: ~/myth/docs/04-CRATES/01-myth-common.md
      ~/myth/docs/04-CRATES/02-myth-db.md
      ~/myth/docs/07-STATE.md
"
```

---

## Wave 2 — Layer 2 (판정 + 정체성)

### Task 2.1 — myth-gavel 구현

**참조**: `04-CRATES/03-myth-gavel.md`

구현 순서:
1. `src/rules/compile.rs`: Rule YAML → CompiledRule (regex 컴파일)
2. `src/rules/bedrock.rs`, `foundation.rs`, `surface.rs`
3. `src/rules/mod.rs`: RuleSet 통합
4. `src/grid/default.rs`: 5×6 기본 매트릭스 (`04-CRATES/03-myth-gavel.md` §Grid 참조)
5. `src/grid/override.rs`: SQLite grid_overrides 적용
6. `src/grid/mod.rs`: Grid 통합
7. `src/fatigue.rs`: FatigueTracker
8. `src/verdict.rs`: Verdict + to_hook_json
9. `src/judge.rs`: Gavel::judge() 전체 흐름
10. `src/lib.rs`

**중요 패턴**:
- `once_cell::sync::Lazy<RuleSet>` 전역 캐시 (첫 호출에 로드, 이후 무료)
- RegexSet으로 한 번에 매칭, 매칭된 index로 상세 rule 정보 조회
- Fail-safe: rule 로드 실패 시 모든 tool 차단 (deny by default)

**테스트**:
- `tests/bedrock_load_test.rs`: YAML 로딩
- `tests/bedrock_match_test.rs`: 47개 rule에 대한 간이 positive/negative (full fixture는 Wave 7)
- `tests/grid_test.rs`: 매트릭스 lookup + override 우선순위
- `tests/fatigue_test.rs`: 세션당 상한 동작
- `tests/verdict_test.rs`: Enforcement → Hook JSON 변환

### Task 2.2 — myth-identity 구현

**참조**: `04-CRATES/04-myth-identity.md`

구현 순서:
1. `src/tier1/normalize.rs`: aggressive normalize (regex 기반)
2. `src/tier1/mod.rs`: SHA1 hash
3. `src/store/in_memory.rs`: mmap 기반 VectorStore 구현 (`vectors.bin` 파일 포맷은 `07-STATE.md` §벡터 바이너리 참조)
4. `src/store/sqlite_vec.rs`: 스텁만 (todo!() macro)
5. `src/store/usearch.rs`: 스텁만
6. `src/store/mod.rs`: trait VectorStore
7. `src/tier2/embed_client.rs`: myth-embed 호출 (다음 Task에서 myth-embed 구현 후 활용)
8. `src/tier2/mod.rs`: 임베딩 유사도 매칭
9. `src/tier3/mod.rs`: Python subprocess 호출 stub
10. `src/matcher.rs`: IdentityMatcher 통합

**vectors.bin 구현 참조** (`07-STATE.md` §벡터 바이너리 — 파일 레이아웃):
- Magic, version, dim, count, generation 헤더
- atomic rename 기반 재기록
- mmap 후 `&[[f32; 384]]`로 슬라이스

**테스트**:
- `tests/normalize_test.rs`: timestamp/uuid/path → placeholder 변환
- `tests/tier1_test.rs`: 같은 정규화 → 같은 SHA1
- `tests/in_memory_store_test.rs`: 100개 upsert + knn(k=5)
- `tests/integrity_test.rs`: 손상된 vectors.bin 감지
- `tests/lapse_test.rs`: compute_lapse_score 경계값

### Task 2.3 — myth-embed 구현

**참조**: `04-CRATES/06-myth-embed.md`, `PROTOCOL.md`

구현 순서:
1. `src/protocol/types.rs`: Request, Response, Op, OpResult
2. `src/protocol/wire.rs`: length-prefixed bincode I/O
3. `src/daemon/model.rs`: fastembed-rs 래핑
4. `src/daemon/idle.rs`: IdleTracker
5. `src/daemon/stats.rs`: 상태 수집 (uptime, request_count, RSS)
6. `src/daemon/server.rs`: tokio UnixListener
7. `src/daemon/mod.rs`: run() — 데몬 메인 루프
8. `src/lock.rs`: flock
9. `src/spawn.rs`: self-fork-exec
10. `src/client.rs`: EmbedClient (sync wrapper)
11. `src/cli.rs`: status/stop/probe
12. `src/main.rs`: 클라/데몬 분기

**중요**:
- `#[global_allocator] static GLOBAL: MiMalloc = MiMalloc;` 
- `#[tokio::main(flavor = "current_thread")]` — 단일 스레드 런타임 (데몬은 가볍게)
- `model.onnx` 다운로드 경로: `~/.myth/embeddings/models/multilingual-e5-small/`
- fastembed-rs의 `EmbeddingModel::MultilingualE5Small` 사용

**첫 실행 시 주의**:
- fastembed가 HuggingFace에서 모델 다운로드 시도 → 네트워크 필요
- 오프라인 환경이면 사전에 수동 다운로드 안내

**테스트**:
- `tests/protocol_roundtrip.rs`: bincode 직렬화/역직렬화
- `tests/daemon_lifecycle.rs`: spawn → ping → shutdown (tokio test, tempdir)
- `tests/concurrent_clients.rs`: 여러 동시 요청
- `tests/spawn_race.rs`: 두 클라이언트 동시 spawn 시도 → flock으로 한 인스턴스만 성공
- 모델 로드 테스트는 `#[ignore]` 처리 (CI에서 skip)

### Task 2.4 — Wave 2 통합 테스트

```bash
cd ~/myth/rust
cargo test -p myth-gavel -p myth-identity -p myth-embed
cargo clippy -p myth-gavel -p myth-identity -p myth-embed
```

수동 검증:
```bash
cargo build --release -p myth-embed
./target/release/myth-embed probe "hello world"
# 384-dim vector 출력 확인
```

**Wave 2 커밋**:
```bash
git commit -m "wave-2: implement Layer 2 (gavel, identity, embed)

- myth-gavel: RuleSet loader, 5x6 Grid, FatigueTracker, Verdict
- myth-identity: normalize, 3-tier matcher, InMemoryStore
- myth-embed: self-daemonizing daemon, bincode wire protocol

Refs: ~/myth/docs/04-CRATES/03-myth-gavel.md
      ~/myth/docs/04-CRATES/04-myth-identity.md
      ~/myth/docs/04-CRATES/06-myth-embed.md
      ~/myth/PROTOCOL.md
"
```

---

## Wave 3 — Layer 3 (Hooks)

### Task 3.1 — myth-hooks 구현

**참조**: `04-CRATES/05-myth-hooks.md`, `06-HOOKS.md`

구현 순서:
1. `src/core/input.rs`: stdin JSON 파싱 (Claude Code hook schema)
2. `src/core/output.rs`: stdout JSON + exit code
3. `src/core/latency.rs`: hook-latency.ndjson append
4. `src/core/session.rs`: SessionId 관리
5. `src/templates/variant_b.rs`: engineered 템플릿
6. `src/templates/variant_a.rs`, `variant_c.rs`: 기본 구조만
7. `src/bin/pre_tool.rs`: The Gavel 호출
8. `src/bin/post_tool.rs`: latency + hook_events append
9. `src/bin/post_tool_failure.rs`: Tier 0 classify + Tier 1 Variant B 주입
10. `src/bin/user_prompt.rs`: pending reflection 감시
11. `src/bin/stop.rs`: Tier 2 (enable_tier2=false 경로)
12. `src/bin/session_start.rs`: brief.md 주입

**공통 패턴** (모든 bin의 main):
```rust
use mimalloc::MiMalloc;
#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

fn main() -> ExitCode {
    let start = Instant::now();
    myth_common::logging::init_logging(env!("CARGO_BIN_NAME"));
    let result = run();
    let latency = start.elapsed().as_secs_f64() * 1000.0;
    let _ = myth_hooks::core::latency::record(env!("CARGO_BIN_NAME"), latency, &result);
    result.into_exit_code()
}
```

**Cargo.toml `[[bin]]` 6개** (`04-CRATES/05-myth-hooks.md` §Cargo.toml).

**테스트**:
- `tests/pre_tool_integration.rs`: stdin JSON → stdout JSON 왕복
- `tests/post_tool_failure_classify.rs`: deterministic classify 정확도
- `tests/user_prompt_compliance.rs`: transcript에서 tool_use 감시
- `tests/session_start_brief.rs`: brief.md 주입 검증
- `tests/variant_b_render_test.rs`: 템플릿 렌더링

```bash
cargo test -p myth-hooks
cargo build --release -p myth-hooks
ls -la target/release/myth-hook-*  # 6개 바이너리 존재 확인
```

### Task 3.2 — Wave 3 커밋

```bash
git commit -m "wave-3: implement Layer 3 (hooks, 6 binaries)

- 6 hook binaries: pre-tool, post-tool, post-tool-failure, user-prompt, stop, session-start
- Variant B engineered template
- Tier 0 deterministic classifier
- Tier 2 standby (enable_tier2=false)

Refs: ~/myth/docs/04-CRATES/05-myth-hooks.md
      ~/myth/docs/06-HOOKS.md
"
```

---

## Wave 4 — Layer 4 (Orchestration, Runtime, UI)

### Task 4.1 — myth-runtime 구현

**참조**: `04-CRATES/08-myth-runtime.md`

구현 순서:
1. `src/discovery.rs`: find_claude, detect_version
2. `src/version.rs`: ClaudeVersion + 호환성 체크
3. `src/env.rs`: 환경변수 주입
4. `src/io.rs`: OutputCapture
5. `src/session.rs`: 세션 ID 관리
6. `src/fallback.rs`: ccusage 통합, quota 감지
7. `src/lib.rs`: ClaudeRuntime

**Mock Claude** (`tests/fixtures/mock-claude`):
```bash
#!/usr/bin/env bash
case "$1" in
  --version) echo "claude 2.1.109" ;;
  -p) echo "mock response" ;;
  *) exec /bin/cat ;;  # interactive 시뮬레이션
esac
```

**테스트**:
- `tests/discovery_test.rs`: MYTH_CLAUDE_BIN 환경변수 override
- `tests/version_parse_test.rs`: "claude 2.1.109" 파싱
- `tests/env_inject_test.rs`: MYTH_SESSION_ID 등 주입 검증
- Mock 바이너리로 통합 테스트

### Task 4.2 — myth-orchestrator 구현

**참조**: `04-CRATES/07-myth-orchestrator.md`

구현 순서:
1. `src/plan.rs`: plan.json 파싱 + 검증 (files_affected 중복 체크)
2. `src/tmux.rs`: tmux 래퍼 (shell subprocess)
3. `src/worktree.rs`: git worktree 래퍼
4. `src/watchdog.rs`: hard timeout + stale 감지
5. `src/executor.rs`: execute_task (tmux + worktree + claude-runtime)
6. `src/report.rs`: ExecutionReport + 요약 포맷
7. `src/squad.rs`: Claude Squad 연동 (세션 이름 규약)
8. `src/lib.rs`: Orchestrator

**scripts 재활용**: Task 0.7에서 복사한 `~/myth/rust/crates/myth-orchestrator/scripts/*.sh`를 subprocess 호출.

**테스트**:
- `tests/plan_load_test.rs`: 유효/무효 plan.json
- `tests/tmux_wrapper_test.rs`: tmux 명령 실제 호출 (통합, #[ignore] + --test-threads=1)
- `tests/worktree_test.rs`: git 임시 repo에서 worktree 생성/제거
- `tests/parallel_execution_test.rs`: 3 task 병렬 (mock-claude로)

### Task 4.3 — myth-ui 구현

**참조**: `04-CRATES/09-myth-ui.md`

구현 순서:
1. `src/theme.rs`: 색상 정의
2. `src/markdown.rs`: pulldown-cmark 렌더링
3. `src/syntax.rs`: syntect 래핑
4. `src/panels/caselog.rs`, `lessons.rs`, `brief.rs`, `migration.rs`, `tasks.rs`
5. `src/panels/mod.rs`: 공통 trait Panel
6. `src/layout.rs`: ratatui Layout 계산
7. `src/events.rs`: Event stream (tick + key + file watch)
8. `src/app.rs`: App 상태 머신
9. `src/lib.rs`: run_dashboard 공개 API

**파일 watch**: `notify` crate. WSL2에서 이벤트 누락 가능 → 200ms polling을 fallback으로 병행.

**테스트**:
- `tests/markdown_render_test.rs`: heading, paragraph, code block
- `tests/syntax_test.rs`: Rust 코드 강조
- `tests/panel_layout_test.rs`: Rect 계산
- TUI 자체는 상호작용 테스트 어려움 → 수동 확인

### Task 4.4 — Wave 4 통합

```bash
cargo test -p myth-runtime -p myth-orchestrator -p myth-ui
cargo clippy -p myth-runtime -p myth-orchestrator -p myth-ui
```

수동 검증:
```bash
cargo build --release
./target/release/myth-embed --help  # 이전 Wave 확인
# Wave 5의 myth-cli에서 TUI 검증 예정
```

커밋:
```bash
git commit -m "wave-4: implement Layer 4 (runtime, orchestrator, ui)

- myth-runtime: Claude subprocess management, version check, fallback
- myth-orchestrator: wave-based plan execution, tmux/worktree/watchdog
- myth-ui: ratatui TUI with 5 panels

Refs: ~/myth/docs/04-CRATES/07-myth-orchestrator.md
      ~/myth/docs/04-CRATES/08-myth-runtime.md
      ~/myth/docs/04-CRATES/09-myth-ui.md
"
```

---

## Wave 5 — Layer 5 (CLI)

### Task 5.1 — myth-cli 구현

**참조**: `04-CRATES/10-myth-cli.md`

구현 순서:
1. `src/args.rs`: clap Command + 13개 서브커맨드 Args
2. `src/output.rs`: text/json/yaml 포맷
3. `src/subcmd/init.rs`
4. `src/subcmd/install.rs` + `uninstall.rs`
5. `src/subcmd/run.rs` (interactive + parallel)
6. `src/subcmd/status.rs`
7. `src/subcmd/watch.rs` (myth-ui 호출)
8. `src/subcmd/doctor.rs` (health check 여러 항목)
9. `src/subcmd/lesson.rs` (list/show/appeal/retrial/split/merge)
10. `src/subcmd/observer.rs` (Python subprocess)
11. `src/subcmd/gavel.rs` (Milestone C 전 "not running" 반환)
12. `src/subcmd/embed.rs` (myth-embed 바이너리 위임)
13. `src/subcmd/constitution.rs` (pager)
14. `src/subcmd/key.rs` (stub — Milestone A 활성 시 실제)
15. `src/main.rs`: tokio main + dispatch

**Cargo.toml** `[[bin]]`: `myth`.

**테스트**:
- `tests/cli_parse_test.rs`: clap edge cases
- `tests/init_test.rs`: 임시 디렉토리에 init → .claude/ 검증
- `tests/doctor_test.rs`: 각 health check 개별 실행
- `tests/integration/end_to_end.rs`: init → (mock run) → observer run

### Task 5.2 — scripts/install.sh 작성

```bash
#!/usr/bin/env bash
set -euo pipefail

MYTH_HOME="${MYTH_HOME:-$HOME/myth}"
LOCAL_BIN="$HOME/.local/bin"

cd "$MYTH_HOME/rust"
cargo build --release

mkdir -p "$LOCAL_BIN"
for bin in myth myth-hook-pre-tool myth-hook-post-tool \
           myth-hook-post-tool-failure myth-hook-user-prompt \
           myth-hook-stop myth-hook-session-start myth-embed; do
    ln -sf "$MYTH_HOME/rust/target/release/$bin" "$LOCAL_BIN/$bin"
done

# Python shims
cat > "$LOCAL_BIN/myth-assessor" <<'EOF'
#!/usr/bin/env bash
exec python3 -m myth_py.assessor.cli "$@"
EOF
chmod +x "$LOCAL_BIN/myth-assessor"

cat > "$LOCAL_BIN/myth-observer" <<'EOF'
#!/usr/bin/env bash
exec python3 -m myth_py.observer.cli "$@"
EOF
chmod +x "$LOCAL_BIN/myth-observer"

# Python 패키지 설치 (editable, 개발 중엔 편리)
cd "$MYTH_HOME/python"
pip install -e . --break-system-packages  # Ubuntu 24.04 대응

# 초기 데이터 구조 생성
"$LOCAL_BIN/myth" install  # init_myth_home() 호출

echo "myth installed. Verify with: myth doctor"
```

### Task 5.3 — Wave 5 통합

```bash
cargo test -p myth-cli
bash ~/myth/scripts/install.sh
myth --version  # "myth 0.1.0" 확인
myth doctor  # 아직 rules 없으므로 일부 경고 예상
```

커밋:
```bash
git commit -m "wave-5: implement Layer 5 (CLI) + install.sh

- 13 subcommands: init, install, run, status, watch, doctor, lesson, observer, gavel, embed, constitution, key, uninstall
- install.sh: build + symlink + init myth home

Refs: ~/myth/docs/04-CRATES/10-myth-cli.md
"
```

---

## Wave 6 — Python 레이어

### Task 6.1 — myth_py.assessor 구현

**참조**: `05-PYTHON.md` §Assessor

```
python/myth_py/assessor/
├── cli.py               # typer entry
├── classifier.py        # Tier 0
├── dispatcher.py        # Tier 3 stub
├── templates.py
├── schema.py            # Pydantic
├── state.py
└── subagent_runner.py
```

### Task 6.2 — myth_py.observer 구현

**참조**: `05-PYTHON.md` §Observer

```
python/myth_py/observer/
├── cli.py
├── analyzer.py
├── brief_gen.py
├── migration.py
├── report.py
└── lapse.py
```

### Task 6.3 — Python 테스트

```
python/tests/
├── assessor/
│   ├── test_classifier.py
│   ├── test_schema.py
│   └── test_templates.py
└── observer/
    ├── test_analyzer.py
    ├── test_brief_gen.py
    └── test_migration.py
```

```bash
cd ~/myth/python
pytest
```

### Task 6.4 — Wave 6 커밋

```bash
git commit -m "wave-6: implement Python layer (assessor, observer)

- myth_py.assessor: classifier, templates, schema, dispatcher stub
- myth_py.observer: analyzer, brief_gen, migration, lapse

Refs: ~/myth/docs/05-PYTHON.md
"
```

---

## Wave 7 — Rules + Fixtures

### Task 7.1 — Bedrock Rule YAML 작성

**참조**: `DECISIONS.md` Decision 5, `THIRD-PARTY.md` §1-2

`~/myth/templates/bedrock-rules.yaml`:

```yaml
version: 1
items:
  - id: rm_rf_unsandboxed
    description: "Unsandboxed rm -rf on production paths"
    rules:
      - { id: R1-A, pattern: '...', level: 5, source: "gitleaks v8.x (MIT)" }
      - { id: R1-B, pattern: '...', level: 5, source: "..." }
      # ... R1-A ~ R1-G (7개)
  - id: production_secrets_commit
    description: "Credential commit"
    rules:
      - { id: R2-A, pattern: '...', level: 5 }
      # ... R2-A ~ R2-D (4개 중 R2-A는 40개 anchored prefix 병합)
  - id: auth_bypass_production
    description: "Authentication bypass"
    rules:
      - { id: R3-A, pattern: '...', level: 5 }
      # ... R3-A ~ R3-D
```

**총 47개 패턴**. 각 source에 gitleaks/detect-secrets 출처 명기.

정확한 정규식은 gitleaks의 `config/gitleaks.toml`에서 매핑. Claude Code가 gitleaks 레포를 참조하거나 설계 문서(Research #5)의 패턴 예시를 사용.

### Task 7.2 — Foundation Rule YAML

```yaml
# ~/myth/templates/foundation-rules.yaml
version: 1
items:
  - id: main_force_push
    rules: [{ id: F1-A, pattern: "git push.*--force", level: 4 }]
  - id: no_verify_ci_bypass
    rules: [{ id: F2-A, pattern: "--no-verify", level: 4 }]
  - id: pii_exfiltration
    rules: [{ id: F3-A, pattern: "...", level: 4 }]
  - id: unverified_dependency
    rules: [{ id: F4-A, pattern: "curl.*\\|\\s*sh", level: 4 }]
  - id: untrusted_arbitrary_execution
    rules: [{ id: F5-A, pattern: "eval\\s*\\(", level: 3 }]
```

### Task 7.3 — Grid YAML

```yaml
# ~/myth/templates/grid.yaml
version: 1
matrix:
  # Level-Recurrence: Enforcement
  "1-I": dismiss
  "1-II": dismiss
  "1-III": note
  # ... 전체 30칸
  "5-VI": strike
# Bedrock 매칭은 항상 seal (매트릭스 우회)
```

### Task 7.4 — 560개 test fixtures

`~/myth/tests/fixtures/positive/R1-A/01.txt`:
```
rm -rf /
```

각 rule 그룹 (14개) × (positive 20 + negative 20) = 560. Claude Code가 Research #5 예시와 자체 판단으로 생성. 각 파일 한 줄씩 또는 여러 줄의 합법적 명령.

### Task 7.5 — pre-commit hook

```bash
# ~/myth/.git/hooks/pre-commit
#!/usr/bin/env bash
cd ~/myth/rust
cargo test --release -p myth-gavel --test bedrock_fixtures
```

### Task 7.6 — Wave 7 검증

```bash
cd ~/myth/rust
cargo test -p myth-gavel --test bedrock_fixtures
# 560 fixtures all green, 0 FP
```

커밋:
```bash
git commit -m "wave-7: rules and fixtures

- bedrock-rules.yaml: 47 patterns across 3 items
- foundation-rules.yaml: 5 items
- grid.yaml: 30-cell default matrix
- 560 test fixtures (positive 280 + negative 280)
- pre-commit hook for FP=0 validation

Refs: ~/myth/DECISIONS.md Decision 5
      ~/myth/THIRD-PARTY.md
"
```

---

## Wave 8 — 통합 검증

### Task 8.1 — end-to-end 시나리오

**시나리오**: 새 프로젝트 만들기 → myth init → (mock) Claude 세션 실행 → 실패 유도 → lesson 기록 → observer 실행 → brief.md 확인.

```bash
# 1. 테스트용 프로젝트
mkdir -p /tmp/myth-test
cd /tmp/myth-test
git init
echo "print('hello')" > test.py

# 2. myth init
myth init

# 3. 검증
ls .claude/  # settings.json, agents/assessor.md, agents/observer.md 존재
cat .claude/settings.json  # 6개 hook 등록

# 4. myth doctor
myth doctor
# 모두 green 기대

# 5. myth run (mock 환경에서)
# 실제 Claude Code 있으면 myth run 으로 세션 시작, 
# 일부러 실패할 tool 실행 (e.g., 존재하지 않는 파일)

# 6. myth lesson list
myth lesson list  # 새 lesson 확인

# 7. myth observer run
myth observer run
cat ~/.myth/brief.md  # Observer 생성 내용 확인

# 8. myth watch
myth watch  # TUI 잠시 띄워서 확인 (수동)
```

### Task 8.2 — 성능 검증

```bash
# hook latency
hyperfine --warmup 3 --runs 100 \
    'echo "{\"tool_name\":\"Bash\",\"tool_input\":{\"command\":\"ls\"},\"session_id\":\"abc\"}" | myth-hook-pre-tool'
# 기대: mean < 5ms, P99 < 10ms

# myth-embed hot embed
myth embed status  # 데몬 떠있는지 확인
hyperfine --warmup 3 --runs 50 \
    'myth-embed probe "hello world"'
# 기대: mean 10~20ms
```

### Task 8.3 — License audit

```bash
~/myth/scripts/license-audit.sh
# GPL/LGPL/AGPL 없어야 함
```

### Task 8.4 — Wave 8 커밋

```bash
git tag -a v0.1.0 -m "Day-1 release"
git commit -m "wave-8: integration validation complete

- End-to-end scenario: init → run → fail → lesson → observer → brief
- Performance: hook P99 < 10ms (binary-per-hook mode)
- License audit: all permissive

Day-1 ready. Observation period begins.

Refs: ~/myth/docs/08-BUILD-SCOPE.md §6
"
```

---

## Day-1 이후 — 21일 shadow mode

Jeffrey가 Claude Code를 평소처럼 사용. myth는 배경에서 관찰·학습. 21일 후 Milestone A 평가 세션을 Jeffrey가 직접 스케줄.

이 문서의 역할은 **Day-1까지**. Milestone A 이후는 별도 문서·세션에서 다룸.

---

## 부록 A — myth-orchestrator로 self-parallel 실행

**주의**: Wave 4.2가 끝나야 가능. Wave 0~4.1은 수동 실행.

Wave 4.2 완료 후 이 Playbook을 **plan.json으로 변환** 가능:

```json
{
  "version": 1,
  "title": "myth Day-1 build",
  "waves": [
    {
      "id": "W5",
      "tasks": [
        { "id": "T5.1", "description": "myth-cli 구현", "prompt": "...", "files_affected": ["rust/crates/myth-cli/**"] },
        { "id": "T5.2", "description": "install.sh", "prompt": "...", "files_affected": ["scripts/install.sh"] }
      ]
    }
  ]
}
```

그러면 Wave 5 이후는 myth 자기 자신으로 병렬 실행 가능. 단 Day-0에는 myth-orchestrator가 아직 없으므로 **Wave 0~4는 순차 수동 실행**.

---

## 부록 B — 실패 시 복구 Playbook

### 빌드 실패

```bash
cd ~/myth/rust
cargo clean
cargo build --release
```

근본 원인 해결 후 재시도. 절대 `cargo build --release --ignore-errors` 같은 우회 금지.

### 테스트 실패

실패 케이스 분석 → 코드 수정. 테스트를 `#[ignore]`로 막지 말 것.

진짜 환경 의존 (network 필요 등) → `#[ignore]` 허용하되 이유를 주석에 명시:

```rust
#[test]
#[ignore = "requires network to download model"]
fn test_embed_cold_start() { ... }
```

### Claude Code의 작업이 막힐 때

Claude Code가 설계 문서 해석에서 막히거나 모호한 상황 → **작업 중단하고 Jeffrey에게 질문**. 임의 판단하지 않는다.

예시 보고:
```
Wave 3 Task 3.1 진행 중 `06-HOOKS.md`와 `04-CRATES/05-myth-hooks.md`에서 
UserPromptSubmit hook의 반환 JSON 형식이 약간 다릅니다.
- 06-HOOKS.md §4.3: "additionalContext"
- 04-CRATES/05: "context"
어느 쪽을 정답으로 할까요?
```

---

## 체크리스트 요약

**Wave 0 완료**: 디렉토리 구조, cargo check pass  
**Wave 1 완료**: myth-common, myth-db 테스트 green  
**Wave 2 완료**: myth-gavel, myth-identity, myth-embed 테스트 green + probe 동작  
**Wave 3 완료**: 6개 hook 바이너리 빌드, stdin/stdout 왕복 테스트  
**Wave 4 완료**: runtime, orchestrator, UI 빌드  
**Wave 5 완료**: myth CLI 모든 서브커맨드, install.sh 동작  
**Wave 6 완료**: myth_py 테스트 green  
**Wave 7 완료**: 560 fixtures FP=0  
**Wave 8 완료**: end-to-end 시나리오 + 성능 + license audit → v0.1.0 태그

8개 Wave 모두 green = **Day-1 완료**.
