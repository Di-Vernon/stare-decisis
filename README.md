# stare-decisis

> A Claude Code orchestrator where past failures become binding precedent.

**⚠️ v0.1.1 — early testing phase. Not production-ready. Actively looking for testers and feedback.**

---

## What is this?

`stare-decisis` turns Claude Code's hook system into a learning governance layer. When a tool call fails, the system records the pattern. On future runs, similar patterns trigger just-in-time context injection — Claude sees the precedent before making the same mistake twice.

The name comes from the common-law principle *stare decisis et non quieta movere* — "to stand by things decided and not disturb what is settled." Every resolved failure becomes binding precedent for what follows.

## Why?

Claude Code is powerful but has no cross-session memory of what went wrong. Repeat the same kind of mistake → repeat the same kind of fix. This project addresses that gap with four principles:

- **Remembers** failures without leaking them to unrelated sessions
- **Injects** relevant precedent just-in-time, not as permanent prompt bloat
- **Refuses** a narrow set of catastrophic patterns outright (the never-list)
- **Appeals** — a precedent you disagree with can be overturned

It runs as a hook layer around `claude`, not a replacement. Your existing Claude Code workflow keeps working exactly as before.

## How it works

Three tiers, intentionally ordered by cost and latency:

| Tier | Role | Tech | Latency |
|------|------|------|---------|
| **Trial Court** | Pattern match; allow / block / warn | Rust binary + YAML rules | < 50 ms |
| **Reflector** | Failure → lesson extraction | Claude Haiku (API) | ~2 s per failure |
| **Curator** | Weekly precedent consolidation | Claude Sonnet (API) | weekly batch |

Trial Court runs on every tool call and needs no network. Reflector runs only when a failure is observed. Curator runs weekly to merge, retire, and rank precedents. No data leaves your machine unless Reflector or Curator is invoked — both require an Anthropic API key and use pay-per-call pricing.

> Production cost benchmarks are not yet published. Expect early releases to optimize for correctness over cost; tune after observation.

---

## Quick start — let Claude install it for you

The easiest way to try `stare-decisis` is to hand [**give-claude.md**](give-claude.md) to Claude. Claude will walk you through installation, first run, and feedback — no other prompts required from you.

**Recommended**: paste into **Claude Code** in a new terminal for hands-free setup.
**Also works**: claude.ai, Claude Desktop, or any Claude interface (Claude will hand you commands to run yourself).

```text
1. Open give-claude.md in this repo
2. Copy the entire file
3. Paste into a new Claude session
4. Follow along
```

---

## Manual installation

If you prefer to install by hand:

### Prerequisites

- Linux or WSL2 Ubuntu (macOS best-effort, not tested)
- Rust **1.82+** (edition 2021)
- Python **3.11+**
- Claude Code **2.1.27+** (PostToolUseFailure hook is required)
- `git`, `cargo`, `clang`, `mold`
- `~/.local/bin` on your `PATH`

SQLite is bundled into the Rust binary via `rusqlite`, so no system SQLite install is needed.

### Install binaries

```bash
git clone https://github.com/Di-Vernon/stare-decisis.git ~/stare-decisis
cd ~/stare-decisis/rust
cargo build --release
./target/release/myth install
```

This symlinks binaries to `~/.local/bin` and initializes `~/.myth/` with rule templates and a SQLite state DB.

Verify:

```bash
myth doctor
```

Expected output: 4–5 `[ok]` lines, possibly one `[warn]` about the embed socket (auto-spawned on first use — not an error).

### Wire hooks into a test project

From inside a Claude Code project:

```bash
myth link
```

This registers six myth hooks into the project's `.claude/settings.json` (or `.claude/settings.local.json` if it already exists — myth prefers the local file).

Properties you can rely on:
- **Non-destructive** — any existing non-myth hook entries are preserved
- **Idempotent** — safe to re-run; duplicate myth entries will not be created
- **Backed up** — the original settings file is copied to `.pre-myth-{timestamp}` before modification
- **Atomic** — write happens via temp file + rename, no partial-write corruption

Open the settings file after running to see what was added.

To unwire:

```bash
myth unlink
```

This removes only the myth-authored entries; anything else in `.claude/settings.json` stays untouched.

> ⚠️ **Use a sandbox project, not production**, for early testing. Shadow mode is safe-by-design but v0.1.1 is experimental.

### Everyday commands

```bash
myth link [PATH]          # wire myth hooks into a Claude Code project
myth unlink [PATH]        # remove myth hooks from a project
myth status               # what's been captured
myth lesson list          # current precedent pool
myth doctor               # health check
myth uninstall            # reverse binary install
myth constitution         # open CONSTITUTION.md in $PAGER
```

See `myth --help` for the full command surface.

---

## What to expect in v0.1.x (shadow mode)

This release series is deliberately conservative:

- **Observation-first.** Hooks record envelopes and failure patterns to a local SQLite database. No blocking, no context injection — just data collection.
- **Never-list is active.** A small hand-curated set of universally-catastrophic patterns (e.g. `rm -rf /`) is blocked regardless of mode. This is the one exception to shadow mode.
- **No silent network calls.** Reflector and Curator only run when you explicitly invoke them.
- **Your workflow is unchanged.** If `stare-decisis` did nothing at all, you'd still have `claude` doing what it always did.

The 21-day shadow period is about **calibration** — learning what normal looks like for your project before flipping any enforcement switches.

---

## Philosophy

> Perfection is not arrival. Perfection is convergence. Convergence is not accident. Convergence is law.

Precedents are never prescribed in advance. The system starts rough and converges through use — closer to how case law accumulates than to how a static rulebook is written. Two design consequences:

1. **Pre-blocking only for Level 5 (catastrophic) patterns.** Everything below that emerges from actual failures observed in actual sessions.
2. **Seeding from public failure data.** No governance system ever started from nothing. Initial precedents draw on publicly-available Claude Code error patterns.

See [`CONSTITUTION.md`](CONSTITUTION.md) for the full design document.

---

## Known limitations

- **Pattern identity** is currently SHA1-based string hashing. Embedding-based identity is in active development.
- **Appeal interface** is CLI only — no web UI yet.
- **Cross-project learning** not yet supported — each project has its own precedent database.
- **Stub commands**: `myth gavel`, `myth lesson split`, `myth lesson merge`, and parts of `myth key` are placeholders for future milestones.
- **Documentation** is being reorganized from internal-first to external-first; some pages still assume project history.

---

## Feedback

This is the whole point of the testing phase. All of these are welcome:

- 🐛 **Bugs** → [GitHub Issues](https://github.com/Di-Vernon/stare-decisis/issues), label `bug`
- 💡 **Feature requests** → same place, label `enhancement`
- 🧠 **Precedent-quality observations** → label `precedent-quality`. These are the highest-signal feedback.
- 🎭 **False positives** (something was blocked that shouldn't have been) → label `false-positive` with a minimal repro.
- 💬 **Open-ended discussion** → [GitHub Discussions](https://github.com/Di-Vernon/stare-decisis/discussions) (if enabled)

---

## Roadmap

- **v0.1.x** (current) — Shadow-mode stabilization; Day-1 hook contract frozen; `myth link` / `myth unlink` CLI integration.
- **v0.2** — Remand integration (experimental: failed actions get a second attempt with precedent context).
- **v0.3** — Dirty working-tree warnings; security-context generalization; cross-platform polish.
- **v1.0** — Embedding-based identity; web appeal UI; multi-project precedent sharing.

See [`CHANGELOG.md`](CHANGELOG.md) for release history.

---

## A note on naming

The repository is `stare-decisis` but the internal crate, CLI binary, and Python package are named `myth`. You will see both. This is a rename-in-progress from the pre-public era; the internal name will eventually align with the repo name.

## License

[MIT](LICENSE). Third-party notices: [`NOTICE`](NOTICE) and [`THIRD-PARTY.md`](THIRD-PARTY.md).

---

**Not affiliated with Anthropic.** `stare-decisis` is a community tool built on Claude Code's official extension points (hooks, subagents, MCP, skills). The name "Claude" and related marks are property of Anthropic.
