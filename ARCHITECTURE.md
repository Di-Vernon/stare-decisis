# myth — Architecture

이 문서는 myth의 **실행 모델**과 **시간에 따른 구조 변경 조건**을 박제한다. `docs/01-OVERVIEW.md`가 "무엇을 만드는가"를 설명한다면, 이 문서는 "어떤 모델로 돌아가는가" + "언제 어떻게 바뀔 수 있는가"를 설명한다.

## 1. 아키텍처 개요 — Option 4+ Hybrid Wrapper

myth는 Claude Code를 **감싸는 wrapper**다. Claude Code를 대체하지 않고, 그 앞뒤에 앉아서 행동을 관찰·차단·학습한다.

```
사용자 입력
    ↓
[myth CLI (또는 hook 자동 발동)]
    ↓
[Claude Code subprocess]
    ├─ PreToolUse → The Gavel (myth-gavel binary)
    ├─ PostToolUse → (간단한 로깅만)
    ├─ PostToolUseFailure → Assessor (Python subprocess)
    ├─ UserPromptSubmit → Assessor 준수 감시
    └─ SessionStart → brief.md 주입
    ↓
[Tool 실행 결과]
    ↓
[myth Observer (주간 실행)]
    ├─ caselog.jsonl 분석
    ├─ brief.md 갱신
    └─ Migration Readiness 보고
```

**Option 4+ "Hybrid Wrapper"란**: 
- Claude Code는 **그대로 실행**하되
- myth가 **hook 시스템**을 통해 모든 tool 호출에 개입
- myth 자체는 **Rust(60-65%) + Python(30-35%)** 하이브리드
- Rust는 hook 임계 경로 (low latency), Python은 LLM 호출·Agent SDK (유연성)

## 2. v1 API 계약 — 6개 고정점

**이 섹션의 내용은 myth v1에서 영구 고정이다.** Milestone C·D·E 등 미래 구조 변경이 일어나도 이 6개는 바뀌지 않는다. 사용자(Claude Code, `.claude/settings.json`, shell scripts 등)가 이 계약에 의존하기 때문이다.

### Contract 1 — Hook Entry Point Path

```
~/.local/bin/myth-hook-pre-tool       (The Gavel)
~/.local/bin/myth-hook-post-tool       
~/.local/bin/myth-hook-post-tool-failure  (Assessor trigger)
~/.local/bin/myth-hook-user-prompt
~/.local/bin/myth-hook-stop
~/.local/bin/myth-hook-session-start
```

이 경로들은 v3(가벨 daemon 전환) 이후에도 **그대로 유지**된다. 데몬 모드로 가도 같은 바이너리가 같은 경로에서 호출된다(자동 spawn 로직이 내부에서 처리).

### Contract 2 — stdin/stdout Protocol

모든 hook 바이너리는 Claude Code의 **표준 Hook JSON schema**를 stdin으로 받고 **`SyncHookJSONOutput`**을 stdout으로 낸다.

```
stdin (예: PreToolUse):
{
  "tool_name": "Bash",
  "tool_input": {...},
  "tool_use_id": "...",
  "session_id": "...",
  ...
}

stdout:
{
  "continue": true,
  "hookSpecificOutput": {
    "hookEventName": "PreToolUse",
    "permissionDecision": "allow" | "deny" | "ask",
    "additionalContext": "..."
  }
}
```

스키마 확장은 **추가 필드만 허용**, 필드 삭제·의미 변경 금지.

### Contract 3 — Exit Code Semantics

```
0   : allow (실행 허가)
2   : block (실행 차단, stderr가 Claude에게 피드백)
기타 : non-blocking error (hook 자체 오류, tool 실행은 계속)
```

이 매핑은 Claude Code 자체 규약이자 myth 내부 Grid의 Enforcement Action과 연결된다.

### Contract 4 — Environment Variables

myth가 **읽는** 환경 변수 (Claude Code 2.1.114 실측 기준):

```
CLAUDECODE=1                  # Claude Code 하에서 실행 중임을 나타내는 플래그
CLAUDE_CODE_ENTRYPOINT        # 예: "sdk-cli", interactive 등 진입 방식
CLAUDE_CODE_EXECPATH          # claude 바이너리의 절대 경로
CLAUDE_PROJECT_DIR            # 현재 프로젝트 루트 (= stdin JSON의 cwd와 동일)
CLAUDE_ENV_FILE               # SessionStart hook 전용, session-env shell 경로
```

**primary 소스는 stdin JSON**. `CLAUDE_TRANSCRIPT_PATH`, `CLAUDE_SESSION_ID`,
`CLAUDE_HOOK_EVENT`, `CLAUDE_TOOL_INPUT`, `CLAUDE_FILE_PATHS` 등 초안에
있던 변수는 **실제로 환경변수로 주입되지 않는다**. 동일 정보는 stdin
JSON의 `session_id` / `transcript_path` / `hook_event_name` / `tool_input`
필드로 제공된다. myth hook 바이너리는 stdin 파싱이 주 경로, 환경변수는
플래그·경로 용도. 상세는 `docs/06-HOOKS.md §환경 변수` 참조.

myth가 **쓰는/사용하는** 환경 변수:

```
MYTH_SESSION_ID            # UUID, 세션당 고유
MYTH_CORRELATION_ID        # reminder_id (Assessor trigger 추적)
CLAUDE_REVIEW_ACTIVE       # 재귀 hook 호출 방지 플래그
MYTH_DISABLE               # myth 비활성 모드 (디버깅)
MYTH_NO_EMBED_DAEMON       # embed daemon 비활성 (fallback only)
MYTH_ANTHROPIC_API_KEY     # Tier 3 증축 시 (fallback: ~/.config/myth/api_key)
```

### Contract 5 — SQLite Schema Forward-Only

`~/.myth/state.db`의 스키마는 `PRAGMA user_version` 기반 forward-only 마이그레이션.

```sql
PRAGMA user_version = 1;  -- v1 초기 스키마
```

- 필드 추가: OK
- 필드 삭제/이름 변경: 금지 (새 필드 만들고 기존 deprecation)
- 테이블 삭제: 금지
- 새 테이블: OK

마이그레이션 파일 위치: `~/myth/rust/crates/myth-db/migrations/`.

### Contract 6 — Config Paths

```
~/.config/myth/config.yaml         # 사용자 설정 (XDG 표준)
~/.myth/bedrock-rules.yaml         # Bedrock Rule 정의
~/.myth/foundation-rules.yaml       # Foundation Rule
~/.myth/surface-rules.yaml          # Surface Rule (프로젝트별 override 가능)
~/.myth/grid.yaml                   # Level×Recurrence 처분 매트릭스
~/.myth/state.db                    # SQLite
~/.myth/vectors.bin                 # 임베딩 벡터 파일
~/.myth/caselog.jsonl              # 실패 원본 기록
~/.myth/lesson-state.jsonl         # lesson 상태 변화 시계열
~/.myth/audit.jsonl                # Merkle audit log
~/.myth/brief.md                   # Observer 주간 브리프
~/.local/state/myth/               # 런타임 상태 (XDG_STATE_HOME)
    ├── hook-latency.ndjson
    ├── embed-daemon.log
    ├── gavel-daemon.log  (Milestone C 이후만)
    └── tier3-dispatch.jsonl  (Milestone A 이후만)
$XDG_RUNTIME_DIR/myth/embed.sock   # myth-embed daemon Unix socket
```

이 경로들은 고정. v1에서 v2, v3으로 가도 바뀌지 않는다.

## 3. Execution Model — The Gavel (binary-per-hook, v1)

### 현재 모델

`~/.local/bin/myth-hook-pre-tool`은 **단일 발사 Rust 바이너리**다. 각 PreToolUse 이벤트는 새 프로세스를 fork한다.

```
bash PreToolUse 이벤트:
    Claude Code가 fork+exec → myth-hook-pre-tool
      ├─ stdin에서 Hook JSON 읽음 (~0.1ms)
      ├─ SQLite state.db open + WAL (~0.5ms)
      ├─ bedrock/foundation/surface-rules.yaml 로드 (LazyLock 캐시)
      ├─ 47개 정규식 DFA 매칭 (~0.1ms)
      ├─ HashMap Grid lookup (~0.01ms)
      ├─ 판정 → stdout JSON (~0.1ms)
      └─ exit
    총 지연: P50 ~2.3ms, P99 ~6-8ms (목표)
```

### 빌드 프로파일

`~/myth/rust/Cargo.toml` workspace-level:

```toml
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

`~/myth/rust/.cargo/config.toml`:

```toml
[build]
rustflags = ["-C", "target-cpu=x86-64-v3"]

[target.x86_64-unknown-linux-gnu]
linker = "clang"
rustflags = ["-C", "link-arg=-fuse-ld=mold"]
```

글로벌 allocator: `mimalloc` (각 바이너리 `main.rs` 상단 등록).

정규식: `regex-automata` + `include_bytes!` 직렬화 DFA. 런타임 `Regex::new` 호출 0회.

### 유지 조건

이 binary-per-hook 모델은 **P99 hook latency가 15ms 이하를 유지하는 한** 유효하다. 초과 시 Milestone C 전환 조건 체크.

## 4. Milestone A~E — 수렴 트리거

myth는 Phase 개념을 쓰지 않는다. 대신 **각 기능이 실측 조건을 만족할 때** 구조가 진화한다. 이것이 Master Principle("완벽은 도달이 아니라 수렴이다. 수렴은 우연이 아니라 법이다")의 아키텍처 구현이다.

### Milestone A — Assessor Tier 2/3 증축 판단

**트리거**: Day-1 + 21일(3주) 실사용 경과.  
**측정 소스**: `~/.myth/metrics/reflector-shadow.jsonl` (Day-1부터 자동 수집).

**분석 수행자**: Jeffrey + Claude Desktop 공동 (재평가 세션).

**결정 변수** (Tier 1 준수율):
- ≥85% → Tier 2/3 비활성 유지, Day-1 상태 영속
- 70~85% → Tier 2(Stop block 재주장) 활성
- <70% → Tier 2 + Tier 3(Anthropic SDK dispatch) 활성

**Tier 3 활성 시 수행 작업**:
1. Jeffrey가 Anthropic Console에서 API key 발급
2. `myth key set` 실행 → `~/.config/myth/api_key` (mode 0600)
3. Anthropic Console workspace spend limit $10 hard cap 설정
4. `~/.config/myth/config.yaml` 에 `enable_tier3: true` flag
5. `myth doctor --check-tier3`로 검증

**Day-1 준비 상태**: Tier 2/3 관련 코드는 **전부 구현되어 있음**. 활성만 flag 토글.

### Milestone B — Vector Store Migration

**트리거 조건** (AND):
- 레코드 수 > 20,000
- in-memory brute-force P99 search latency > 50ms (실측)

**측정 소스**: `~/.myth/state.db`의 `SELECT COUNT(*) FROM lessons`, Observer 주간 리포트의 벡터 검색 P99.

**전환 대상**:
- 후보 1: sqlite-vec (그 시점 Rust crate가 상류 안정 버전 따라잡았다면)
- 후보 2: usearch (sqlite-vec DiskANN 불안정 시)

**전환 방식**: `myth-identity/src/store/` 아래 구현체 파일 추가 (`sqlite_vec.rs` 또는 `usearch.rs`). `VectorStore` trait 구현. config로 선택:

```yaml
# ~/.config/myth/config.yaml
vector_store: in_memory  # → "sqlite_vec" 또는 "usearch"로 변경
```

### Milestone C — The Gavel Daemon

**트리거 조건** (AND):
1. `~/.local/state/myth/hook-latency.ndjson` 주간 집계에서 **P99 > 15ms가 2주 연속**
2. Research #2 §4 빌드 프로파일 완전 적용 상태 (`myth doctor --perf-check` 통과)
3. WSL2 운영 체크리스트 그린 상태 (`myth doctor --wsl-check` 통과)
4. PGO+BOLT 적용 시도 후에도 P99 > 15ms 유지 (`~/myth/scripts/pgo-build.sh` 결과)

**비상 트리거**: 단일 hook event의 P99 > 50ms 측정 시 **즉시 검토** (2주 누적 불필요).

**Observer 주간 브리프 고정 섹션**:

```markdown
## Migration Readiness — The Gavel (Milestone C)

- [ ] P99 > 15ms for 2 weeks  (current: X.Yms)
- [ ] Build profile applied (verified by myth doctor)
- [ ] WSL2 checklist green
- [ ] PGO+BOLT attempted with insufficient improvement

Status: No migration needed / Consider migration / Migration required
```

**전환 방식**: Self-daemonizing (emacsclient 패턴). 같은 `myth-hook-pre-tool` 바이너리가 `--daemon` 플래그로 데몬 모드 전환. 클라이언트 모드(기본)가 `$XDG_RUNTIME_DIR/myth/gavel.sock`에 Unix socket 연결 시도, ECONNREFUSED 시 `self-fork-exec --daemon`.

**투명성 5요소**:
- `myth gavel status` (PID, uptime, 요청 수, RSS)
- `myth gavel stop`
- 30분 유휴 자가 종료
- `~/.local/state/myth/gavel-daemon.log` (JSON Lines)
- `--no-gavel-daemon` 탈출구

**v1 API 계약 6개 전부 유지**. 외부 관점에서 같은 바이너리가 더 빠를 뿐.

### Milestone D — Semantic Detection

**트리거**: Observer 주간 리포트에서 **Bedrock Rule miss 관찰**. 즉 정규식 clear로 통과한 tool_input 중 Observer가 **"이건 Bedrock 위반이었어야 한다"** 판정한 사례 누적.

**전제 조건**: Milestone C 완료 (daemon 인프라 재사용).

**활성 메커니즘**:
- The Gavel daemon에 `bge-small` 또는 그 시점 최적 코드 임베딩 모델 추가 (별도 myth-embed namespace)
- 각 Bedrock Rule에 anchor paraphrase 10~30개 수집
- 정규식 clear 경로에서 임베딩 추출 → anchor centroid와 cosine 비교
- cosine ≥ 0.82 → hard-block + lesson 자동 생성
- 0.70~0.82 → ask user
- < 0.70 → allow

**Shadow mode 선행 1주**: 임베딩 결정을 로그만 기록, 실제 차단은 정규식으로만.

### Milestone E — AST-Based Validation

**트리거 조건** (OR):
- Milestone D 활성 후 semantic FP > 5% 실측
- semantic FN > 2% 실측
- Bash 도구 호출에서 체계적 miss 패턴 관찰

**전제 조건**: Milestone D 활성 후 ≥4주 데이터.

**활성 메커니즘**: `tree-sitter-bash` + `ast-grep-core` 도입. The Gavel에 Tier 2 검증 단계 추가 (정규식 → 임베딩 → AST 순).

**범위 제한**: Bash 도구만. Python/JS/Go/Rust는 명시적 비용-효과 재평가 후 opt-in.

### Milestone 간 독립성

Milestone A~E는 **독립적으로 트리거**된다. 순서도 강제되지 않는다:

- A가 Day-1+21일에 발동한다고 B가 발동하는 것은 아님 (레코드 수 조건 별도)
- C가 발동해도 D가 자동 발동은 아님 (조건 별도)
- E가 C보다 먼저 발동하는 것은 **구조상 불가** (D가 C 전제이므로 E도 C 전제)

Observer가 매주 각 Milestone 상태를 브리프에 리포트. 사용자(Jeffrey)가 해당 전환을 실제 승인·실행.

## 5. myth-embed daemon 아키텍처

임베딩 모델(multilingual-e5-small)의 500ms~2초 콜드 로드를 amortize. **Day-1 포함 구조** (Milestone 없이 기본 활성).

```
[myth-hook-post-tool-failure]      ← 단명 바이너리
    │
    │ Unix socket: $XDG_RUNTIME_DIR/myth/embed.sock
    │ Protocol: length-prefixed bincode
    │
    ↓
[myth-embed daemon]                 ← 상주 프로세스
    ├─ 첫 spawn: 500ms~2s (모델 로드)
    ├─ 이후 요청: ~8-15ms
    ├─ 메모리: ~150MB
    └─ 15분 유휴 → self-exit
```

### Self-daemonizing 패턴

동일 `myth-embed` 바이너리가 두 모드:

**클라이언트 모드** (기본):
1. Unix socket connect 시도
2. 성공 → bincode 요청 전송 → 응답 수신 → 종료
3. ECONNREFUSED → `self-fork-exec --daemon`, 100ms 대기, 재시도

**데몬 모드** (`--daemon`):
1. `flock` 획득 (동시 spawn race 방지)
2. 스테일 소켓 `unlink()`
3. 모델 로드 (fastembed-rs)
4. Unix socket bind + listen
5. tokio 이벤트 루프 진입
6. 15분 유휴 감지 시 self-exit

### 프로토콜 (bincode v1)

```rust
#[derive(Serialize, Deserialize)]
struct Request {
    version: u8,  // = 1
    id: Uuid,
    op: Op,
}

enum Op {
    Embed { text: String },
    Ping,
    Shutdown,
}

struct Response {
    version: u8,  // = 1
    id: Uuid,
    result: OpResult,
}

enum OpResult {
    Embedded { vector: Vec<f32> },  // 384 floats
    Pong { uptime_secs: u64, request_count: u64, rss_bytes: u64 },
    ShuttingDown,
    Error { message: String },
}
```

Framing: `u32-le length prefix + bincode payload`. 자세한 wire format은 `PROTOCOL.md`.

## 6. 데이터 흐름

### 실패 시나리오 (가장 복잡한 경로)

```
1. 사용자 → Claude (자연어)
2. Claude → Bash tool 호출
3. Claude Code → PreToolUse hook 발동
   └─ myth-hook-pre-tool
      ├─ 정규식 47개 매칭
      ├─ state.db Grid lookup
      └─ 결과: allow
4. Bash 실행 → 실패 (exit code != 0)
5. Claude Code → PostToolUseFailure hook 발동
   └─ myth-hook-post-tool-failure
      ├─ Tier 0 classify: "schema_mismatch" (deterministic)
      │  └─ 결정 가능 → Level 3 즉시 기록
      ├─ OR Tier 1 classify: "ambiguous" 
      │  ├─ Variant B additionalContext 생성
      │  ├─ reminder_id 발급
      │  ├─ lesson-state.jsonl에 pending_reflection append
      │  └─ additionalContext 반환 (Claude에게 전달됨)
6. 다음 턴 (Claude가 응답 시작):
   - UserPromptSubmit hook → myth-hook-user-prompt
      ├─ pending_reflection 스캔
      ├─ 이전 turn에 Task(assessor) 호출 확인
      └─ 성공 시 compliant 상태, 실패 시 tier=2 escalation
7. Assessor subagent 호출 (Claude가 Task tool로)
   └─ .claude/agents/assessor.md (Haiku)
      ├─ 실패 payload 분석
      ├─ 4+1축 분해
      ├─ Level 1~5 + category 판정
      ├─ identity_hash 계산 (SHA1 + embedding query)
      │  └─ myth-embed daemon 통신
      │     └─ multilingual-e5-small 임베딩
      └─ JSON 반환 (parent에게)
8. parent Claude가 복구 action 결정
9. caselog.jsonl + state.db에 lesson 갱신
10. (주간) Observer → brief.md 재생성 + Migration Readiness 보고
```

### 성공 시나리오 (흔한 경로)

```
1-4 동일, 단 Bash 실행 성공
5. PostToolUse hook → myth-hook-post-tool
   └─ latency 기록만 (hook-latency.ndjson)
   └─ 종료
```

## 7. 파일 시스템 레이아웃

### myth 본체 (Git 저장소)

```
~/myth/
├── README.md, CONSTITUTION.md, ARCHITECTURE.md, ...
├── docs/            # 설계 문서 25+
├── rust/            # Rust workspace
│   ├── Cargo.toml
│   └── crates/
│       ├── myth-common/
│       ├── myth-db/
│       ├── myth-gavel/
│       ├── myth-identity/
│       ├── myth-hooks/
│       ├── myth-embed/
│       ├── myth-orchestrator/
│       ├── myth-runtime/
│       ├── myth-ui/
│       └── myth-cli/
├── python/
│   └── myth_py/
│       ├── assessor/
│       └── observer/
├── templates/       # myth init 복사 원본
│   ├── .claude/
│   │   ├── settings.json.template
│   │   ├── agents/{assessor.md, observer.md}
│   │   └── hooks/
│   └── commons/seed-lessons.yaml
└── scripts/
    ├── install.sh
    └── pgo-build.sh
```

### 시스템 설치물 (사용자 환경)

```
~/.local/bin/myth*               # PATH에 있는 실행 파일들
~/.config/myth/                   # 사용자 설정 (XDG)
~/.myth/                          # 사용자 데이터 (런타임)
~/.local/state/myth/              # 런타임 상태 (XDG_STATE_HOME)
$XDG_RUNTIME_DIR/myth/            # Unix socket (tmpfs)
```

### 프로젝트 단위 (myth init 결과)

```
~/project/프로젝트A/
└── .claude/
    ├── settings.json            # hook 등록, PATH
    ├── agents/
    │   ├── assessor.md → ~/.myth/templates/agents/assessor.md (symlink)
    │   └── observer.md → ~/.myth/templates/agents/observer.md
    ├── hooks/                   # Claude Code hook 설정
    └── CLAUDE.md                # (선택) 프로젝트별 myth 컨텍스트
```

프로젝트별 override:
- `.myth/surface-rules.yaml` (프로젝트 루트) → 전역 surface rule과 병합

## 8. 보안·권한 경계

### 파일 권한

```
~/.config/myth/api_key       : 0600 (rw-------)
~/.config/myth/              : 0700
$XDG_RUNTIME_DIR/myth/       : 0700
~/.myth/                     : 0700
```

### 네트워크

myth 본체는 **네트워크 요청 없음**. 예외 2가지:
- `myth-embed` 초기 다운로드 시 HuggingFace에서 모델 파일 (SHA-256 검증)
- Milestone A 이후 `myth_py.assessor.dispatcher` → Anthropic API (명시적 opt-in)

### SQLite

WAL 모드 + busy_timeout 5000ms + synchronous NORMAL. 단일 writer 원칙 (hook이 읽기 중심, daemon 전환 시 채널 직렬화).

## 9. 장애 모드와 graceful degradation

### myth-embed daemon 다운

Hook이 ECONNREFUSED 감지:
1. Self-fork-exec 재시도 1회
2. 여전히 실패 → `--no-embed-daemon` 모드로 인프로세스 fastembed (500ms 페널티 감수)
3. 여전히 실패 → Tier 2 identity 생략 (Tier 1 SHA1만, recurrence 정확도 저하 감수)
4. caselog는 계속 기록

### The Gavel (Milestone C 이후) daemon 다운

유사한 fallback:
1. ECONNREFUSED → 재spawn 시도
2. 실패 → binary-per-hook 모드로 자동 fallback (latency 악화 감수)
3. SessionStart에 경고 주입

### SQLite 손상

1. startup 시 `PRAGMA integrity_check` 자동 수행
2. 실패 감지 → `state.db.corrupted-{timestamp}`로 이동, 새 DB 초기화
3. `caselog.jsonl`에서 lesson 복원 (append-only라 무결)
4. 사용자 알림

### Bedrock-rules.yaml 파싱 실패

**Fail-safe 원칙**: 파싱 실패 시 The Gavel은 **모든 tool 실행을 차단** (deny by default). Jeffrey가 수동으로 파일 수정 후 재시작.

## 10. 관찰성 (observability)

### 로그 파일

```
~/.local/state/myth/
├── hook-latency.ndjson          # 모든 hook 실행 latency (Day-1)
├── embed-daemon.log             # myth-embed daemon 로그 (JSON Lines)
├── gavel-daemon.log             # The Gavel daemon 로그 (Milestone C 이후)
└── tier3-dispatch.jsonl         # Tier 3 API 호출 로그 (Milestone A 이후)
```

### Metrics

Observer 주간 리포트가 `brief.md`에 포함:

- Hook P50/P99 (7일)
- Bedrock/Foundation/Surface 매칭 횟수
- 새 lesson 수, lapsed lesson 수
- Tier 1/2/3 준수율 (Assessor shadow metric)
- Tier 3 비용 (Milestone A 이후)
- Migration Readiness 체크리스트 (각 Milestone)

### myth doctor

```bash
myth doctor                    # 전체 health check
myth doctor --perf-check       # 빌드 프로파일 검증
myth doctor --wsl-check        # WSL2 환경 검증
myth doctor --migration        # 모든 Milestone 상태 요약
```

## 11. 변경 이력

| 날짜 | 버전 | 변경 |
|---|---|---|
| 2026-04-19 | v1.0 | 초기 작성. Decision 1~8 반영. |

---

미래에 이 문서가 수정되려면:
1. 관련 Decision 추가 (DECISIONS.md)
2. 이 문서의 해당 섹션 갱신
3. 변경 이력에 기록
4. CONSTITUTION.md의 관련 Article과 모순 없는지 검증

**v1 API 계약 6개 (섹션 2)는 영구 불변.**
