# ADR-0005 — audit의 지위: evidence sensor이자 보고 단계의 ladder 한 칸

- 날짜: 2026-10-06
- 상태: 확정 (Jeffrey 비준)
- 관련: audit 설계 D1

## 맥락

사용자 교정이 들어오기 전에도 실수를 잡으려면, claim과 실행 증거를 대조하는 audit이 필요하다. 그런데 audit에 독자적인 규칙과 권한을 주면 myth 안에 판단 주체가 둘 생긴다. 그러면 precedent 체계와 audit의 결정이 서로 충돌한다.

## 선택

- audit은 **evidence sensor**다. 별도의 규칙집을 갖지 않는다. 규칙은 precedent에서만 나온다.
- audit의 개입(피드백·차단)은 codification ladder의 한 칸으로 다룬다. 다른 칸과 같은 측정·강등 규칙을 적용받는다.
- audit의 산출물은 **audit incident**다. `provenance: audit`, `trust: low`로 기록한다.
- audit incident 하나만으로는 holding 후보가 되지 않는다. 독립 신호가 1개 이상 일치해야 한다. 독립 신호는 사용자 교정, PermissionDenied 후 승인, interrupt, 검증 결과 중 하나다.
- L3/L4 승격 근거를 셀 때 audit incident는 넣지 않는다.

## 근거

- 단일 LLM 추출 결과는 노이즈가 크다. Tang et al.(arXiv 2605.29442)에서는 추출된 에피소드 29,896개 중 2단 검증을 통과한 것이 16,118개(53.9%)뿐이었다. 원문 인용 근거를 요구하는 검증을 거친 뒤에야 정밀도가 0.93이 됐다. [B, 실사용]
- "성공"으로 저장된 기억의 52.9–59.5%가 실제로는 실패였다(arXiv 2606.15017). 자기 판정만으로 라벨을 붙이면 학습이 오염된다. [A]
- 오탐 주입은 누락보다 해롭다(arXiv 2604.27283). [A/B]

## 영향 범위

- case file 스키마에 `provenance`, `trust` 필드를 둔다.
- holding 추출기는 audit incident를 단독 근거로 쓰지 않는다.
