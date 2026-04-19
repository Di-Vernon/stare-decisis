# myth — Day-1 빌드 범위

이 문서는 **Day-0부터 Day-1 릴리스까지 구현해야 할 것의 전체 목록**이다. `ARCHITECTURE.md`가 "무엇이 있어야 하는가"라면, 이 문서는 "**얼마나 구현하느냐**"와 "**무엇을 나중으로 미루느냐**"의 경계선이다.

헌법 Article 19 (Rough Start)에 따라, Day-1은 완벽을 추구하지 않는다. **실측 가능한 최소 기능**을 Day-1에 구현하고, 나머지는 Milestone 트리거로 미룬다.

## 1. Day-1 활성 기능

Day-1에 구현되어 **즉시 동작**하는 것들.

### 1.1 The Gavel (Pre-execution Blocking)

- [ ] Bedrock Rule YAML 로더 + 3 items / 47 patterns
- [ ] Foundation Rule YAML 로더 + 5-10 items
- [ ] Surface Rule YAML 로더 (프로젝트별 override 지원)
- [ ] RegexSet 기반 정규식 매칭 (런타임 컴파일 + LazyLock 캐시)
- [ ] Grid 5×6 기본 매트릭스 + `grid_overrides` 테이블 적용
- [ ] FatigueTracker (세션당 상한: advisory 2, caution 3, warn 2)
- [ ] Verdict → Hook JSON 변환
- [ ] 560개 test fixture (positive 280 + negative 280)
- [ ] pre-commit hook으로 FP=0 자동 검증
- [ ] 실행체: `myth-hook-pre-tool` binary (mimalloc + LTO fat)

### 1.2 Assessor (Post-failure Analysis)

- [ ] Tier 0: Python `classifier.py` + Rust deterministic classify
- [ ] Tier 1: Variant B engineered template
- [ ] Shadow mode metrics (`reflector-shadow.jsonl`)
- [ ] Pending reflection 감시 (UserPromptSubmit hook)
- [ ] `.claude/agents/assessor.md` (Haiku subagent 정의)
- [ ] Pydantic schema 검증
- [ ] 실행체: `myth-hook-post-tool-failure`, `myth-assessor` (Python)

### 1.3 Observer (Weekly Analysis)

- [ ] caselog 주간 분석 (`analyzer.py`)
- [ ] Lesson top-N 랭킹
- [ ] Lapse score 계산 + status 전환
- [ ] Migration Readiness 5개 (A~E) 평가
- [ ] Tier 1 compliance rate 계산
- [ ] brief.md 생성 + Migration Readiness 섹션
- [ ] 실행체: `myth-observer` (Python, `myth observer run` CLI)

### 1.4 Identity (3-Tier Matching)

- [ ] Tier 1: aggressive normalize + SHA1 hash
- [ ] Tier 2: multilingual-e5-small 임베딩 (via myth-embed)
- [ ] Tier 3: 인터페이스만 (활성은 Milestone A)
- [ ] `trait VectorStore` + `InMemoryStore` 구현
- [ ] `vectors.bin` 포맷 (magic/version/dim/count/generation)
- [ ] `vector_metadata` SQLite 테이블
- [ ] `integrity_check` + 복구 메커니즘

### 1.5 myth-embed Daemon

- [ ] Self-daemonizing 바이너리
- [ ] Unix socket + bincode 프로토콜 v1
- [ ] `Embed`, `Ping`, `Shutdown` 3개 Op
- [ ] `flock`-based spawn race 방지
- [ ] 15분 유휴 자가 종료
- [ ] 실행체: `myth-embed` binary (client/daemon 통합)
- [ ] 투명성: `status`, `stop`, `probe` CLI

### 1.6 Parallel Execution (myth-orchestrator)

- [ ] Wave-based `plan.json` 파싱
- [ ] tmux 세션 관리
- [ ] git worktree 관리
- [ ] Watchdog (hard timeout 11min, stale 2min)
- [ ] 최대 동시 3~4 task
- [ ] Execution report 생성
- [ ] 기존 harness-orchestrator shell 재사용

### 1.7 Storage Layer

- [ ] SQLite schema v1 (migrations/001_initial.sql)
- [ ] PRAGMA 설정 (WAL, synchronous NORMAL, busy_timeout 5000)
- [ ] JSONL writer with `fcntl` flock
- [ ] Merkle audit chain (blake3)
- [ ] 4개 JSONL: caselog, lesson-state, audit, hook-latency

### 1.8 Hook System

- [ ] 6개 hook 바이너리 (pre-tool, post-tool, post-tool-failure, user-prompt, stop, session-start)
- [ ] 모두 binary-per-hook 모드 (Day-1은 데몬화 없음)
- [ ] hook-latency.ndjson 자동 수집
- [ ] `.claude/settings.json` 템플릿 생성 (`myth init`)
- [ ] `CLAUDECODE=1`, `MYTH_DISABLE`, `CLAUDE_REVIEW_ACTIVE` 플래그 처리

### 1.9 CLI (myth-cli)

- [ ] `myth init` — 프로젝트 스캐폴딩
- [ ] `myth install` / `myth uninstall` — 바이너리 심볼릭 링크
- [ ] `myth run` — Claude Code 실행 (interactive + parallel mode)
- [ ] `myth status` — 짧은 요약
- [ ] `myth watch` — TUI 대시보드
- [ ] `myth doctor` — health check (기본 / --perf-check / --wsl-check / --migration)
- [ ] `myth lesson list/show/appeal/retrial/split/merge`
- [ ] `myth observer run [--dry]`
- [ ] `myth embed status/stop/probe`
- [ ] `myth constitution` (less pager)
- [ ] `myth gavel status/stop` — stub 단계 (Milestone C 전 "not running" 반환)

### 1.10 TUI (myth-ui)

- [ ] ratatui + crossterm 초기화
- [ ] 3분할 레이아웃 (caselog / tasks / lessons | brief / migration)
- [ ] 5개 패널 (caselog, tasks, lessons, brief, migration)
- [ ] 마크다운 렌더링 (pulldown-cmark)
- [ ] 구문 강조 (syntect)
- [ ] vim 스타일 키 바인딩
- [ ] 실시간 파일 watch (notify crate, 200ms polling fallback)

### 1.11 Installation & Docs

- [ ] `scripts/install.sh` (cargo build → symlink to ~/.local/bin → init ~/.myth/)
- [ ] `scripts/uninstall.sh`
- [ ] `scripts/pgo-build.sh` (Milestone C 대비 대기 상태)
- [ ] `~/myth/CONSTITUTION.md` (v2.3)
- [ ] 28개 설계 문서 전부 (현재 작업 중)

## 2. Day-1 대기 기능 (코드는 있으나 비활성)

Day-1 빌드에 **포함되지만 flag로 비활성** 상태. 조건 충족 시 토글만으로 활성.

### 2.1 Assessor Tier 2 / Tier 3

- 코드: `myth-hooks/src/bin/stop.rs` (Tier 2 logic)
- 코드: `myth_py/assessor/dispatcher.py` (Tier 3 Anthropic SDK — stub)
- 활성 플래그: `~/.config/myth/config.yaml`
  ```yaml
  assessor:
    tier_2_enabled: false  # Milestone A에서 toggle
    tier_3_enabled: false
  ```
- 활성 조건: Milestone A (21일 사용 후 compliance rate 측정)

### 2.2 Semantic Detection

- 코드: `myth-gavel/src/rules/semantic.rs` (스텁)
- 스튜브 메서드만 정의, 구현 없음
- 활성 조건: Milestone D (Bedrock miss 관찰 시)

### 2.3 API Key Management

- 코드: `myth-cli/src/subcmd/key.rs` (전체 구현)
- UI에서 숨김 (`myth key --help` 시 "post-Milestone A" 안내)
- 활성 조건: Milestone A Tier 3 활성 시 Jeffrey가 수동으로

## 3. Day-1 미구현 (의도적 지연)

코드 자체가 Day-1 빌드에 **없음**. 해당 Milestone 시점에 신규 crate/모듈 추가.

### 3.1 The Gavel Daemon

- 파일: 없음 (v2 신규 작성)
- 조건: Milestone C (P99 > 15ms 2주 + 빌드 프로파일 확인 + WSL2 green + PGO 시도)
- 변경: `myth-gavel`에 daemon mode 추가, `myth-hook-pre-tool`에 client/server 분기

### 3.2 Vector Store 대안

- 파일: `myth-identity/src/store/sqlite_vec.rs` (스텁만, 구현 없음)
- 파일: `myth-identity/src/store/usearch.rs` (스텁만)
- 조건: Milestone B (레코드 20K + P99 > 50ms)
- 구현: `trait VectorStore` 구현체 1개 추가 → config로 선택

### 3.3 AST-Based Validation

- 파일: 없음 (Milestone E에서 신규 crate `myth-ast` 또는 `myth-gavel::ast`)
- 의존성: `tree-sitter-bash`, `ast-grep-core`
- 조건: Milestone E (Milestone D 활성 후 semantic FP>5% 또는 FN>2%)

## 4. 크기 예산

| 영역 | LOC | 빌드 시간 |
|---|---|---|
| Rust (10 crate) | ~11,000 | ~8분 (release) |
| Python (assessor + observer) | ~2,500 | N/A |
| Shell scripts | ~800 | N/A |
| YAML rules | ~600 | N/A |
| Test fixtures (text) | ~2,000 lines | N/A |
| 문서 (md) | ~15,000 줄 | N/A |
| **총 구현** | **~32,000 LOC** | **Day-1 빌드 ~8분** |

## 5. Day-0 → Day-1 작업 순서

**Day-0**: Jeffrey 검토 통과 + 28개 문서 완성. 시작 신호.

**Day-1 (구현 단계)**:

Claude Code가 다음 순서로 작업한다 (상세: `09-CLAUDE-PROMPTS.md`).

```
Wave 0 — 스캐폴딩
  0.1 cargo new --workspace myth
  0.2 10 crate 생성 (--lib or --bin)
  0.3 pyproject.toml + myth_py 디렉토리
  0.4 .cargo/config.toml
  0.5 workspace Cargo.toml 의존성 선언

Wave 1 — Layer 0~1 (기반)
  1.1 myth-common 구현 (types.rs, error.rs, paths.rs)
  1.2 myth-db 구현 (SQLite schema, JSONL writer, audit chain)
  1.3 Wave 1 유닛 테스트 전부 green

Wave 2 — Layer 2 (판정)
  2.1 myth-gavel 구현 (rules, grid, fatigue, verdict)
  2.2 myth-identity 구현 (3-tier matcher, InMemoryStore)
  2.3 myth-embed 구현 (daemon + client + bincode protocol)
  2.4 Wave 2 유닛 테스트 green

Wave 3 — Layer 3 (hooks)
  3.1 myth-hooks 구현 (6개 bin)
  3.2 Variant B 템플릿
  3.3 Wave 3 통합 테스트 (stdin/stdout 왕복)

Wave 4 — Layer 4 (orchestration + runtime + UI)
  4.1 myth-runtime 구현 (Claude subprocess)
  4.2 myth-orchestrator 구현 (tmux + worktree + watchdog)
  4.3 myth-ui 구현 (ratatui 패널들)
  4.4 Wave 4 통합 테스트

Wave 5 — Layer 5 (CLI)
  5.1 myth-cli 구현 (13개 서브커맨드)
  5.2 Wave 5 end-to-end 테스트

Wave 6 — Python
  6.1 myth_py.assessor 구현
  6.2 myth_py.observer 구현
  6.3 Pydantic schema 검증

Wave 7 — Rules & Fixtures
  7.1 Bedrock/Foundation/Surface YAML 작성
  7.2 560개 test fixtures 작성
  7.3 FP=0 검증 통과

Wave 8 — 통합 검증
  8.1 scripts/install.sh
  8.2 실제 프로젝트에서 myth init → myth run → myth observer run 흐름
  8.3 myth doctor 전 항목 green
  8.4 21일 shadow mode 시작 대기 상태
```

**예상 시간**: Claude Code가 full-auto로 돌리면 **5~10일** (Wave 1~8 기준, 병렬 효과 포함).

## 6. Day-1 출시 기준 (Definition of Done)

다음 모두 green이면 Day-1 완료:

- [ ] `cargo test --workspace` 전부 pass
- [ ] `cargo clippy --workspace` warning 0
- [ ] `cargo build --release` 성공
- [ ] 560개 fixtures FP=0
- [ ] `pytest python/tests/` pass
- [ ] `myth doctor` 전 항목 green
- [ ] `myth init` + `myth run` end-to-end 시나리오 성공
- [ ] `myth observer run --dry` 유효한 brief.md 생성
- [ ] `myth-embed probe "hello"` 384-dim 벡터 반환
- [ ] 28개 설계 문서 전부 존재
- [ ] Git initial commit

## 7. Day-1 이후 첫 3주 (Day-22 = Milestone A)

Day-1 릴리스 후 3주간 **무간섭 관찰**.

**실측 수집**:
- `hook-latency.ndjson`에 Pre/Post hook 모든 latency
- `reflector-shadow.jsonl`에 Tier 1 compliance 매 실패마다
- `caselog.jsonl`에 모든 실패 이벤트
- `lesson-state.jsonl`에 모든 상태 변화

**Jeffrey의 역할**:
- 평소처럼 Claude Code 사용
- 가끔 `myth status` 확인
- 주간 brief.md 읽기
- 필요 시 `myth lesson appeal`

**Claude의 역할** (이 세션 또는 후속 세션):
- Day-21 분석 세션에서 shadow metrics 해석
- Tier 1 compliance rate 계산
- Tier 2/3 활성 권고 생성

Day-22에 Jeffrey와 Claude 공동으로 Milestone A 판단.

## 8. 릴리스 버저닝

myth 자체 버전:
- **v0.1**: Day-1 (Rough Start). 모든 Milestone 비활성.
- **v0.2**: Milestone A 이후 Tier 2/3 추가 (있다면)
- **v0.3**: Milestone B 또는 C (먼저 오는 쪽)
- **v1.0**: 모든 Milestone 도달 + 6개월 안정 운영

v0.x는 API 계약 6개 지키되 내부 구조 변경 허용.
v1.0부터는 SemVer 엄격 적용.

Cargo 버전 (`rust/Cargo.toml`):
```toml
[workspace.package]
version = "0.1.0"
```

Python 버전 (`pyproject.toml`):
```toml
version = "0.1.0"
```

Git tag:
```bash
git tag -a v0.1.0 -m "Day-1 release"
```

## 9. 미리 준비해야 하는 것 (Day-0 전제)

Claude Code가 작업 시작 전에 환경에 있어야:

- [ ] `~/myth/` 디렉토리 (현재 작업 중)
- [ ] 28개 설계 문서 전부 (현재 작업 중)
- [ ] `CONSTITUTION.md v2.3` 완성
- [ ] WSL2 환경 체크 (`WSL2-SETUP.md` 기준)
- [ ] Claude Code 2.1.27+ 설치
- [ ] mold, clang, Rust stable, Python 3.11+, tmux
- [ ] 기존 `~/project/harness-orchestrator/` 보존 (참조용)

Day-0 신호 = Jeffrey가 "09-CLAUDE-PROMPTS.md를 Claude Code에 붙여넣자"라고 결정하는 시점.

## 10. 체크리스트 요약

**Day-0 이전 (지금 진행 중)**:
- [x] Phase 1~4 문서 23개
- [ ] Phase 5 문서 5개 (현재 작성)
- [ ] Phase 6 문서 4개

**Day-0 신호 조건**:
- [ ] 28개 문서 완성
- [ ] Jeffrey 검토 통과
- [ ] WSL2 체크리스트 그린
- [ ] CONSTITUTION v2.3

**Day-1 완료 기준** (섹션 6 반복):
- [ ] 모든 테스트 pass
- [ ] 모든 fixture FP=0
- [ ] `myth doctor` green
- [ ] end-to-end 시나리오 성공

## 관련 문서

- `09-CLAUDE-PROMPTS.md` — Claude Code 실행 지시서 (이 문서의 Wave를 Task로 분해)
- `10-VALIDATION.md` — 각 Wave의 검증 방법
- `11-RISKS.md` — 구현 중 예상되는 리스크
- `12-DEPLOYMENT.md` — Day-1 이후 배포·유지보수
