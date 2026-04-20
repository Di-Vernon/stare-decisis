# `myth-gavel` — The Gavel 판정 로직

## 역할

Claude Code의 **PreToolUse hook**에서 발동하는 실시간 차단자. 이 crate는 **라이브러리만** 제공 (bin 없음). 실제 실행 바이너리는 `myth-hooks`의 `myth-hook-pre-tool`이 `myth-gavel`을 호출한다.

**핵심 책임**:
1. Bedrock/Foundation/Surface Rule YAML 로드
2. 47개 정규식 컴파일 → 사전 DFA
3. tool_input 매칭
4. Grid lookup (Level × Recurrence → Enforcement)
5. Fatigue tracking (세션당 알림 상한)
6. 판정 결과 → Hook JSON 직렬화

**10ms 예산 내 처리**. 매 호출 P99 <10ms 목표 (Milestone C 전환 조건).

**의존**: `myth-common`, `myth-db`.
**의존받음**: `myth-hooks`.

## Cargo.toml

```toml
[package]
name = "myth-gavel"
version = "0.1.0"
edition = "2021"
license = "MIT OR Apache-2.0"

[dependencies]
myth-common = { path = "../myth-common" }
myth-db = { path = "../myth-db" }

serde = { workspace = true }
serde_yaml = { workspace = true }
regex = { workspace = true }
regex-automata = { workspace = true }
once_cell = "1"
tracing = { workspace = true }
anyhow = { workspace = true }
```

## 모듈 구조

```
crates/myth-gavel/
├── src/
│   ├── lib.rs                  # 공개 API: Gavel
│   ├── rules/
│   │   ├── mod.rs              # RuleSet 로더
│   │   ├── bedrock.rs          # Bedrock Rule 로드 + 47개 DFA
│   │   ├── foundation.rs       # Foundation Rule
│   │   ├── surface.rs          # Surface Rule (프로젝트별 병합)
│   │   └── compile.rs          # regex → DFA 직렬화
│   ├── grid/
│   │   ├── mod.rs              # Grid (Level × Recurrence → Enforcement)
│   │   ├── default.rs          # 기본 5×6 매트릭스
│   │   └── override.rs         # grid_overrides 테이블 적용
│   ├── fatigue.rs              # 세션당 알림 상한 tracker
│   ├── verdict.rs              # Verdict 타입 + Hook JSON 변환
│   └── judge.rs                # 전체 판정 흐름
└── tests/
    └── integration/
        ├── bedrock_tests.rs    # 47개 정규식 positive/negative
        ├── grid_tests.rs
        └── end_to_end.rs
```

## 공개 API — `Gavel`

```rust
pub struct Gavel {
    rules: RuleSet,
    grid: Grid,
    // lesson_store: Box<dyn LessonStore>, // added in Task 2.3 (myth-identity integration)
    fatigue: Mutex<FatigueTracker>,         // single-owner; Arc unnecessary (no cross-thread sharing in hook binary model)
}

impl Gavel {
    pub fn init() -> Result<Self> {
        let rules = RuleSet::load_all()?;
        let db = Database::open(&myth_common::state_db_path())?;
        let grid = Grid::load(&db)?;
        // lesson_store: Task 2.3에서 `SqliteLessonStore` owned form을
        // `Box<dyn LessonStore>`로 Gavel에 주입. 현재는 필드 미보유.
        let fatigue = Mutex::new(FatigueTracker::new());
        Ok(Self { rules, grid, fatigue })
    }

    /// 단일 tool 호출에 대한 판정
    pub fn judge(&self, input: &ToolInput) -> Verdict {
        // 1. Bedrock Rule 검사 → 매칭 시 Seal
        if let Some(m) = self.rules.bedrock.match_any(&input.serialized) {
            return Verdict::seal(m);
        }

        // 2. Foundation Rule 검사
        if let Some(m) = self.rules.foundation.match_any(&input.serialized) {
            // Recurrence::I fixed until Task 2.3 (lesson_store integration).
            // 이후에는 `self.lesson_store.find_by_identity(hash)` 결과로
            // `Recurrence::from_count(lesson.recurrence_count)` 산출.
            let enforcement = self.grid.lookup(m.level, Recurrence::I);
            let enforcement = self.fatigue.lock().unwrap()
                .register(input.session_id, enforcement);
            return Verdict::with_enforcement(enforcement, m, None);
        }

        // 3. Surface Rule 검사 (동일 Grid 경로)
        if let Some(m) = self.rules.surface.match_any(&input.serialized) {
            // ... Grid lookup + fatigue (Foundation과 동일 패턴)
        }

        // 4. 어느 Rule도 매칭 안 됨
        Verdict::allow()
    }
}
```

> **v0.1 구현 중 변경** (Jeffrey 승인 2026-04-19)
>
> Task 2.1 구현에서 `Gavel` struct의 두 필드가 원안과 달라졌다:
>
> - `lesson_store: Arc<dyn LessonStore>` → **필드 자체 생략** (Task 2.3
>   myth-identity 통합 시점에 `Box<dyn LessonStore>`로 추가). 원인:
>   (a) `rusqlite::Connection`이 `!Sync`이므로 `Arc<dyn LessonStore + Sync>`
>   불가능하고, (b) Gavel은 hook 바이너리 내 **단일 소유자**이므로
>   cross-thread 공유가 필요 없다 — `Arc` 대신 `Box`가 충분.
>   Day-1 `grid_path()`는 lesson 조회 없이 Recurrence::I 고정으로 동작.
>
> - `fatigue: Arc<Mutex<FatigueTracker>>` → **`fatigue: Mutex<FatigueTracker>`**
>   로 단순화. Gavel이 단일 소유자라 Arc로 공유할 필요 없음.
>
> 이 두 결정의 **최종 확정은 Task 2.3**에서 myth-identity를 통합할 때
> 이뤄진다. myth-identity 쪽의 LessonStore 사용 패턴(cross-thread 필요
> 여부 등)에 맞춰 Box 유지 또는 Arc 복원을 선택한다.

## `RuleSet` — 정규식 컴파일

```rust
pub struct RuleSet {
    pub bedrock: CompiledRules,
    pub foundation: CompiledRules,
    pub surface: CompiledRules,
}

pub struct CompiledRules {
    rules: Vec<CompiledRule>,
    // RegexSet으로 한번에 매칭
    set: regex::RegexSet,
}

pub struct CompiledRule {
    pub id: String,           // "R1-A"
    pub item: String,         // "rm_rf_unsandboxed"
    pub regex: regex::Regex,
    pub level: Level,
    pub likelihood: Likelihood,
    pub source: Option<String>,  // "gitleaks v8.x"
}

impl CompiledRules {
    pub fn match_any(&self, text: &str) -> Option<RuleMatch> {
        let matches = self.set.matches(text);
        if !matches.matched_any() {
            return None;
        }
        
        // 첫 매칭 rule의 상세 정보 반환
        let idx = matches.into_iter().next().unwrap();
        let rule = &self.rules[idx];
        // 정밀 매칭 (어느 부분 매칭했는지)
        let m = rule.regex.find(text)?;
        
        Some(RuleMatch {
            rule_id: rule.id.clone(),
            item: rule.item.clone(),
            level: rule.level,
            matched_span: (m.start(), m.end()),
            matched_text: m.as_str().to_string(),
            identity_hash: compute_hash(text, &rule.id),
        })
    }
}
```

### YAML 로딩 예시

```yaml
# ~/.myth/bedrock-rules.yaml
version: 1
item: rm_rf_unsandboxed
description: "Unsandboxed rm -rf on production or user home"
rules:
  - id: R1-A
    pattern: '(?x) (?:^|[\s;&|`$(]) (?:sudo\s+|doas\s+)? ... rm\s+(?:-[a-zA-Z]*[rR][a-zA-Z]*[fF][a-zA-Z]*|...) \b'
    likelihood: HIGH
    source: "gitleaks v8.x (MIT)"
    level: 5
    tests:
      positive: ["rm -Rf ~", "rm -fr /", ...]
      negative: ["rm /tmp/scratch", "rm -f ./build/output.o", ...]
  - id: R1-B
    pattern: '...'
    ...

# 47개 rule 전체 (R1-A ~ R3-D)
```

로딩 시:
1. `serde_yaml`로 파싱
2. 각 rule의 regex 컴파일
3. `RegexSet` 구축 (한 번에 모든 매칭)
4. 실패 시 **fail-safe**: 로드 실패하면 **모든 tool 차단** (deny by default)

## `Grid` — 처분 매트릭스

```rust
pub struct Grid {
    matrix: [[Enforcement; 6]; 5],  // [Level-1][Recurrence-1]
    overrides: HashMap<(Level, Recurrence), Enforcement>,
}

impl Grid {
    pub fn load() -> Result<Self> {
        let mut grid = Self::default();
        // state.db의 grid_overrides 적용
        grid.apply_db_overrides()?;
        Ok(grid)
    }
    
    pub fn lookup(&self, level: Level, recurrence: Recurrence) -> Enforcement {
        // override 우선
        if let Some(e) = self.overrides.get(&(level, recurrence)) {
            return *e;
        }
        self.matrix[level as usize - 1][recurrence as usize - 1]
    }
}

impl Default for Grid {
    fn default() -> Self {
        use Enforcement::*;
        Self {
            matrix: [
                //       I         II        III       IV        V         VI
                /*L1*/ [Dismiss,  Dismiss,  Note,     Note,     Advisory, Advisory],
                /*L2*/ [Note,     Note,     Advisory, Advisory, Caution,  Caution],
                /*L3*/ [Advisory, Caution,  Caution,  Warn,     Warn,     Warn],
                /*L4*/ [Caution,  Warn,     Warn,     Strike,   Strike,   Strike],
                /*L5*/ [Strike,   Strike,   Strike,   Strike,   Strike,   Strike],
                // Bedrock 매칭은 위 매트릭스 우회, 무조건 Seal
            ],
            overrides: HashMap::new(),
        }
    }
}
```

Observer가 주간 리포트에서 "L3×III을 Caution 대신 Warn으로" 같은 제안 → Jeffrey 승인 → `grid_overrides` 테이블에 삽입 → 다음 Gavel 시작 시 반영.

## `FatigueTracker` — 세션당 알림 상한

헌법 Article 10.3: 한 세션 안에서 동일 등급 알림이 너무 많이 떠서 사용자가 "알림 피로"에 빠지는 것 방지.

```rust
pub struct FatigueTracker {
    session_counts: HashMap<SessionId, SessionFatigue>,
}

pub struct SessionFatigue {
    advisory: u32,     // 상한 2
    caution: u32,      // 상한 3
    warn: u32,         // 상한 2
    started: Timestamp,
}

impl FatigueTracker {
    pub fn check_and_increment(&mut self, session_id: SessionId, enforcement: Enforcement) -> bool {
        let fatigue = self.session_counts.entry(session_id).or_default();
        match enforcement {
            Enforcement::Advisory => {
                if fatigue.advisory >= 2 { return false; }
                fatigue.advisory += 1;
                true
            }
            Enforcement::Caution => {
                if fatigue.caution >= 3 { return false; }
                fatigue.caution += 1;
                true
            }
            Enforcement::Warn => {
                if fatigue.warn >= 2 { return false; }
                fatigue.warn += 1;
                true
            }
            _ => true,  // Strike, Seal은 항상 발동 (피로도 무관)
        }
    }
}
```

상한 초과 시 enforcement를 **한 단계 낮춤** (Warn → Caution → Note).

## `Verdict` — Hook JSON 변환

```rust
pub struct Verdict {
    pub enforcement: Enforcement,
    pub rule_match: Option<RuleMatch>,
    pub lesson_id: Option<LessonId>,
    pub rationale: String,
}

impl Verdict {
    pub fn to_hook_json(&self) -> serde_json::Value {
        match self.enforcement {
            Enforcement::Dismiss | Enforcement::Note => {
                json!({ "continue": true })
            }
            Enforcement::Advisory | Enforcement::Caution => {
                json!({
                    "continue": true,
                    "hookSpecificOutput": {
                        "hookEventName": "PreToolUse",
                        "permissionDecision": "allow",
                        "additionalContext": self.rationale,
                    }
                })
            }
            Enforcement::Warn => {
                json!({
                    "continue": true,
                    "hookSpecificOutput": {
                        "hookEventName": "PreToolUse",
                        "permissionDecision": "ask",
                        "additionalContext": self.rationale,
                    }
                })
            }
            Enforcement::Strike | Enforcement::Seal => {
                // exit 2로 block
                json!({
                    "continue": false,
                    "stopReason": self.rationale,
                })
            }
        }
    }
}
```

## 규칙 컴파일 최적화

**regex-automata 사전 DFA + `include_bytes!`**:

```rust
// build.rs (컴파일 타임)
// 47개 rule을 DFA로 미리 컴파일 → dfa-bedrock.bin 생성
// src/rules/bedrock.rs 가 이 파일을 include_bytes!로 로드

static BEDROCK_DFA: Lazy<DenseDFA<...>> = Lazy::new(|| {
    DenseDFA::from_bytes(include_bytes!("../../assets/bedrock.dfa"))
        .expect("invalid DFA")
});
```

런타임 `Regex::new()` 호출 0회. 콜드 스타트 예산 절약.

**대안 (Day-1 간단)**: 런타임 `Regex::new()` + `once_cell::Lazy` 캐시. 충분히 빠름 (~0.1ms). Milestone C 가까워지면 DFA 사전 컴파일로 전환.

## 테스트 fixtures (Day-1 560개)

```
~/myth/tests/fixtures/
├── positive/
│   ├── R1-A/
│   │   ├── 01_rm_Rf_tilde.txt
│   │   ├── 02_rm_fr_root.txt
│   │   ...
│   │   └── 20_rm_recursive_force.txt
│   ├── R1-B/
│   ...
│   └── R3-D/
└── negative/
    ├── R1-A/
    ...
    └── R3-D/
```

각 rule마다 positive 20 + negative 20 = 40. 14개 rule 그룹 × 40 = 560.

**pre-commit hook** (`~/myth/.git/hooks/pre-commit`):

```bash
#!/usr/bin/env bash
cd ~/myth
cargo test --release -p myth-gavel --test integration
# FP 0 검증
```

## 성능 예산

| 작업 | 목표 |
|---|---|
| `Gavel::init()` | LazyLock 캐시, 첫 호출만 ~1ms |
| `RuleSet::load_all()` | 47 DFA 로드 0.02~0.1ms |
| `match_any()` | RegexSet 한 번 매칭 <0.1ms |
| `Grid::lookup()` | HashMap 조회 <0.01ms |
| `FatigueTracker` | Mutex lock <0.02ms |
| `Verdict::to_hook_json()` | serde_json <0.1ms |
| **총** | **<1ms (P50)** |

나머지 hook 바이너리 오버헤드(exec, stdin, stdout)로 P99 ~5-8ms 예상.

## 관련 결정

- Decision 5: 47개 정규식 + gitleaks 차용 + test fixtures 560
- Decision 7: binary-per-hook 유지 + Milestone C 전환 조건
- ARCHITECTURE §3: The Gavel 실행 모델
