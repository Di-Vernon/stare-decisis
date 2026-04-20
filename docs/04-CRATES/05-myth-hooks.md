# `myth-hooks` — Claude Code Hook 바이너리

## 역할

Claude Code의 6개 hook 이벤트에 바인딩되는 **실제 실행 바이너리**들을 제공한다. 각 바이너리는 단일 책임이고, 공통 로직은 `myth-hooks::core` 모듈로 추출.

**의존**: `myth-common`, `myth-db`, `myth-gavel`, `myth-identity`.
**의존받음**: 없음 (최종 실행체).

## Cargo.toml

```toml
[package]
name = "myth-hooks"
version = "0.1.0"
edition = "2021"
license = "MIT OR Apache-2.0"

[dependencies]
myth-common = { path = "../myth-common" }
myth-db = { path = "../myth-db" }
myth-gavel = { path = "../myth-gavel" }
myth-identity = { path = "../myth-identity" }

serde = { workspace = true }
serde_json = { workspace = true }
uuid = { workspace = true }
chrono = { workspace = true }
mimalloc = { workspace = true }
tracing = { workspace = true }
anyhow = { workspace = true }

# bin targets
[[bin]]
name = "myth-hook-pre-tool"
path = "src/bin/pre_tool.rs"

[[bin]]
name = "myth-hook-post-tool"
path = "src/bin/post_tool.rs"

[[bin]]
name = "myth-hook-post-tool-failure"
path = "src/bin/post_tool_failure.rs"

[[bin]]
name = "myth-hook-user-prompt"
path = "src/bin/user_prompt.rs"

[[bin]]
name = "myth-hook-stop"
path = "src/bin/stop.rs"

[[bin]]
name = "myth-hook-session-start"
path = "src/bin/session_start.rs"
```

**6개 bin target**. 각 ~100 LOC 수준의 얇은 래퍼.

## 모듈 구조

```
crates/myth-hooks/
└── src/
    ├── lib.rs               # 공통 API (core 모듈)
    ├── core/
    │   ├── mod.rs
    │   ├── input.rs         # stdin JSON 파싱 (Hook JSON schema)
    │   ├── output.rs        # stdout JSON + exit code
    │   ├── latency.rs       # hook-latency.ndjson 기록
    │   └── session.rs       # SessionId 관리
    ├── templates/
    │   ├── variant_a.rs     # Minimal additionalContext
    │   ├── variant_b.rs     # Engineered (기본)
    │   └── variant_c.rs     # Conditional (Sonnet 4.5 튜닝)
    └── bin/
        ├── pre_tool.rs
        ├── post_tool.rs
        ├── post_tool_failure.rs
        ├── user_prompt.rs
        ├── stop.rs
        └── session_start.rs
```

## 공통 `main()` 패턴

모든 hook 바이너리가 따르는 공통 흐름:

```rust
use mimalloc::MiMalloc;
#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

fn main() -> ExitCode {
    let start = std::time::Instant::now();
    myth_common::logging::init_logging("myth-hook-pre-tool");
    
    let result = run();
    
    // latency 기록 (fire-and-forget, 실패해도 hook은 계속)
    let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;
    let _ = myth_hooks::core::latency::record("pre_tool", elapsed_ms, &result);
    
    match result {
        HookResult::Allow => ExitCode::SUCCESS,
        HookResult::Block { output } => {
            // stdout에 JSON 쓰고 exit 2
            println!("{}", serde_json::to_string(&output).unwrap());
            ExitCode::from(2)
        }
        HookResult::Error(e) => {
            eprintln!("myth hook error: {}", e);
            ExitCode::SUCCESS  // non-blocking error: hook 자체 오류, tool은 계속
        }
    }
}
```

## bin 1 — `myth-hook-pre-tool` (The Gavel)

```rust
// src/bin/pre_tool.rs

fn run() -> HookResult {
    let input = read_hook_input()?;  // stdin JSON
    
    if std::env::var("MYTH_DISABLE").is_ok() {
        return HookResult::Allow;
    }
    
    if std::env::var("CLAUDE_REVIEW_ACTIVE").is_ok() {
        // 재귀 방지: myth 자체가 발동한 도구 호출은 skip
        return HookResult::Allow;
    }
    
    let gavel = Gavel::init()?;
    let verdict = gavel.judge(&input.into());
    
    match verdict.enforcement {
        Enforcement::Dismiss | Enforcement::Note => HookResult::Allow,
        Enforcement::Advisory | Enforcement::Caution => {
            HookResult::AllowWithContext(verdict.to_hook_json())
        }
        Enforcement::Warn => {
            HookResult::Ask(verdict.to_hook_json())
        }
        Enforcement::Strike | Enforcement::Seal => {
            HookResult::Block { output: verdict.to_hook_json() }
        }
    }
}
```

**시간 예산**: P50 ~2.3ms, P99 <10ms (Milestone C 조건).

## bin 2 — `myth-hook-post-tool`

단순 기록만. Tool이 성공한 경우.

```rust
fn run() -> HookResult {
    let input = read_hook_input()?;
    
    // latency 기록은 main()에서 공통 처리
    // 추가로 hook_events 테이블에 이벤트 append
    let db = Database::open(&myth_common::state_db_path())?;
    let events = EventStore::new(db);
    events.log_success(&input)?;
    
    HookResult::Allow
}
```

~1ms 목표.

## bin 3 — `myth-hook-post-tool-failure` (Assessor 트리거)

Decision 3의 Tier 0 + Tier 1 간소 Hybrid 구현.

```rust
fn run() -> HookResult {
    let input = read_hook_input()?;
    
    // Tier 0: deterministic classify
    if let Some(classification) = classify_deterministic(&input) {
        // 결정적 분류 성공 → LLM 호출 없이 직접 lesson 기록
        record_lesson(&input, classification)?;
        return HookResult::Allow;
    }
    
    // Tier 1: Option B (additionalContext로 Assessor Task 호출 유도)
    let reminder_id = ReminderId::new();
    let template = templates::variant_b::render(&input, reminder_id);
    
    // lesson-state.jsonl에 pending_reflection append
    let pending = PendingReflection {
        reminder_id,
        session_id: input.session_id,
        turn_n: input.turn_number,
        tool_name: input.tool_name.clone(),
        status: "pending_reflection".into(),
        ts: now(),
    };
    let writer = JsonlWriter::new(myth_common::myth_home().join("lesson-state.jsonl"));
    writer.append(&pending)?;
    
    // caselog.jsonl에 원본 기록
    let caselog = JsonlWriter::new(myth_common::caselog_path());
    caselog.append(&FailureRecord::from(&input))?;
    
    // shadow metrics
    let shadow = ShadowMetric {
        reminder_id,
        ts: now(),
        tier_resolved: 1,  // Tier 1에서 시도
        variant: "B",
    };
    JsonlWriter::new(myth_common::myth_home().join("metrics/reflector-shadow.jsonl"))
        .append(&shadow)?;
    
    // additionalContext 반환
    HookResult::AllowWithContext(json!({
        "continue": true,
        "hookSpecificOutput": {
            "hookEventName": "PostToolUseFailure",
            "additionalContext": template,
        }
    }))
}
```

### `classify_deterministic`

Decision 3의 Tier 0 — 정규식·exit code로 분류 가능한 실패를 LLM 없이 처리.

```rust
pub fn classify_deterministic(input: &PostToolFailureInput) -> Option<DeterministicClassification> {
    // Network timeout
    if TIMEOUT_RE.is_match(&input.error) {
        return Some(DeterministicClassification {
            level: Level::Low,
            category: Category::Process,
            rationale: "transient_network".into(),
        });
    }
    
    // Rate limit
    if input.error.contains("429") || input.error.contains("rate limit") {
        return Some(DeterministicClassification {
            level: Level::Low,
            category: Category::Process,
            rationale: "rate_limit".into(),
        });
    }
    
    // File not found (특정 시그니처)
    if FILE_NOT_FOUND_RE.is_match(&input.error) {
        return Some(DeterministicClassification {
            level: Level::Medium,
            category: Category::Correctness,
            rationale: "file_not_found".into(),
        });
    }
    
    // Permission denied (syscall)
    // Syntax error with line number
    // Git conflict marker 감지
    // ...
    
    None  // 애매한 경우 Tier 1로
}
```

**예상 커버리지**: 전체 실패의 20~30%. 실측 후 Milestone A에서 재평가.

## bin 4 — `myth-hook-user-prompt` (준수 감시)

Tier 1이 발동한 뒤 다음 턴에 Claude가 실제로 Assessor Task를 호출했는지 검증.

> **v0.1 Task 3.2 단순화** (Jeffrey 승인 2026-04-21)
>
> Day-1 user-prompt bin은 **read-only scan**만 수행한다:
>
> - `~/.myth/lesson-state.jsonl`의 존재·라인 수만 확인 (tracing::debug)
> - transcript 파싱 + tool_use 감시 + compliant/missed 상태 기록은 **구현하지 않음**
>
> 이유 3가지:
> 1. compliant 판정의 선행 조건 — `post-tool-failure`가 `pending_reflection` 레코드를
>    먼저 생성해야 — 는 Task 3.5에서 완성된다. Task 3.2 시점에는 비교할 pending record가
>    아직 없다.
> 2. transcript(JSONL) 파싱은 Claude Code 2.1.x의 내부 형식 실측이 별도로 필요하다. Wave 3
>    범위에 포함 여부는 Task 3.6에서 재검토.
> 3. H2 (실측: UserPromptSubmit에 `turn_number` 없음) — 원안의 `turn_n` 기반 매칭은
>    재설계 필요. Task 3.5/3.6에서 실제 데이터 구조 확정 후 구현.
>
> 아래 의사코드는 **Milestone A 이후 목표 형태**로 남겨둔다. Day-1 실구현은 위 두 줄
> (존재 확인 + 라인 수 로그)만.

```rust
fn run() -> HookResult {
    let input = read_hook_input()?;
    
    // 이전 턴의 pending_reflection 스캔
    let pending_list = scan_pending_reflections(input.session_id)?;
    
    for pending in pending_list {
        // transcript 파일에서 직전 assistant 메시지의 tool_use 검사
        let tool_uses = load_previous_turn_tool_uses(&input.transcript_path)?;
        
        let compliant = tool_uses.iter().any(|tu| {
            tu.tool_name == "Task" 
                && tu.input.get("subagent_type") == Some(&json!("assessor"))
                && tu.input.get("prompt").and_then(|p| p.as_str())
                    .map(|s| s.contains(&pending.reminder_id.to_string()))
                    .unwrap_or(false)
        });
        
        let status = if compliant { "compliant" } else { "missed" };
        
        // lesson-state.jsonl에 상태 업데이트 append (append-only이므로 새 레코드)
        let update = PendingUpdate {
            reminder_id: pending.reminder_id,
            status: status.into(),
            ts: now(),
        };
        JsonlWriter::new(myth_common::myth_home().join("lesson-state.jsonl")).append(&update)?;
        
        // shadow metrics
        let shadow = ShadowMetric {
            reminder_id: pending.reminder_id,
            ts: now(),
            tier_resolved: if compliant { 1 } else { 2 },  // 1 = Tier 1 성공, 2 = Tier 2 필요
            variant: "B",
        };
        JsonlWriter::new(myth_common::myth_home().join("metrics/reflector-shadow.jsonl"))
            .append(&shadow)?;
    }
    
    HookResult::Allow
}
```

## bin 5 — `myth-hook-stop` (Tier 2 대기)

Decision 3에서 **Day-1 비활성**. 코드는 있지만 `enable_tier2: false` 플래그로 no-op 반환.

```rust
fn run() -> HookResult {
    if !is_tier2_enabled()? {
        // Milestone A 전까지 이 경로로 빠짐
        return HookResult::Allow;
    }
    
    let input = read_hook_input()?;
    
    // stop_hook_active 체크 (무한 루프 방지)
    if input.stop_hook_active {
        return HookResult::Allow;
    }
    
    // missed pending_reflection 있는지 확인
    let missed_list = scan_missed_reflections(input.session_id)?;
    if missed_list.is_empty() {
        return HookResult::Allow;
    }
    
    // 첫 missed에 대해 Variant B 재주장
    let missed = &missed_list[0];
    let reinforced_template = templates::variant_b::render_reinforced(missed);
    
    HookResult::Block { output: json!({
        "decision": "block",
        "reason": reinforced_template,
    }) }
}

fn is_tier2_enabled() -> Result<bool> {
    // ~/.config/myth/config.yaml의 assessor.tier_2_enabled 플래그
    let config = load_config()?;
    Ok(config.assessor.tier_2_enabled)
}
```

## bin 6 — `myth-hook-session-start`

brief.md를 Claude에게 주입. 새 세션이 시작될 때마다.

```rust
fn run() -> HookResult {
    let brief_path = myth_common::brief_path();
    
    if !brief_path.exists() {
        return HookResult::Allow;  // brief 없으면 주입 없이 그냥 진행
    }
    
    let brief = std::fs::read_to_string(&brief_path)?;
    
    // 10K 문자 cap
    let brief_truncated = if brief.len() > 10_000 {
        format!("{}\n\n... (truncated, see ~/.myth/brief.md)", &brief[..9_900])
    } else {
        brief
    };
    
    let context = format!(
        "<myth-brief>\n\
         The following is the current Observer brief for the myth system.\n\
         It summarizes active lessons and recent patterns.\n\
         \n\
         {}\n\
         </myth-brief>",
        brief_truncated
    );
    
    HookResult::AllowWithContext(json!({
        "continue": true,
        "hookSpecificOutput": {
            "hookEventName": "SessionStart",
            "additionalContext": context,
        }
    }))
}
```

## Variant B 템플릿

`src/templates/variant_b.rs`:

```rust
pub fn render(input: &PostToolFailureInput, reminder_id: ReminderId) -> String {
    format!(
        r#"<user-prompt-submit-hook>
<instructions>
The tool call in the previous turn failed with a non-recoverable error.
Per myth project policy, before any remediation attempt you must dispatch
the 'assessor' subagent to analyze this failure.
</instructions>

<decision_rule>
IF error_class IN {{transient_network, rate_limit_429}}: retry once; no assessor.
ELSE IF error_class IN {{schema_mismatch, permission_denied, logic_error, assertion_fail}}:
  your next action is a single Task tool call (no other output).
ELSE: ask the user.
</decision_rule>

<example>
Task(
  subagent_type="assessor",
  description="Analyze failure of {tool_name}",
  prompt="Failure payload: {{compact_json}}. Return the strict JSON per assessor contract. reminder_id: {rid}"
)
</example>

<rationale>
Skipping reflection on this failure class causes cascading errors.
The assessor returns a JSON verdict within one turn on Haiku 4.5.
</rationale>
<assessor-trigger-id>{rid}</assessor-trigger-id>
</user-prompt-submit-hook>"#,
        tool_name = input.tool_name,
        rid = reminder_id,
    )
}

pub fn render_reinforced(missed: &MissedReflection) -> String {
    // Tier 2 재주장용 — 더 강한 어조
    format!(
        "The assessor trigger {} was not followed in the previous turn. \
         This is the second reminder. Call Task(subagent_type=\"assessor\", ...) now.",
        missed.reminder_id
    )
}
```

## `core/latency.rs`

```rust
pub fn record(event: &str, latency_ms: f64, result: &HookResult) -> Result<()> {
    let record = LatencyRecord {
        ts: now(),
        event: event.to_string(),
        latency_ms,
        result: result.label().to_string(),
        session_id: current_session_id(),
    };
    
    let writer = JsonlWriter::new(myth_common::hook_latency_path());
    writer.append(&record)
}
```

fire-and-forget. 실패 시 eprintln 경고만.

## 공통 성능 예산

각 bin의 main() 전체:

- exec + ld.so: ~0.5-1.2ms (bedrock)
- allocator init (mimalloc): ~0.1ms
- logging init: ~0.2ms
- stdin JSON 파싱: ~0.1ms
- 본 로직: crate별 (pre_tool 1~5ms, post_tool <0.5ms, etc.)
- latency 기록: ~0.1ms (fire-and-forget)
- exit/dtors: ~0.1ms

`pre_tool` 외 나머지는 대부분 2-3ms 이하.

## 테스트

```
tests/
├── pre_tool_integration.rs    # stdin JSON → 판정 → stdout JSON 왕복
├── post_tool_failure_test.rs  # deterministic classify 정확도
├── user_prompt_compliance.rs  # tool_use 감시 로직
├── session_start_brief.rs     # brief 주입
└── variant_template_test.rs   # 템플릿 렌더링
```

Mock Claude Code Hook JSON 고정 입력 → bin 실행 → stdout 검증.

## 관련 결정

- Decision 3: Tier 0 + Tier 1 (Day-1), Tier 2 (코드 있음, 비활성), Tier 3 (미구현)
- Decision 7: binary-per-hook 유지, 측정 인프라 (`latency.rs`)
- ARCHITECTURE Contract 1/2/3: 진입점·stdin/stdout·exit code
- 네이밍 카테고리 1/6: Assessor, brief, caselog
