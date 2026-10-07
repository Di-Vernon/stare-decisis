# myth v2 — 설계 문서

myth v2는 재설계 중이다. v1(`v0.1.1`)은 아카이브 상태다. 이 디렉터리에는 v2의 설계 문서와 결정 기록(ADR)이 있다.

- 개발 원칙: docs-first. 설계 커밋과 구현 커밋은 분리한다. 커밋 1개에는 결정 1개만 담는다.
- 용어: myth 신조어는 원문으로 쓴다. 용어집은 [ADR-0004](adr/0004-terminology.md)에 있다.
- 결정 기록은 `adr/`에 번호순으로 쌓는다. 결정이 뒤집히면 원 결정을 지우지 않고, 후속 ADR을 추가해 원 결정의 상태를 `대체됨(→ ADR-NNNN)`으로 바꾼다.

## ADR 목록

| 번호 | 제목 | 상태 |
|---|---|---|
| [0001](adr/0001-build-vs-fork.md) | Build vs Fork — 자체 구축 | 확정 |
| [0002](adr/0002-repository-strategy.md) | 저장소 전략 — 같은 repo의 main에서 진행 | 확정 |
| [0003](adr/0003-license-files.md) | 라이선스 파일 분리 | 확정 |
| [0004](adr/0004-terminology.md) | 용어 규칙 — 신조어 원문 표기 | 확정 |
| [0005](adr/0005-audit-role.md) | audit의 지위 — evidence sensor이자 ladder 한 칸 | 확정 |
| [0006](adr/0006-audit-excludes-intent-misreading.md) | intent 오독은 audit 대상에서 제외 | 확정 |
| [0007](adr/0007-audit-staged-rollout.md) | audit 단계 도입 — M0에는 A0·A1만 | 확정 |
| [0008](adr/0008-audit-a0-checks.md) | A0 결정적 검사 목록 | 확정 |
| [0009](adr/0009-completion-report.md) | completion report 형식 | 확정 |
| [0010](adr/0010-change-tree.md) | change tree — 변경 사항을 기록으로 렌더링 | 확정 |
| [0011](adr/0011-audit-protected-paths.md) | 보호 대상 — restraint lock 확장 | 확정 |
| [0012](adr/0012-audit-a1-labeling.md) | A1 라벨 수집과 차단 권한 해제 기준 | 확정 |
| [0013](adr/0013-stack-ts-mod-plugin.md) | 스택 — TypeScript mod 중심 Claude Code plugin | 확정 |
| [0014](adr/0014-storage.md) | 저장 형식 — `~/.myth/` 불변 파일 event sourcing | 확정 |
| [0015](adr/0015-llm-path-and-model.md) | LLM 사용 — 승격 제안 시 holding 초안만, 세션 모델에 fork | 확정 |
| [0016](adr/0016-behavior-only-detection.md) | 감지는 행동 신호로만 — 초기 버전에 어휘 사전 없음 | 확정 |
