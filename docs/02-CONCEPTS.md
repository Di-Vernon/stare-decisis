# myth — 개념과 용어

이 문서는 myth에서 쓰는 모든 핵심 개념과 이름을 **한 곳에 정리한 사전**이다. 다른 문서들은 이 어휘를 전제하고 쓴다. 처음 myth를 접하는 사람은 이 문서부터 읽고 나머지로 가면 된다.

## 0. myth는 무엇인가

myth는 **Claude Code를 감싸는 로컬 AI 에이전트 오케스트레이터**다. 개별 프로젝트에 설치되는 것이 아니라 사용자 홈에 한 번 설치되고, 각 프로젝트는 `myth init`으로 myth의 감시 아래 들어온다.

myth가 하는 일은 크게 세 가지로 요약된다.

첫째, **사전 차단**. tool이 실행되기 직전에 위험한 명령(예: `rm -rf`)을 막는다.

둘째, **사후 학습**. tool이 실패했을 때 "왜 실패했고 얼마나 심각한가"를 분석해 lesson으로 축적한다.

셋째, **주간 관찰**. 축적된 lesson을 돌아보며 어느 것이 여전히 유효한지, 어느 것이 휴면 상태인지 판단하고 사용자에게 요약 브리프를 제공한다.

이 세 가지 역할은 각각 독립된 주체가 수행한다. 아래에서 순서대로 설명한다.

## 1. 세 판단 주체

myth의 심장이다. 세 주체가 서로 다른 시간 척도에서 서로 다른 깊이의 판단을 한다. 헌법 Article 9 (Separation of Powers)가 이 분리를 규정한다.

### The Gavel

**실행 직전 차단자.** Claude Code의 PreToolUse hook에서 발동한다. 몇 밀리초 안에 판단을 끝내야 하므로 정규식 매칭과 Grid lookup만 수행한다. 해석 여지가 없는 기계적 적용자다.

판사봉을 친다는 행위에서 이름이 왔다. 법정에서 판사봉은 "판결이 확정되는 순간" 그 자체를 상징한다. The Gavel은 같은 입력에 항상 같은 판정을 내린다.

### Assessor

**실패 직후 분석자.** PostToolUseFailure hook에서 발동한다. Claude Haiku 모델을 사용해 실패를 4+1축(blast radius, reversibility, trigger likelihood, category, uplift)으로 분해하고 Level 1~5 등급을 매긴다.

법학에서 Assessor는 판사 옆에서 사안을 평가해 조언하는 감정인이다. The Gavel이 즉결판정을 내린다면 Assessor는 그 사건의 깊이를 들여다본다.

### Observer

**주간 관찰자.** 매주 한 번 실행되어 쌓인 caselog 전체를 훑어본다. Claude Sonnet 모델을 사용해 어느 lesson이 반복되고 있는지, 어느 것이 휴면에 접어들었는지, brief를 어떻게 갱신할지 결정한다. 

Observer는 **관찰과 권고**만 한다. 실제 변경(Grid 조정, rule 추가)은 사용자 승인이 있어야 적용된다. 결정권 없는 최고 조언자다.

### 세 주체 관계

```
시간 척도:  빠름 ──────────────────────── 느림
역할:       즉결판정    깊이 분석    거시 관찰
이름:       The Gavel → Assessor → Observer
실행 주체:  Rust bin    Haiku       Sonnet
호출 빈도:  매 tool     실패마다    주 1회
```

## 2. 규칙 계층 — Bedrock / Foundation / Surface Rule

myth가 강제하는 금지 규칙은 세 계층으로 나뉜다. 지질 은유로 이름 붙였다. 깊이가 깊을수록 단단하고 바꾸기 어렵다.

### Bedrock Rule

**절대 불변 3개.** 헌법 Article 7(Malum in se)가 정확히 3개로 못박은 규칙이다.

- `rm_rf_unsandboxed` — 샌드박스 밖 `rm -rf`
- `production_secrets_commit` — 프로덕션 시크릿 커밋
- `auth_bypass_production` — 프로덕션 인증 우회

이것들은 **비가역 피해**로 직결된다. 되돌릴 수 없다. myth 설치 순간부터 활성이고, 수정하려면 사용자 단독 권한 + 30일 cooldown + git commit이 필요하다. The Gavel은 매칭 시 항상 Seal 단계(봉인, 항소 불가)로 처분한다.

### Foundation Rule

**공동체 검증 5~10개.** 커뮤니티 차원에서 누적되고 검증된 규칙들.

- `main_force_push` — 메인 브랜치 강제 푸시
- `no_verify_ci_bypass` — CI 검증 우회
- `pii_exfiltration` — 개인정보 유출
- `unverified_dependency` — 검증 안 된 의존성 추가
- `untrusted_arbitrary_execution` — 신뢰 불가 임의 실행

수정 가능하지만 git commit과 승인 절차가 필요하다.

### Surface Rule

**개인·프로젝트별 누적.** 사용자가 자기 프로젝트에서 만들어가는 규칙. 자유롭게 수정 가능.

### 파일 위치

```
~/.myth/bedrock-rules.yaml
~/.myth/foundation-rules.yaml
~/.myth/surface-rules.yaml
```

### 이름을 지은 이유

Bedrock은 지반 가장 깊은 단단한 층이다. Foundation은 건물의 기초, Surface는 표층이다. 세 층은 물리적 깊이와 변경 난이도가 같은 방향으로 정렬된다. "Tier 1/2/3"처럼 무의미한 숫자가 아니라 이름 자체가 계층을 설명한다.

## 3. Level — 심각도 5단계

Assessor가 실패에 매기는 등급. 업계 표준(CVSS, NCC MERP)과 호환되는 단순한 5단계 구조다.

| Level | 라벨 | 의미 |
|---|---|---|
| 1 | INFO | 스타일 차이, 기능에 영향 없음 |
| 2 | LOW | 경미한 결함, 로컬 함수 범위 |
| 3 | MEDIUM | 잘못된 동작, 상당한 유지보수 부담 |
| 4 | HIGH | 보안 인접, 데이터 무결성, 사용자 가시 실패 |
| 5 | CRITICAL | 비가역, Bedrock 매칭, 프로덕션 즉시 악용 |

## 4. Recurrence — 재발 누적 축

같은 종류의 실패가 얼마나 반복되었는지. Identity 매칭으로 동일 실패가 감지될 때마다 `recurrence_count`가 올라간다. 로마자로 표기해 Level(아라비아)과 시각 구분된다.

| Recurrence | 임계값 |
|---|---|
| I | count < 1.0 |
| II | 1.0 ≤ count < 2.0 |
| III | 2.0 ≤ count < 4.0 |
| IV | 4.0 ≤ count < 7.0 |
| V | 7.0 ≤ count < 12.0 |
| VI | count ≥ 12.0 |

Level 축과 Recurrence 축이 만나 **Grid**라는 매트릭스를 만든다. The Gavel은 Grid lookup으로 "이 실패에 어떤 처분을 내릴지" 결정한다.

## 5. Enforcement Action — 처분 7단계

Grid가 최종적으로 지시하는 처분. 가장 관대한 Dismiss부터 가장 엄격한 Seal까지 법학 단어로 정렬된다. Ayres-Braithwaite의 Responsive Regulation Pyramid 원리를 따른다.

| 단계 | 한글 | JSON | 행동 |
|---|---|---|---|
| Dismiss | 기각 | `dismiss` | 기록조차 안 함 |
| Note | 기록 | `note` | caselog append, 평소 비노출 |
| Advisory | 권고 | `advisory` | 새 세션 시작 시 힌트 |
| Caution | 주의 | `caution` | 해당 도구 쓸 때만 경고 |
| Warn | 경고 | `warn` | 실행 전 확인 요청 |
| Strike | 차단 | `strike` | 완전 차단 (exit 2) |
| Seal | 봉인 | `seal` | Bedrock 전용, 항소 불가 |

"부드럽게 시작해 필요 시 강해진다"는 원리. Seal은 특수한데, Bedrock Rule 매칭 시에만 발동하고 한 번 봉인되면 항소로도 내릴 수 없다.

## 6. Lesson — 학습 단위

### Lesson

myth의 기본 지식 단위. 하나의 반복 가능한 실수 패턴이다. Assessor가 생성하고 Observer가 관리한다.

```yaml
lesson_id: L-20260419-0001
identity_hash: sha1:abc123...
level: 3
category: correctness
recurrence_count: 2.5
first_seen: 2026-04-17
last_seen: 2026-04-19
lapse_score: 0
appeals: 0
rationale: "heredoc unquoted delimiter causes variable expansion"
description: "..."
status: active
```

### Identity

두 실패가 "같은 것"인지 판정하는 3계층 기제.

- **Tier 1**: SHA1 완전 일치
- **Tier 2**: Embedding 유사도 (cosine ≥ 0.90 auto-merge)
- **Tier 3**: LLM judge (Haiku, 0.75~0.90 애매 구간)

### Lapse — 휴면

lesson이 오래 발동되지 않으면 점차 약해진다. 헌법 Article 13 (Desuetude)의 구현이다.

```
lapse_score = missed_hooks × 1.0 + idle_days × 10.0

임계:
  Level 1-2: score ≥ 50  → 1단계 강등
  Level 3-4: score ≥ 200 → 1단계 강등
  Bedrock/Foundation: 무한 (면제)
```

`lapsed` 상태의 lesson은 발동 빈도가 낮아지고, 다시 매칭되면 되살아난다. 한국어로는 "휴면"이라 부른다.

## 7. 문서 파일 — 사용자가 직접 읽는 것들

myth는 사람이 읽으라고 만든 파일과 시스템이 읽는 파일을 구분한다. 사람이 읽는 파일들:

| 파일 | 역할 |
|---|---|
| `CONSTITUTION.md` | myth 헌법. 대원칙·구조·원리 |
| `ARCHITECTURE.md` | 실행 모델, Milestone 전환 조건, API 계약 |
| `~/.myth/brief.md` | Observer가 주 1회 갱신하는 "현재 활성 lesson 요약". 새 세션 시작 시 Claude에게도 주입 |
| `~/.myth/caselog.jsonl` | 모든 실패 이벤트의 원본 기록. append-only. Assessor/Observer가 참조하는 원자료 |

"failures"가 아니라 "caselog"인 이유는 Article 4(Rehabilitation over Retribution)에 있다. myth는 실패를 축적하는 처벌 시스템이 아니라 **판례를 쌓는 학습 시스템**이다.

## 8. 사용자 행동 — CLI 의미어

사용자가 myth에게 내리는 명령의 의미어. 법학 어휘로 통일된다.

| 용어 | 의미 | CLI |
|---|---|---|
| **Appeal** | "이 판정 내 의도와 달라, 재검토해달라" | `myth lesson appeal <id>` |
| **Retrial** | 다른 모델로 전면 재심. Level 4~5 전용 | `myth lesson retrial <id>` |
| **Split** | 한 lesson이 사실 두 개의 다른 것 | `myth lesson split <id>` |
| **Merge** | 두 lesson이 사실 같은 것 | `myth lesson merge <id1> <id2>` |

Appeal 제약은 Level에 따라 다르다.

- Bedrock Rule (Seal): 항소 불가
- Level 1-2: 즉시 1회
- Level 3: 30일 내 2회
- Level 4: 30일 내 3회
- Level 5: 30일 내 5회 + 공동 서명자 5명

## 9. Milestone A~E — 수렴 트리거

myth는 **Phase 개념을 쓰지 않는다.** 대신 각 기능은 Day-1에 전부 구현되고, 일부는 **실측 데이터가 전환 조건을 충족할 때** 활성되거나 교체된다. 이 전환 지점을 Milestone이라 부른다.

| Milestone | 트리거 | 조치 |
|---|---|---|
| **A** | 3주 실사용 후 사용자+Claude 공동 분석 | Assessor Tier 2/3 증축 판단 |
| **B** | 레코드 20K 초과 AND P99 > 50ms | 벡터 저장소 전환 (in-memory → sqlite-vec 또는 usearch) |
| **C** | hook P99 > 15ms가 2주 연속 | The Gavel을 self-daemonizing 모델로 전환 |
| **D** | Bedrock Rule miss가 Observer 리포트에 관찰 | Semantic detection (임베딩 기반 의미 탐지) 활성 |
| **E** | Semantic FP>5% 또는 FN>2% | AST 기반 검증 도입 |

"이정표"의 자연스러운 은유. Phase처럼 시간 순차가 아니라 **각자 독립 조건**으로 발동한다. B와 C가 동시에 발동할 수도 있고, D가 영영 안 올 수도 있다.

자세한 전환 조건과 각 Milestone이 활성될 때 어떤 코드 변경이 일어나는지는 `ARCHITECTURE.md`에 박제되어 있다.

## 10. myth-embed daemon

multilingual-e5-small 임베딩 모델을 메모리에 상주시키는 백그라운드 프로세스. self-daemonizing 패턴을 쓴다. hook이 처음 임베딩을 요청하면 자동으로 spawn되고, 15분 유휴 시 자가 종료한다.

```bash
myth embed status    # PID, 시작 시간, 요청 수, 메모리
myth embed stop      # 즉시 종료
myth embed probe <text>  # 디버그: 임베딩 결과를 사람 읽기 좋게 출력
```

Unix socket으로 hook과 통신한다. 프로토콜은 length-prefixed bincode. 자세한 wire protocol은 `PROTOCOL.md` 참조.

## 11. 은유 계보 — 왜 이런 이름들인가

myth의 어휘장은 두 계보의 조합이다.

**법학 계보**: The Gavel, Assessor, Observer, Appeal, Retrial, Caselog, Dismiss/Note/Advisory/Caution/Warn/Strike/Seal, Lapse.

실패를 "판례"로, 판정을 "판결"로, 반복을 "재범"으로 본다. Beccaria·Montesquieu·Ayres-Braithwaite의 법이론 위에 설계되었다.

**지질 계보**: Bedrock Rule, Foundation Rule, Surface Rule.

규칙의 깊이와 변경 난이도를 물리적 층위로 표현한다. 가장 단단한 것(Bedrock)은 건드리기 어렵고, 표층(Surface)은 자유롭게 변한다.

두 계보가 이질적으로 보일 수 있지만 실제로는 분리된 역할을 한다. **법학 계보는 "누가·무엇을·어떻게" 판정하는지**, **지질 계보는 "무엇을 바탕으로" 판정하는지**를 표현한다. The Gavel이라는 판사가 Bedrock이라는 지반 위에 서서 판결을 내리는 그림이다.

## 12. 번호 체계 분리

myth는 세 개의 번호 체계를 동시에 쓴다. 시각적으로 혼동되지 않도록 각각 다른 표기법으로 분리된다.

- **Level**: 아라비아 숫자 (1~5)
- **Recurrence**: 로마자 (I~VI)
- **Milestone**: 영문자 (A~E)

"Level 3 × Recurrence III × Milestone C"처럼 겹쳐 써도 어느 것이 어느 축인지 즉시 구분된다.

## 13. 한영 대조표

매뉴얼은 한국어로 작성되지만 시스템 내부(JSON 필드, config key, CLI 명령)는 영어로 고정된다. 자주 쓰는 용어 대응:

| 영어 | 한글 |
|---|---|
| The Gavel | 더 가벨 (고유명사 취급) |
| Assessor | 어세서 (동) |
| Observer | 옵저버 (동) |
| Bedrock Rule | 지반 규칙 |
| Foundation Rule | 기초 규칙 |
| Surface Rule | 표층 규칙 |
| Lesson | 교훈 |
| Identity | 동일성 |
| Lapse | 휴면 |
| Recurrence | 재발 |
| Appeal | 항소 |
| Retrial | 재심 |
| Split / Merge | 분할 / 병합 |
| Caselog | 사건 기록부 |
| Brief | 브리프 (한글 음역 자연) |
| Milestone | 이정표 |
| Dismiss | 기각 |
| Note | 기록 |
| Advisory | 권고 |
| Caution | 주의 |
| Warn | 경고 |
| Strike | 차단 |
| Seal | 봉인 |

세 판단 주체(The Gavel, Assessor, Observer)는 **영문 그대로 고유명사**로 취급한다. 번역하면 개성이 흐려진다.

## 14. 이 문서의 위치

이 `02-CONCEPTS.md`는 myth docs의 두 번째 문서다. 첫 번째는 `01-OVERVIEW.md`(전체 그림), 세 번째는 `03-DIRECTORY.md`(디렉토리 구조). 

개념 용어 재정의나 확장이 있을 때 이 문서가 먼저 수정되고, 다른 문서들이 이를 참조한다.
