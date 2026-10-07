# ADR-0013 — 스택: TypeScript mod 중심 Claude Code plugin

- 날짜: 2026-10-07
- 상태: 확정 (Jeffrey 비준)
- 관련: 핸드오프 §9-2, §9-5

## 맥락

v2는 공개 OSS 사용자가 쉽게 설치할 수 있어야 한다. 후보는 셋이었다.

- (A) TypeScript mod 중심 plugin
- (B) settings command hook + Python
- (C) settings command hook + Rust 바이너리

Claude Code 전용이 되는 것은 문제가 아니다(Jeffrey 확인).

mod의 사양은 Claude Code 공식 문서(v2.1.287 기준)로 확인했다. mod는 plugin 안의 JS/TS 이벤트 핸들러이며 Claude Code 프로세스 안에서 실행된다.

## 선택

**(A)**. 세부 결정은 다음과 같다.

1. **배포 형태**: plugin 하나에 mod, skills, 필요한 settings hook을 함께 담는다. 설치는 marketplace의 `/plugin install`로 한다.
2. **외부 런타임 없음**: 사용자 PC에 Node, Python, Rust가 필요 없다. Claude Code가 `.ts` 모듈을 직접 로드한다.
3. **core / adapter 분리**: 도메인 로직(case file, holding, ladder, counters, 통계)은 `$`에 의존하지 않는 순수 TS로 작성한다. mod는 이벤트를 core로 넘기는 얇은 adapter만 맡는다.
4. **v1 코드는 재사용하지 않는다**: redaction 패턴 같은 교훈만 데이터로 옮긴다.
5. **의존성은 0을 원칙으로 한다**: hooks module은 plugin 안의 상대 경로 import만 허용되기 때문이다.

## 근거

myth의 요구와 mod 기능을 대응시키면 다음과 같다.

| myth 요구 | mod 기능 |
|---|---|
| hook 간 상태 유지 | 모듈 변수 공유. v1 결함 #3(hook마다 새 프로세스)이 구조적으로 사라진다 |
| 비동기 LLM 판정 | `$.model.complete`/`classify`가 세션 자격 증명(사용자 plan 또는 API key)으로 호출한다. `claude -p` 하위 프로세스와 재귀 가드가 필요 없다 |
| L3/L4 집행 | `tool.call`의 `{ deny }`, `tool.check`의 `ask`/`deny` |
| 선택적 주입 | `prompt.submit`의 context |
| audit, change tree | `classic.Stop` / `turn.complete`, `$.process.run`, `$.ui.status`/`log` |
| ratification UI | `$.ui.ask`, 명령 등록 |
| 테스트 | `claude plugin test` (`.test.ts`) |

hook은 터미널, VS Code 확장, `claude -p`, SDK, 클라우드 세션에서 모두 실행된다.

(B)와 (C)는 다음 이유로 탈락했다.
- 프로세스 기동 비용이 매번 든다.
- hook 간 상태를 파일로만 공유할 수 있다.
- LLM 호출에 `claude -p`나 API key가 필요하다.
- (B)는 런타임을 사용자 PC에 설치해야 하고, (C)는 OS별 바이너리를 배포해야 한다. plugin에는 설치 단계가 없다.

## 제약과 대응

| 제약 | 대응 |
|---|---|
| Claude Code v2.1.287 이상 필요 | 일반 사용자는 최신 버전을 쓴다고 가정한다. README에 최소 버전을 명시한다 |
| Desktop 앱의 WSL 세션에서는 plugin이 로드되지 않음 | 지원 환경으로 명시한다. WSL **터미널**의 `claude`는 정상 동작한다 |
| 관리자가 `allowManagedModsOnly`, `allowManagedHooksOnly`, managed `disableAllHooks`를 켜면 mod가 로드되지 않음 | 아래 degraded mode 원칙, 그리고 관리자용 조직 mod 배포 안내(관리 디렉터리 marketplace + `prependPlugins`) |
| 아무 설정이 없는 Team/Enterprise 환경에서는 내장 guard가 먼저 실행됨 | 영향 없음. guard는 deny 규칙에 걸린 호출을 mod가 승인하는 것만 막는데, myth는 승인하지 않는다 |
| hook 자체 실행 시간 10초 제한 | 무거운 작업은 타이머와 `$.model.*` 대기로 넘긴다. API 대기 시간은 제한에 포함되지 않는다 |
| mod API가 신생이라 변경 위험이 있음 | core / adapter 분리, 빌드별 타입 파일로 검증 |

## degraded mode 원칙

mod가 로드되지 않아도 **이미 정적 설정으로 컴파일한 법은 계속 동작**해야 한다.

- 남는 것: L1 `.claude/rules`, L3/L4의 `permissions.deny`와 `autoMode` 규칙. 이것들은 hook이 아니라 Claude Code가 직접 읽는 설정이다.
- 멈추는 것: evidence 수집, case file 생성, holding 추출, L2 경고, audit, 측정, ratification.

따라서 L1·L3·L4는 mod 내부 판정에만 의존하지 않고 정적 미러를 함께 둔다. 정적 미러를 안전하게 쓰는 경로는 spike S1에서 정한다.

## spike S1 (M0 첫 작업)

- 세션당 실제 저장 바이트 수
- 타이머 콜백의 시간 제한 유무
- `$.fs` 쓰기 중 잘린 쓰기의 처리
- 정적 미러를 쓸 경로 (`~/.claude/settings.json`은 원자적 쓰기가 없어 Claude Code와 동시에 쓰면 손상될 위험이 있다)
- npm 의존성 vendoring 가능 여부

## 참조

- https://code.claude.com/docs/en/plugins/mods/overview
- https://code.claude.com/docs/en/plugins/mods/api
- https://code.claude.com/docs/en/plugins/mods/events
- https://code.claude.com/docs/en/plugins/mods/reference
- https://code.claude.com/docs/en/plugins/mods/admin
