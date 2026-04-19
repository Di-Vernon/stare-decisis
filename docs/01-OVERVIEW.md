# myth — Overview

이 문서는 myth 설계의 **전체 그림**을 한 번에 보여준다. 세부는 다른 문서로 분산되어 있고, 이 문서는 "**이 시스템이 대략 무엇인가**"를 빠르게 파악하게 해준다.

처음 myth를 접하는 사람은 `02-CONCEPTS.md`(용어) → `01-OVERVIEW.md`(이 문서, 전체 그림) → `ARCHITECTURE.md`(실행 모델) 순으로 읽으면 된다.

## 1. myth가 해결하려는 문제

Claude Code를 오래 쓰다 보면 세 가지 문제가 누적된다.

첫째, **같은 실수의 반복**. 같은 heredoc 따옴표 문제, 같은 Python venv 누락, 같은 `rm -rf` 위험 — Claude는 세션이 바뀌면 이전에 배운 것을 잊는다. 사용자는 같은 피드백을 반복해서 줘야 한다.

둘째, **재앙적 명령의 사전 차단 부재**. Claude Code 자체에는 "이 명령은 비가역 피해 발생"을 알아채는 기제가 제한적이다. `rm -rf /`, 프로덕션 시크릿 커밋, 인증 우회 같은 것들이 실행 직전에 멈춰야 한다.

셋째, **학습의 구조화 부재**. 어떤 실수는 Level 1(미미), 어떤 실수는 Level 5(치명)다. 이걸 구분 없이 모두 같은 무게로 기록하면 금세 노이즈가 된다. 심각도와 재발 빈도에 따라 다른 대응이 필요하다.

myth는 이 세 문제에 대한 로컬 답이다.

## 2. 핵심 아이디어 3줄 요약

1. **The Gavel** — tool 실행 직전에 47개 정규식으로 비가역 위험을 막는다.
2. **Assessor** — tool 실패 직후 Claude Haiku가 4+1축으로 분석해 Level·category를 매긴다.
3. **Observer** — 매주 한 번 Claude Sonnet이 모든 기록을 돌아보며 패턴을 찾고 brief를 갱신한다.

이 세 주체가 **시간 척도가 다른 세 판단**을 각각 수행한다. 빠름(ms) → 중간(수 초) → 느림(분).

## 3. Day-1 빌드 범위

myth는 **Phase 단계로 쪼개지 않고** Day-1에 모든 기능을 구현한다. 일부는 비활성 상태로 대기하다가 실측 조건(Milestone A~E)이 충족되면 활성된다.

### Day-1에 구현되어 활성 상태

**학습 레이어**:
- The Gavel: Bedrock Rule 47개 정규식, Foundation Rule, Surface Rule 전체
- Grid (Level × Recurrence 매트릭스)
- Assessor: Tier 0 deterministic classifier + Tier 1 Option B (engineered Variant B template)
- Observer: 주간 분석, brief.md 재생성, Migration Readiness 리포트
- Identity 3-Tier: SHA1, Embedding (multilingual-e5-small), LLM judge 설계
- Vector Store: in-memory + mmap + `trait VectorStore` 추상화
- Lapse tracking (Article 13 Desuetude)

**인프라**:
- myth-embed daemon (self-daemonizing, Unix socket, bincode)
- SQLite WAL (state.db)
- JSONL append 로그 (caselog, lesson-state, audit)
- Merkle audit log
- Appeal system (Level별 제약)
- Config hot-reload

**CLI**:
- `myth run`, `myth install`, `myth init`
- `myth status`, `myth doctor`
- `myth lesson` (list/show/appeal/retrial/split/merge)
- `myth observer run`
- `myth gavel status/stop`
- `myth embed status/stop/probe`
- `myth constitution`

**TUI**:
- ratatui 기반 대시보드
- 마크다운 렌더링 (syntect + pulldown-cmark)
- 진행률 표시, 키 바인딩

**Parallel execution** (기존 harness-orchestrator 통합):
- Claude Squad 연동
- tmux + worktree 병렬
- `harness-orchestrator/lib/execute.sh` 재활용

**Fallback**:
- Max quota 소진 시 Extra Usage
- API key 없는 graceful degradation
- Daemon 없는 regex-only fallback

### Day-1에 구현되어 대기 상태

- **Tier 2/3 Assessor**: PostToolUseFailure + UserPromptSubmit + Stop hook 3개 이벤트는 등록되고, Stop block 로직과 Anthropic SDK dispatcher는 빌드에 포함. `enable_tier2: false`, `enable_tier3: false` 플래그로 비활성. Milestone A 발동 시 토글.

- **Semantic detection**: bge-small 또는 코드 임베딩 모델 통합 코드. The Gavel의 정규식 clear 경로에 semantic check 훅. Milestone D까지 비활성.

- **AST 검증**: tree-sitter-bash + ast-grep-core 의존성 추가 가능하나 Day-1 빌드 미포함. Milestone E 도달 시 crate 추가.

- **The Gavel daemon**: self-daemonizing 모드 로직은 **미구현**. Milestone C 트리거 시 추가 개발.

### Day-1 미구현 (의도적)

- **The Gavel daemon** (위 설명 참조)
- **API key 관리 인프라**: `myth key set/show/clear`는 Milestone A 시점에 개발. Day-1엔 CLI에 미노출.
- **sqlite-vec / usearch** 벡터 저장소 대안: Milestone B 발동 시 crate 추가.

## 4. 기술 스택

### Rust (60~65%)

**용도**: hook 임계 경로, 데몬, CLI 메인.

**주요 crate**:
- Workspace: `~/myth/rust/` 아래 10개 crate
- Runtime: 없음 (sync 위주). Tokio는 daemon용만 (current_thread)
- DB: `rusqlite` (bundled SQLite)
- 정규식: `regex-automata` 직렬화 DFA
- 직렬화: `serde` + `serde_json` (작은 페이로드), `bincode` (Unix socket)
- 임베딩: `fastembed-rs` (ONNX int8)
- Allocator: `mimalloc` global
- UI: `ratatui` + `crossterm`, `syntect`, `pulldown-cmark`

### Python (30~35%)

**용도**: Assessor/Observer LLM dispatch, Agent SDK 통합, 복잡한 JSON schema 검증.

**주요 패키지**:
- `anthropic` (Milestone A Tier 3 활성 시)
- `pydantic` (JSON schema)
- `typer` (하위 CLI, myth-cli가 주체)

### Shell (5%)

**용도**: install.sh, pgo-build.sh, 기존 `harness-orchestrator/lib/` 재활용.

## 5. 예상 크기

| 영역 | LOC |
|---|---|
| Rust (10 crate) | ~11,000 |
| Python (assessor + observer) | ~2,500 |
| Shell scripts | ~800 |
| 정규식 rule files (YAML) | ~600 |
| Test fixtures | ~2,000 (텍스트) |
| 문서 (md) | ~15,000 (한글·영어) |
| **총합** | **~32,000 LOC** |

## 6. 외부 의존 요약

### 필수 (Day-1)
- Claude Code 2.1.x+ (공식 CLI)
- Claude Max 구독
- Rust toolchain (stable)
- Python 3.11+
- SQLite 3.45+ (Ubuntu 24.04 기본)
- mold linker
- tmux (병렬 실행)
- WSL2 Ubuntu 24.04

### 라이선스 차용
- gitleaks (MIT) — 40개 anchored provider prefix
- detect-secrets (Apache-2.0) — keyword+entropy 패턴

### 런타임 downloads
- multilingual-e5-small ONNX int8 (~116MB) — 첫 실행 시

## 7. myth의 "왜" — 철학

### Master Principle

```
완벽은 도달이 아니라 수렴이다.
수렴은 우연이 아니라 법이다.
```

완벽한 규칙을 처음부터 설계할 수 없다. 대신 **수렴 메커니즘**을 설계한다:
- 실패 → Assessor 분석 → lesson
- 반복 → Recurrence 축 상승 → enforcement 강화
- 휴면 → Lapse → 자동 강등
- 주간 관찰 → Observer 권고 → 사용자 승인 → Grid 진화

이 루프가 **법칙처럼 작동**하게 하는 것이 myth 설계 목표.

### 법학 은유

myth는 **Beccaria / Montesquieu / Ayres-Braithwaite**의 법이론 위에 설계되었다.

- **Beccaria**: "처벌의 확실성이 엄격성을 지배한다". → 우리는 사후 처벌이 아닌 **사전 차단(The Gavel) + 확실한 기록(caselog)**을 강조.
- **Montesquieu**: 권력 분립. → 세 주체(The Gavel, Assessor, Observer)가 서로 다른 시간 척도·도구·권한으로 분리.
- **Ayres-Braithwaite Responsive Regulation Pyramid**: 부드럽게 시작, 필요 시 강해짐. → Dismiss/Note/Advisory/Caution/Warn/Strike/Seal 7단계.

### Rough Start 원칙

헌법 Article 19: **"증거 없는 상태 구조 금지"**. 

- 미리 최적화하지 않는다.
- 기능은 필요 시점에 추가하지 않고, Day-1에 구현하되 활성만 실측 기반.
- Phase 대신 Milestone: **조건이 충족될 때 전환**.

## 8. myth가 하지 않는 것

**myth는 Claude Code의 대체품이 아니다.** Claude Code를 감싸는 wrapper. Claude Code의 모든 기능은 그대로 작동.

**myth는 프로젝트 코드를 리팩토링하거나 테스트하지 않는다.** 그건 Claude Code의 역할.

**myth는 외부 네트워크를 관찰하지 않는다.** 로컬 hook 기반. 예외는 Milestone A 이후 Tier 3 Anthropic API 호출(명시 opt-in).

**myth는 팀 단위 동기화를 하지 않는다.** v1은 단일 사용자, 로컬 전용. 팀 동기화는 별도 프로젝트 범위.

**myth는 "완벽한" 금지 리스트를 주장하지 않는다.** Bedrock 47개 규칙도 실측 튜닝 대상 (Research #5 FP 검증으로 조정).

## 9. 다음 읽어볼 문서

- `02-CONCEPTS.md` — 모든 용어 정의
- `ARCHITECTURE.md` — 실행 모델 + Milestone 전환 조건
- `03-DIRECTORY.md` — 디렉토리·파일 레이아웃 상세
- `04-CRATES/00-overview.md` — Rust crate 간 의존 관계

구현 단계에서는:
- `08-BUILD-SCOPE.md` — Day-1 빌드 범위 체크리스트
- `09-CLAUDE-PROMPTS.md` — Claude Code 구현 지시서

## 10. 변경 이력

| 날짜 | 버전 | 변경 |
|---|---|---|
| 2026-04-19 | v1.0 | 초기 작성. Decision 1~8 + 네이밍 재작업 반영. |
