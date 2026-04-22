# Changelog

All notable changes to myth are recorded here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) with additional
wave-by-wave traceability back to the `docs/09-CLAUDE-PROMPTS.md` build
plan.

## [0.1.0] — Day-1 (pending v0.1.0 tag)

### Release shape

- **Binaries** (Rust, 8 total): `myth`, `myth-hook-pre-tool`,
  `myth-hook-post-tool`, `myth-hook-post-tool-failure`,
  `myth-hook-user-prompt`, `myth-hook-stop`, `myth-hook-session-start`,
  `myth-embed`
- **Python console scripts**: `myth-assessor`, `myth-observer`
- **Target platform**: Linux / WSL2 (primary) — macOS/Linux portable
- **Licence**: MIT OR Apache-2.0 (dual)

### Added

- **Layer 0-1** (Wave 1): `myth-common` (Level, Enforcement, XDG paths,
  timestamps, IDs) + `myth-db` (SQLite WAL schema v1: `lessons`,
  `vector_metadata`, `vector_generation`, `hook_events`, `appeal_history`,
  `grid_overrides`, `sessions`). Forward-only migration driven by
  `PRAGMA user_version`.
- **Layer 2** (Wave 2): `myth-gavel` (RuleSet + Grid + FatigueTracker +
  Verdict), `myth-identity` (aggressive normalisation, SHA1 tier-1 hash,
  in-memory + mmap vector store), `myth-embed` (length-prefixed bincode
  Unix-socket daemon wrapping fastembed-rs multilingual-e5-small).
- **Layer 3** (Wave 3): 6 hook binaries (`pre_tool`, `post_tool`,
  `post_tool_failure`, `user_prompt`, `stop`, `session_start`) + Tier 0
  deterministic classifier.
- **Layer 4** (Wave 4): `myth-runtime` (UsageTracker, ClaudeLauncher),
  `myth-orchestrator` (plan.json execution), `myth-ui` (ratatui TUI).
- **Layer 5** (Wave 5): `myth-cli` with 13 subcommands + `myth install /
  uninstall / doctor / watch / run / init / observer / lesson / embed /
  appeal / retrial / constitution / key / status`. Drift 7 items synced
  in Wave 5.5.
- **Python layer** (Wave 6): `myth_py.assessor` (Tier 0 classifier,
  Variant B template, dispatcher stub), `myth_py.observer` (brief-gen,
  lapse detection, archive migration).
- **Bedrock Rule catalogue** (Wave 7): 15 rule entries × 209 alternation
  branches = 54 distinct detection signatures across 3 items
  (`rm_rf_unsandboxed` R1-A..G, `production_secrets_commit` R2-A..D with
  40 gitleaks provider prefixes embedded, `auth_bypass_production`
  R3-A..D).
- **Foundation Rule seed** (Wave 7): 5 entries (main_force_push,
  no_verify_ci_bypass, pii_exfiltration, unverified_dependency,
  untrusted_arbitrary_execution).
- **Sentencing Grid** (Wave 7): 30-cell 5×6 Level × Recurrence matrix,
  default hardcoded + DB override table.
- **Test fixtures** (Wave 7): 560 cases (280 positive + 280 negative)
  with an `a01_harness_sanity` gate and an `a02_fixtures_full_sweep`
  harness proving FP=0 / FN=0 at entry-level.
- **Tier 0 concurrent write test** (Wave 7 Task 7.6): N=8 subprocess
  race with distinct identities, warm-up isolation, WAL + busy_timeout=5s
  assertion.
- **Lesson split/merge** (Wave 8 Task 8.1): real persistence via
  `meta_json` (schema v1 preserved — no migration 002). Parent ↔ children
  and sources ↔ merged relations tracked as JSON in the existing
  `meta_json` column.
- **Python auto-install** (Wave 8 Task 8.2): `myth install` invokes
  `uv pip install -e .[dev]` (falls back to `pip3`, then `pip`). Failure
  degrades to a warning with the exact manual command; the Rust-side
  install still exits 0.
- **Tier 3 dispatch wiring** (Wave 8 Task 8.3): subprocess call path
  from Rust hook to Python `myth_py.assessor.cli classify --input <path>`
  built but gated off (`tier3_gate_active() == false` on Day-1). Flips
  at Milestone A when Tier 1 compliance drops below Decision 3's 70%
  threshold.
- **THIRD-PARTY audit** (Wave 8 Task 8.5): Rust 395 crates + Python 36
  packages, all OSI-approved permissive licences. No GPL / AGPL
  obligations. `r-efi` offers LGPL as one of three alternatives; MIT
  chosen by myth's redistribution.
- **LICENSE**, **NOTICE** (Wave 8 Task 8.5): dual-licence manifest,
  with NOTICE enumerating upstream adaptations (gitleaks, detect-secrets,
  multilingual-e5-small).

### Changed

- **fs2 → fs4 migration** (Wave 7 Task 7.5): the unmaintained `fs2`
  crate (last update 2020) is replaced with `fs4` 0.13 across myth-db
  (`jsonl.rs`), myth-embed (`lock.rs`, `flock_race.rs`), and their
  `Cargo.toml` entries. `try_lock_exclusive` now returns
  `io::Result<bool>`; call sites adapted to distinguish
  contention-vs-error. Bench ceilings preserved (pre_tool 31.82 ms,
  post_tool 32.06 ms, post_tool_failure_tier0 37.27 ms — all under their
  Wave 6 ceilings).

### Fixed (self-drift resolution, Wave 7 stop condition #13)

- **Regex** coverage tightening (7 items): R1-D `git clean` flag
  permutation bidirectional; R2-B URL schemes extended to cover
  postgres/mysql/redis/mongodb/amqp; R2-C adds bare `token` keyword and
  quote-wrapper around all keywords; R2-D adds quote-wrapper around
  CJK labels; R3-A merges `verify=False` + `verify:false` under one
  quote-wrapped branch; R3-C accepts case-insensitive values; R1-A and
  R1-C drop `$HOME` from their path lists (R1-G owns the env-sentinel
  territory — overlap removed).
- **Fixture** reclassification (13 items): R2-A 4 length-vs-charset
  corrections; R2-D 3 value replacements where provider-prefix values
  were triggering R2-A; R1-A/R1-C 3 cases moved from env-sentinel to
  system paths; 3 near-miss negatives reframed to avoid genuine
  other-rule matches.

### Deferred to Milestone C (carry-forward)

- **myth-db migration cold-start race**: `migration.rs` applies schema
  outside a transaction, so N concurrent cold `Database::open` calls
  can race on `CREATE TABLE`. Wave 7 Task 7.6 isolates its own assertion
  with a warm-up invocation; the proper fix is batched with the Gavel
  daemon SQLite rework.
- **lesson split/merge index**: relations in `meta_json` JSON without
  indexed `parent_lesson_id` / `superseded_by` columns. Day-1 call
  frequency is too low to matter; revisit with the daemon access-pattern
  re-evaluation.

### Observation-period expectation

Per Decision 3 / Decision 7, Day-1 ships `shadow mode` metrics
(`~/.myth/metrics/reflector-shadow.jsonl`) and a 21-day observation
window before Milestone A is evaluated. Tier 1 compliance < 70%
triggers Tier 2+3 activation; otherwise Tier 3 stays inactive
indefinitely.

---

For the authoritative commit-level record see `git log --grep wave-`
and `docs/09-CLAUDE-PROMPTS.md §Wave 7/8 drift sync`.
