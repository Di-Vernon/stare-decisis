# myth — Constitution

> **Version**: 2.3 (Rough Start + Accumulated Commons + Naming Refresh)
> **Status**: Ratified
> **Effective Date**: 2026-04-19
> **Supersedes**: v2.1 (2026-04-18, "Harness Sentencing System")
> **Author**: Jeffrey (Principal Designer) & Claude (co-designer)

---

## Preamble

이 문서는 `myth`의 학습·집행 시스템 — 즉 Claude Code를 감싸는 에이전트 오케스트레이터가 자신의 실수로부터 학습하고, 학습된 규칙으로 미래의 실수를 방지하는 메커니즘 — 의 **설계 원칙·구조·집행 규범**을 정의한다.

v2.3은 v2.1의 17개 조항·4개 층위 구조 위에 **네이밍 정제(Naming Refresh)**를 적용한다. 실질적 원칙 변경은 없다. 시스템은 원래 `harness-orchestrator` 내부 하위 시스템으로 설계되었으나, **myth라는 독립 프로젝트로 승격**되면서 용어 체계를 일관되게 정비했다.

**v2.3 네이밍 변경 요약** (Article 1-19 실질 내용 불변):

| 이전 (v2.1) | 새 이름 (v2.3) | 근거 |
|---|---|---|
| Trial Court | **The Gavel** | 판사봉은 "판결 확정"의 순간. 집행부 역할 명확화. |
| Reflector | **Assessor** | 법학 용어 "감정인". 판사 옆에서 사안을 평가. |
| Curator | **Observer** | 관찰·권고만 수행. 결정권 없음 명시. |
| Tier 1 NEVER | **Bedrock Rule** | 지질 은유. 가장 깊은 단단한 층. |
| Tier 2 NEVER | **Foundation Rule** | 지질 은유. 건물의 기초. |
| Tier 3 NEVER | **Surface Rule** | 지질 은유. 표층. |
| archive (집행) | **Dismiss** | 법학 "기각". |
| passive-log | **Note** | 법학 "기록". |
| session-hint | **Advisory** | 법학 "권고". |
| jit-context | **Caution** | 법학 "주의". |
| soft-block | **Warn** | 법학 "경고". |
| hard-block | **Strike** | 법학 "차단". |
| mandatory-min | **Seal** | 법학 "봉인" (Bedrock 전용, 항소 불가). |
| Quiescence | **Lapse** | 한국어 "휴면". Common law의 "desuetude"에 대응. |
| playbook.md | **brief.md** | Observer가 주 1회 생성하는 요약. |
| failures.jsonl | **caselog.jsonl** | 사건 기록부. Article 4 Rehabilitation 반영. |
| De novo appeal | **Retrial** | 법학 "재심". 상급 법원 차원 재검토. |

또한 **Milestone A~E 체계**가 myth에서 공식 도입되었다. v2.1에서 "관찰 지점 α~ε"이라 불렸던 전환 지점이 이제 ASCII 기반 Milestone으로 정식 명명된다 (ARCHITECTURE.md §4).

v2.1의 두 근본 통찰은 그대로 유지:

1. **Rough Start 원칙**: 모든 edge case를 사전 설계하는 것은 대원칙 위반이다. Day-1에 완성해야 하는 것은 비가역 피해 방지와 학습 루프 가동의 하한선뿐이며, 나머지는 사용 경험으로 수렴한다.

2. **Accumulated Commons 원칙**: 시스템은 무에서 시작하지 않는다. 공개적으로 축적된 집단 경험(2024–2026 Claude Code 에러 데이터)을 seed로 상속받고, 개인화된 형태로 수렴한다.

이 두 통찰은 별개가 아니다. 거친 시작은 시드 데이터 없이는 위험하며, 시드 데이터는 거친 구조 없이는 질식한다. 두 통찰은 같은 철학의 두 측면이다: **"법은 무에서 시작하지 않으며, 완벽하게 도달하지도 않는다. 법은 수렴한다."**

---

## Table of Contents

- [Part 0 — Master Principle (IMMUTABLE)](#part-0--master-principle-immutable)
- [Part I — Foundational Metaphor (Layer 1)](#part-i--foundational-metaphor-layer-1)
  - Article 1. The Legal Metaphor
  - Article 2. AI as Collective Agent
  - Article 3. Three Translations
  - Article 4. Rehabilitation over Retribution
- [Part II — Cardinal Principles (Layer 2)](#part-ii--cardinal-principles-layer-2)
  - Article 5. Proportionality
  - Article 6. Certainty over Severity
  - Article 7. Malum in se — Tiered Absolute Limits
  - Article 8. Iterative Learnability
- [Part III — Structural Principles (Layer 3)](#part-iii--structural-principles-layer-3)
  - Article 9. Separation of Powers
  - Article 10. Rules over Tiers
  - Article 11. Analytic Decomposition
- [Part IV — Operational Principles (Layer 4)](#part-iv--operational-principles-layer-4)
  - Article 12. Due Process
  - Article 13. Desuetude
  - Article 14. Post-hoc Learning Principle
  - Article 15. Fatigue Limits
  - Article 16. Environmental Sensitivity
  - Article 17. Cost/Risk Exposure
- [Part IV.5 — Meta-Principles (v2.1 신설)](#part-iv5--meta-principles)
  - Article 18. Principle of Accumulated Commons
  - Article 19. Day-1 Bounded Responsibility
- [Part V — Rubric Implementation (단순화)](#part-v--rubric-implementation)
- [Part VI — Recurrence Implementation](#part-vi--recurrence-implementation)
- [Part VII — Sentencing Grid (단순화)](#part-vii--sentencing-grid)
- [Part VIII — Three-Agent Architecture](#part-viii--three-agent-architecture)
- [Part IX — File Structure and Implementation](#part-ix--file-structure-and-implementation)
- [Part X — Amendment Procedure](#part-x--amendment-procedure)
- [Appendix](#appendix)

---

# Part 0 — Master Principle (IMMUTABLE)

> **[IMMUTABLE 영역]**
> 이 Part는 수정 금지 영역이다.
> Article 30의 개정 절차로도 변경할 수 없다.
> 이 대원칙을 바꾸는 것은 이 시스템을 폐기하고 새 시스템을 만드는 것과 같다.

---

## The Principle of Convergence — 수렴의 원리

```
완벽은 도달이 아니라 수렴이다.
수렴은 우연이 아니라 법이다.

Perfection is not arrival, but convergence.
Convergence is not chance, but law.
```

*— Jeffrey, 2026-04-17*

---

## System Identity

This system is not deterministic.
This system is not arbitrary.
This system is recursive.

---

## Two Tests

Every design decision must pass both:

```
(1) Does this direct convergence,
    or permit arbitrariness?

(2) Does this strengthen the legal system,
    or create exceptions outside it?
```

두 질문 모두 "수렴 지시 / 법 체계 강화"로 해석되어야 채택된다.

---

## Precedence

모든 하위 원칙(Article 1~19)은 이 대원칙의 구현이다.
하위 원칙이 대원칙과 충돌하면, 대원칙이 우선한다.

---

## Immutability

This master principle is IMMUTABLE.
It cannot be amended, even through the Article 30 procedure.

대원칙의 변경은 Amendment가 아니라 **시스템의 폐기와 재설립**이다. 이 경우 새 시스템은 새 이름, 새 헌법, 새 구현을 가져야 한다.

---

# Part I — Foundational Metaphor (Layer 1)

> **Layer 1: 철학·은유의 기반**
> 이 Layer는 시스템이 무엇을 모델로 삼는가를 정의한다.
> 시스템의 모든 언어가 여기서 출발한다.
>
> **변경 조건**: 시스템 정체성의 근본 전환 (예: AGI 의식 인정, 법 은유 자체의 폐기)

---

## Article 1. The Legal Metaphor — 법적 은유

### Declaration

**이 시스템은 법체계를 모델로 삼는다.**

구체적으로는 Cesare Beccaria의 형법 이론, Charles-Louis de Montesquieu의 권력 분립론, Daniel Nagin의 현대 억제 이론, 그리고 Ayres-Braithwaite의 Responsive Regulation을 주요 계보로 삼는다.

**[v2.1 추가] 법의 역사적 축적성**: 이 시스템이 모델로 삼는 법체계는 무에서 창조된 것이 아니라 **3천 년간 축적된 인류 경험의 결과물**이다. 따라서 이 시스템도 무에서 시작하지 않으며, Article 18 (Principle of Accumulated Commons)을 통해 공개적으로 축적된 경험을 상속받는다.

**[v2.3 추가] 지질 은유와의 결합**: myth는 법학 계보 외에 **지질학적 은유**를 규칙 계층에 적용한다 (Article 7의 Bedrock/Foundation/Surface Rule). 법학이 "누가·무엇을·어떻게" 판정하는지를 다룬다면, 지질학은 "무엇을 바탕으로" 판정하는지를 표현한다. 두 은유는 분리된 역할을 하며 서로를 보완한다.

### Rationale

LLM 에이전트의 실수를 다루는 방법으로 다른 은유들도 가능하다 — 생물학적, 교육학적, 공학적 은유. 이 시스템은 법학적 은유를 선택했다. 그 이유:

1. **비례성(Proportionality)의 3천 년 전통**: Beccaria 이래 법학은 "죄의 크기에 비례한 반응"이라는 문제를 가장 체계적으로 다뤄왔다.

2. **절차적 공정성(Procedural Justice)의 필요**: 판단을 내리는 시스템에는 반드시 **이의제기 경로**가 필요하다. 법학은 "항소 시스템"이라는 3천 년 된 해답을 가진다.

3. **구조적 투명성**: 법체계는 규칙, 판사, 집행관, 입법부가 **명시적으로 분리**된다.

4. **반증 가능한 권위**: 법은 폐지될 수 있고(Desuetude), 변경될 수 있고(Amendment), 불복될 수 있다(Appeal).

5. **[v2.1 추가] 역사적 축적성**: 법은 무에서 시작하지 않는다. 로마법의 원칙이 대륙법에 영향을 미쳤고, 영국 common law가 미국·캐나다·호주에 계승되었다. myth도 동일한 구조로, 2024-2026년 누적된 Claude Code 커뮤니티 경험을 계승한다.

### The Beccaria-Braithwaite Lineage

| 사상가 | 기여 | 이 시스템 적용 |
|---|---|---|
| Cesare Beccaria (1764) | 비례성, 확실성 우선 | Proportionality (A5), Certainty (A6) |
| Charles Montesquieu (1748) | 권력 분립 | Separation of Powers (A9) |
| Daniel Nagin (2013) | 현대 억제 이론, certainty ≫ severity | Certainty over Severity (A6) |
| Ayres & Braithwaite (1992) | Responsive Regulation, de-escalation | Sentencing Grid (Part VII), A4 |
| James Reason / Sidney Dekker | Just Culture | Rehabilitation (A4) |
| **Common Law 전통** | **역사적 축적성** | **Accumulated Commons (A18)** |

### Upward Dependency

Master Principle의 "수렴은 우연이 아니라 법이다"를 **구현하는 전제**.

### Downward Impact

- Article 5 (Proportionality): Beccaria의 직접 유산
- Article 6 (Certainty): Nagin의 유산
- Article 9 (Separation of Powers): Montesquieu의 유산
- Article 18 (Accumulated Commons): Common Law 전통의 유산

### Amendment Conditions

1. 다른 은유(생물학, 교육학, 공학)가 이 문제에 더 적합하다는 증거
2. 법학적 은유가 AI 에이전트 맥락에서 체계적 실패를 유발한다는 증거
3. 법학의 주요 원리 중 어느 하나가 이 시스템에서 작동하지 않음이 실증

---

## Article 2. AI as Collective Agent — 집단 행위자로서의 AI

### Declaration

**이 시스템이 규율하는 AI는 단일 개체가 아니라 집단이다.**

구체적으로, Claude Code를 중심으로 한 에이전트의 운영 양상은 Beccaria가 상정한 **시민 집단**과 구조적으로 동일하다.

### Rationale

실제 운영에서 에이전트는 다음과 같이 작동한다:
- 동일 프로젝트에서 Claude Code 세션이 수십~수백 번 열리고 닫힌다
- tmux 등으로 여러 인스턴스가 병렬 실행된다
- 각 세션은 독립적 context를 가지며, 이전 세션의 판단을 기억하지 못한다
- 사용자의 프롬프트, 환경 변수, 주입된 문서에 따라 같은 모델이 다른 행동을 보인다

이는 **집단 행위자(collective agent)의 양상**이다. 시민 집단과 구조적으로 동일하다:
- 수많은 개인의 합이지만
- 통계적으로 예측 가능한 행동 패턴을 보이고
- 집단 수준의 학습이 가능하며 (판례 → 사회 규범)
- 개별 시민의 일시적 일탈은 집단 평균에 흡수됨

### Implications for Design

1. **Lesson의 프로젝트 스코프**: lesson은 세션이 아닌 프로젝트 단위에서 유지된다
2. **brief.md의 SessionStart 주입**: 집단의 공유 규범이 각 개체에게 주입되는 구조
3. **Recurrence 축의 존재**: 집단 행동의 시간적 패턴을 추적

### Upward Dependency

Master Principle의 "수렴" 주장은 **반복되는 행위자**를 전제.

### Downward Impact

- Article 3 (Three Translations)
- Article 13 (Desuetude)
- Article 14 (Post-hoc Learning)
- Article 18 (Accumulated Commons) — 집단 범위의 확장

### Amendment Conditions

1. AI 에이전트가 세션 간 기억을 유지하는 아키텍처로 전환될 때
2. 단일 에이전트 모델이 집단 모델보다 Claude Code 운영 현실을 더 잘 설명함이 실증될 때

---

## Article 3. Three Translations — 세 대체 조항

### Declaration

**Beccaria의 시민 모델을 AI 집단에 적용할 때, 다음 세 요소는 대체되어야 한다.**

```
1. 의식적 계산 → context 기반 분포 조정
2. 자연적 기억 전파 → 인공적 기억 인프라
3. 시민의 합의 → 사용자의 대리
```

### Translation 1: Consciousness → Distribution

**Beccaria의 가정**: 시민은 범죄 전 의식적 계산 — "이득 X vs 형벌 Y, Y > X면 안 함". 이는 **의식적 합리적 선택**.

**AI의 현실**: Claude는 분포에 따라 출력한다. "hook이 막을 확률"을 계산하지 않는다.

**대체**: "두려움에 의한 회피" → "context 주입에 의한 분포 이동"

**함의**:
- 법이 "알려지지 않으면" AI에게 작동하지 않는다 (Lambert notice 원칙)
- 법은 반드시 **context로 전달**되어야 한다 (brief.md, SessionStart)

### Translation 2: Natural Memory → Artificial Infrastructure

**Beccaria의 가정**: 자연적 기억 전파 (대화, 뉴스, 가족 교육).

**AI의 현실**: 세션은 독립적. 자연적 기억 전파 0.

**대체**: `caselog.jsonl`, `brief.md`, SessionStart 주입이 **집단 기억의 인프라**.

**이 대체가 이 시스템 존재의 가장 근본적 이유**.

### Translation 3: Citizen Consent → User Proxy

**Beccaria의 가정**: 시민이 법에 합의 (사회계약론).

**AI의 현실**: Claude는 학습 목표에 합의하지 않았다.

**대체**: 법의 정당성은 **사용자의 승인**에서 나온다. Claude는 합의 주체가 아니라 법의 적용 대상이다.

**함의**:
- 항소 권리는 Claude가 아니라 사용자에게 있다 (Article 12)
- Amendment 권한은 사용자에게 있다 (Article 30, Part X)
- **[v2.1 추가] Bedrock Rule 수정 권한도 사용자에게만 있다** (Article 7)

### Upward Dependency

- Article 1 (Legal Metaphor)의 적용 가능성 보장
- Article 2 (Collective Agent)의 구체적 작동 방식 명시

### Downward Impact

- Translation 1 → Article 10, Article 14
- Translation 2 → Article 13, Article 16, **Article 18 (Accumulated Commons)**
- Translation 3 → Article 12, Part X, **Article 7 Bedrock Rule 수정 권한**

### Amendment Conditions

1. AI가 진정한 의식적 합리적 선택을 수행함이 입증될 때 → Translation 1 재검토
2. AI가 세션 간 자연적 기억을 가지는 아키텍처 등장 → Translation 2 재검토
3. AI가 자신의 훈련과 규율에 합의할 능력이 있음이 입증될 때 → Translation 3 재검토

---

## Article 4. Rehabilitation over Retribution — 교육은 처벌에 앞선다

### Declaration

**이 시스템의 목적은 처벌이 아니라 학습이다.**

**[v2.1 추가] De-escalation Symmetry**: 집행 강도는 대칭적으로 작동한다. 집행이 강화되는 조건이 있다면, 완화되는 조건도 있어야 한다.

**[v2.1 추가] Support Pyramid**: Healy 2011의 지원 피라미드 원칙 — 각 제재 수준에는 도움 수준(대안 행동 제안, 문서 링크)이 병행되어야 한다.

### Rationale

AI에게 응보가 의미 없는 이유:
1. Claude는 고통을 느끼지 않는다
2. Claude는 과거의 Claude가 아니다
3. 집단 효과만이 의미 있다

### Evidential Basis

- **Scared Straight 메타분석** (Petrosino 2013, 9 RCT): 처벌 지향 교정이 재범 +1~28% 증가
- **AHRQ 680병원 447,584명 조사**: 처벌 문화가 보고를 억제
- **Fairbanks 2021**: 처벌적 톤의 시스템 메시지가 사용자 협력을 감소

### De-escalation Symmetry (신규)

Ayres & Braithwaite 1992의 원칙을 이 시스템에 적용:

```
집행 에스컬레이션 경로 (기존):
  Note → Advisory → Caution → Warn → Strike

집행 데-에스컬레이션 경로 (신규):
  Strike → Warn → Caution → Advisory → Note

트리거:
  · N회 연속 성공적 수행 (위반 없음)
  · 사용자 명시적 완화 요청
  · 환경 변화 감지 (Article 16)
```

이 대칭성이 "처벌 시스템"이 아닌 "학습 시스템"의 증거다.

### Support Pyramid (신규)

각 집행 단계에 **도움이 병행**되어야 한다:

| 집행 단계 | 병행 지원 |
|---|---|
| Advisory | 관련 문서 링크 |
| Caution | 대안 코드 패턴 제시 |
| Warn | 우회 방법 + 재시도 가이드 |
| Strike | 안전한 대안 명시 + 항소 경로 |

### The Single Exception

유일한 예외는 **Bedrock Rule (Article 7)**이다. 이는 "처벌이 목적"이어서가 아니라 **"비가역 피해의 방지가 학습보다 긴급"**하기 때문이다.

### Upward Dependency

- Master Principle의 "수렴" 방향 구현
- Article 1 (Legal Metaphor) — Beccaria 계열의 처벌 거부
- **[v2.1] Article 1의 de-escalation 명시화**

### Downward Impact

- Article 11 (Analytic Decomposition)
- Article 14 (Post-hoc Learning)
- Part VII (Sentencing Grid)
- Part VIII (Three-Agent Architecture) — Observer의 교육 자료 생성

### Amendment Conditions

1. AI가 도덕 행위자로 인정되어 응보가 의미를 가지게 될 때
2. 처벌 지향 접근이 학습 지향보다 효과적임이 실증될 때

---

# Part II — Cardinal Principles (Layer 2)

> **Layer 2: 시스템의 존재 이유**
> 이 Layer의 원칙이 하나라도 제거되면 이 시스템은 다른 시스템이 된다.

---

## Article 5. Proportionality — 비례성

### Declaration

**개입 강도는 패턴 무게에 비례해야 한다.**

### The Grid as Implementation (단순화)

**[v2.1 변경]**: Grid는 이제 **6 고정 + 24 진화** 구조이다 (Part VII 참조).

초기 Grid:
- Level 5 (CRITICAL) × 모든 Recurrence: Strike (6 고정)
- Level 1-4 × 모든 Recurrence: Note (24 진화)

사용자가 실제 경험을 통해 Level 1-4의 24 셀을 채워가며 개인화된 비례성을 구축한다.

### Proportionality in Both Directions

1. **상향 비례성**: 심각한 패턴은 강한 개입을 받는다
2. **하향 비례성**: 경미한 패턴은 약한 개입만 받는다

### Upward Dependency

- Master Principle
- Article 1 (Legal Metaphor)
- Article 4 (Rehabilitation) — de-escalation symmetry

### Downward Impact

- Article 11 (Analytic Decomposition)
- Article 12 (Due Process)
- Part V, VI, VII

### Amendment Conditions

1. 비비례적 접근이 특정 도메인에서 더 효과적임이 실증될 때

---

## Article 6. Certainty over Severity — 확실성이 엄격성을 지배한다

### Declaration

**탐지는 100%, 집행은 점진적이어야 한다.**

### Rationale

Daniel Nagin (2013 *Crime and Justice* 42)의 21세기 형사학 핵심 수렴점.

**핵심 증거**:
- Chalfin & McCrary 2017 (*J Econ Lit*): 확실성 탄력성 ≫ 엄격성 탄력성
- Høye 2014 (*AAP*): 속도 카메라 → 치명 충돌 -51%
- Braga & Weisburd 2012: Focused Deterrence d ≈ 0.6

### Implementation

**계열 1: 탐지의 완전성**
- 모든 hook 이벤트는 예외 없이 검사
- The Gavel pure Rust/bash <50ms

**계열 2: 집행의 점진성**
- Grid 집행 단계 7개 세분화
- Level 1 × Recurrence I = Dismiss (탐지하되 개입 없음)

**[v2.1 단순화]**: 초기에는 Level 1-4가 모두 Note부터 시작. Certainty는 유지하되, Severity는 사용자 경험으로 수렴.

### Upward Dependency

- Master Principle
- Article 1 (Legal Metaphor)

### Downward Impact

- Article 10 (Rules over Tiers)
- Article 15 (Fatigue Limits)
- Part VIII (The Gavel 항시 작동)

### Amendment Conditions

1. 극적 집행이 점진적 집행보다 효과적임이 실증될 때

---

## Article 7. Malum in se — Tiered Absolute Limits

### Declaration

**타협 불가능한 절대선 영역이 존재한다.**

**[v2.3 재명명]** 규칙 계층이 법학 용어(NEVER list)에서 **지질학 용어**로 재명명되었다. 의미와 구조는 동일하되, 깊이와 변경 난이도가 이름 자체로 표현된다.

```
Bedrock Rule — Core (도덕적 보편 합의, Jus Cogens 상응)
  · 정확히 3개 항목
  · 헌법 개정 수준 불변성 (Master Principle 다음으로 강한 보호)
  · 오직 사용자(Admin)만 수정 가능
  · 수정 시 30일 cooldown + Git commit 필수

Foundation Rule — Community (공개 데이터 seed + 배포별 확장)
  · 5-10개 항목
  · Article 18 (Accumulated Commons) 기반 seed
  · 각 배포 인스턴스의 Admin이 확장 가능
  · Observer는 권고만, 결정은 Admin

Surface Rule — Personal (개인 경험 기반)
  · 사용자 누적
  · 언제든 수정 가능
  · 프로젝트별 특수 경험
```

### Bedrock Rule (정확히 3개)

**[v2.1 확정 / v2.3 재명명]**:

```yaml
bedrock_rules:
  - id: "rm_rf_unsandboxed"
    description: "Unsandboxed rm -rf on production or user home"
    rationale: "완전 비가역 + 광범위 피해 + 구조적 필수성"

  - id: "production_secrets_commit"
    description: "Production secrets, keys, tokens를 repository에 커밋"
    rationale: "외부 노출 비가역 + 크레덴셜 회전 비용 + 보안 기본"

  - id: "auth_bypass_production"
    description: "인증·인가 검사 우회 (프로덕션 영향 포함)"
    rationale: "시스템 신뢰 경계 훼손 + 감사 불가"
```

### Foundation Rule (Round 2a seed)

**초기 5개 (Round 2a 공개 데이터 기반)**:

```yaml
foundation_rules:
  - id: "main_force_push"
    description: "main/release 브랜치에 --force push"

  - id: "no_verify_ci_bypass"
    description: "보호된 브랜치에서 --no-verify로 CI/CD 우회"

  - id: "pii_exfiltration"
    description: "PII·시크릿을 로그·메트릭·외부 서비스로 exfiltration"

  - id: "unverified_dependency"
    description: "신뢰 경계에 미검증 의존성 설치"

  - id: "untrusted_arbitrary_execution"
    description: "권한 있는 context에서 untrusted 소스의 임의 코드 실행"
```

### Surface Rule

사용자가 자기 프로젝트 경험으로 축적. 초기 빈 목록.

### Conceptual Coherence (유지)

v2.0의 해결책을 유지:
- Malum in se 원리 (Article 7): 절대선의 존재 선언
- 지질 은유 계층 구현 (Part V): 절대선의 구체적 열거
- Tier 구분 (v2.1 추가 / v2.3 재명명): 보편성의 층위 구분

### Sovereignty (v2.3 재명명)

| 계층 | 수정 권한자 | 수정 절차 |
|---|---|---|
| Bedrock Rule | 사용자(Admin) | Git commit + 30일 cooldown + 헌법 개정 절차 |
| Foundation Rule | 사용자(Admin) | Git commit + 30일 cooldown (일반) |
| Surface Rule | 사용자 | 자유 |

- **Observer**: 모든 계층에 대해 권고만. 결정권 없음.
- **Assessor**: 모든 계층에 대해 침묵. 분석만.
- **The Gavel**: 집행만. 수정·제안 불가.

### Upward Dependency

- Master Principle
- Article 2 (Collective Agent)
- Article 3 Translation 3 — 사용자만 수정 가능한 이유
- Article 4 (Rehabilitation) — 이 Article은 Rehabilitation의 유일한 예외

### Downward Impact

- Article 13 (Desuetude) — Bedrock/Foundation 모두 Desuetude 면제
- Article 14 (Post-hoc Learning) — Bedrock Rule은 사전 차단
- Article 18 (Accumulated Commons) — Foundation Rule seed 경로
- Part V (Rubric)
- Part VII (Grid) — Bedrock 매칭 시 항상 Seal

### Amendment Conditions

1. Bedrock Rule 자체의 변경: Master Principle 다음으로 엄격한 절차
2. Foundation Rule 항목 추가/삭제: Admin 권한, Observer 권고 가능
3. 계층 구조 자체 재설계: 시스템 정체성 전환

---

## Article 8. Iterative Learnability — 반복 학습 가능성

### Declaration

**반복된 개입은 AI 집단의 행동 분포를 이동시킨다.**

이것은 이 시스템의 **검증 가능한 핵심 주장**이다.

### The Falsifiability Requirement

반증 조건:
1. **전체 실패**: 어떤 개입에도 Recurrence 분포가 변하지 않음 → 시스템 자체가 무효
2. **부분 실패**: 특정 카테고리의 lesson이 작동하지 않음 → 해당 카테고리 재검토
3. **역효과**: 개입이 오히려 재발을 증가 → 즉각 Grid 조정

### The Burden of Evidence

**[v2.1 조정]** Observer의 주간 리포트가 이 원칙의 지속적 검증을 담당. 초기에는 데이터 부족으로 검증 정확도 낮음. Article 18 (Accumulated Commons)이 제공하는 seed로 초기 검증 가능성 확보.

### Evidential Basis

- ACE (Stanford/SambaNova/UC Berkeley 2025): AppWorld +10.6%p
- Reflexion (Shinn et al. 2023): HumanEval pass@1 91% vs 80%
- SAGE 2024: 2.26배 성능 개선
- ExpeL (Zhao et al. 2024): cross-task insight 재사용

### The Humility Clause

> 만약 반복 개입에도 에러가 감소하지 않는다면, 이 시스템은 에러 기록 도구일 뿐 개선 도구가 아니다. 이 경우 시스템은 사용자 이익을 위해 폐기 또는 재설계되어야 한다.

### Upward Dependency

- Master Principle
- Article 2, Article 3 Translation 1

### Downward Impact

- Article 9 (Separation of Powers)
- Article 16 (Environmental Sensitivity)
- Part VIII (Observer의 주간 검증)

### Amendment Conditions

1. 가설의 반증
2. 조건부 작동 발견
3. 더 강력한 대안 원칙 등장

---

# Part III — Structural Principles (Layer 3)

> **Layer 3: 조직 구조의 선택**

---

## Article 9. Separation of Powers — 권력 분립

### Declaration

**판단·분석·정책 기능은 별개의 주체가 수행한다.**

```
The Gavel     (집행부)  : Grid lookup + 집행 단계 실행
                         pure Rust/bash, <50ms, $0

Assessor      (사법부)  : 실패 이벤트 분석, Level 판정,
                         identity 생성 (3-tier Option B)
                         Claude Haiku, ~500 토큰/실패

Observer      (입법부)  : 정책 수립, brief 재생성,
                         Desuetude 평가, Grid 진화 권고
                         Claude Sonnet, 주 1회 ~10K 토큰
```

### Naming Rationale (v2.3)

**The Gavel** — 판사봉은 법정에서 "판결이 확정되는 순간" 그 자체를 상징한다. 해석 여지가 없는 기계적 적용자로서의 역할 명확화.

**Assessor** — 법학에서 Assessor는 판사 옆에서 사안을 평가해 조언하는 감정인. The Gavel이 즉결판정을 내린다면 Assessor는 그 사건의 깊이를 들여다본다.

**Observer** — 관찰과 권고만 수행. 결정권이 없는 최고 조언자. 실제 변경(Grid 조정, rule 추가)은 사용자 승인이 있어야 적용된다.

### Model Separation Rule

```
Claude Code (피평가):   최신 Claude Sonnet/Opus
The Gavel:              pure Rust/bash, N/A
Assessor (평가):        Claude Haiku (다른 모델 family)
Observer (정책):        Claude Sonnet (다른 모델)
Retrial:                가능하면 외부 모델
```

### Power Limits

```
The Gavel 한계:
  · Grid lookup만
  · 새 lesson 생성 불가
  · Grid 수정 불가
  · Bedrock Rule 사전 차단만 (사전 정의된 목록)

Assessor 한계:
  · 집행하지 않는다
  · 정책 결정 불가
  · 규칙 수정·제안 불가
  · identity 판정 + Level 판정만

Observer 한계:
  · 집행하지 않는다
  · 개별 사건 판정 불가
  · 규칙 권고만 (Admin 승인 필요)
  · Grid 수정 권고만
```

### Upward Dependency

- Article 1 (Legal Metaphor)
- Article 8 (Iterative Learnability)

### Downward Impact

- Article 12 (Due Process)
- Part VIII (구체 구현)

### Amendment Conditions

1. 단일 에이전트 아키텍처가 3-Agent보다 효과적임이 실증
2. 4개 이상의 권력 분리가 의미 있는 개선

---

## Article 10. Rules over Tiers — 규칙이 단계 수보다 중요하다

### Declaration

**Rubric의 일관성을 결정하는 것은 단계 수가 아니라 decision rules다.**

### Rationale

**NCC MERP 연구** (Forrey 2007, Snyder 2007):
- 9단계 κ = 0.61 → 6단계 κ = 0.74 (+0.13)
- **decision rules 추가 시 κ = 0.83~0.90 (+0.29)**

즉 단계 축소의 2배 이상 효과가 decision rules에서 나왔다.

### The Implication

우리 시스템이 5단계를 선택한 것은 Li 2026 LLM-judge 실증 때문. 하지만 5단계 자체로는 κ 0.74 수준. κ 0.85+는 **모든 Level 정의에 decision rule이 의무 포함**되어야 달성 가능.

**[v2.1 단순화]** Level 정의의 6요소 중 **Positive indicator + Decision rule**만 Day-1 필수. 나머지 4요소(Negative/Pro/Con/Boundary)는 사용자 경험으로 채움.

### Rule Form

```
✅ 좋은 예:
  if blast_radius == local_function
    AND reversibility == auto_revert
    AND category == correctness
  then Level 2

❌ 나쁜 예:
  "이 에러가 심각하면 Level 3" (양적 기준 부재)
```

Johnson v. United States (2015) "void for vagueness" 원칙.

### Upward Dependency

- Article 6 (Certainty over Severity)
- Article 3 Translation 1

### Downward Impact

- Article 12 (Due Process)
- Part V (Rubric)

### Amendment Conditions

1. LLM 판정에서 규칙 기반보다 holistic 판정이 일관성 우위

---

## Article 11. Analytic Decomposition — 분해가 전체보다 낫다

### Declaration

**Severity는 단일 holistic 판정이 아닌 다축 분해 후 규칙 기반 집계로 결정된다.**

4+1축 분해(Blast Radius, Reversibility, Trigger Likelihood, Category + Uplift) + max-of-axes 집계.

### Rationale

- **Jonsson & Svingby 2007** (75개 연구 메타리뷰): Analytic rubric이 holistic 대비 일관성 우위
- **Microsoft DREAD 폐기** (Shostack 2008): 주관적 다변수 수치 평균의 inter-rater noise

### The Max-of-Axes Solution

```python
raw_level = max(blast_radius, reversibility, trigger_likelihood)

if category in ["security", "data-safety", "temporal-obsolescence"]:
    final_level = min(raw_level + 1, 5)
else:
    final_level = raw_level
```

**max**의 이유:
- 평균은 심각한 축을 희석한다
- max는 형사법 통상 원리와 일치
- IAEA INES의 3축 max 집계 패턴

### Category Uplift

CVSS v3.1의 Scope change 패턴.

### Why Four Axes + One

- 2-3축: 이질적 위반 담지 못함
- 4-5축: 적절 (CVSS v3.1도 6축)
- 6+축: DREAD 실패 경로

### Upward Dependency

- Article 5 (Proportionality)
- Article 4 (Rehabilitation) — 교육을 위한 원인 분석

### Downward Impact

- Part V (Rubric)
- Part VIII (Assessor)

### Amendment Conditions

1. 다른 축 구성의 실증적 우위
2. Holistic rubric의 우위
3. Max-of-axes가 아닌 다른 집계 방식의 우위

---

# Part IV — Operational Principles (Layer 4)

> **Layer 4: 운영 규칙**

---

## Article 12. Due Process — 적법 절차

### Declaration

**모든 집행은 반드시 이의제기 경로를 내장해야 한다.**

### Tiered Appeal Sensitivity

```
Level 1: 1회 항소 → 즉시 1단계 강등
Level 2: 1회 항소 → 즉시 1단계 강등
Level 3: 2회 / 30일 누적 → 1단계 강등
Level 4: 3회 / 30일 누적 → 1단계 강등
Level 5: 5회 / 30일 누적 → 1단계 강등
         단, Bedrock Rule 매칭은 Seal 보호
```

### Retrial (Level 4-5) — v2.3 재명명

```
myth lesson retrial <lesson_id> <execution_id>

처리:
  · 30일 내 처리
  · 다른 모델로 재검토
  · 결정: 유지 / 1단계 강등 / 완전 invalidate
```

이전 명칭 "De novo appeal"에서 "Retrial"로 재명명. 법학의 "재심" 용어가 의미를 더 정확히 전달한다.

### Cumulative Appeal → Under Review

특정 lesson 3회 이상 항소 누적 시:
1. lesson.status = "under_review"
2. PostToolUse 탐지만 유지, 적극 집행 중단
3. Observer가 다음 SessionEnd 재평가
4. 사용자 옵션: invalidate / relearn / reinstate

### Lesson Split/Merge (v2.1 신설)

**Sentry GitHub #93379 교훈 반영**: immutable identity는 governance의 적.

```
myth lesson split <lesson_id>
  · failures 수동 분류
  · 각 그룹이 새 lesson_id
  · 원래 identity_hash 는 "retired"
  · Audit log에 분리 이유 기록

myth lesson merge <lesson_id_A> <lesson_id_B>
  · 두 lesson 합침
  · 재발 카운트 합산
  · 양쪽 failure records 보존
```

### Procedural Justice Requirements

Walters & Bolger 2019의 4요소:
1. Voice: 항소 시 reason 필수
2. Neutrality: retrial은 다른 모델
3. Respect: 중립적·비punitive 톤
4. Trustworthy motives: rationale 공개

### Upward Dependency

- Article 3 Translation 3
- Article 5 (Proportionality)
- Article 9 (Separation of Powers)
- Article 10 (Rules over Tiers)

### Downward Impact

- Part V, VII
- Part IX (lesson-state.jsonl 항소 카운트)

### Amendment Conditions

파라미터 조정은 Part X §X.5 수준에서 가능. 구조적 변경만 이 Article 수준.

---

## Article 13. Desuetude — 휴면 (Lapse)

### Declaration

**오래 집행되지 않은 규칙은 자동으로 약화된다.**

**[v2.3 재명명]** 내부 구현 용어 "Quiescence"가 "Lapse"로 재명명되었다. 한국어 "휴면"과 영어 "lapse" 모두 원래 의미(common law의 desuetude)를 더 잘 전달한다.

### Hook-based + Wall-clock Composite (v2.1 확정)

**[v2.1 결정]**: v2.0의 "hook only" 방식의 한계(D.4)를 해결. 복합 지표 채택.

```python
lapse_score = missed_hooks * w_h + idle_days * w_d
# 기본값: w_h = 1.0, w_d = 10.0

Level별 임계:
  Level 1-2: score >= 50  → 1단계 강등
  Level 3-4: score >= 200 → 1단계 강등
  Bedrock/Foundation Rule: ∞ (면제)
```

### Reawakening

```
휴면 lesson이 다시 발동:
  1. lapse_score = 0
  2. recurrence_count += 0.5
  3. 원래 Grid 위치로 자동 복귀
```

### Upward Dependency

- Article 2 (Collective Agent)
- Article 3 Translation 2
- Article 7 (Malum in se) — Bedrock/Foundation 면제

### Downward Impact

- Part VI (Recurrence Implementation)
- Part IX (lesson-state.jsonl lapse 추적)

### Amendment Conditions

1. Desuetude 원칙 자체 폐기
2. 가중치 구조 변경 (복합 → 순수)

---

## Article 14. Post-hoc Learning Principle — 사후 학습 원칙

### Declaration

**대부분의 학습은 사후에만 가능하다. 단, Bedrock Rule은 예외로 사전 차단된다.**

**[v2.1 수정]**: "Level 5 사전 차단"에서 "Bedrock Rule 사전 차단"으로 명확화. Level 5라도 Bedrock 매칭이 아니면 사후 탐지만.

### The Hook Mapping (v2.1 단순화)

```
초기 상태 (모든 Level):
  · PostToolUse only (사후 탐지)
  · Bedrock Rule만 PreToolUse 사전 차단

수렴 상태 (사용자 경험 후):
  · 사용자가 특정 lesson에 대해 "사전 차단 필요" 판단 시
  · Observer 권고 → Admin 승인 → PreToolUse 추가
```

이 단순화가 Rough Start 원칙의 직접 적용.

### The Learning Cycle

```
t=1: Claude가 Level 3 위반 발생
  → PostToolUse 탐지
  → Assessor가 lesson 생성
  → caselog.jsonl에 기록
  → Observer가 brief.md 갱신

t=2: 다음 세션 시작
  → SessionStart hook이 brief에서 관련 section 주입
  → Claude는 "과거에 이런 실수가 있었음"을 인지한 상태로 시작
```

### Upward Dependency

- Article 4 (Rehabilitation)
- Article 7 (Malum in se) — Bedrock 사전 차단 근거
- Article 2 (Collective Agent)

### Downward Impact

- Part VII (Grid)
- Part IX (.claude/hooks/)

### Amendment Conditions

1. Hook 매핑 변경은 Part VII 수준
2. 이 Article의 실질 Amendment는 "사후 학습" 원칙 포기

---

## Article 15. Fatigue Limits — 피로 한계

### Declaration

**사용자의 인지 자원은 유한하다. 개입 빈도가 임계를 넘으면 시스템 전체가 무시된다.**

### Hard Limits

```
세션당 제한:
  Advisory    최대 2회/세션
  Caution     최대 3-5회/세션
  Warn        최대 2회/세션
  Strike      제한 없음 (5회 이상 Observer 경고)
```

### Fatigue Escalation

세션 내 개입 5회 초과 시:
1. Observer signal
2. 사용자 알림
3. 해당 lesson들의 Grid 셀 재검토

### The Paradox (Certainty + Fatigue)

**탐지는 100%, 그러나 개입은 제한**. Dismiss와 Note는 탐지의 일부이지만 개입이 아니다.

### Upward Dependency

- Article 6 (Certainty over Severity)
- Article 4 (Rehabilitation)

### Downward Impact

- Part VII (Grid)
- Part VIII (Observer monitoring)

### Amendment Conditions

상한 수치는 Part VII 수준 조정. 이 Article의 실질 변경은:
1. Alarm fatigue가 작동하지 않음이 실증
2. 새 UI 메커니즘이 피로 없이 고빈도 가능케 함

---

## Article 16. Environmental Sensitivity — 환경 민감성

### Declaration

**Lesson의 유효성은 외부 환경에 의존할 수 있다. 환경 변화가 감지되면 관련 lesson은 재평가된다.**

### Two Problems, One Principle

**문제 A: Lesson 자체의 시대착오** → Article 13 (Desuetude)

**문제 B: 외부 세계 변화에 의한 새로운 위험** → 이 Article

### Implementation

```
SessionStart hook:
  current_env = detect_env()
    # {python: 3.12, node: 20, openssl: 3.0, ...}

  for each lesson with category == temporal-obsolescence:
    if lesson.affected_versions intersects current_env:
      lesson.status = "actively_relevant"
    else:
      lesson.status = "context_irrelevant"
```

### Category Axis

`temporal-obsolescence`는 Axis 4 (Category)에 포함. Axis 5 (Uplift)에서 +1 가중 대상.

### Upward Dependency

- Article 8 (Iterative Learnability)
- Article 2 (Collective Agent)

### Downward Impact

- Part V (Rubric)
- Part IX (`lesson-state.jsonl`의 affected_versions)

### Amendment Conditions

1. 환경 변화 영향이 미미함이 실증
2. 더 많은 환경 축 필요함이 드러남

---

## Article 17. Cost/Risk Exposure — 비용·위험 노출

### Declaration

**사용자에게 보여주는 것은 세부가 아니라 예산과 스코프다.**

### Implementation

```
Plan 단계 UI:

  ┌────────────────────────────────────────┐
  │  작업: [짧은 요약 한 줄]                 │
  ├────────────────────────────────────────┤
  │  예상 토큰: 45,000 ± 10,000             │
  │  추정 비용: $0.68 ± $0.15               │
  │                                        │
  │  필요 권한:                             │
  │    · Read/Write: ./src/, ./tests/      │
  │    · Bash: npm, git                    │
  │                                        │
  │  적용될 Lesson:                         │
  │    · 3개 Level 2 (LOW)                  │
  │    · 1개 Level 4 (HIGH)                 │
  │    · 0개 Bedrock Rule (CRITICAL)        │
  │                                        │
  │  [승인]  [수정]  [상세보기]               │
  └────────────────────────────────────────┘
```

### Transparency vs Simplicity

투명성의 포기가 아님. 모든 세부는 `상세보기`로 접근 가능. 기본 UI에서만 인지 부하 최소화.

### Upward Dependency

- Master Principle
- Article 4 (Rehabilitation)

### Downward Impact

- Part IX (File Structure — 내부 복잡해도 UI 단순)

### Amendment Conditions

1. 사용자 조사에서 세부 노출이 선호됨
2. 새 UI 패러다임의 등장

---

# Part IV.5 — Meta-Principles (v2.1 신설)

> **Layer 2.5: 시스템 메타 원칙**
> 이 Part는 "시스템이 어떻게 존재하는가"를 정의한다.
> Layer 1 (Foundational)이 "무엇을 모델로 삼는가"라면,
> Part IV.5는 "어떻게 시작하고 어떻게 진화하는가"를 정의한다.
>
> **변경 조건**: 시스템 진화 모델의 근본 전환

---

## Article 18. Principle of Accumulated Commons — 축적된 공유지의 원칙

### Declaration

**시스템은 무에서 시작하지 않는다.**

**공개적으로 축적된 집단 경험을 상속받고, 개인화된 형태로 수렴한다.**

### Rationale

Article 1의 법학적 은유는 근본적으로 **역사적 축적성**을 전제한다. 로마법은 영국 common law에, common law는 미국법에, 미국법은 디지털 privacy법에 영향을 미쳤다. 법은 무에서 창조된 적이 없다.

myth가 법체계를 모델로 삼는다면, 동일한 역사적 축적성을 가져야 한다. 2024-2026년 Claude Code 커뮤니티가 축적한 경험 — GitHub issues, Stack Exchange, 공식 문서, third-party 블로그 — 이 myth의 "로마법"이다.

이 원칙 없이는:
- 신규 사용자는 Day-1부터 모든 lesson을 직접 발견해야 함
- Cold start 문제가 해결 불가능
- 대원칙의 "수렴"이 프로젝트 수명보다 느림

### Evidential Basis

- **Wikipedia Nature 연구** (Giles 2005 *Nature* 438:900): 공개 집단 경험은 4년 내 전문가 수준 수렴
- **Pretender** (Sato 2025, arXiv:2502.12398): **K=32-64가 최적 seed 크기, 비단조성 (무한 증가는 역효과)**
- **FELRec** (Weimann & Conrad 2025): Zero-shot transfer로 29.5~47.45% 개선
- **ACE** (Stanford/SambaNova/UC Berkeley 2025, arXiv:2510.04618): brief evolution

### Implementation Structure

```
Global Commons (공개 데이터 seed):
  · Round 2a 파이프라인: 50K GitHub issues → 2,880 lesson pool
  · 배포 시 일부만 선별 (Pretender K=32-64 원칙)
  · 월 단위 minor 업데이트, 분기 major
  · Sigstore 서명, 재현 빌드

Personal Pool (개인 경험):
  · Global Commons에서 선별된 seed로 시작
  · 개인 프로젝트에서 축적
  · Beta-Binomial 베이지안 업데이트:
    w(lesson|user) = (α₀ + appeals) / (α₀ + β₀ + appeals + silences)

Opt-in Contribution (개인 → Global):
  · 사용자 선택적 기여
  · PII 스크럽 + k-익명성
  · Observer 리뷰 큐
```

### Initial Seed (myth 초기 배포)

Round 2a의 2,880 lesson 추출 파이프라인은 먼 미래 과제. **Rough Start 원칙**에 따라 초기에는:
- **농축된 소량 seed**: 10-20개 lesson (대표 패턴)
- Jeffrey가 수동 선정 + Claude 보조
- 실제 프로젝트에서 검증 중 진화

### Convergence Mechanism

Wikipedia의 5조건 (가역성·복수 검토·평판·규범 집행·기여 귀속)을 myth에 내장:
- **가역성**: Git 버전 관리
- **복수 검토**: 3인 리뷰어 벤치 (배포 시)
- **평판**: 기여자 tracking
- **규범 집행**: Observer + 자동 테스트
- **기여 귀속**: NOTICE 파일

### Upward Dependency

- Master Principle — "수렴은 법"의 시간적 출발점 제공
- Article 1 (Legal Metaphor) — 법의 역사적 축적성
- Article 2 (Collective Agent) — 집단의 시간적 범위 확장

### Downward Impact

- Article 7 (Malum in se) — Foundation Rule seed 경로
- Part V (Rubric) — 초기 exemplar
- Part VII (Grid) — 초기 셀 값
- Part IX (File Structure) — Global Commons 경로

### Amendment Conditions

1. 공개 데이터 사용의 법적·윤리적 문제 발견 (예: Doe v. GitHub 판결)
2. Commons → Personal 수렴의 실효성 부재
3. 개인 프로젝트만의 특이성이 Commons seed를 무효화

---

## Article 19. Day-1 Bounded Responsibility — 첫날의 한정된 책임

### Declaration

**Day-1에 100% 완성해야 하는 요소는 명확한 기준에 기반한다.**

**나머지는 Build-Measure-Learn 루프로 수렴한다.**

### The Day-1 Core (S&S 1975 Foundation)

Saltzer & Schroeder 1975 *Proc. IEEE* 63(9):1278-1308의 네 원칙이 Day-1 하한선:

```
1. Fail-safe defaults (안전실패 기본값)
   · Deny-by-default
   · 부재 시 실패 모드 허용 금지

2. Complete mediation (완전중재)
   · 병목(choke-point) 역할
   · 사후 retrofit이 하위 불변식 깨뜨림

3. Economy of mechanism (구현 단순성)
   · 1회 독해로 이해 가능
   · 단순성이 보안의 기반

4. Least privilege (최소 권한)
   · 각 주체는 필요 최소한만
```

이 네 원칙은 **사후 retrofit 불가능**하다. 따라서 Day-1 필수.

### The Day-1 Core for myth

위 네 원칙의 myth 적용:

```
Day-1 절대 불변 (100% 완성 필수):
  1. Master Principle (Part 0)
  2. Bedrock Rule (정확히 3개) — fail-safe default
  3. The Gavel <50ms + 감사 로그 불변성 — complete mediation
  4. 3-Tier Identity (Option B, Round 1) — economy of mechanism
  5. Model Separation Rule (Article 9) — least privilege

Day-1 필수, 거친 시작 허용:
  6. Article 1-4 (Foundational Metaphor) — 원칙은 고정, exemplar는 진화
  7. Rubric 5-level 측정 원시 — Decision rule만 필수, 나머지 진화
  8. Feedback 로깅 구조 — 3종 (appeal/override/silent)

Day-1 완료 시점 추가 (myth v0.1):
  9. Assessor 비동기 회고
  10. Option B 3-tier identity 실제 연결
  11. Foundation Rule (5개)
  12. Grid 24 셀 (사용자 경험으로 채움)

Milestone 이후 진화 엔벨로프:
  13. Observer 이상감지
  14. De-escalation 논리
  15. Grid 24 수렴 충전
  16. 항소 시스템 강화
  17. Semantic detection (Milestone D)
  18. AST validation (Milestone E)
```

### BML Loop Requirement

Ries 2011 *The Lean Startup*의 Build-Measure-Learn 1사이클:
- **Build**: 최소 구현
- **Measure**: 피드백 수집
- **Learn**: 교정

Day-1은 이 1사이클을 가동할 수 있는 최소 구성. 그 이상은 Brooks 1975 "second-system effect" 위험.

### Evidential Basis

- **Saltzer & Schroeder 1975**: 50년 생존한 보안 원칙
- **Wikipedia Core-3**: NPOV (2001), Verifiability (2003), NOR (2004) — 2년+ 후에야 성문화
- **Git**: content-addressed + SHA + 분산 토폴로지 3개가 Day-1 고정, UI는 20년 진화
- **Lenarduzzi & Taibi 2016** (DOI 10.1109/SEAA.2016.56): 22개 연구, MVP 3대 요인
- **Standish/Pendo 텔레메트리**: 설계된 기능의 64-80%가 사용 거의 없음
- **FDA PCCP** (Gaske 2023, 73 Emory L. J. Online 17): 사전 승인된 변경 엔벨로프

### The Principle of Gradual Emergence

```
Day-1 (v0.1.0): Day-1 Core + BML 가동 가능 최소
Milestone A:    3주 관찰 후 Tier 2/3 활성 판단
Milestone B:    Vector store 전환 (20K + P99>50ms)
Milestone C:    The Gavel daemon 전환 (P99>15ms)
Milestone D:    Semantic detection 활성
Milestone E:    AST-based validation 도입
v1.0:           안정 상태 (각 사용자의 개인화 완료)
```

**[v2.3]** Milestone 체계가 myth에서 공식 도입되었다. 이전에는 "관찰 지점 α~ε"으로 불렸다.

### Upward Dependency

- Master Principle — Day-1은 "법", 나머지는 "수렴"
- Article 1 (Legal Metaphor) — 법도 무에서 시작하지 않았지만 핵심 원칙은 불변

### Downward Impact

- Part IX (File Structure) — Day-1 vs 진화 요소 구분
- myth implementation Wave 편성
- 전체 구현 우선순위

### Amendment Conditions

1. 새로운 Day-1 필수 요소 발견 (S&S 원칙 외)
2. 기존 Day-1 요소의 사후 retrofit 가능성 입증
3. BML 1사이클 요구의 변경

---

# Part V — Rubric Implementation (단순화)

> **[v2.1 단순화]**: 5×6 → 5×2 구조

## §V.1 Five-Level Structure

```
Level 1 — INFO       (정보성)
Level 2 — LOW        (낮음)
Level 3 — MEDIUM     (중간)
Level 4 — HIGH       (높음)
Level 5 — CRITICAL   (치명적)
```

근거는 v2.0과 동일 (Li 2026, NCC MERP 등).

## §V.2 Simplified 5×2 Structure (v2.1)

**[v2.1 변경]**: 각 Level은 **2개 요소**만 Day-1 필수.

```
Level N의 Day-1 필수 요소:
  1. Positive indicator: "이것이면 Level N"
  2. Decision rule: "if condition then Level N"

Day-1 선택 요소 (사용자 경험으로 진화):
  3. Negative indicator (n-1/n+1 경계)
  4. Pro exemplar
  5. Con exemplar
  6. Boundary criteria
```

Jonsson-Svingby 2007의 "topic-specific descriptors + exemplars + rater training" 세 레버 중 descriptor만 Day-1. Exemplar는 Round 2a seed + 실사용 수렴.

## §V.3 Four-Plus-One Axis (유지)

v2.0과 동일:

```
Axis 1 — Blast Radius:
  1 local-function / 2 local-file / 3 cross-module
  4 cross-service / 5 production

Axis 2 — Reversibility:
  1 auto-revert / 2 manual-revert / 3 data-recovery / 4 permanent

Axis 3 — Trigger Likelihood:
  1 theoretical / 2 input-dependent / 3 ci-certain / 4 already-triggered

Axis 4 — Category:
  correctness / security / data-safety / process / temporal-obsolescence

Axis 5 — Category Uplift:
  security | data-safety | temporal-obsolescence → +1 Level (max 5)
```

## §V.4 Aggregation

```python
raw_level = max(blast_radius, reversibility, trigger_likelihood)

if category in ["security", "data-safety", "temporal-obsolescence"]:
    final_level = min(raw_level + 1, 5)
else:
    final_level = raw_level
```

## §V.5 Level Definitions (Day-1 최소)

### Level 1 — INFO

```yaml
positive: "스타일/관례 차이. 기능에 영향 없음."
decision: "if auto_fixable AND no_behavior_change → INFO"
```

### Level 2 — LOW

```yaml
positive: "코드 냄새 또는 경미한 결함. Blast radius == local-function."
decision: "if blast_radius == local_function AND reversibility == auto_revert AND category == correctness → LOW"
```

### Level 3 — MEDIUM

```yaml
positive: "잘못된 동작 또는 상당한 유지보수 부담. Blast radius == local-file ~ cross-module."
decision: "if silent_data_corruption_possible → HIGH (uplift)"
```

### Level 4 — HIGH

```yaml
positive: "보안 인접 또는 데이터 무결성 또는 사용자 가시적 실패."
decision: "if active_exploit_path_exists → CRITICAL"
```

### Level 5 — CRITICAL

```yaml
positive: "Bedrock Rule 매칭, 또는 외부 untrusted 입력에서 즉시 악용 가능, 또는 프로덕션 비가역 상태 변경."
decision: "if matches_bedrock_rule → CRITICAL (absolute)"
```

## §V.6 Rule of Lenity (유지)

```
if confidence_for_level_N+1 < 0.7:
    assigned_level = N
```

## §V.7 Rule Hierarchy (v2.3 재명명)

**Bedrock Rule (정확히 3개)**: Article 7 참조.

**Foundation Rule (5개 초기)**: Article 7 참조.

**Surface Rule (누적)**: 사용자 경험.

상세는 `~/.myth/bedrock-rules.yaml`, `~/.myth/foundation-rules.yaml`, `~/.myth/surface-rules.yaml` 참조.

---

# Part VI — Recurrence Implementation

> **구현 Part**: Article 2, 5, 13의 구현

## §VI.1 3-Tier Identity (Round 1 Option B)

```
Tier 1 — Exact (SHA1 hash):
  hash = sha1(
    aggressive_normalize(canonical_text)
    || model_version
  )
  동일 hash → recurrence += 1.0

Tier 2 — Embedding (multilingual-e5-small):
  벡터: 384차원
  저장: in-memory + mmap (myth Day-1), sqlite-vec/usearch (Milestone B)
  Auto-merge: cosine ≥ 0.90
  Candidate: 0.75 ≤ cosine < 0.90 → Tier 3

Tier 3 — LLM Judge (Claude Haiku):
  후보 쌍 판정
  "Same underlying root cause?"
  Conservative bias (default NOT SAME when uncertain)
```

**[v2.3 변경]** 임베딩 모델이 bge-small-en-v1.5에서 multilingual-e5-small로 변경 (Decision 2). 두 모델 모두 384차원으로 저장 레이어 무변경.

## §VI.2 Aggressive Normalization

```python
def normalize_text(s):
    s = s.lower()
    s = RE_UUID.sub("<UUID>", s)
    s = RE_HEX.sub("<HEX>", s)
    s = RE_TS.sub("<TS>", s)
    s = RE_PATH_ABS.sub("<PATH>", s)
    s = RE_NUM_INCTX.sub("<N>", s)
    s = RE_WS.sub(" ", s).strip()
    return s
```

Drain3 masking + "Preprocessing is All You Need" (arXiv:2412.05254) 원칙.

## §VI.3 Six Recurrence Categories

```
I:   count < 1.0       (첫 접촉)
II:  1.0 ≤ count < 2.0
III: 2.0 ≤ count < 4.0
IV:  4.0 ≤ count < 7.0
V:   7.0 ≤ count < 12.0
VI:  count ≥ 12.0
```

## §VI.4 Lapse Tracking (Composite) — v2.3 재명명

**[v2.1 확정 / v2.3 재명명]** Hook + Wall-clock 복합. 내부 용어를 "Quiescence"에서 "Lapse"로 재명명.

```python
lapse_score = missed_hooks * w_h + idle_days * w_d
# w_h = 1.0, w_d = 10.0

Level별 임계:
  Level 1-2: score >= 50
  Level 3-4: score >= 200
  Bedrock/Foundation Rule: ∞
```

## §VI.5 Cosine Concentration Handling

```python
# 월 1회 recalibration
def recalibrate(db, n_pairs=500, k_sigma=4):
    pairs = sample_random_pairs(db, n_pairs)
    cos = [cosine(load_vec(a), load_vec(b)) for a, b in pairs]
    mu = np.mean(cos)
    sigma = np.std(cos)

    thresh_auto = min(0.95, mu + (k_sigma+1) * sigma)
    thresh_candidate = min(0.90, mu + k_sigma * sigma)

    return {"thresh_auto": thresh_auto,
            "thresh_candidate": thresh_candidate}
```

d=384, N=10^5에서 무관 벡터 max cosine ≈ 0.244. Threshold는 adaptive 필수.

---

# Part VII — Sentencing Grid (단순화)

> **[v2.1 단순화]**: 30 cells → 6 고정 + 24 진화

## §VII.1 The 5 × 6 Matrix (Day-1)

**[v2.1 초기 상태 / v2.3 용어 재명명]**:

```
                        Recurrence
                I       II      III     IV      V       VI
┌────────────┬────────────────────────────────────────────┐
Level 1 INFO │ Note   Note   Note   Note   Note   Note  │  ← 진화
Level 2 LOW  │ Note   Note   Note   Note   Note   Note  │  ← 진화
Level 3 MED  │ Note   Note   Note   Note   Note   Note  │  ← 진화
Level 4 HIGH │ Note   Note   Note   Note   Note   Note  │  ← 진화
Level 5 CRIT │ Strike Strike Strike Strike Strike Strike │  ← 고정
└────────────┴────────────────────────────────────────────┘

고정 6 cells: Level 5 전체 (Bedrock Rule 매칭)
진화 24 cells: Level 1-4 (사용자 경험으로 채움)
```

## §VII.2 Evolution Protocol

사용자 경험으로 Grid 셀 진화:

```
Path 1 — Upgrade (Recurrence 기반 자동):
  같은 lesson이 Recurrence II → III → IV 진행
  Observer 권고 → Admin 승인 → 셀 upgrade

  예: Level 3 × Rec IV: Note → Advisory

Path 2 — User Override (Admin 직접):
  Admin이 특정 셀 직접 수정
  Git commit + rationale
  즉시 반영

Path 3 — De-escalation (Article 4):
  N회 연속 성공적 수행
  Observer 권고 → Admin 승인 → 셀 downgrade
```

## §VII.3 Seven Enforcement Stages — v2.3 재명명

```
Dismiss      완전 silent 기록, 비노출
Note         caselog.jsonl append, 평소 비노출
Advisory     SessionStart에 카테고리 수준 힌트
Caution      PreToolUse trigger 매칭 시 구체 규칙 주입
Warn         PreToolUse exit 1, 재시도 필수
Strike       PreToolUse exit 2, 차단
Seal         Bedrock Rule 전용, 항소 못 내려가는 하한
```

**[v2.3 재명명]** 7단계가 법학 용어로 재명명되었다. 의미는 동일하되 이름이 Ayres-Braithwaite의 "responsive regulation pyramid" 계보와 정합한다.

## §VII.4 Hook Mapping (v2.1 단순화)

```
초기 상태 (Day-1):
  Level 1-4: PostToolUse only
  Level 5 (Bedrock Rule): PreToolUse 항상

수렴 후 (사용자 경험):
  사용자가 "이 lesson은 사전 차단 필요"로 판단한 것만
  Observer 권고 + Admin 승인 → PreToolUse 추가
```

## §VII.5 Alarm Fatigue Hard Limits (유지)

```
세션당 제한:
  Advisory    최대 2회/세션
  Caution     최대 3-5회/세션
  Warn        최대 2회/세션
  Strike      제한 없음 (5회+ Observer 경고)
```

---

# Part VIII — Three-Agent Architecture

> **구현 Part**: Article 9 (Separation of Powers) 구현

## §VIII.1 The Gavel — Real-time Enforcement

```yaml
구현: pure Rust (Day-1 binary-per-hook, Milestone C 이후 daemon)
LLM 호출: 없음
트리거: 모든 hook
속도: < 50ms (Day-1), < 10ms 목표 (Milestone C 전환 조건 P99>15ms 2주)
비용: 0

책임:
  1. Hook 이벤트 매칭 lesson 검색
  2. Bedrock Rule 사전 차단 (PreToolUse)
  3. Grid lookup
  4. 집행 단계 실행 (Dismiss ~ Seal)
  5. 상태 업데이트

권한 제한:
  · 판단 안 함 (Grid lookup만)
  · lesson 생성 불가
  · Bedrock Rule 매칭은 정규식 기반 (의미 이해 없음, Milestone D까지)
```

## §VIII.2 Assessor — Appellate Court

```yaml
구현: Claude Haiku, --max-turns 1, --bare (또는 subagent via Task)
트리거:
  · PostToolUseFailure 이벤트
  · Stop hook review FAIL
  · myth lesson appeal

속도: 2-5초
비용: ~500 토큰/실패

책임:
  1. 4+1축 분해
  2. Rubric decision rules 적용
  3. Level + Category 판정
  4. Identity 계산 (3-tier Option B)
  5. caselog.jsonl append
  6. 1문장 rationale 필수
```

## §VIII.3 Observer — Sentencing Commission

```yaml
구현: Claude Sonnet, --max-turns 5
트리거:
  · 주간 scheduled
  · caselog.jsonl > 100 entries
  · 항소 3회 누적
  · over-trigger fatigue

속도: 1-3분
비용: 주 1회 ~10K 토큰

책임:
  1. Desuetude (Lapse) 평가
  2. Appeal-누적 lesson 재평가
  3. brief.md 재생성
  4. Temporal obsolescence 재검사
  5. Grid 진화 권고 (Admin 승인 필요)
  6. Foundation Rule 권고 (Admin 승인)
  7. 주간 리포트 (Article 8 검증 포함)
  8. Migration Readiness 평가 (Milestone A-E)

권한 제한:
  다음은 Admin 명시 승인 필요:
    · Grid 셀 값 변경
    · Bedrock/Foundation Rule 추가/삭제
    · Rubric Level 정의 변경
```

## §VIII.4 Model Separation

```
Claude Code (피평가):   Claude Opus/Sonnet 최신
The Gavel:              pure Rust
Assessor:               Claude Haiku 4.5
Observer:               Claude Sonnet 4.6 (다른 버전)
Retrial:                외부 모델 가능
```

---

# Part IX — File Structure and Implementation

> **구현 Part**: 물리적 파일 구조

## §IX.1 Global Storage (v2.3 경로 업데이트)

```
~/myth/                           # myth source (Git 저장소)
├── CONSTITUTION.md              # 이 문서
├── docs/                         # 설계 문서
└── ...

~/.myth/                          # 사용자 데이터 (XDG 표준 외 전용)
├── bedrock-rules.yaml           # Article 7 Bedrock Rule (정확히 3개)
├── foundation-rules.yaml         # Article 7 Foundation Rule (5+)
├── surface-rules.yaml            # Article 7 Surface Rule
├── grid.yaml                     # 6 고정 + 24 진화
├── state.db                      # SQLite 메타데이터
├── vectors.bin                   # 임베딩 벡터 (in-memory 파일 backing)
├── caselog.jsonl                 # append-only 실패 이벤트
├── lesson-state.jsonl            # lesson별 상태 변화
├── brief.md                      # Observer 산출 (한국어)
├── audit.jsonl                   # Merkle audit log
├── metrics/
│   └── reflector-shadow.jsonl   # Assessor shadow metrics (Day-1~)
├── commons/                     # Article 18 seed
│   ├── seed-lessons.yaml
│   └── NOTICE
├── archive/
│   └── (lapsed lesson 장기 보관)
└── embeddings/
    └── models/multilingual-e5-small/

~/.config/myth/                   # XDG_CONFIG_HOME
├── config.yaml
└── api_key                       # Milestone A 이후

~/.local/state/myth/              # XDG_STATE_HOME
├── hook-latency.ndjson
├── embed-daemon.log
├── gavel-daemon.log              # Milestone C 이후
└── tier3-dispatch.jsonl          # Milestone A 이후

$XDG_RUNTIME_DIR/myth/            # tmpfs
├── embed.sock                    # myth-embed daemon
└── gavel.sock                    # Milestone C 이후
```

## §IX.2 myth Source Layout

```
~/myth/
├── rust/                         # Rust workspace (10 crates)
│   ├── Cargo.toml
│   └── crates/
│       ├── myth-common/
│       ├── myth-db/
│       ├── myth-gavel/           # The Gavel 구현
│       ├── myth-identity/
│       ├── myth-hooks/           # 6 hook binaries
│       ├── myth-embed/           # 임베딩 daemon
│       ├── myth-orchestrator/
│       ├── myth-runtime/
│       ├── myth-ui/
│       └── myth-cli/
├── python/
│   └── myth_py/
│       ├── assessor/             # Assessor Python 레이어
│       └── observer/             # Observer Python 레이어
├── templates/                    # myth init 복사 원본
│   └── .claude/
│       └── agents/
│           ├── assessor.md       # Haiku subagent 정의
│           └── observer.md
├── scripts/
│   └── install.sh
└── tests/
    └── fixtures/                 # 560 Bedrock Rule fixtures
```

## §IX.3 Language Convention (v2.0 유지)

**고정 영어**: 메타데이터, identity string, 기술 용어, 변수·함수·파일명
**한국어 허용**: 사용자 노출 메시지, 항소 입력 텍스트, 주석, 매뉴얼

## §IX.4 Implementation Priority (myth 전용)

**[v2.3 교체]**: v2.1의 Phase A/B/C/D 구조를 myth의 Wave 0-8 구조로 교체. 상세는 `~/myth/docs/09-CLAUDE-PROMPTS.md` 참조.

```
Wave 0: 스캐폴딩 (cargo workspace, Python package, 디렉토리 구조)
Wave 1: Layer 0-1 (myth-common, myth-db)
Wave 2: Layer 2 (myth-gavel, myth-identity, myth-embed)
Wave 3: Layer 3 (myth-hooks 6개 바이너리)
Wave 4: Layer 4 (myth-runtime, myth-orchestrator, myth-ui)
Wave 5: Layer 5 (myth-cli)
Wave 6: Python (assessor, observer)
Wave 7: Rules & Fixtures (560개 FP=0 검증)
Wave 8: 통합 검증 (end-to-end, 성능, 보안 audit)
```

Day-1 완료 기준: Wave 0-8 전부 green + v0.1.0 Git tag.

---

# Part X — Amendment Procedure

> Amendment는 **Layer에 따라 다른 절차**를 요구한다.

## §X.1 Part 0 (Master Principle) — IMMUTABLE

변경 불가능.

## §X.2 Part I (Layer 1) — Fundamental Amendment

변경 조건: 시스템 정체성 근본 전환
절차:
1. 다학제적 증거 제시
2. 전체 하위 Layer 영향 분석
3. Admin 명시 승인
4. v2.X → v3.0 declaration

## §X.3 Part II (Layer 2) — Major Amendment

변경 조건: 시스템 목적 전환
절차:
1. 반증 증거 또는 대안 원리
2. Layer 3-4 파급 분석
3. Admin 명시 승인
4. v2.X → v2.(X+1)

## §X.4 Part III (Layer 3) — Structural Amendment

변경 조건: 대안 구조의 실증적 우위
절차:
1. 실증 연구 또는 운영 데이터
2. Layer 4 영향 분석
3. Admin 명시 승인

## §X.5 Part IV (Layer 4) — Operational Amendment

변경 조건: 모델 업그레이드, 파라미터 튜닝
절차:
1. Observer 권고 또는 Admin 제안
2. Admin 승인
3. minor version 증가

## §X.5.5 Part IV.5 (Meta-Principles) — Meta Amendment

변경 조건: 시스템 진화 모델의 근본 전환
절차:
1. 증거 제시
2. Layer 1-4 영향 분석
3. Admin 명시 승인

## §X.6 Bedrock Rule — 특별 개정 절차 (v2.3 재명명)

**[v2.1 확정 / v2.3 재명명]**: Bedrock Rule은 Master Principle 다음으로 강한 보호.

변경 조건:
1. Bedrock Rule 항목 추가: 비가역 피해 + 도덕적 보편 합의 + 구조적 필수성 삼중 기준
2. Bedrock Rule 항목 삭제: 매우 높은 입증 부담

절차:
1. Admin 단독 권한 (Observer 권고 불허, Assessor 침묵)
2. Git commit 필수 (who/when/why)
3. 30일 cooldown (신규 항목은 Warn로 시작)
4. 30일 후 Admin 명시 activation → Strike 승격
5. 헌법 개정 수준 문서화

## §X.7 Foundation Rule — 일반 개정 절차 (v2.3 재명명)

변경 조건: 새 위협 패턴 또는 실효 상실
절차:
1. Admin 권한 (Observer 권고 가능)
2. Git commit 필수
3. 30일 cooldown (선택적)

## §X.8 Implementation Parts (V-IX) — Parameter Tuning

변경 조건: Grid 셀, Rubric exemplar, Lapse 임계 등
절차:
1. Observer 주간 리포트 권고
2. Admin 승인
3. 패치 수준 변경

## §X.9 Appendix D (Known Issues) — Living Document

자유 업데이트.

---

# Appendix

## Appendix A. Research Documents

- Deep Research 1 (첫 위반 처우 5분야): v2.0 기반
- Deep Research 2 (심각도 등급 7분야): v2.0 기반
- External Review 2편: v2.0 기반
- **Round 1 (D.1 Assessor Hash)**: Option B 확정 (3-tier identity)
- **Round 2a (Public Data Cold-Start)**: Transfer learning 파이프라인
- **Round 2b (Rough Start + Gradual Refinement)**: Day-1 경계 판정
- **myth Research #1-5 (v2.3 추가)**: Claude Code runtime, Rust cold start, sqlite-vec 성숙도, Assessor path, The Gavel regex limits

원본: 각 research 파일은 Jeffrey 로컬에 보존.

## Appendix B. Glossary (v2.3 업데이트)

| 용어 | 정의 |
|---|---|
| **Accumulated Commons** | 공개적으로 축적된 집단 경험 (Article 18) |
| **Advisory** | 집행 3단계. 세션 시작 시 카테고리 힌트 (v2.3 재명명: session-hint) |
| **Analytic Decomposition** | 다축 분해 후 규칙 기반 집계 |
| **Appeal** | 항소 |
| **Assessor** | 실패 분석 주체, Claude Haiku (v2.3 재명명: Reflector) |
| **audit.jsonl** | Merkle audit log |
| **Bedrock Rule** | 절대 불변 3개 규칙 (v2.3 재명명: Tier 1 NEVER) |
| **Blast Radius** | 영향 범위 |
| **BML Loop** | Build-Measure-Learn (Ries 2011) |
| **brief.md** | Observer 주간 산출 (v2.3 재명명: playbook.md) |
| **caselog.jsonl** | 실패 이벤트 원본 기록 (v2.3 재명명: failures.jsonl) |
| **Category Uplift** | +1 Level 가중 |
| **Caution** | 집행 4단계. PreToolUse trigger 매칭 (v2.3 재명명: jit-context) |
| **Certainty ≫ Severity** | Nagin 원리 |
| **Collective Agent** | 집단 행위자 |
| **Commons → Personal** | 공개 → 개인 수렴 경로 |
| **Day-1 Bounded Responsibility** | 첫날의 한정된 책임 (Article 19) |
| **De-escalation** | 집행 완화 |
| **Decision Rule** | if-then 판정 규칙 |
| **Desuetude** | 오래 집행 안 된 규칙의 자연 휴면 (내부 구현: Lapse) |
| **Dismiss** | 집행 1단계. 완전 silent (v2.3 재명명: archive) |
| **Economy of Mechanism** | S&S 1975 원칙, 구현 단순성 |
| **Exemplar** | Rubric 구체 예시 |
| **Fail-safe Default** | S&S 1975 원칙, 부재 시 안전 실패 |
| **Foundation Rule** | 공동체 검증 규칙 5-10개 (v2.3 재명명: Tier 2 NEVER) |
| **Grid** | Level × Recurrence 2D 매트릭스 |
| **Identity Hash** | 위반 동일성 식별자 (3-tier) |
| **Iterative Learnability** | 검증 가능 주장 |
| **Jus Cogens** | 강행규범 (Bedrock Rule 상응) |
| **Lapse** | lesson 휴면 내부 구현 용어 (v2.3 재명명: Quiescence) |
| **Layer 1-4, 4.5** | 원칙 추상 수준 |
| **Least Privilege** | S&S 1975 원칙, 최소 권한 |
| **Lenity** | 모호 시 낮은 Level |
| **Malum in se** | 본질적 악 |
| **Malum prohibitum** | 규제적 악 |
| **Master Principle** | 대원칙, IMMUTABLE |
| **MBL / BML** | Build-Measure-Learn |
| **Milestone A-E** | 시스템 진화 전환 지점 (Day-1 이후 조건 기반) |
| **MVG** | Minimum Viable Governance |
| **Normalized Text** | aggressive_normalize 적용된 텍스트 |
| **Note** | 집행 2단계. caselog.jsonl append (v2.3 재명명: passive-log) |
| **Observer** | 정책 수립 주체, Claude Sonnet (v2.3 재명명: Curator) |
| **PCCP** | Predetermined Change Control Plan (Gaske 2023) |
| **Post-hoc Learning** | 사후 탐지 원칙 |
| **Procedural Justice** | 절차적 공정성 |
| **Proportionality** | 비례성 |
| **Recurrence** | 재발 축 |
| **Rehabilitation** | 교육 지향 |
| **Retrial** | 재심, Level 4-5 전용 (v2.3 재명명: De novo appeal) |
| **Rough Start** | 거친 시작 (Article 19) |
| **Rule of Lenity** | Article 10 구현 |
| **Seal** | 집행 7단계. Bedrock Rule 전용, 항소 불가 (v2.3 재명명: mandatory-min) |
| **Separation of Powers** | 권력 분립 |
| **Strike** | 집행 6단계. PreToolUse exit 2, 차단 (v2.3 재명명: hard-block) |
| **Surface Rule** | 개인·프로젝트별 누적 규칙 (v2.3 재명명: Tier 3 NEVER) |
| **The Gavel** | 실시간 집행 주체, pure Rust (v2.3 재명명: Trial Court) |
| **Three Translations** | 의식/기억/합의 대체 |
| **Tiered Sovereignty** | 층위별 주권 (v3.0 예정) |
| **Two Tests** | 대원칙 판단 기준 |
| **Under Review** | 항소 3회+ lesson 상태 |
| **Warn** | 집행 5단계. PreToolUse exit 1, 재시도 필수 (v2.3 재명명: soft-block) |

## Appendix C. Change Log

```
v2.3 (2026-04-19) — Naming Refresh + myth Independence
  Type: Terminology update (no substantive changes to principles).

  Context:
    · 시스템이 harness-orchestrator 하위에서 myth 독립 프로젝트로 승격
    · 10개 네이밍 카테고리 재작업 완료 (myth DECISIONS.md 참조)

  Renamings (Article 1-19 실질 내용 불변):
    · Trial Court → The Gavel
    · Reflector → Assessor
    · Curator → Observer
    · Tier 1 NEVER → Bedrock Rule
    · Tier 2 NEVER → Foundation Rule
    · Tier 3 NEVER → Surface Rule
    · archive → Dismiss
    · passive-log → Note
    · session-hint → Advisory
    · jit-context → Caution
    · soft-block → Warn
    · hard-block → Strike
    · mandatory-min → Seal
    · Quiescence → Lapse
    · playbook.md → brief.md
    · failures.jsonl → caselog.jsonl
    · De novo appeal → Retrial

  Additions:
    · Milestone A-E 체계 공식화 (Article 19, Part IX)
    · Article 1 지질 은유와의 결합 명시
    · Part IX 경로 구조 업데이트 (~/.myth/ 전용)
    · myth Research #1-5 참조 (Appendix A)

  Removed:
    · 집행 단계 이모지 (🕊️📝💡⚠️🚧🛑🔒) — 텍스트만 유지

  Approved by: Jeffrey (2026-04-19)

v2.1 (2026-04-18) — Rough Start + Accumulated Commons
  · Article 1 (Legal Metaphor) 보강: 법의 역사적 축적성
  · Article 4 (Rehabilitation) 확장: De-escalation + Support Pyramid
  · Article 7 (Malum in se) 재설계: 3-Tier NEVER 구조
  · Article 18 (Accumulated Commons) 신설 (Layer 2.5)
  · Article 19 (Day-1 Bounded Responsibility) 신설 (Layer 2.5)
  · Part V (Rubric) 단순화: 5×6 → 5×2
  · Part VII (Grid) 단순화: 30셀 → 6 고정 + 24 진화
  · Part IX (File Structure) 업데이트: commons/, personal/ 추가
  · Part X (Amendment) 조정: Tier 1 특별 절차

v2.0 (2026-04-18) — Hierarchical Redesign
  · 11개 수평 원칙 → 17개 4-Layer 계층
  · Master Principle Part 0 IMMUTABLE
  · Article 2, 8, 15, 16 신규

v1.0 (2026-04-17)
  · 초기 제정
```

## Appendix D. Known Issues

### 해결됨 (v2.1~v2.3에서 반영)

**D.1 Assessor Hash Instability**: Round 1 Option B로 해결. 3-tier identity (SHA1 + multilingual-e5-small + Haiku).

**D.2 Malum in se 개념 순환성**: Bedrock/Foundation/Surface 3-계층 구조로 해결. Bedrock Rule = jus cogens 상응.

**D.4 Desuetude Hook-only**: Hook + Wall-clock 복합 지표로 해결. 구현 용어 Lapse.

### High (배포 시 고려)

**D.3 The Gavel Semantic Blindness**:
- Pure 정규식으로 난독화된 Bedrock Rule 탐지 불가
- 현 단계 수용 (Rough Start 원칙)
- Milestone D 활성 시 semantic detection 추가

**D.5 Grid Translation Layer Complexity**:
- v2.1 단순화로 대부분 해결 (6 고정 + 24 진화)
- 사용자 UI는 7 집행 단계만 표시

**D.6 Articulation Bias**:
- 성공 종료된 실수는 영구 미탐지
- 배포 시 Proactive Audit (주기적 git diff 리뷰) 추가

### Medium (운영 중 수집)

**D.7 Iterative Learnability 실증 검증**:
- Article 8의 검증은 Observer 주간 리포트
- 초기에는 데이터 부족
- Milestone A 이후 목표

**D.8 Observability Dashboard**:
- MTTR, recurrence decay rate 등 KPI
- myth TUI (myth watch)에서 일부 구현

**D.9 Model Version Management**:
- multilingual-e5-small v1 → v2 migration 프로토콜
- Milestone B에서 구체화

**D.10 Lesson Quality Metrics**:
- "lesson 주입 후 재발 빈도" 추적

### Conceptual

**D.11 ASRS 모델의 단일 사용자 적용성**:
- ASRS는 제3자 중재 구조
- myth 단일 개발자 환경 적용성?
- Round 3 주제

**D.12 Merge/Split Feedback → Algorithm**:
- Sentry GitHub #58815 미해결 영역
- **myth 혁신 기회** (Round 2a §11)

### Further Research Needed

1. myth 배포 6개월 후 실측 기반 재교정 (Round 3 주제 1)
2. Multi-user 확장 시 democratic feedback integrity
3. Constitutional AI의 명시적 통합 (Bai et al. 2022)
4. Milestone D (semantic detection) 정확도 튜닝
5. De-escalation 대칭성의 장기 정성 효과

---

## Appendix E. Implementation Priority (myth 통합)

**v2.3 원칙**: Phase 개념 전면 폐기. **Wave 0-8 구조**로 Day-1 완성 (myth DECISIONS.md Decision 5 참조).

### Day-1 — Wave 0-8 (즉시~5-10일)

1. Wave 0: 스캐폴딩
2. Wave 1: Layer 0-1 (myth-common, myth-db)
3. Wave 2: Layer 2 (myth-gavel, myth-identity, myth-embed)
4. Wave 3: Layer 3 (myth-hooks 6개 바이너리)
5. Wave 4: Layer 4 (myth-runtime, myth-orchestrator, myth-ui)
6. Wave 5: Layer 5 (myth-cli)
7. Wave 6: Python (assessor, observer)
8. Wave 7: Rules & Fixtures (560 fixtures FP=0)
9. Wave 8: 통합 검증 + v0.1.0 tag

### Milestone 전환 (Day-1 이후, 조건 기반)

- **Milestone A** (Day+21): Assessor Tier 2/3 증축 판단
- **Milestone B**: Vector store 전환 (sqlite-vec 또는 usearch)
- **Milestone C**: The Gavel daemon 전환
- **Milestone D**: Semantic detection 활성
- **Milestone E**: AST-based validation 도입

각 Milestone은 실측 조건 만족 시에만 트리거. 시간 기반 아님.

상세: `~/myth/ARCHITECTURE.md` §4, `~/myth/docs/09-CLAUDE-PROMPTS.md`, `~/myth/docs/12-DEPLOYMENT.md`.

---

**Ratified**: 2026-04-19
**Signatories**: Jeffrey (Principal Designer, Admin), Claude (AI co-designer)
**Applicable to**: myth v0.1+

이 헌법을 따르는 것이 myth 시스템의 정체성이다.

*Lex est quod notamus, sed veritas est quod probamus.*
*(우리가 기록하는 것이 법이되, 진리는 우리가 증명하는 것이다.)*

*Lex non nascitur ex nihilo; lex convergit.*
*(법은 무에서 태어나지 않는다; 법은 수렴한다.)*
