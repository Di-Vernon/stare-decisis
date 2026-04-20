# myth — Hook 시스템

## 역할

Claude Code의 **21개 hook 이벤트 전체 지형**을 보여주고, myth가 그 중 **어떤 6개를 쓰는지 + 왜 그 6개인지** 설명한다. 또한 `.claude/settings.json`의 정확한 구조와 myth가 기대하는 stdin/stdout 스키마를 박제한다.

Hook 시스템은 myth 전체의 **사용자 가시 경계**. myth는 이 경계를 통해서만 Claude Code에 개입한다.

## Claude Code 2.1.x의 21개 Hook 이벤트

Research #1에 따라 전수 목록:

| # | 이벤트명 | 발동 시점 | myth 사용 |
|---|---|---|---|
| 1 | `SessionStart` | 세션 시작 시 | ✅ brief.md 주입 |
| 2 | `SessionEnd` | 세션 종료 시 | ✕ |
| 3 | `UserPromptSubmit` | 사용자가 엔터 친 직후 | ✅ 이전 turn의 assessor compliance 검증 |
| 4 | `PreToolUse` | tool 실행 직전 | ✅ The Gavel 판정 |
| 5 | `PostToolUse` | tool 성공 직후 | ✅ latency 로깅 |
| 6 | `PostToolUseFailure` | tool 실패 직후 (2.1.27+) | ✅ Assessor trigger |
| 7 | `Stop` | turn 종료 직전 | ✅ (Day-1 비활성, Milestone A 활성) |
| 8 | `SubagentStart` | Task subagent 시작 | ✕ |
| 9 | `SubagentStop` | Task subagent 종료 | ✕ |
| 10 | `Compact` | /compact 명령 | ✕ |
| 11 | `Resume` | 세션 재개 | ✕ |
| 12 | `Fork` | 세션 fork | ✕ |
| 13 | `Checkpoint` | checkpoint 생성 | ✕ |
| 14 | `ModelChange` | 모델 전환 | ✕ |
| 15 | `ContextWindowWarn` | 컨텍스트 75% 도달 | ✕ |
| 16 | `TaskStart` | Task tool 호출 | ✕ |
| 17 | `TaskStop` | Task tool 종료 | ✕ |
| 18 | `PermissionDecision` | 권한 요청 응답 | ✕ |
| 19 | `MCPConnect` | MCP 서버 연결 | ✕ |
| 20 | `MCPDisconnect` | MCP 서버 연결 끊김 | ✕ |
| 21 | `PostToolUseInterrupt` | 사용자가 도중 중단 | ✕ |

## myth가 사용하는 6개 Hook — 정당화

### `SessionStart` → `myth-hook-session-start`

**목적**: brief.md를 Claude에게 주입.

**왜 이 이벤트인가**: 매 세션 시작 시 Claude가 "지금 활성 lesson이 뭐가 있는지, Migration Readiness 상태는 어떤지"를 context에 포함해야 한다. SessionStart는 딱 한 번만 발동하므로 비용 낮음.

**대안**: UserPromptSubmit마다 주입 — context 낭비 + 토큰 비용. 비추.

### `UserPromptSubmit` → `myth-hook-user-prompt`

**목적**: 직전 turn의 `pending_reflection` 준수 여부 검증.

**왜 이 이벤트인가**: Claude가 응답을 막 시작하기 직전. 여기서 이전 turn의 결과(Task tool 호출 여부)를 transcript에서 확인할 수 있다. Stop hook이 더 늦지만 Stop에서 block 시 재귀 위험 있어 **감시는 UserPromptSubmit, 재주장은 Stop**으로 분리.

### `PreToolUse` → `myth-hook-pre-tool` (The Gavel)

**목적**: 재앙적 명령 차단. myth의 **핵심 차단 경로**.

**왜 이 이벤트인가**: tool 실행 직전이 유일한 차단 기회. PostToolUse에서는 이미 실행 완료 후.

**성능 제약**: P99 <15ms (Milestone C 조건).

### `PostToolUse` → `myth-hook-post-tool`

**목적**: hook latency 기록 + 성공한 이벤트 간단 로깅.

**왜 이 이벤트인가**: latency 분포를 알아야 Migration Readiness 판단 가능. 성공한 이벤트도 통계에 포함.

**Day-1 경량**: ~1ms 목표. state.db write 최소화.

### `PostToolUseFailure` → `myth-hook-post-tool-failure`

**목적**: 실패 시 Assessor 호출 유도 (Tier 1 Variant B).

**왜 이 이벤트인가**: PostToolUse와 별개로 2.1.27부터 실패 전용 이벤트. Research #4 권고: 실패만 분리해서 처리하면 성공 경로 성능 보존.

**주의**: 2.1.27 미만 버전은 이 이벤트 없음 → PostToolUse로 fallback (exit code로 실패 감지).

### `Stop` → `myth-hook-stop`

**Day-1 비활성**. `enable_tier2: false`로 no-op.

**Milestone A 이후 활성 시 목적**: Tier 1에서 Claude가 assessor를 호출 안 했으면, Stop 시점에 block하고 Variant B 재주장 (Tier 2).

**재귀 위험**: `stop_hook_active` 플래그로 방지. 한 세션에서 Tier 2 재주장 최대 3회.

## myth가 사용하지 않는 15개 — 이유

**SessionEnd**: 정보 없음 — session-end-hook 내부 로직 대체는 `myth observer run`의 주간 cron이 더 적절.

**SubagentStart/Stop**: Task subagent(assessor)는 Claude 본체가 관리. myth가 개입할 필요 없음.

**Compact, Resume, Fork, Checkpoint**: 세션 구조 이벤트. myth는 세션 내용에만 관심.

**ModelChange, ContextWindowWarn, MCPConnect/Disconnect**: 플랫폼 상태. myth가 개입할 이유 없음.

**TaskStart/Stop, PermissionDecision, PostToolUseInterrupt**: 만약 **Observer 고도화** 시점에서 유의미해질 수 있음. Day-1 범위 밖.

## `.claude/settings.json` 구조

`myth init` 생성물:

```json
{
  "hooks": {
    "SessionStart": {
      "command": "/home/miirr/.local/bin/myth-hook-session-start"
    },
    "UserPromptSubmit": {
      "command": "/home/miirr/.local/bin/myth-hook-user-prompt"
    },
    "PreToolUse": {
      "command": "/home/miirr/.local/bin/myth-hook-pre-tool"
    },
    "PostToolUse": {
      "command": "/home/miirr/.local/bin/myth-hook-post-tool"
    },
    "PostToolUseFailure": {
      "command": "/home/miirr/.local/bin/myth-hook-post-tool-failure"
    },
    "Stop": {
      "command": "/home/miirr/.local/bin/myth-hook-stop"
    }
  }
}
```

**절대 경로** 사용. PATH 의존하면 Claude Code 자식 프로세스 환경 차이로 실패 가능.

**Scope**: `.claude/settings.json` (Project scope). `~/.claude/settings.json` (User scope)와 `.claude/plugins/*/settings.json` (Plugin scope) 중 **Project scope**만 사용 (Research #4 Issue #24788 회피).

## Hook 입력 JSON 스키마 (stdin)

Claude Code 2.1.114가 모든 hook에 보내는 공통 필드:

```json
{
  "session_id": "uuid-v4-string",
  "transcript_path": "/home/<user>/.claude/projects/<escaped-cwd>/<session>.jsonl",
  "cwd": "/path/to/project",
  "hook_event_name": "PreToolUse",
  "stop_hook_active": false,
  "permission_mode": "default" | "acceptEdits" | "bypassPermissions" | "plan" | "..."
}
```

`permission_mode`는 현재 세션의 권한 모드. `Stop` 외 대부분 이벤트에 포함.

이벤트별 추가 필드:

### `PreToolUse` 추가

```json
{
  "tool_name": "Bash",
  "tool_use_id": "toolu_abc123",
  "tool_input": {
    "command": "rm -rf /tmp/build",
    "description": "cleanup"
  }
}
```

### `PostToolUse` 추가 (성공 케이스)

```json
{
  "tool_name": "Bash",
  "tool_use_id": "toolu_abc123",
  "tool_input": { /* PreToolUse와 동일 */ },
  "tool_response": {
    "stdout": "hello world",
    "stderr": "",
    "interrupted": false,
    "isImage": false,
    "noOutputExpected": false
  }
}
```

`tool_response`에 `exit_code` / `duration_ms`는 **없다**. 성공 판정은 "이 이벤트 자체가 발동했다"로 간주 — 실패는 배타적으로 `PostToolUseFailure`가 대신 발동.

### `PostToolUseFailure` 추가 (실패 케이스, 2.1.27+)

```json
{
  "tool_name": "Bash",
  "tool_use_id": "toolu_01Nv3...",
  "tool_input": { /* PreToolUse와 동일 */ },
  "error": "Exit code 1\ncat: /nonexistent_file: No such file or directory",
  "is_interrupt": false
}
```

**주의**: `tool_response` 객체가 아니라 단일 문자열 `error`. 첫 줄에 보통 `"Exit code N"`이 온다 (Bash 기준). 파서는 multiline string으로 취급해야 하고, exit code가 필요하면 이 문자열의 첫 줄을 파싱해야 한다.

`is_interrupt`는 사용자가 도중에 중단했는지 여부(취소).

### `UserPromptSubmit` 추가

```json
{
  "prompt": "사용자가 방금 입력한 텍스트"
}
```

`turn_number`는 **없다**. Turn 추적이 필요한 hook은 자체적으로 카운터를 유지하거나 transcript 파일의 라인 수로 추정해야 한다.

### `SessionStart` 추가

```json
{
  "source": "startup" | "resume" | "clear"
}
```

### `Stop` 추가

```json
{
  "stop_hook_active": false,
  "last_assistant_message": "응답 텍스트의 마지막 메시지"
}
```

**주의**: 초안 문서는 `stop_reason: "end_turn" | "max_tokens"`를 제시했으나 실제 2.1.114 runtime은 `last_assistant_message` (마지막 assistant 응답의 본문 문자열)를 보낸다. 의미와 용도가 다르다. "왜 멈췄는지"를 알려주지 않고 "무엇을 말하고 멈췄는지"를 알려준다.

### 이벤트 간 관계 (실측 기반)

- **`PreToolUse`는 모든 tool 호출에서 발동** (성공·실패 무관).
- **`PostToolUse`와 `PostToolUseFailure`는 배타적** — 같은 `tool_use_id`에 대해 둘 중 하나만 발동한다. 성공 → `PostToolUse`, 실패 → `PostToolUseFailure`.
- **`PreToolUse`가 `continue: false` + exit 2로 block했을 때의 후속 이벤트 발동 여부는 미실측**. Task 3.4(pre-tool) 구현 중 실제 Strike/Seal 경로를 확인하며 추가 검증 예정.

> **v0.1 Task 3 사전 실측 기반 업데이트** (Jeffrey 승인 2026-04-21, Claude Code 2.1.114)
>
> 이 섹션은 2026-04-21 `/tmp/myth-hook-probe/`에서 Claude Code 2.1.114
> 런타임에 빈 dump 스크립트를 bind해 stdin JSON을 **직접 캡처**한
> 결과로 작성되었다. 초안(설계 당시의 Research #1)과 실제 런타임
> 사이에 다섯 가지 유의미한 차이가 있었다:
>
> 1. **공통 필드에 `permission_mode` 추가** — 세션 권한 모드가 모든 이벤트 stdin에 포함.
> 2. **`UserPromptSubmit`에 `turn_number` 없음** — `prompt` 단독.
> 3. **`Stop`의 `stop_reason` 대신 `last_assistant_message`** — 필드명·의미 모두 다름.
> 4. **`PostToolUse`의 `tool_response` 필드 교체** — `{stdout, stderr, interrupted, isImage, noOutputExpected}`. `exit_code` / `duration_ms` 없음.
> 5. **`PostToolUseFailure` 스키마 전면 교체** — `tool_response` 객체가 아니라 `error` (multiline string) + `is_interrupt` (bool). 이 차이가 myth 학습 루프의 핵심이므로 가장 중대.
>
> 관계 관찰(추가):
> - `PostToolUse`와 `PostToolUseFailure`는 배타.
> - `PreToolUse`는 성공·실패 양쪽 경로에서 발동.
> - Block(exit 2) 이후 흐름은 이번 실측 범위 밖. Task 3.4에서 검증.
>
> myth-hooks의 `core/input.rs`는 이 스키마를 primary 소스로 파싱하도록
> 구현한다. 환경변수는 docs(architecture): hook env contract를 참고.

## Hook 출력 JSON 스키마 (stdout)

**`SyncHookJSONOutput`**:

```json
{
  "continue": true,
  "stopReason": "optional reason if continue=false",
  "hookSpecificOutput": {
    "hookEventName": "PreToolUse",
    "permissionDecision": "allow" | "deny" | "ask",
    "additionalContext": "string inserted into Claude's context"
  }
}
```

`continue: false` + exit code 2 → **Claude가 tool 실행 중단**하고 `stopReason`을 받음.
`continue: true` + `additionalContext` → **Claude context에 주입**. Claude가 반응.

### myth가 반환하는 구체 형태

**Advisory / Caution** (The Gavel):
```json
{
  "continue": true,
  "hookSpecificOutput": {
    "hookEventName": "PreToolUse",
    "permissionDecision": "allow",
    "additionalContext": "Caution: This command matches a known failure pattern (lesson L-20260419-0012, recurrence III)."
  }
}
```

**Warn** (The Gavel):
```json
{
  "continue": true,
  "hookSpecificOutput": {
    "hookEventName": "PreToolUse",
    "permissionDecision": "ask",
    "additionalContext": "Warn: potential data loss. The user must confirm."
  }
}
```

**Strike / Seal**:
```bash
# stdout
{"continue": false, "stopReason": "Bedrock Rule R1-A matched: rm_rf_unsandboxed"}
# exit 2
```

**Assessor trigger** (PostToolUseFailure):
```json
{
  "continue": true,
  "hookSpecificOutput": {
    "hookEventName": "PostToolUseFailure",
    "additionalContext": "<Variant B template with reminder_id>"
  }
}
```

## Exit Code 의미

```
0   allow (정상 실행)
2   block (tool 실행 차단, stderr는 Claude에게 전달)
기타 non-blocking error (hook 자체 오류, tool은 정상 진행)
```

myth는 **예외 상황에서만 exit 2**. hook 내부 에러는 항상 exit 0 + stderr 로그.

## 환경 변수 (Claude Code → hook)

Claude Code 2.1.114가 실제로 hook 프로세스에 주입하는 환경변수 (실측):

```bash
CLAUDECODE=1                     # Claude Code 하에서 실행 중임을 나타내는 플래그
CLAUDE_CODE_ENTRYPOINT=sdk-cli   # (또는 interactive 등) 진입 방식
CLAUDE_CODE_EXECPATH=...         # claude 바이너리의 절대 경로
CLAUDE_PROJECT_DIR=/path/to/project   # 현재 프로젝트 루트 (= stdin JSON의 cwd와 동일)
# SessionStart hook에만 추가로:
CLAUDE_ENV_FILE=/home/<user>/.claude/session-env/<session>/sessionstart-hook-0.sh
# 시스템 env:
XDG_RUNTIME_DIR=/run/user/<uid>/
XDG_DATA_DIRS=...
```

**primary 소스는 stdin JSON**. 초안이 제시한 `CLAUDE_TRANSCRIPT_PATH`, `CLAUDE_SESSION_ID`, `CLAUDE_HOOK_EVENT`, `CLAUDE_TOOL_INPUT`, `CLAUDE_FILE_PATHS` 등은 **환경변수에 존재하지 않는다**. 해당 정보는 모두 stdin JSON의 필드로 이동했다:

| 초안 env | 실제 정보 위치 |
|---|---|
| `CLAUDE_TRANSCRIPT_PATH` | stdin JSON `transcript_path` |
| `CLAUDE_SESSION_ID` | stdin JSON `session_id` |
| `CLAUDE_HOOK_EVENT` | stdin JSON `hook_event_name` |
| `CLAUDE_TOOL_INPUT` | stdin JSON `tool_input` (객체) |
| `CLAUDE_FILE_PATHS` | 해당 없음 — `tool_input`의 필드로 접근 |

myth hook 바이너리는 **stdin JSON 파싱을 주된 입력 경로로** 구현한다. 환경변수는 다음 용도에 한정:
- `CLAUDECODE`: myth 실행 여부 확인
- `CLAUDE_PROJECT_DIR`: 프로젝트 루트 가드(stdin `cwd`와 교차 검증)
- `CLAUDE_CODE_EXECPATH`: 필요 시 claude 바이너리 재호출(Day-1에는 미사용)
- `CLAUDE_ENV_FILE` (SessionStart 전용): 아직 Day-1 사용 계획 없음. Task 3.2 구현 중 필요 발견 시 추가 실측.

> **v0.1 Task 3 사전 실측 — 환경변수 소스 전환** (Jeffrey 승인 2026-04-21, Claude Code 2.1.114)
>
> 초안 env 목록(7개)과 실측 결과(4개 + SessionStart 전용 1개)의 차이가
> 단순 누락이 아니라 **primary source의 아키텍처 전환**이다:
>
> - **제거**: `CLAUDE_TRANSCRIPT_PATH / CLAUDE_SESSION_ID / CLAUDE_HOOK_EVENT / CLAUDE_TOOL_INPUT / CLAUDE_FILE_PATHS` — 전부 env에 없음. 동일 정보가 stdin JSON의 필드로 존재.
> - **신규**: `CLAUDE_CODE_ENTRYPOINT / CLAUDE_CODE_EXECPATH` — 모든 hook에, `CLAUDE_ENV_FILE` — SessionStart에만.
> - **유지**: `CLAUDECODE / CLAUDE_PROJECT_DIR`.
>
> 즉 Claude Code 2.1.x는 "env via shell" 방식에서 "structured JSON on stdin" 방식으로 이동했다. myth hook 구현은 stdin JSON 파서를 1차 소스로 삼고, env는 boolean 플래그·경로 정도로만 활용. `ARCHITECTURE.md` Contract 4도 같은 기준으로 수정.

## 환경 변수 (myth → 자식 프로세스)

myth가 **설정하거나 읽는** 변수:

```bash
MYTH_SESSION_ID=uuid            # myth 고유 (Claude session_id와 1:1이지만 별도 생성)
MYTH_CORRELATION_ID=reminder-id # Assessor trigger 추적
MYTH_ACTIVE=1                   # myth supervision 아래임을 알림
MYTH_DISABLE=1                  # myth 비활성 (디버깅)
MYTH_NO_EMBED_DAEMON=1          # embed daemon 비활성 (fallback only)
CLAUDE_REVIEW_ACTIVE=1          # myth 자체가 발동한 Claude 호출 (재귀 방지)
```

## Hook 바이너리 공통 규약

1. **stdin에서 JSON 읽기**. EOF 즉시.
2. **작업 수행 (짧게)**. 예산: 1~15ms (이벤트에 따라).
3. **stdout에 JSON 한 줄**. 생략 시 allow.
4. **stderr에 로그** (사용자 보이지 않게 표시, debug 용).
5. **timeout 안전**: Claude Code가 5~10초 timeout 설정. myth는 항상 그 안에 완료.

## 재귀 방지

myth hook에서 다른 Claude Code tool이 실행되면 **hook 재귀 발동** 위험.

예: `myth-hook-session-start`이 `subprocess.run(["claude", "..."])` 호출 → 새 Claude 세션 → 그것도 hook 발동 → 무한 루프.

**방지 메커니즘**:
1. `CLAUDE_REVIEW_ACTIVE=1` 환경변수 → myth가 발동한 호출이면 PreToolUse는 allow로 즉시 통과.
2. `stop_hook_active` 필드 → Stop hook 중 재귀 감지 시 즉시 종료.
3. hook 바이너리는 **Claude CLI를 호출하지 않음** (myth orchestration은 주 CLI에서만).

## Hook 설치 검증

`myth doctor`의 health check 중 hook 등록 확인:

```python
def check_hook_registration():
    project = Path.cwd()
    settings = project / ".claude" / "settings.json"
    
    if not settings.exists():
        return CheckResult.Fail("`.claude/settings.json` missing. Run `myth init`.")
    
    data = json.loads(settings.read_text())
    hooks = data.get("hooks", {})
    
    expected = {
        "SessionStart": "myth-hook-session-start",
        "UserPromptSubmit": "myth-hook-user-prompt",
        "PreToolUse": "myth-hook-pre-tool",
        "PostToolUse": "myth-hook-post-tool",
        "PostToolUseFailure": "myth-hook-post-tool-failure",
        "Stop": "myth-hook-stop",
    }
    
    missing = [k for k in expected if k not in hooks]
    if missing:
        return CheckResult.Fail(f"missing hooks: {missing}")
    
    wrong_path = []
    for event, expected_bin in expected.items():
        cmd = hooks[event]["command"]
        if not cmd.endswith(expected_bin):
            wrong_path.append((event, cmd))
    if wrong_path:
        return CheckResult.Warn(f"hook paths look suspicious: {wrong_path}")
    
    return CheckResult.Pass("All 6 hooks registered correctly")
```

## 문제 해결

### 증상: hook이 호출되지 않는다

1. `.claude/settings.json`이 **프로젝트 루트**에 있는지 확인
2. `myth doctor` 실행
3. `claude --debug` 로 실행해서 hook 로딩 로그 확인
4. `~/.local/bin/myth-hook-pre-tool`가 실행 가능한지 (`chmod +x`)
5. `CLAUDECODE=1` 있는지 (Claude Code가 실행 중인 모드인지)

### 증상: hook은 호출되지만 판정이 반영 안 됨

1. `hookSpecificOutput.hookEventName`이 올바른지 (이벤트 이름 오타)
2. exit code가 2가 아닌데 block 시도 (continue: false만으로는 부족)
3. **Plugin scope 사용 중**이면 Project scope로 전환 (Research #4 Issue #24788)

### 증상: `additionalContext`가 Claude에게 전달 안 됨

- Windows + MCP + Plugin scope 조합 버그. Project scope + WSL2 환경 확인.
- `continue: true` 여야 함 (false면 additionalContext 무시)

## 관련 결정

- ARCHITECTURE Contract 1~4: hook 경로·프로토콜·exit code·환경변수
- Decision 3 (Tier 0/1/2/3): Hook이 Assessor를 부르는 경로
- Research #1: Claude Code 2.1.x 전체 hook 이벤트
- Research #4: PostToolUseFailure, Issue #24788, Project scope 사용
