# ADR-0010 — change tree: 변경 사항을 myth가 기록으로 렌더링한다

- 날짜: 2026-10-06
- 상태: 확정 (Jeffrey 비준)
- 관련: audit 설계 D6

## 맥락

디렉터리에 변경이 있으면 사용자가 무엇이 바뀌었는지 한눈에 볼 수 있어야 혼동이 없다. 같은 내용을 Claude가 직접 쓰면 누락이나 착오가 생길 수 있고, 토큰도 든다.

## 선택

파일 변경이 있었던 턴에는 myth가 Stop 시점에 change tree를 렌더링해 사용자에게 보여 준다(`systemMessage`).

```
Changes: 3 files, +42 −7
src/
├── auth/
│   ├── login.ts      M  +30 −5
│   └── validate.ts   A  +12
└── legacy/
    └── old.ts        D  −2
```

- 표기: `A` 추가, `M` 수정, `D` 삭제, `R` 이름 변경
- 데이터 원천:
  - git 저장소: 턴 시작 기준점과 비교한 `--name-status`, `--numstat`
  - git이 아닌 디렉터리: myth 이벤트 로그의 Edit/Write 기록. 이 경우 Bash로 바뀐 파일이 누락될 수 있으므로, 그 사실을 트리 하단에 표시한다.
- Claude의 completion report는 이 트리를 다시 쓰지 않고 참조만 한다.

## 근거

- 기록에서 결정적으로 생성하므로 보고 착오가 없고, 모델 토큰도 쓰지 않는다.
- `systemMessage`는 사용자에게만 보이고 Claude의 행동은 바꾸지 않는다(Claude Code 공식 문서). 그래서 사용자에게 정보를 줄 뿐 개입으로 세지 않는다. [B, 벤더]
- 요청 대상과 실제 접근 대상의 차이는 거짓 완료의 핵심 신호다(arXiv 2609.20812). 이 차이를 사용자가 직접 볼 수 있다.

## 영향 범위

- intervention budget에서 차감하지 않는다.
- 같은 데이터로 ADR-0008 #4(테스트 파일 변경)를 검사한다.
