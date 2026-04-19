# myth — Documentation Index

myth 프로젝트의 **모든 설계 문서**에 대한 내비게이션. 어떤 질문을 하든 여기서 시작.

## 목차

### 최상위 (`~/myth/`)

| 파일 | 역할 | 대상 독자 |
|---|---|---|
| [`README.md`](../README.md) | 프로젝트 첫인상, 5분 요약 | 누구나 |
| [`CONSTITUTION.md`](../CONSTITUTION.md) | 불변 원칙, 법률 문서 | 의사결정자 |
| [`ARCHITECTURE.md`](../ARCHITECTURE.md) | 실행 모델, v1 API 계약, Milestone | 구현자, 운영자 |
| [`DECISIONS.md`](../DECISIONS.md) | 설계 결정 이력 (Decision 1~9) | 설계 이해 |
| [`PROTOCOL.md`](../PROTOCOL.md) | myth-embed wire protocol 스펙 | 프로토콜 구현자 |
| [`WSL2-SETUP.md`](../WSL2-SETUP.md) | 환경 체크리스트 | 설치자 |
| [`THIRD-PARTY.md`](../THIRD-PARTY.md) | 라이선스 귀속 | 법률·컴플라이언스 |

### 설계 문서 (`~/myth/docs/`)

#### 시작 문서

| 파일 | 역할 |
|---|---|
| [`00-INDEX.md`](00-INDEX.md) | 이 문서 |
| [`01-OVERVIEW.md`](01-OVERVIEW.md) | 전체 그림, Day-1 빌드 범위, 철학 |
| [`02-CONCEPTS.md`](02-CONCEPTS.md) | 용어집, 네이밍 맵, 한영 대조 |
| [`03-DIRECTORY.md`](03-DIRECTORY.md) | 디렉토리·파일 레이아웃 (4-way 분산) |

#### Rust Crate 상세 (`04-CRATES/`)

| 파일 | Crate | 역할 |
|---|---|---|
| [`04-CRATES/00-overview.md`](04-CRATES/00-overview.md) | (모두) | 10개 crate 관계도, 의존성 레이어 |
| [`04-CRATES/01-myth-common.md`](04-CRATES/01-myth-common.md) | myth-common | 기반 타입, XDG 경로 |
| [`04-CRATES/02-myth-db.md`](04-CRATES/02-myth-db.md) | myth-db | SQLite + JSONL + audit chain |
| [`04-CRATES/03-myth-gavel.md`](04-CRATES/03-myth-gavel.md) | myth-gavel | The Gavel 판정 로직 |
| [`04-CRATES/04-myth-identity.md`](04-CRATES/04-myth-identity.md) | myth-identity | 3-Tier identity 매칭 |
| [`04-CRATES/05-myth-hooks.md`](04-CRATES/05-myth-hooks.md) | myth-hooks | 6개 hook 바이너리 |
| [`04-CRATES/06-myth-embed.md`](04-CRATES/06-myth-embed.md) | myth-embed | 임베딩 daemon |
| [`04-CRATES/07-myth-orchestrator.md`](04-CRATES/07-myth-orchestrator.md) | myth-orchestrator | 병렬 실행 |
| [`04-CRATES/08-myth-runtime.md`](04-CRATES/08-myth-runtime.md) | myth-runtime | Claude subprocess 래핑 |
| [`04-CRATES/09-myth-ui.md`](04-CRATES/09-myth-ui.md) | myth-ui | ratatui TUI |
| [`04-CRATES/10-myth-cli.md`](04-CRATES/10-myth-cli.md) | myth-cli | 주 CLI, 13개 서브커맨드 |

#### Python 및 인프라

| 파일 | 역할 |
|---|---|
| [`05-PYTHON.md`](05-PYTHON.md) | myth_py 패키지 (Assessor, Observer) |
| [`06-HOOKS.md`](06-HOOKS.md) | Claude Code hook 21개, myth가 쓰는 6개 |
| [`07-STATE.md`](07-STATE.md) | SQLite 스키마 v1, JSONL 포맷, vectors.bin |

#### 실행 계획

| 파일 | 역할 |
|---|---|
| [`08-BUILD-SCOPE.md`](08-BUILD-SCOPE.md) | Day-1 빌드 범위, 8 Wave 순서, DoD |
| [`09-CLAUDE-PROMPTS.md`](09-CLAUDE-PROMPTS.md) | **Claude Code 실행 지시서** (핵심) |
| [`10-VALIDATION.md`](10-VALIDATION.md) | 테스트 전략, fixtures 560, 성능·보안 검증 |
| [`11-RISKS.md`](11-RISKS.md) | 리스크 카탈로그 (T/P/S/O 20개) |
| [`12-DEPLOYMENT.md`](12-DEPLOYMENT.md) | 설치·업그레이드·롤백·운영 |

## 독자별 경로

### "myth가 뭔지 빨리 알고 싶다"

```
README.md (5분)
  ↓
docs/01-OVERVIEW.md (10분)
```

### "구현하러 왔다 (Claude Code)"

```
docs/09-CLAUDE-PROMPTS.md (실행 지시서)
  ↓ 각 Wave에서 참조
docs/04-CRATES/*.md (해당 crate 상세)
  ↓
docs/07-STATE.md (스키마)
PROTOCOL.md (myth-embed)
```

### "Jeffrey — 검토 완료 후 Day-0 신호"

```
CONSTITUTION.md (원칙 재확인)
  ↓
DECISIONS.md (9개 결정 다시 보기)
  ↓
docs/08-BUILD-SCOPE.md (Day-1 범위 체크)
  ↓
docs/09-CLAUDE-PROMPTS.md (Claude Code에 전달할 문서)
```

### "운영자 (Jeffrey, Day-1 이후)"

```
docs/12-DEPLOYMENT.md (설치·운영)
  ↓
docs/10-VALIDATION.md (모니터링 지표)
  ↓
docs/11-RISKS.md (장애 대응)
  ↓
CONSTITUTION.md (의사결정 시)
```

### "기여자 (future)"

```
CONSTITUTION.md
  ↓
DECISIONS.md
  ↓
docs/04-CRATES/00-overview.md
  ↓
각 crate 상세
```

## 문서 간 관계도

```
              CONSTITUTION.md
                    │
                    ▼
              DECISIONS.md
              ┌─────┴─────┐
              ▼           ▼
      ARCHITECTURE.md   01-OVERVIEW.md
      (API 계약,         (전체 그림,
       Milestone)         Day-1 범위)
              │           │
        ┌─────┼───────────┤
        ▼     ▼           ▼
  02-CONCEPTS 03-DIRECTORY 04-CRATES/*
  (용어)      (경로)       (crate 상세)
        │                   │
        ▼                   ▼
   05-PYTHON           06-HOOKS
   (Python)            (hook 시스템)
                           │
                           ▼
                      07-STATE
                      (저장소)
                      
   08-BUILD-SCOPE → 09-CLAUDE-PROMPTS → 10-VALIDATION
   (범위)            (실행 지시)          (검증)
                                             │
                                             ▼
                                        11-RISKS → 12-DEPLOYMENT
```

## 문서 간 참조 규약

각 문서는 **다른 문서를 참조할 때 경로 포함**. 예:

```markdown
자세한 내용은 [`docs/07-STATE.md §2.3`](07-STATE.md#23) 참조.
```

규약:
- 상대 경로 우선 (`docs/07-STATE.md`)
- 절대 경로는 최상위 문서에서만 (`~/myth/...`)
- 섹션 번호 포함 권장

## 문서 수정 흐름

새 결정이 생기면:

```
1. DECISIONS.md 에 Decision N+1 추가 (삭제하지 않음, 누적)
2. 영향받는 문서 수정 (ARCHITECTURE, 04-CRATES, 등)
3. 02-CONCEPTS.md 용어 변경 있으면 갱신
4. 각 문서 "변경 이력" 섹션에 날짜·변경 내용 기록
5. 00-INDEX.md 필요 시 업데이트
```

## 버전·이력

현재 v0.1 기준 (2026-04-19 기점).

각 문서의 "변경 이력" 섹션이 해당 문서의 역사를 보여준다.

전체 릴리스 이력:
- **v0.1.0**: Day-1 릴리스 (초기)
- (이후 Milestone 별로 버전업)

## 빠진 문서?

현재 문서에서 다루지 않는 주제:

- **팀 배포**: v1 범위 밖. 별도 프로젝트.
- **번역 (영어 매뉴얼)**: 기본 영어 + 한국어 병기. 완전 번역본 없음.
- **GUI**: myth는 TUI만. 웹 대시보드 등 계획 없음.
- **모바일**: 범위 밖.
- **CI/CD 통합**: GitHub Actions 등 연동은 future work.

이런 주제가 필요해지면 `docs/` 아래 새 문서 추가.

## 변경 이력

| 날짜 | 변경 |
|---|---|
| 2026-04-19 | 초기 작성. 28개 문서 전체 내비게이션. |
