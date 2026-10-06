# ADR-0011 — 보호 대상: restraint lock 확장, 테스트 파일은 막지 않음

- 날짜: 2026-10-06
- 상태: 확정 (Jeffrey 비준)
- 관련: audit 설계 D7

## 맥락

작업 중인 에이전트가 테스트, 설정, audit 데이터를 바꾸면 검사를 무력화할 수 있다. 반대로 테스트 파일까지 항상 막으면 정상적인 테스트 작성(TDD 포함)이 깨진다.

## 선택

- **항상 막는 대상**(PreToolUse deny와 permission deny를 함께 건다):
  - myth 설정·데이터
  - `.claude/settings*.json`
  - (A2 도입 후) intent contract 파일
  - 이것은 보안 판정이므로 판정이 실패하면 deny한다.
- **설정 변경 감시**: `ConfigChange` hook으로 project/local 설정 변경을 막고 기록한다.
- **변조 점검**: SessionStart에서 myth 설정·hook 스크립트의 해시를 대조한다. 불일치하면 변조 의심으로 표시한다.
- **설치 위치**: audit hook은 user scope(`~/.claude/settings.json`)에 설치한다.
- **테스트 파일은 막지 않는다.** 대신 피드백 직후의 테스트 파일 변경을 gaming 의심으로 기록한다(ADR-0008 #4).

## 근거

- project의 `.claude/settings.json` 값 하나로 user scope hook이 꺼질 수 있다. `disableAllHooks`는 우선순위 적용 뒤의 값으로 결정되기 때문이다(Claude Code 공식 문서). [B, 벤더]
- 테스트를 숨기면 부정행위가 거의 0이 되지만 정상 성능이 떨어진다. 읽기전용은 그 절충이다. 다만 연산자 오버로딩 같은 우회는 막지 못한다(ImpossibleBench, arXiv 2510.20270). [B, 벤치마크]
- 테스트 작성은 일상 작업이다. 항상 막으면 오탐 차단이 되고, 오탐은 누락보다 해롭다.

## 영향 범위

- 핸드오프 §8.6의 restraint lock 범위에 위 항목을 추가한다.
- 변조 방어의 실제 작동은 E-A8로 점검한다.
