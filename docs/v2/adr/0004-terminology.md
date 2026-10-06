# ADR-0004 — 용어 규칙: myth 신조어는 원문으로 표기한다

- 날짜: 2026-10-06
- 상태: 확정 (Jeffrey 비준)

## 맥락

v2 설계를 한국어로 논의하면서 myth 고유 개념을 번역해 썼다. 그 결과 오히려 뜻을 알기 어려워졌다(예: "의도 계약", "직권 사건"). 같은 개념이 문서마다 다른 번역어로 쓰이면 코드와 문서의 대응도 깨진다.

## 선택

- myth 신조어는 원문(영어)으로만 쓴다. 문장 설명은 한국어로 하되, 아래 용어집의 단어는 번역하지 않는다.
- 코드 식별자, 설정 키, 문서 용어는 같은 단어를 쓴다.
- 새 용어를 만들면 이 용어집에 먼저 추가한다.

## 용어집

| 용어 | 뜻 |
|---|---|
| case file | 사건 하나의 기록. facts(증거 포인터), holding, 메타데이터, counters를 담는다 |
| holding | case file에서 추출한 명령형 한 문장의 교훈 |
| precedent | 사용자 ratification을 거쳐 효력을 가진 holding |
| polarity | holding의 종류. `constrain`(범위 축소), `prescribe`(단계 추가), `restrain`(중단·철회·위임) |
| persistence | holding의 지속 범위. `once` / `session` / `durable` |
| codification ladder | precedent를 적용하는 강도의 사다리. L0 기록 ~ L4 차단 |
| evidence sensor | 사건의 증거를 만드는 감지 장치. 사용자 교정, auto mode 차단, interrupt, 검증 결과, audit 등 |
| audit | 사용자 교정 없이 claim과 실행 증거를 대조하는 evidence sensor |
| audit incident | audit이 만든 사건. 신뢰 등급이 가장 낮다 |
| claim | Claude가 응답에서 한 완료·검증 주장. 예: "모든 테스트 통과" |
| evidence package | audit 판정에 넘기는 구조화 증거 묶음 |
| intent contract | 사용자 프롬프트에서 추출한 goal·scope·forbidden·done criteria |
| completion report | 작업 턴 종료 시 Claude가 쓰는 4칸 보고(Done / Verified / Not done·Unverified / Assumptions) |
| change tree | 턴 동안 바뀐 파일을 myth가 기록으로 그린 트리 |
| intervention budget | 세션 하나에서 사용자에게 보이는 개입(audit 피드백, L2 경고, L3 확인 등)의 합계 상한 |
| shadow mode | 판정은 하되 기록만 하고 개입하지 않는 상태 |
| ratification | 사용자의 명시적 승인. durable 확정, L3/L4 승격, 폐기 등에 필요하다 |
| restraint lock | 작업 중인 에이전트가 myth 데이터·설정·hook을 바꾸지 못하게 막는 장치 |

## 영향 범위

- `docs/v2/` 이후 모든 v2 문서와 코드에 적용한다.
- v1 문서는 소급 수정하지 않는다.
