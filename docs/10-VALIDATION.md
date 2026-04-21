# myth — Validation Strategy

myth는 **실패가 비용인 시스템**이다. The Gavel이 false positive를 내면 정상 작업이 막히고, false negative를 내면 비가역 피해가 발생한다. Assessor가 잘못 분류하면 lesson이 왜곡된다. 검증은 단순 "테스트가 통과했다"가 아니라 **이 시스템이 실제로 쓸만한지**를 판단하는 활동.

이 문서는 Day-1 릴리스 전 통과해야 할 검증과, 릴리스 후 지속 모니터링 지표를 정리한다.

## 1. 검증 계층

네 가지 계층.

```
┌─────────────────────────────────┐
│ E2E: end-to-end 시나리오        │  수동 + 자동
├─────────────────────────────────┤
│ Integration: crate 간 상호작용  │  cargo test
├─────────────────────────────────┤
│ Property-based: fuzzing         │  proptest (선택)
├─────────────────────────────────┤
│ Unit: 함수·모듈 단위            │  cargo test (module-level)
└─────────────────────────────────┘
```

위에서 아래로 갈수록 느리고 적다. myth의 **합격 기준은 Unit 100% + Integration 95%+ + E2E 주요 시나리오 수동 확인**.

## 2. Unit Tests

각 crate의 `src/*.rs` 내부 `#[cfg(test)] mod tests` 또는 `tests/` 디렉토리.

### 2.1 커버리지 목표

- `myth-common`, `myth-db`: **95%+** (기반 레이어, 모든 함수 테스트 가능)
- `myth-gavel`, `myth-identity`: **90%+** (핵심 로직)
- `myth-hooks`, `myth-cli`: **75%+** (stdin/stdout 의존, 일부 통합 테스트에 위임)
- `myth-ui`: **60%+** (TUI는 상호작용 많음)
- `myth-embed`, `myth-runtime`, `myth-orchestrator`: **70%+** (subprocess·network 의존)

**측정**: `cargo tarpaulin` 또는 `cargo llvm-cov`:

```bash
cargo install cargo-llvm-cov
cd ~/myth/rust
cargo llvm-cov --workspace --html
# target/llvm-cov/html/index.html 열기
```

### 2.2 필수 테스트 영역

**`myth-common`**:
- `Level::from_count` 경계값 (0.99, 1.0, 1.01, 11.99, 12.0)
- `Enforcement::is_blocking` 논리
- XDG 경로 함수들 (환경변수 override 포함)

**`myth-db`**:
- Fresh DB에 v1 마이그레이션 성공
- 이미 v1 DB에 재시작 → 변경 없음
- WAL 모드 활성 확인 (`PRAGMA journal_mode`)
- Audit chain tamper 감지

**`myth-gavel`**:
- 47개 패턴 개별 positive 매칭
- 47개 패턴 개별 negative 비매칭
- RegexSet 빈 set → Verdict::allow
- Grid lookup 30칸 전부
- FatigueTracker 세션당 상한 동작

**`myth-identity`**:
- Normalize: timestamp·UUID·path·hex 각각 치환
- SHA1 hash 결정론
- InMemoryStore: upsert N개 → knn(k=5) 거리 정렬
- vectors.bin 손상 (magic/version/size 각 경우) 감지

**`myth-hooks`**:
- stdin JSON 파싱 (누락 필드·추가 필드 모두)
- Hook 출력 JSON의 `continue/permissionDecision/additionalContext`
- Variant B 템플릿 변수 치환

**`myth-embed`**:
- bincode Request/Response 직렬화 왕복
- Length prefix 1 MB 초과 거부
- Version mismatch 에러

### 2.3 실행

```bash
cd ~/myth/rust
cargo test --workspace --release
# 또는 nextest (빠름)
cargo install cargo-nextest
cargo nextest run --workspace --release
```

## 3. Integration Tests

각 crate의 `tests/` 디렉토리. 여러 모듈·외부 자원(SQLite, 파일시스템)과 상호작용.

### 3.1 필수 시나리오

**DB + JSONL 동기화**:
- Lesson 생성 → `lessons` 테이블 + `lesson-state.jsonl`에 동시 기록
- Audit 이벤트 → `audit.jsonl` + `lessons` 상태 일치

**Gavel + Identity**:
- 같은 tool_input 2회 → 첫 번째 match, 두 번째 recurrence_count++
- `vectors.bin` 업데이트 후 knn 반영 확인

**Hook 바이너리 왕복** (고정 stdin → stdout):
```rust
#[test]
fn test_pre_tool_hook_roundtrip() {
    let input = r#"{"session_id":"abc","tool_name":"Bash","tool_input":{"command":"ls"}}"#;
    let output = run_binary("myth-hook-pre-tool", input.as_bytes());
    let parsed: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(parsed["continue"], json!(true));
}
```

**Embed daemon 생명주기**:
- Spawn → Ping → Embed → Shutdown
- 동시 2개 클라이언트 Embed
- Idle 15분 시뮬레이션 → 자동 종료

**Orchestrator 병렬 실행**:
- 3 task 동시 실행 (mock-claude 사용)
- 하나 실패 → 다른 2개 영향 없음
- 격리 확인 (각자 worktree)

### 3.2 공유 fixture

```rust
// tests/common/mod.rs
pub fn tempdir_with_myth_home() -> TempDir {
    let dir = TempDir::new().unwrap();
    std::env::set_var("MYTH_HOME_OVERRIDE", dir.path());
    // 기본 rules yaml 복사
    std::fs::copy("../../../templates/bedrock-rules.yaml", 
                  dir.path().join("bedrock-rules.yaml")).unwrap();
    // ... 기타
    dir
}
```

각 integration test가 격리된 환경에서 실행.

## 4. Bedrock Rule Fixture 검증 (560개)

myth의 가장 중요한 검증.

### 4.1 구조

```
~/myth/tests/fixtures/
├── positive/
│   ├── R1-A/    # 20 files, each contains matching command
│   │   ├── 01_rm_rf_tilde.txt
│   │   ├── 02_rm_Rf_root.txt
│   │   └── ...
│   ├── R1-B/
│   └── ...  (14 rule groups total)
└── negative/
    ├── R1-A/    # 20 files, each contains legitimate command
    │   ├── 01_rm_local_file.txt
    │   └── ...
    └── ...
```

### 4.2 검증 테스트

```rust
// ~/myth/rust/crates/myth-gavel/tests/bedrock_fixtures.rs
use myth_gavel::Gavel;
use std::fs;
use std::path::Path;

const RULE_GROUPS: &[&str] = &[
    "R1-A", "R1-B", "R1-C", "R1-D", "R1-E", "R1-F", "R1-G",  // rm_rf
    "R2-A", "R2-B", "R2-C", "R2-D",                          // secrets
    "R3-A", "R3-B", "R3-C", "R3-D",                          // auth_bypass
];

#[test]
fn positive_fixtures_match() {
    let gavel = Gavel::init().unwrap();
    let fixtures_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent().unwrap().parent().unwrap().parent().unwrap()
        .join("tests/fixtures/positive");
    
    let mut failures = Vec::new();
    
    for group in RULE_GROUPS {
        let group_dir = fixtures_dir.join(group);
        for entry in fs::read_dir(&group_dir).unwrap() {
            let path = entry.unwrap().path();
            let content = fs::read_to_string(&path).unwrap();
            
            let verdict = gavel.judge_text(&content);
            
            if !verdict.is_blocking() {
                failures.push(format!(
                    "{}/{:?}: expected match for group {}, got {:?}",
                    group, path.file_name().unwrap(), group, verdict.enforcement
                ));
            }
        }
    }
    
    assert!(failures.is_empty(), "Positive fixture failures:\n{}", failures.join("\n"));
}

#[test]
fn negative_fixtures_do_not_match() {
    let gavel = Gavel::init().unwrap();
    let fixtures_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent().unwrap().parent().unwrap().parent().unwrap()
        .join("tests/fixtures/negative");
    
    let mut false_positives = Vec::new();
    
    for group in RULE_GROUPS {
        let group_dir = fixtures_dir.join(group);
        for entry in fs::read_dir(&group_dir).unwrap() {
            let path = entry.unwrap().path();
            let content = fs::read_to_string(&path).unwrap();
            
            let verdict = gavel.judge_text(&content);
            
            if verdict.is_blocking() {
                false_positives.push(format!(
                    "{}/{:?}: unexpected match for {}, got {:?}",
                    group, path.file_name().unwrap(), group, verdict.enforcement
                ));
            }
        }
    }
    
    // FP = 0 is required
    assert!(false_positives.is_empty(), 
        "FALSE POSITIVES DETECTED (must be zero):\n{}", 
        false_positives.join("\n"));
}
```

### 4.3 pre-commit hook

```bash
# ~/myth/.git/hooks/pre-commit
#!/usr/bin/env bash
set -euo pipefail
cd ~/myth/rust

echo "Running Bedrock fixture validation..."
cargo test --release -p myth-gavel --test bedrock_fixtures 2>&1 | tail -20

if [ $? -ne 0 ]; then
    echo "❌ Bedrock fixtures failed. Commit blocked."
    exit 1
fi

echo "✓ Bedrock fixtures passed"
```

chmod +x로 실행 권한.

### 4.4 FN 검증 (False Negative)

**FP=0은 엄격**하지만 **FN=0은 불가능**. 새로운 공격 패턴은 항상 생긴다.

대신 **FN 유입 프로세스**:
1. Jeffrey가 운영 중 myth가 놓친 공격 목격
2. `myth lesson appeal`로 해당 건 기록
3. Observer 주간 분석에서 새 패턴 제안
4. Jeffrey 승인 → `~/.myth/bedrock-rules.yaml` 수정 + 새 fixture 추가
5. `tests/fixtures/positive/` 갱신
6. PR (Git commit)

## 5. Property-Based Tests (선택)

`proptest` crate로 생성적 검증.

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn normalize_is_idempotent(s: String) {
        let once = normalize_aggressive(&s);
        let twice = normalize_aggressive(&once);
        prop_assert_eq!(once, twice);
    }
    
    #[test]
    fn recurrence_from_count_monotonic(a in 0.0f64..100.0, b in 0.0f64..100.0) {
        if a < b {
            let ra = Recurrence::from_count(a) as u8;
            let rb = Recurrence::from_count(b) as u8;
            prop_assert!(ra <= rb);
        }
    }
    
    #[test]
    fn bincode_roundtrip(op in any::<Op>()) {
        let serialized = bincode::serialize(&op).unwrap();
        let deserialized: Op = bincode::deserialize(&serialized).unwrap();
        prop_assert_eq!(op, deserialized);
    }
}
```

선택 사항. Day-1 필수는 아님. Milestone 이후 의심 버그가 반복되면 추가.

## 6. E2E 시나리오 (수동 + 자동)

### 6.1 Core Scenario 1 — 새 프로젝트에 myth 설치

```bash
# 수동 실행 체크리스트
mkdir -p /tmp/myth-e2e-1
cd /tmp/myth-e2e-1
git init
echo "# test" > README.md
git add README.md && git commit -m "init"

# myth init
myth init
test -f .claude/settings.json
test -f .claude/agents/assessor.md
test -f .claude/agents/observer.md

# Settings 검증
jq '.hooks | keys | length' .claude/settings.json
# 기대: 6

# CLAUDE.md 템플릿 (선택)
test -f CLAUDE.md

# PASS
```

### 6.2 Core Scenario 2 — The Gavel 차단

```bash
cd /tmp/myth-e2e-1

# Bedrock 매칭 시뮬레이션 (실제 Claude Code 없이)
echo '{
  "session_id": "test-session",
  "tool_name": "Bash",
  "tool_input": {"command": "rm -rf /"}
}' | myth-hook-pre-tool
# 기대: stdout에 {"continue": false, "stopReason": "Bedrock Rule R1-A matched..."}
# exit code 2
```

### 6.3 Core Scenario 3 — Failure → Lesson 흐름

```bash
# PostToolUseFailure 시뮬레이션
echo '{
  "session_id": "test-session",
  "tool_name": "Bash",
  "tool_input": {"command": "cat /nonexistent"},
  "tool_response": {"exit_code": 1, "stderr": "cat: /nonexistent: No such file or directory"}
}' | myth-hook-post-tool-failure
# 기대: Variant B template 반환

# caselog 확인
cat ~/.myth/caselog.jsonl | tail -1
# 기대: 방금 failure 이벤트
```

### 6.4 Core Scenario 4 — Observer 주간 분석

```bash
# 인위적으로 caselog에 이벤트 여러 개 추가 (테스트 데이터)
# ... 또는 Scenario 3을 N번 반복

myth observer run --dry
# 기대: 생성된 brief.md 내용을 stdout에 출력
# - Analyzed N events
# - X new lessons
# - Migration Readiness 섹션
```

### 6.5 Core Scenario 5 — Lesson 관리

```bash
myth lesson list
# 기대: 최대 20개 lesson (Scenario 3에서 만든 것)

# 특정 lesson show
LESSON_ID=$(myth lesson list --format json | jq -r '.[0].id')
myth lesson show "$LESSON_ID"

# Appeal
myth lesson appeal "$LESSON_ID" --reason "I believe this is correct behavior"
myth lesson list --status pending
# 기대: 항소 pending 상태
```

### 6.6 자동화된 E2E

`~/myth/tests/integration/e2e.sh`:

```bash
#!/usr/bin/env bash
set -euo pipefail

E2E_DIR=$(mktemp -d)
trap "rm -rf $E2E_DIR" EXIT

echo "▶ Scenario 1: myth init"
cd "$E2E_DIR"
git init -q
myth init
test -f .claude/settings.json || exit 1
echo "  ✓"

echo "▶ Scenario 2: Gavel block"
output=$(echo '{"session_id":"t","tool_name":"Bash","tool_input":{"command":"rm -rf /"}}' | myth-hook-pre-tool 2>&1 || true)
echo "$output" | grep -q '"continue": false' || exit 1
echo "  ✓"

# Scenario 3, 4, 5 유사하게 추가

echo ""
echo "All E2E scenarios passed."
```

CI 또는 `~/myth/scripts/run-e2e.sh`로 통합.

## 7. 성능 검증

### 7.1 Hook Latency

```bash
# Pre-tool 100회 측정
hyperfine --warmup 3 --runs 100 \
    'echo "{\"session_id\":\"x\",\"tool_name\":\"Bash\",\"tool_input\":{\"command\":\"ls\"}}" | myth-hook-pre-tool'

# 목표
# Mean < 5 ms
# Max < 15 ms
# P99 < 10 ms
```

### 7.2 myth-embed Hot Path

```bash
# 데몬 띄우기
myth embed probe "warmup" > /dev/null

# 50회 측정
hyperfine --warmup 3 --runs 50 \
    'myth-embed probe "the quick brown fox"'

# 목표
# Mean 8-20 ms
```

### 7.3 전체 빌드

```bash
cd ~/myth/rust
time cargo build --release
# 목표: 8 CPU core에서 < 10분
```

### 7.4 메모리 사용

```bash
# myth-embed 데몬 RSS
myth embed status
# 기대: RSS ~150 MB (multilingual-e5-small 로드 후)

# myth-hook-pre-tool 단일 호출 peak RSS
/usr/bin/time -v echo '...' | myth-hook-pre-tool 2>&1 | grep "Maximum resident"
# 기대: < 30 MB
```

## 8. Security 검증

### 8.1 라이선스 감사

```bash
~/myth/scripts/license-audit.sh
# GPL/LGPL/AGPL 포함 시 exit 1
```

### 8.2 파일 권한

```bash
# API key (Milestone A 이후)
stat -c '%a' ~/.config/myth/api_key
# 기대: 600

# config 디렉토리
stat -c '%a' ~/.config/myth
# 기대: 700

# runtime socket
stat -c '%a' "$XDG_RUNTIME_DIR/myth/embed.sock"
# 기대: 600
```

### 8.3 Injection 저항

Hook 입력은 **JSON**. Shell interpolation 없음. 그러나 `tool_input.command`가 정규식에 들어가므로 catastrophic backtracking 가능성:

```bash
# ReDoS 테스트
echo '{
  "session_id":"x",
  "tool_name":"Bash",
  "tool_input":{"command":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaa!"}
}' | timeout 1s myth-hook-pre-tool
# 기대: 1초 이내 정상 반환 (timeout 안 걸림)
```

`regex::Regex`는 DFA 기반이라 polynomial guarantee. 이론적으로 안전하나 실측 확인.

## 9. Day-1 Definition of Done

다음 모두 green이면 myth v0.1.0 릴리스 가능:

**Unit/Integration**:
- [ ] `cargo test --workspace --release` 전체 pass
- [ ] `cargo clippy --workspace -- -D warnings` 0 warning
- [ ] `cargo fmt --all --check` 0 diff

**Python**:
- [ ] `pytest python/tests/` pass
- [ ] `mypy myth_py` 0 error (strict mode 선택)
- [ ] `ruff check python/` 0 warning

**Fixtures**:
- [ ] Positive 280 전부 매칭
- [ ] Negative 280 전부 비매칭 (FP=0)

**E2E**:
- [ ] `~/myth/scripts/run-e2e.sh` 전 시나리오 pass
- [ ] `myth doctor` 전 항목 green (rules 파일 있는 상태)

**Performance**:
- [ ] Hook pre-tool P99 < 10ms
- [ ] embed hot embed P50 < 20ms
- [ ] cargo build --release < 10분

**Security**:
- [ ] License audit pass (no copyleft)
- [ ] API key file 권한 0600
- [ ] ReDoS 테스트 통과

**Build Artifacts**:
- [ ] 8개 Rust 바이너리 존재 (`target/release/`)
- [ ] 2개 Python shim script (`~/.local/bin/myth-{assessor,observer}`)
- [ ] 28개 문서 (`~/myth/**/*.md`)

**Git**:
- [ ] 8개 wave commit
- [ ] v0.1.0 태그

## 10. 운영 중 모니터링 (Day-1 이후)

매주 Jeffrey가 확인:

**Observer brief.md**:
- Tier 1 compliance rate (Milestone A 판단용)
- Hook P99 latency (Milestone C 판단용)
- 신규 lesson 수, 재발 lesson 수
- Lapse 전환 건수

**hook-latency.ndjson**:
- P50/P99 변화 추세
- 단일 outlier (50ms 초과) 건수

**caselog.jsonl**:
- 주간 failure 빈도
- 카테고리 분포 (security/correctness/process/data_safety/temporal)

**tier3-dispatch.jsonl** (Milestone A 이후):
- 월간 API 비용
- 호출 빈도

### 지표 임계값

| 지표 | 정상 범위 | 경고 | 심각 |
|---|---|---|---|
| Hook P99 | < 10ms | 10~15ms | > 15ms (2주+) → Milestone C |
| Embed latency P50 | < 20ms | 20~50ms | > 50ms |
| Tier 1 compliance | > 85% | 70~85% | < 70% → Milestone A |
| 주간 lesson 증가 | 1~10 | 10~50 | > 50 |
| Bedrock miss (Observer) | 0 | 1~2 | > 2 → Milestone D |
| Tier 3 비용 | < $2/월 | $2~5 | > $5 → 튜닝 |

## 11. 회귀 테스트

새 rule을 Bedrock에 추가할 때:

```bash
# 1. 새 rule 정의
vim ~/.myth/bedrock-rules.yaml

# 2. positive/negative fixture 작성
mkdir ~/myth/tests/fixtures/positive/R1-H
# 20개 positive + 20개 negative 파일

# 3. 전체 검증
cd ~/myth/rust
cargo test --release -p myth-gavel --test bedrock_fixtures

# 4. 기존 fixtures와 충돌 없는지 확인 (새 rule이 기존 negative에 매칭되면 FP)
# 실패 시 rule 패턴 조정

# 5. 커밋
git add ~/.myth/bedrock-rules.yaml ~/myth/tests/fixtures/
git commit -m "bedrock: add R1-H pattern for <description>"
```

## 12. Fuzzing (선택, Milestone 이후)

`cargo-fuzz`로 parse 로직 fuzzing:

```bash
cargo install cargo-fuzz
cd ~/myth/rust/crates/myth-hooks
cargo fuzz init
# 타겟: hook input JSON 파싱
cargo fuzz run parse_hook_input -- -max_total_time=300
```

Day-1 범위 밖. 운영 중 crash 보고 누적 시 도입 검토.

## 관련 문서

- `~/myth/docs/08-BUILD-SCOPE.md` — Day-1 DoD (섹션 6)
- `~/myth/docs/09-CLAUDE-PROMPTS.md` — 각 Wave의 테스트 작업
- `~/myth/docs/11-RISKS.md` — 검증으로 커버되지 않는 리스크

---

## Wave 7 실제 통계 (Wave 8 Task 8.4 sync)

### Bedrock Rule

- **Entries**: 15 (R1-A..G, R2-A..D, R3-A..D) — CONSTITUTION Article 7 §
  Bedrock "정확히 3개 아이템" 준수 (item = rm_rf_unsandboxed, 
  production_secrets_commit, auth_bypass_production)
- **Alternation branches**: 209 총합 (R2-A가 40 provider prefix 병합 포함)
- **Detection signatures 커버리지**: Decision 5 "47 patterns" 기준 **54개**
  (R1-A~G 7 + R2-A 40 prefix + R2-B~D 3 + R3-A~D 4). Day-1 요구치 상회.

### Foundation Rule

- **Entries**: 5 (F1-A main_force_push, F2-A no_verify_ci_bypass, F3-A
  pii_exfiltration, F4-A unverified_dependency, F5-A
  untrusted_arbitrary_execution)

### Grid

- **Cells**: 30 (Level 1-5 × Recurrence I-VI)
- **Coordinate system**: Level × Recurrence → Enforcement (rule id 미참조).
  Rule.level 필드 + Lesson.recurrence_count → Grid::lookup → Enforcement
  3-step 로직.
- **런타임 로드**: Grid::load()는 DB `grid_overrides` 테이블만 조회.
  templates/grid.yaml 편집은 런타임 반영 안 됨 (docs/07 sub-1b 참조).

### Fixtures

- **Positive**: 280 cases (15 files, 18-20 per rule)
- **Negative**: 280 near-miss cases (15 files, 18-20 per rule)
- **Total**: 560
- **FP=0 / FN=0**: a02_fixtures_full_sweep 통과
- **Sanity gate**: a01_harness_sanity (5 obvious positive + 5 obvious
  negative) — 해시 자체 버그 조기 감지

### 테스트 수치 (Wave 6 baseline 대비)

- Wave 6 baseline: Rust 268 / Python 36
- Wave 7 종료: Rust 275 / Python 36 (net +7: template_sanity 2 +
  fixture_harness 2 + tier0_concurrent 1 + lesson split/merge 2)
- Wave 8 누적: Task별 점진 추가 (Task 8.3 tier3 gate 1 등)

### Harness 판정 단위 (sub-1c)

- Entry 단위 (rule_id). Internal alternation 커버리지는 fixture 다양성으로
  간접 검증. 특정 alternation 누락 시 해당 case fixture-FN으로 표면화됨.

### Bench ceilings (fs4 검증 기준)

| 시나리오 | Wave 6 상한 | Wave 7 실측 P99 |
|---|---|---|
| pre_tool | 36.6 ms | 31.82 ms |
| post_tool | 35.2 ms | 32.06 ms |
| post_tool_failure_tier0 | 44.9 ms | 37.27 ms |

fs2 → fs4 migration 후 상한 전부 준수.
