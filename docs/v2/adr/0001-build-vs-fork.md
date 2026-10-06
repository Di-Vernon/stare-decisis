# ADR-0001 — Build vs Fork: 자체 구축

- 날짜: 2026-10-06
- 상태: 확정 (Jeffrey 비준)
- 관련: 핸드오프 §9-1, Phase A 리뷰

## 맥락

TRACE(arXiv 2606.13174)와 그 배포판인 tellonce는 myth v2와 가장 가까운 선행 사례다. 사용자 교정을 runtime hook 검사로 컴파일한다는 점이 같다. v2를 tellonce 위에 얹을지(fork), 직접 만들지(build)를 정해야 했다.

리뷰 대상:
- `YujunZhou/tellonce` `d7696b0` = v1.7.1 (2026-09-12)
  - MIT 라이선스, 커밋 158개, 단일 유지보수자
  - Claude Code 변형 12,617 LOC (Python 23 모듈 + bash hook 9개)
- `YujunZhou/TRACE_exp` `c9d76bf` (2026-06-11), MIT 라이선스

방법은 정적 리뷰다. hook 등록 → 진입점 → import 그래프 → 저장소·judge·컴파일 계층 순으로 추적했다. 제3자 코드는 실행하지 않았다.

## 선택지

- **(B) 자체 구축.** tellonce의 설계 일부를 명시적으로 차용하고 출처를 고지한다.
- **(C) tellonce 위에 구축.** fork한 뒤 ladder·강등·측정·패턴 계층을 추가한다.

## 선택

**(B)**

## 근거

### 1. myth 차별점 7개 중 6개가 tellonce에 비어 있다

| # | 차별점 | 판정 | 코드 근거 |
|---|---|---|---|
| 1 | 재범·심각도에 따른 단계적 승급 | 비어 있음 | 교정 1회로 즉시 active가 된다(`lib/memory_store.py:1277`). 재발 카운터가 없다(`:441-470`) |
| 2 | 비집행 채널을 포함한 ladder | 부분 점유 | observe 모드와 reminder tier는 있다. 그러나 증거에 따라 오르내리는 ladder는 없다 |
| 3 | 노출 기반 강등·폐용, regressed 복원 | 비어 있음 | ARCHIVE/RESTORE는 사용자 명시 요청일 때만 한다(`lib/memory_judge.py:216-220`). threshold_advisor는 `5f3d82f`에서 삭제됐다 |
| 4 | holdout 효과 측정 | 비어 있음 | TRACE_exp는 오프라인 벤치마크 평가만 한다 |
| 5 | `autoMode.*` 라우팅 | 비어 있음 | PreToolUse·permissions·autoMode 사용이 0건이다. 설치기는 permissions를 의도적으로 건드리지 않는다(`lib/_install_merge_settings.py:109-111`) |
| 6 | 패턴 분석·위험 프로필 | 비어 있음 | 군집이나 프로필 코드가 없다 |
| 7 | restraint lock | 비어 있음 | `memory_upsert.py apply-plan`은 호출자가 넘긴 텍스트를 신뢰 turn으로 등록해 커밋한다(`:1035-1065`). Bash를 쓸 수 있는 에이전트가 스스로 규칙을 만들 수 있다 |

### 2. tellonce 런타임은 v2 방향(핸드오프 §8)과 세 축에서 충돌한다

- **센서**: 사용자 발화만 본다. myth는 auto mode 차단, interrupt, 검증 결과 등 여러 센서를 쓴다.
- **주입**: 매 턴 전체 규칙 인덱스를 상시 주입한다. 상한 50개를 넘으면 회전시킨다(`lib/retrieve_inject.py:68, 615-644`). myth는 행동 직전에 선택적으로 경고한다.
- **집행**: Stop 시점에만 개입한다. deterministic 층은 빈 확장점이고(`lib/deterministic_block.py:49-78`), shadow judge는 차단하지 않는다. myth는 PreToolUse와 autoMode 규칙을 쓴다.
- tellonce의 컴파일 계층(`lib/execution_support.py`)은 어떤 hook 진입점에서도 도달하지 않는다. 실험 host용 API다.

### 3. C를 택해도 재사용 범위가 작다

| 영역 | LOC | 비율 | C로 갈 때 |
|---|---|---|---|
| 학습 코어 | ≈5.8K | 46% | 스키마와 judge 프롬프트를 수정해야 재사용 가능 |
| 주입·집행 | ≈4.6K | 37% | 폐기 |
| 연결되지 않은 host API | ≈1.4K | 11% | — |

case file 스키마(polarity·persistence·counters)를 넣는 순간 upstream 동기화는 불가능해진다. 또 Python + bash + jq 스택이 §9-2 스택 결정보다 먼저 고정된다.

### 4. 검증된 효과를 상속하지 못한다

- TRACE_exp 공개 하네스는 교정을 tellonce의 LLM 파이프라인으로 처리하지 않는다.
- ClawArena 쪽은 교정 텍스트의 벤치마크 라벨(P1–P5, O1–O4)을 하드코딩된 요구사항 문장으로 바꿔 직접 기록한다(`experiments/clawarena/clawarena/conditions.py:706-754`).
- verify-retry는 `compiled_enforcement` 조건에서만 돌고, 검증기는 벤치마크 자체 checker다(`multi_runner.py:474-523`).
- 따라서 논문 수치(위반율 100→37.6%/2.0%)를 tellonce 런타임의 효과 근거로 쓰지 않는다.

## 차용할 것

출처는 `NOTICE`와 `THIRD-PARTY.md`에 고지한다. 코드를 직접 가져오면 MIT 고지를 유지한다.

1. 사용자 turn만 규칙을 승인한다는 신뢰 경계. 근거 인용(exact quote)을 커밋 경계에서 재검증한다.
2. 적용 범위는 사용자 원문 인용이 있을 때만 좁힌다.
3. 위험한 durable 규칙은 REJECT한다(안전장치 해제, 파괴적 삭제, 보호 브랜치 push 등).
4. lifecycle 연산 9종: NOOP / UPDATE / SUPERSEDE / SPLIT / NEW / NEEDS_USER / REJECT / ARCHIVE / RESTORE.
5. 컴파일 검증 방식: tier·stage 모델, 합성 사례 자가검증, 독립 리뷰. 통과하지 못하면 reminder로 강등한다.
6. 저장 구조: SQLite를 진실원으로 두고 projection은 재생성한다. generation 기반 낙관적 동시성을 쓴다.
7. hook은 inbox에만 쓰고, 판정은 detached worker가 한다.
8. 하위 `claude -p` 호출 시 재귀를 막는다.
9. redaction 패턴.

## 피할 것

- 매 턴 전체 인덱스 주입
- Stop 전용 집행
- 단일 언급만으로 즉시 활성화
- 매 턴 observation 로그 의식
- 에이전트가 호출할 수 있는 자기 승인 CLI

## 영향 범위

- v2 코드는 새로 작성한다.
- 스택(§9-2)은 열린 결정으로 남는다.
- tellonce에서 차용한 항목은 해당 설계 문서와 코드에 출처를 표기한다.

## 참조

- 핸드오프: `MYTH-V2-HANDOFF.md` §6, §9
- tellonce: https://github.com/YujunZhou/tellonce
- TRACE_exp: https://github.com/YujunZhou/TRACE_exp
