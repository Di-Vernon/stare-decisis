# myth — Directory Structure

이 문서는 myth의 **모든 디렉토리와 파일의 위치·역할·권한**을 박제한다. `ARCHITECTURE.md`의 §7에서 간략히 다뤘지만, 이 문서는 **레퍼런스 수준의 상세**를 제공한다.

myth는 **네 개의 독립된 위치**에 분산된다:

1. `~/myth/` — myth 본체 (Git 저장소)
2. `~/.local/bin/` — 실행 파일 (PATH)
3. `~/.config/myth/` + `~/.myth/` — 사용자 설정 + 데이터
4. `~/.local/state/myth/` + `$XDG_RUNTIME_DIR/myth/` — 런타임 상태 + socket

각각 XDG Base Directory Specification을 따른다.

## 1. `~/myth/` — myth 본체

Git 저장소. 소스 코드, 문서, 템플릿이 이곳에 있다. 사용자가 `git clone`으로 받고, `scripts/install.sh`로 설치한다.

```
~/myth/
├── README.md                    # 프로젝트 첫인상 (짧음, 5분 요약)
├── CONSTITUTION.md              # 헌법 (Part 0~X, ratified v2.3)
├── ARCHITECTURE.md              # 실행 모델, Milestone, API 계약
├── PROTOCOL.md                  # myth-embed wire protocol
├── WSL2-SETUP.md                # WSL2 운영 체크리스트
├── THIRD-PARTY.md               # gitleaks·detect-secrets 등 라이선스 귀속
├── DECISIONS.md                 # 설계 결정 이력 (번호 누적)
├── LICENSE                      # MIT OR Apache-2.0 (듀얼)
├── docs/                        # 설계 문서 (25+ 파일)
│   ├── 00-INDEX.md              # 문서 전체 목차·지도
│   ├── 01-OVERVIEW.md           # 전체 그림
│   ├── 02-CONCEPTS.md           # 용어집
│   ├── 03-DIRECTORY.md          # 이 파일
│   ├── 04-CRATES/
│   │   ├── 00-overview.md       # crate 간 관계도
│   │   ├── 01-myth-common.md
│   │   ├── 02-myth-db.md
│   │   ├── 03-myth-gavel.md
│   │   ├── 04-myth-identity.md
│   │   ├── 05-myth-hooks.md
│   │   ├── 06-myth-embed.md
│   │   ├── 07-myth-orchestrator.md
│   │   ├── 08-myth-runtime.md
│   │   ├── 09-myth-ui.md
│   │   └── 10-myth-cli.md
│   ├── 05-PYTHON.md             # Assessor/Observer Python 레이어
│   ├── 06-HOOKS.md              # Hook 시스템 (21개 이벤트)
│   ├── 07-STATE.md              # 상태 저장소 (SQLite + JSONL + audit)
│   ├── 08-BUILD-SCOPE.md        # Day-1 범위 + Milestone 조건
│   ├── 09-CLAUDE-PROMPTS.md     # Claude Code 구현 지시서
│   ├── 10-VALIDATION.md         # 테스트, fixtures, FP 검증
│   ├── 11-RISKS.md              # 리스크 목록 + 대응
│   └── 12-DEPLOYMENT.md         # 배포, 설치, 버전 관리
├── rust/                        # Rust workspace
│   ├── Cargo.toml               # workspace 선언
│   ├── Cargo.lock
│   ├── .cargo/
│   │   └── config.toml          # rustflags, linker 설정
│   ├── crates/                  # 10개 crate
│   │   ├── myth-common/
│   │   │   ├── Cargo.toml
│   │   │   ├── src/lib.rs
│   │   │   └── src/...
│   │   ├── myth-db/
│   │   ├── myth-gavel/
│   │   ├── myth-identity/
│   │   ├── myth-hooks/
│   │   ├── myth-embed/
│   │   ├── myth-orchestrator/
│   │   ├── myth-runtime/
│   │   ├── myth-ui/
│   │   └── myth-cli/
│   └── target/                  # 빌드 산출물 (gitignore)
├── python/
│   └── myth_py/                 # Python 패키지
│       ├── __init__.py
│       ├── pyproject.toml       # Poetry 또는 pip
│       ├── assessor/
│       │   ├── __init__.py
│       │   ├── cli.py           # entry: myth-assessor
│       │   ├── classifier.py    # Tier 0 deterministic
│       │   ├── dispatcher.py    # Tier 3 Anthropic SDK (stub)
│       │   ├── templates.py     # Variant A/B/C
│       │   ├── schema.py        # Pydantic 모델
│       │   └── state.py         # lesson-state.jsonl
│       └── observer/
│           ├── __init__.py
│           ├── cli.py           # entry: myth-observer
│           ├── analyzer.py      # caselog 분석
│           ├── brief_gen.py     # brief.md 생성
│           ├── migration.py     # Migration Readiness 평가
│           └── report.py        # 주간 리포트 포맷
├── templates/                   # myth init 복사 원본
│   ├── .claude/
│   │   ├── settings.json.template
│   │   ├── agents/
│   │   │   ├── assessor.md      # Haiku agent frontmatter
│   │   │   └── observer.md      # Sonnet agent frontmatter
│   │   └── hooks/
│   │       └── (hook 등록 템플릿)
│   ├── commons/
│   │   └── seed-lessons.yaml    # 초기 seed lesson
│   └── CLAUDE.md.template       # 프로젝트별 CLAUDE.md 샘플
├── scripts/
│   ├── install.sh               # 전체 설치 (빌드 + 심볼릭 링크)
│   ├── uninstall.sh
│   ├── pgo-build.sh             # PGO 빌드 (Milestone C 대비)
│   └── dev-setup.sh             # 개발자 환경 준비
├── tests/
│   ├── fixtures/
│   │   ├── positive/            # Bedrock Rule 양성 케이스 (280 files)
│   │   │   ├── R1-A/
│   │   │   ├── R1-B/
│   │   │   └── ...
│   │   └── negative/            # Bedrock Rule 음성 케이스 (280 files)
│   │       ├── R1-A/
│   │       └── ...
│   ├── integration/             # end-to-end 시나리오
│   └── shadow/                  # Assessor Shadow mode 샘플
└── .github/                     # CI 설정 (선택)
    └── workflows/
        └── (규칙 검증, 빌드, 테스트)
```

### `~/myth/`의 특징

- **Git 저장소**. Jeffrey가 직접 관리·수정·커밋.
- **읽기 전용 아님** (사용자 편집 가능).
- 설치는 **`~/.local/bin/`으로 심볼릭 링크** 또는 copy (install.sh 설정).

## 2. `~/.local/bin/` — 실행 파일

PATH에 포함된 위치. `install.sh` 실행 후 채워진다.

```
~/.local/bin/
├── myth                          # 주 CLI entry (symlink → ~/myth/rust/target/release/myth-cli)
├── myth-hook-pre-tool           # The Gavel (symlink)
├── myth-hook-post-tool
├── myth-hook-post-tool-failure
├── myth-hook-user-prompt
├── myth-hook-stop
├── myth-hook-session-start
├── myth-embed                   # embed daemon (클라이언트/데몬 통합 바이너리)
├── myth-assessor                # Python entry (shim script)
└── myth-observer                # Python entry (shim script)
```

**bin 스크립트가 Python entry를 감싸는 방식**:

```bash
#!/usr/bin/env bash
# ~/.local/bin/myth-assessor
exec python3 -m myth_py.assessor.cli "$@"
```

사용자 PATH에 `~/.local/bin`이 있어야 함. `install.sh`가 `.bashrc` 확인·안내.

## 3. `~/.config/myth/` — 사용자 설정 (XDG_CONFIG_HOME)

```
~/.config/myth/
├── config.yaml                  # 사용자 설정 (mode 0600)
└── api_key                      # Anthropic API key (Milestone A 이후)
```

### `config.yaml` 예시

```yaml
# ~/.config/myth/config.yaml
myth_version: 1
language: ko  # 사용자 매뉴얼 언어

gavel:
  daemon_enabled: false  # Milestone C 전까지 false
  latency_log_enabled: true

embed_daemon:
  enabled: true
  idle_timeout_minutes: 15

assessor:
  tier_0_enabled: true
  tier_1_enabled: true
  tier_2_enabled: false  # Milestone A에서 토글
  tier_3_enabled: false  # Milestone A에서 토글
  shadow_mode: true  # Day-1 ~ 21일간 true

observer:
  weekly_cron: "0 9 * * MON"  # 매주 월요일 오전 9시
  brief_language: ko

vector_store: in_memory  # Milestone B에서 "sqlite_vec" 또는 "usearch"

# Milestone A 이후 Tier 3 활성 시:
# anthropic:
#   api_key_path: ~/.config/myth/api_key
#   model: claude-haiku-4-5-20251001
#   spend_limit_usd: 10
```

### `api_key` 파일 (Milestone A 이후)

```
sk-ant-api03-xxxxxxxxxxxxxx...
```

단 한 줄, 끝에 개행. Mode 0600.

## 4. `~/.myth/` — 사용자 데이터

```
~/.myth/
├── bedrock-rules.yaml            # 3 items, 47 patterns (Jeffrey + 30일 cooldown)
├── foundation-rules.yaml         # 5~10 items (Jeffrey + git commit)
├── surface-rules.yaml            # 개인·프로젝트 (자유 수정)
├── grid.yaml                     # Level × Recurrence 처분 매트릭스
├── state.db                      # SQLite (lesson 메타데이터, audit 인덱스)
├── state.db-wal                  # WAL 파일 (SQLite 자동 생성)
├── state.db-shm                  # Shared memory (SQLite 자동)
├── vectors.bin                   # 임베딩 벡터 (in-memory store 파일 back)
├── caselog.jsonl                 # 모든 실패 이벤트 (append-only)
├── lesson-state.jsonl            # lesson 상태 변화 시계열 (append-only)
├── audit.jsonl                   # Merkle audit log (append-only)
├── brief.md                      # Observer 주간 브리프 (매주 재생성)
├── metrics/
│   ├── reflector-shadow.jsonl    # Assessor shadow metrics (Day-1~)
│   └── weekly-summary.jsonl      # Observer 주간 요약 history
└── archive/
    └── (lapsed lesson 장기 보관)
```

### 중요 파일 설명

**`bedrock-rules.yaml` / `foundation-rules.yaml` / `surface-rules.yaml`**: Decision 5에서 확정한 47개 정규식 구조. 형식:

```yaml
rules:
  - id: R1-A
    item: rm_rf_unsandboxed  # 속한 Bedrock item 3개 중 하나
    pattern: "..."
    likelihood: HIGH
    source: "gitleaks v8.x (MIT) rule:..."  # 차용 출처
    tests:
      positive: ["rm -Rf ~", "rm -fr /", ...]
      negative: ["rm -rf /tmp/test", ...]
```

**`state.db`**: SQLite. 테이블 개요 (상세는 `07-STATE.md`):
- `lessons` (id, identity_hash, level, category, ...)
- `hook_events` (hook 실행 이력)
- `appeal_history`
- `grid_overrides` (Admin 수동 조정)

**`caselog.jsonl`**: append-only. 한 줄당 하나의 JSON:

```
{"ts":"2026-04-19T14:23:45Z","session_id":"...","event":"post_tool_failure","tool":"Bash","tool_input":{...},"error":"...","classified_level":3}
{"ts":"2026-04-19T14:24:12Z", ...}
```

**`brief.md`**: Observer가 매주 재생성. 새 세션 시작 시 Claude에게도 주입 (SessionStart hook). Decision 7의 Migration Readiness 섹션 포함.

**권한**: `~/.myth/` 는 0700, 내부 파일은 0600.

## 5. `~/.local/state/myth/` — 런타임 상태 (XDG_STATE_HOME)

```
~/.local/state/myth/
├── hook-latency.ndjson           # 모든 hook latency (Day-1)
├── embed-daemon.log              # myth-embed daemon 로그 (JSON Lines)
├── gavel-daemon.log              # Milestone C 이후만 생성
├── tier3-dispatch.jsonl          # Milestone A Tier 3 활성 이후
└── observer-runs/                # Observer 실행 이력
    └── 2026-W16/
        ├── analysis.json
        └── brief-generated.md
```

### `hook-latency.ndjson` 예시

```
{"ts":"2026-04-19T14:23:45Z","event":"pre_tool","latency_ms":3.2,"result":"allow","session_id":"..."}
{"ts":"2026-04-19T14:23:46Z","event":"pre_tool","latency_ms":2.8,"result":"allow","session_id":"..."}
{"ts":"2026-04-19T14:23:47Z","event":"pre_tool","latency_ms":12.1,"result":"deny","session_id":"..."}
```

Observer가 주간 집계해 `brief.md`의 "Migration Readiness" 섹션에 반영.

### 로그 rotation

logrotate 사용 권장. Day-1 install.sh가 `/etc/logrotate.d/myth` 등록 (권한 있을 시) 또는 사용자에게 안내.

```
~/.local/state/myth/*.log {
    weekly
    rotate 4
    compress
    missingok
    notifempty
}
```

## 6. `$XDG_RUNTIME_DIR/myth/` — Runtime (Unix socket)

일반적으로 `/run/user/<UID>/myth/`. tmpfs, 재부팅 시 사라짐. systemd 관리.

```
$XDG_RUNTIME_DIR/myth/
├── embed.sock                    # myth-embed daemon socket
├── gavel.sock                    # Milestone C 이후 The Gavel daemon socket
├── embed.lock                    # flock (동시 spawn race 방지)
└── gavel.lock                    # Milestone C 이후
```

WSL2에서 `$XDG_RUNTIME_DIR` 설정 확인 필요. systemd-user 활성화 전제. 만약 없으면 fallback `/tmp/myth-$UID/`.

**권한**: 디렉토리 0700, 소켓 0600.

## 7. 프로젝트 단위 — `myth init` 결과

개별 프로젝트에서 `myth init` 실행 시:

```
~/project/프로젝트A/
├── .claude/
│   ├── settings.json             # hook 등록 + PATH
│   ├── agents/
│   │   ├── assessor.md           # ~/myth/templates/...로 symlink
│   │   └── observer.md
│   ├── hooks/
│   │   └── (hook 설정 세부)
│   └── CLAUDE.md                 # (선택) 프로젝트별 myth 지시
└── .myth/                        # (선택) 프로젝트별 override
    ├── surface-rules.yaml        # 프로젝트 전용 Surface Rule
    └── config.local.yaml         # 프로젝트 설정 override
```

`~/project/프로젝트A/.claude/settings.json` 예시:

```json
{
  "hooks": {
    "PreToolUse": {
      "command": "/home/user/.local/bin/myth-hook-pre-tool"
    },
    "PostToolUseFailure": {
      "command": "/home/user/.local/bin/myth-hook-post-tool-failure"
    },
    "UserPromptSubmit": {
      "command": "/home/user/.local/bin/myth-hook-user-prompt"
    },
    "Stop": {
      "command": "/home/user/.local/bin/myth-hook-stop"
    },
    "SessionStart": {
      "command": "/home/user/.local/bin/myth-hook-session-start"
    }
  }
}
```

**Project scope** (`/path/to/project/.claude/settings.json`) 사용. Plugin scope는 회피 (Research #4 Issue #24788 — Windows+MCP+plugin scope에서 additionalContext drop).

## 8. 경로 해석 우선순위 (config override)

복수 config가 존재할 때 우선순위:

```
1. 프로젝트 ~/.myth/config.local.yaml (프로젝트별)
2. 사용자 ~/.config/myth/config.yaml
3. myth 내장 기본값 (컴파일 타임)
```

rule도 동일:

```
surface rules:
  1. 프로젝트 .myth/surface-rules.yaml
  2. 전역 ~/.myth/surface-rules.yaml
  (두 개 병합, 프로젝트가 우선)

foundation rules: 전역만 (~/.myth/foundation-rules.yaml)
bedrock rules: 전역만 (~/.myth/bedrock-rules.yaml)
```

## 9. 권한 요약

| 경로 | 권한 | 사유 |
|---|---|---|
| `~/myth/` | 0755 (일반) | Git 저장소, 공개 가능 |
| `~/.local/bin/myth*` | 0755 | 실행 파일 |
| `~/.config/myth/` | 0700 | 설정 디렉토리 |
| `~/.config/myth/api_key` | 0600 | 비밀 |
| `~/.config/myth/config.yaml` | 0600 | 설정 |
| `~/.myth/` | 0700 | 데이터 디렉토리 |
| `~/.myth/*.yaml` | 0600 | 설정·규칙 |
| `~/.myth/*.jsonl` | 0600 | 로그 |
| `~/.myth/state.db*` | 0600 | DB |
| `~/.local/state/myth/` | 0700 | 상태 디렉토리 |
| `$XDG_RUNTIME_DIR/myth/` | 0700 | socket 디렉토리 |
| `$XDG_RUNTIME_DIR/myth/*.sock` | 0600 | socket |

`install.sh`와 myth-embed·myth-gavel 시작 시 `umask 0077` 설정으로 자동 확보.

## 10. 변경 이력

| 날짜 | 버전 | 변경 |
|---|---|---|
| 2026-04-19 | v1.0 | 초기 작성. 4-way 분산(본체/bin/config+data/runtime) 확정. |
