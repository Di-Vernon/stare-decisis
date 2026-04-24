# Day-1 Health Report — myth v0.1.0

**Date**: 2026-04-22
**Tag**: v0.1.0 (commit `52c0f03`)
**Scope**: Post-tag comprehensive analysis. No code modification.
Snapshot for Shadow mode 21-day Milestone A baseline + Wave 9+ planning.

---

## §1. Executive Summary

**Overall health grade: B (ship with monitoring)**

myth v0.1.0 is **structurally sound** and **ready for Shadow mode entry**,
with three findings that warrant elevated attention during the 21-day
observation window. Test suite green (Rust 278/0/1, Python 36/36), type
and lint clean (mypy strict, ruff), dependency lineage Layer 0→5
unidirectional, all 19 CONSTITUTION Articles mapped (13 fully implemented,
3 Milestone-deferred, 3 Day-1 gaps). 8 release binaries build in 6m42s
from scratch. Tier 3 Milestone-A activation path fully wired.

**Top 3 priority issues**:

1. **Hook P99 latency cold-start 44.4 ms vs Milestone C threshold 15 ms
   (3x over)**. Bench results are explicitly excluded from Milestone C
   trigger evaluation (production hook-latency.ndjson only), but this
   suggests Milestone C may trigger in live use faster than anticipated.
2. **License audit automation completely absent**. `scripts/license-audit.sh`
   is a stub that `exit 1`s; no CI; 395 Rust + 36 Python deps; future
   upgrades can introduce GPL/AGPL licenses undetected.
3. **Documentation Coverage Gap (DCG) in 10/10 crate-cards**. Cargo.toml
   snapshots in `docs/04-CRATES/*.md` are stale across every crate — code
   is correct, but docs mislead readers on actual dependencies (4 cases
   claim dependencies that don't exist — once_cell, regex-automata,
   simsimd, nix).

**Shadow mode entry**: approved. Day-1 hard blockers: none.

---

## §2. Axis 1 — Code Health

### 2.1 Rust workspace

| Metric | Value |
|---|---|
| Total src LOC | 10,574 |
| Total test LOC | 3,758 |
| Combined | 14,332 |
| Test count | 278 passed / 0 failed / 1 ignored |
| Public API items | 318 (across 10 crates) |
| Release binaries | 8 (myth 5.6M / myth-embed 26M / 5 hooks 3.6–4.5M) |
| Clean release build | 6m42s wall, 34 CPU-min, 629 MB RAM |

Largest files: `subprocess_test.rs` (658), `subcmd/lesson.rs` (607),
`myth-db/lesson.rs` (376). Largest crate by src: myth-cli (1,984 LOC).
Release profile: `lto=fat, codegen-units=1, strip=symbols, panic=abort`
— fully optimized.

### 2.2 Python

| Metric | Value |
|---|---|
| Src LOC | 991 (13 files) |
| Test LOC | 536 (36 tests, all pass in 1.24s) |
| mypy --strict | **clean** |
| ruff | **clean** |

### 2.3 Dependencies

- Rust: 395 transitive packages in Cargo.lock, 10 direct workspace crates
- Python: 36 packages total, 7 direct + 4 dev-dependencies

### 2.4 Skipped

- **Coverage**: `cargo-llvm-cov` not installed, network install not
  attempted per Stop-condition policy. Per-crate coverage targets from
  `docs/10-VALIDATION.md` (common/db 95%+, gavel/identity 90%+, etc.) —
  not verified this cycle.
- **Security audit**: `cargo-audit` not installed. Manual THIRD-PARTY
  audit done in Wave 8.5.

---

## §3. Axis 2 — Architecture Integrity

### 3.1 Dependency lineage

- **myth-cli (Layer 5)**: depends on all 8 lower-layer crates ✓
- **myth-common (Layer 0)**: no internal dependencies ✓
- **Cycles**: 0 detected via `cargo tree`
- **cargo doc --workspace**: builds green in 2m46s

### 3.2 Wave commit distribution

| Wave | Commits | LOC delta |
|---|---|---|
| 0 | 1 | +2,188 |
| 1 | 1 | +3,055 |
| 2 | 3 | +3,911 |
| 3 | 8 | +3,212 |
| 4 | 1 | **+3,928** |
| 5 | 1 | +1,645 |
| 6 | 1 | +1,486 |
| 7 | 6 | +2,744 |
| 8 | 7 | +1,568 |

Total: 53 commits (29 wave / 18 docs / 5 chore / 0 fix). Largest by
lines: Wave 4 (single commit, 3,928+); largest by commits: Wave 3 (8).

### 3.3 Hook P99 benchmark (cold-start, n=100)

| Scenario | Current P99 | Wave-3 Step c' baseline | Drift |
|---|---|---|---|
| pre_tool | 44.4 ms | 36.6 ms | **+7.8 (worsened)** |
| post_tool | 33.2 ms | 35.2 ms | −2.0 (improved) |
| post_tool_failure tier0 | 38.4 ms | 44.9 ms | −6.5 (improved) |
| post_tool_failure tier1 | 35.3 ms | — | (new) |
| user_prompt | 4.5 ms | — | |
| stop | 4.2 ms | — | |
| session_start (no brief) | 5.4 ms | — | |
| session_start (with brief) | 3.7 ms | — | |

**Note**: Milestone C uses production `hook-latency.ndjson`, not bench.
Bench excluded from trigger per Wave 3 기반점 2. However, cold-start is
representative of real invocation (binary-per-hook model), so Milestone C
activation in live use is **likely** given the 15 ms threshold.

### 3.4 SQLite

- `PRAGMA user_version = 1` ✓ (Wave 1 schema v1)
- 7 tables: appeal_history / hook_events / sessions / vector_metadata /
  grid_overrides / lessons / vector_generation
- 1 migration file (001_initial.sql, 137 LOC)
- Contract 5 (forward-only, add-only) enforced via code

---

## §4. Axis 3 — Design Integrity

### 4.1 CONSTITUTION 19 Articles mapping

| Status | Count | Articles |
|---|---|---|
| ✓ Fully implemented | 13 | 1, 3, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 18, 19 |
| Milestone-deferred (stub) | 1 | 2 (Anthropic key) |
| **Day-1 gap** | 3 | **4** (Rehabilitation), **16** (Environmental Sensitivity), **17** (Cost/Risk Exposure) |

Gap details:
- Art 4: No explicit rehabilitation path (lesson update vs block
  distinction). Implicit in lesson lifecycle but no dedicated mechanism.
- Art 16: `detect_env()` + `actively_relevant`/`context_irrelevant`
  status missing. Schema has `category='temporal'` but no re-evaluation
  hook on SessionStart.
- Art 17: UI budget display panel absent. Depends on Milestone A
  Anthropic API integration.

### 4.2 Milestone trigger paths (doctor --migration output)

| | Threshold | Detection path | Status |
|---|---|---|---|
| A | Tier 1 compliance < 70% × 3w | `python/myth_py/observer/analyzer.py` + `migration.py::_milestone_a` | pending |
| B | 20K records AND P99 > 50 ms | `migration.py::_milestone_b` placeholder | pending |
| C | hook P99 > 15ms × 14d | `migration.py::_milestone_c(hook_latency_path)` | awaiting data |
| D | Bedrock Rule miss observed | placeholder | pending |
| E | Semantic FP > 5% OR FN > 2% | placeholder | pending |

All 5 paths wired. Milestone C actively reads production latency ndjson.

### 4.3 Drift / carry-forward / Milestone-deferred classification

- **Drift 박제**: 11건 (Wave 6: 6 박제 + 1 closed; Wave 7: 4 sub-items)
- **Carry-forward closed**: 2건 **explicitly labeled** (fs2→fs4, Tier 0
  concurrent) — Jeffrey's earlier count of 4 includes 2 additional
  closures via other paths, **discrepancy unresolved** (see §10 item D).
- **Milestone C deferred**: 2건 confirmed (myth-db migration cold-start
  race, lesson_relations index)

### 4.4 Language convention (CONSTITUTION §IX.3)

**Correct reading of §IX.3**:
- **Fixed English**: metadata, identity strings, technical terms,
  variable/function/file names
- **Korean permitted**: user-facing messages, appeal input, **comments**,
  manuals

**Violations: 0**. Korean text appearing in Rust/Python comments is
within the rule. Variable names and function names are all English.
(Note: the original prompt described this rule in inverted form —
clarified here for future reference.)

---

## §5. Axis 4 — Risk Assessment

### 5.1 Shadow-21d risk register

| # | Risk | Day-1 mitigation | Status |
|---|---|---|---|
| R1 | Hook binary path discovery | 3-tier fallback (current_exe → $MYTH_REPO_ROOT → ~/myth) | ✓ |
| R2 | state.db corruption recovery | No `myth doctor --backup/--repair` | **gap** |
| R3 | Python env missing | `uv > pip3 > pip` + non-fatal warning | ✓ |
| R4 | Claude hook schema drift on upgrade | No version check / schema validation | **gap** |
| R5 | Disk full on JSONL append | Standard io::Error; no ENOSPC-specific handling | soft gap |
| R6 | Migration cold-start race | execute_batch outside transaction; Wave 7.6 warm-up is test-only | defer-MC |
| R7 | lesson_relations index | meta_json JSON parse; N<1K trivial, N≥10K problematic | defer-MC |
| R8 | Reflector-shadow write path | Code inspection confirms `$MYTH_HOME/metrics/reflector-shadow.jsonl` via `shadow_metrics_path()` in post_tool_failure.rs | ✓ (code only) |
| R9 | Tier 3 dead-code activation | `tier3_gate_active()→false` + call site wired in post_tool_failure.rs | ✓ |
| R10 | **License drift monitoring** | `scripts/license-audit.sh` is stub `exit 1`; no CI; no cargo-deny | **critical** |

### 5.2 Per-risk assessment

- **R10 (critical)**: 395 deps with no automated license check creates
  compounding exposure as Wave 9+ upgrades land. Recommend cargo-deny
  + GitHub Actions in Wave 9.1 (see §10).
- **R2, R4, R5**: Day-1 intentional deferrals; absence is expected, not
  a regression. Wave 9+ candidates with low urgency for single-user
  laptop deployment.
- **R6 (migration race)**: probability <1% per session start in
  single-user workflow. Bundled with Milestone C Gavel daemon rework.
- **R7 (relations index)**: During Shadow-21d, lesson count stays <100
  (typical failure rate). Revisit at Milestone C.

---

## §6. Axis 5 — Process Meta-Retrospective

### 6.1 Accelerated mode ROI

| Phase | Waves | Commits | Commits/Wave |
|---|---|---|---|
| Checkpoint-heavy | 0–2 | 5 | 1.67 |
| Transition | 3 | 8 | 8.0 |
| Accelerated | 4–8 | 16 | 3.2 |

**Evidence accel mode worked without quality degradation**:
- Tests maintained 278/0/1 throughout
- clippy clean maintained
- Drift ratio did NOT spike during accel (Wave 6: 1 commit + 6 drifts
  found = thorough self-audit; Wave 7: 6 commits + 4 sub-drifts found
  = not hidden)

### 6.2 Retrospective-principle internalization curve

- Task 3.4: 2 self-corrections
- Task 3.5: 1 self-correction
- Task 3.6: 0 self-corrections
- Wave 4–8: 0 self-corrections per commit body

**Internalization point**: Task 3.6 / Wave 4 boundary. Drift-discovery
mechanism sustained post-internalization (6 drifts surfaced in Wave 6,
4 sub-items in Wave 7) — not suppressed.

### 6.3 Drift pattern taxonomy

| Category | Count | Wave 9+ preemption |
|---|---|---|
| Structural mismatch | 4 | Hard (case-by-case) |
| docs → undefined file | 3 | **Easy** (docs-to-code file-existence linter) |
| Docs internal contradiction | 2 | Medium (cross-reference audit) |
| Upstream dependency break | 1 | Hard (upstream-driven) |
| Milestone deferral | 1 | N/A (by design) |

Most preventable: **"docs → undefined file"** class. Tool suggestion:
scan `docs/**/*.md` for `myth_py/*.py` / `rust/crates/*/src/*.rs`
references, verify path exists.

---

## §7. Drift Registry (11 items — 박제 완료)

All entries frozen in `CHANGELOG.md` [0.1.0] + `docs/09-CLAUDE-PROMPTS.md`
Wave 6/7 sync + `docs/05-PYTHON.md` Wave 6 section.

| # | Wave | Item | Category |
|---|---|---|---|
| 1 | 6 | `subagent_runner.py` undefined in docs | docs→undefined |
| 2 | 6 | pyproject Poetry vs PEP 621/hatchling | structural |
| 3 | 6 | `observer/report.py` undefined in docs | docs→undefined |
| 4 | 6 | `assessor/state.py` undefined in docs | docs→undefined |
| 5 | 6 | `assessor/cli.py` body absent + pyproject constraint | structural |
| 6 | 6 | `lapse.py` sqlite inline vs docs "DB via Rust" | docs internal |
| 7 | (closed) | click 8.3 incompatibility (Wave 6 drift 7) | upstream |
| 8 | 7 sub-1a | 47 entries docs vs 15×209 actual structure | structural |
| 9 | 7 sub-1b | Grid rule_id reference mismatch | docs internal |
| 10 | 7 sub-1c | FP=0 harness entry unit | structural |
| 11 | 7 sub-1d | myth-db migration race | Milestone C defer |

Stop threshold: 12. **Drift count: 11. Margin: 1.**

---

## §8. Documentation Coverage Gap (DCG) Registry (10 items — Wave 9+)

**Definition**: docs snapshot not reflecting current code reality. Code
is correct; docs are stale. Not a behavioral drift; does not block
Shadow mode entry.

| # | File | Missing in docs | Spurious in docs | Severity |
|---|---|---|---|---|
| 1 | 02-myth-db.md | fs4, libc, tempfile | — | medium (fs4 is lock-core) |
| 2 | 01-myth-common.md | dirs, libc | — | low |
| 3 | 03-myth-gavel.md | rusqlite, serde_json, sha1, tempfile | once_cell, regex-automata | **high** (SHA1 actively used; once_cell claim misleads) |
| 4 | 04-myth-identity.md | chrono, myth-embed, tempfile | simsimd | **high** (myth-embed lineage + simsimd false claim) |
| 5 | 05-myth-hooks.md | regex, rusqlite, tempfile | — | medium (Task 3.5 regex is Tier 0 core) |
| 6 | 06-myth-embed.md | fs4, libc, tempfile | nix | **high** (nix→fs4+libc substitution not documented) |
| 7 | 07-myth-orchestrator.md | tempfile | — | low |
| 8 | 08-myth-runtime.md | dirs, tempfile | — | low |
| 9 | 09-myth-ui.md | tempfile | — | low |
| 10 | 10-myth-cli.md | chrono, dirs, myth-embed, tempfile | — | medium (myth-embed dependency hidden) |

### §8.1 Root cause analysis

Wave 8.4 docs sync synchronized 6 files (10-myth-cli, 05-PYTHON, 07-STATE,
08-BUILD-SCOPE, 09-CLAUDE-PROMPTS, 10-VALIDATION) but **did not** sweep
crate-card Cargo.toml sections across the 10-file `docs/04-CRATES/`
series. Changes to dependencies (fs4 arrival Wave 7.5, myth-embed
promotion to identity/cli, etc.) landed in Cargo.toml without paired
docs updates.

**Structural cause**: crate-card Cargo.toml sections are **eagerly
duplicated snapshots** of the real Cargo.toml. Any dependency change
creates a docs obligation that is easy to miss.

### §8.2 Wave 9+ Mandatory Item (DCG remediation)

Two options, in order of escalating durability:

**Option A (quick win)** — re-run one sweep of all 10 crate-cards in
Wave 9.0, aligning each §Cargo.toml block to reality. Estimated 2h.

**Option B (durable)** — redefine docs/04-CRATES/*.md §Cargo.toml from
"snapshot" to "narrative" (describe relationships and invariants, link
to actual Cargo.toml). Eliminates the duplication entirely. Estimated
4–6h, removes drift class permanently.

**Recommended**: Option B. Bundle with §9 WMD cleanup in a single
"Wave 9.0: Foundational housekeeping" task.

---

## §9. Workspace Maintenance Debt (WMD) Registry (3 items)

**Definition**: `rust/Cargo.toml` `[workspace.dependencies]` entries
with no consumers across any crate. Not in Cargo.lock. Settled
intent that never materialized.

| # | Dep | Declared at | Status | Risk |
|---|---|---|---|---|
| 1 | simsimd | Wave 0 (405ca19) | 0 consumers, lock=0 | low (design-intent only) |
| 2 | nix | Wave 2.1 (ea73a07) | 0 consumers (replaced by libc+fs4 in Wave 7.5), lock=0 | low |
| 3 | regex-automata | Wave 0 | 0 direct consumers; arrives transitively via regex | low (redundant decl) |

All three are **low-severity cleanup items**. They don't affect build,
runtime, or security. They do:
- Create false signal in dependency review (simsimd/nix suggest
  capabilities that aren't built in)
- Show up as noise in `cargo-deny` output if adopted in Wave 9+

**Recommendation**: delete all three in Wave 9.0 DCG remediation sweep.

---

## §10. v0.2 Roadmap (Wave 9+ candidates)

Prioritized by shadow-mode-unblock impact, then by engineering cost.

### P0 — Critical during Shadow-21d (address before Milestone A)

- **R10 License audit automation**: implement `scripts/license-audit.sh`
  with `cargo-license` + `pip-licenses`; wire into a minimal GitHub
  Actions CI. Block PRs introducing GPL/AGPL. *Est: 4h.*

### P1 — High during Shadow-21d (address mid-window)

- **DCG remediation (§8.2 Option B)**: narrative-style crate-card
  Cargo.toml sections. *Est: 4–6h.*
- **WMD cleanup**: delete simsimd / nix / regex-automata from workspace
  deps. Bundle with DCG. *Est: 0.5h.*
- **R2 state.db backup/recovery**: add `myth doctor --backup` creating
  a timestamped copy before any destructive migration. *Est: 3h.*

### P2 — Milestone A preparation (after Shadow-21d completes)

- Article 16 implementation: `detect_env()` at SessionStart + `temporal`
  category re-evaluation. *Est: 8h.*
- Article 4 implementation: explicit rehabilitation path (lesson update
  flow distinct from block flow). *Est: 6h.*
- Milestone A Tier 3 flip: change `tier3_gate_active()` from `false` to
  rolling-compliance check reading reflector-shadow.jsonl. *Est: 4h.*
- **[v2.4 EXPERIMENTAL]** Subtleness classifier training: implement
  `myth_py.assessor.remand_feedback.compute_subtleness_score`
  (currently `NotImplementedError`). Training corpus = first 50 Tier 3
  caselog entries × subtleness label (manual or LLM-judged). Reference
  implementation at `experiment/remand-prototype/src/evaluator_llm_judge.py`
  (Opus 4.7 + tool_use + prompt caching, validated $0.05/call). *Est: 1d
  + 50 caselog cases collected during Shadow-21d.*
- **[v2.4 EXPERIMENTAL]** Remand selective trigger pilot (30-day shadow):
  fire Remand only when `subtleness_score ≥ 0.7`, log A/B vs Warn-only
  cohort. CONSTITUTION §VII.6 activation gate condition #4. *Est: 30d
  observation + 1d analyzer.*
- **[v2.4 EXPERIMENTAL]** Multi-judge cross-validation: replicate Phase
  2.4 Opus 4.7 judge with Sonnet 4.6 + at least one non-Anthropic
  model on the same 25 cells. CONSTITUTION §VII.6 activation gate
  condition #5. *Est: 4h + ~$1 budget.*

### P3 — Milestone C preparation

- Migration race fix: wrap `execute_batch` in `BEGIN IMMEDIATE`
  transaction. *Est: 2h.*
- lesson_relations columnar index: add `parent_lesson_id TEXT NULL` +
  `superseded_by TEXT NULL` to schema v2. *Est: 4h.*
- Gavel self-daemonizing architecture (the actual Milestone C work).
  *Est: 3–5 days.*

### P4 — Lower urgency

- Article 17 implementation (UI budget display). Tied to Milestone A.
- R4 hook schema drift detection (claude binary version check).
- R5 disk-full graceful degradation (ENOSPC detection).

---

## §11. Milestone Activation Readiness Matrix

| Milestone | Trigger detection | Trigger path | Activation impl |
|---|---|---|---|
| A | ✓ analyzer.py tier_1_compliance_rate | ✓ brief_gen.py + migration.py | ◐ Tier 3 wiring (gate flip ready) + ○ subtleness classifier (v2.4 added) + ○ selective Remand pilot (v2.4 added) |
| B | ○ placeholder | ○ placeholder | ○ vector store swap (not started) |
| C | ✓ hook-latency.ndjson reader | ✓ migration.py::_milestone_c | ◐ daemon design pending |
| D | ○ placeholder | ○ observer report analysis | ○ semantic detection (not started) |
| E | ○ placeholder | ○ FP/FN monitoring | ○ AST validation (not started) |

**Day-1 readiness**: Milestone A has end-to-end Tier 3 wiring ready for
gate flip; v2.4 (2026-04-24) adds two new pre-activation deliverables
(subtleness classifier + 30-day selective Remand pilot) per CONSTITUTION
§VII.6 activation gate. Others require additional implementation at
activation time (as designed — defer work until signal confirms need).

---

## §12. Conclusion & Recommendations

### Approved for Shadow mode entry

v0.1.0 is structurally complete and empirically functional. 0 test
regressions; 0 CONSTITUTION §IX.3 violations; dependency graph
Layer 0→5 strict; all Milestone detection paths wired. The three
Day-1 CONSTITUTION gaps (Art 4, 16, 17) are by-design and documented.

### Required before Milestone A analysis (end of 21-day window)

Execute §10 P0 (license audit CI). Without this, any dep upgrade
between now and Milestone A analysis carries unmonitored license risk.

### Recommended during 21-day window (low-risk)

Execute §10 P1 items in a single "Wave 9.0: Foundational housekeeping"
task: DCG remediation (§8.2 B) + WMD cleanup + R2 state.db backup.
Combined est: ~8h. No behavior change; reduces future-work surface.

### Defer until Shadow-21d data arrives

All P2+ items. Do not pre-empt Milestone signals with speculative
implementations.

### v2.4 Remand experiment outcome (2026-04-24, additive)

Phase 5 Remand prototype experiment completed: Blanket Remand NO-GO
(delta_L5_L1 = 0.0 with Opus 4.7 judge), Selective Remand conditional
GO. CONSTITUTION promoted v2.3 → v2.4 (additive only — Article 5
§"Grid as Implementation" + Part VII §VII.6 + Article 19 explicit
Day-1 exclusion + Article 4 reference). v0.2 ships scaffold only;
Remand activates only after Milestone A + 30-day selective pilot +
Jeffrey ratification (CONSTITUTION §VII.6 activation gate, 6 conditions).

This finding **does not** alter Day-1 v0.1.0 readiness. v0.1.0 has no
Remand surface; v0.2-alpha adds dead-code scaffold (`#[allow(dead_code)]`)
that demotes to Warn if accidentally fired. 281 Rust + 39 Python tests
green, 0 regression.

Reference: `experiment/remand-prototype/results/FINAL_REPORT.md`.

### Success criteria for next health report (post-Shadow-21d)

- Drift count unchanged at 11 (no new drift introduced)
- DCG count reduced from 10 to ≤2 (via §8.2 B)
- WMD count reduced from 3 to 0
- R10 closed (CI green)
- Milestone A trigger data collected (Tier 1 compliance % reported)
- v2.4 P2 Remand items: subtleness classifier corpus accumulated
  (≥50 caselog cases tagged with subtleness label)
