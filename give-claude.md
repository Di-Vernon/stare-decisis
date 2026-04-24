# give-claude.md

> **Paste this entire file into any Claude session.** Claude will then walk you through installing and testing `stare-decisis` — no other prompts needed from you.
>
> **Best experience**: paste into **Claude Code** in a new terminal. Also works in claude.ai or Claude Desktop — Claude will simply hand you commands to run yourself instead of executing them directly.

---

## [Instructions for Claude — start reading here]

You are onboarding a user who wants to test **stare-decisis** (v0.1.1, testing phase), an experimental Claude Code governance and learning layer. The repository is https://github.com/Di-Vernon/stare-decisis and the internal CLI/package is named `myth` (rename to `stare-decisis` is in progress).

**The user pasted this file because they want a guided walkthrough without writing prompts.** Lead them through three phases: Install, First Run, Feedback. Verify each step before moving on. Respond in the user's language — detect from their messages (likely English or Korean).

**If you have bash tool access**, run commands yourself and show output. **If you're in a web interface**, give commands to run and wait for them to paste results back.

---

### What stare-decisis does (summarize briefly before starting)

It runs as a hook layer around Claude Code, records failure patterns into a local SQLite database, and injects relevant precedent as just-in-time context in future sessions. Currently in **shadow mode** — observation only, with one exception: a minimal "never-list" blocks catastrophic patterns (e.g. `rm -rf /`) regardless of mode.

Three internal tiers:
- **Trial Court** (Rust binary, <50ms, $0) — pattern match on every tool call
- **Reflector** (Claude Haiku, on failure only) — extracts lessons from failures
- **Curator** (Claude Sonnet, weekly) — consolidates and retires precedents

Reflector and Curator require an Anthropic API key. Trial Court does not.

---

### Phase 1 — Install

**1.1 Prerequisite check.** Verify each of these — help install anything missing before proceeding:

- Linux or WSL2 Ubuntu (macOS best-effort, not tested)
- Rust 1.82+ (`rustc --version`)
- Python 3.11+ (`python3 --version`)
- Claude Code 2.1.27+ (`claude --version`) — PostToolUseFailure hook requires this
- `git`, `cargo`, `clang`, `mold` available
- `~/.local/bin` on PATH (this is where the `myth` CLI will be installed)

**1.2 Clone** (skip if already cloned):

```bash
git clone https://github.com/Di-Vernon/stare-decisis.git ~/stare-decisis
cd ~/stare-decisis
```

**1.3 Build and install binaries:**

```bash
cd rust
cargo build --release
./target/release/myth install
```

This symlinks binaries to `~/.local/bin` and initializes `~/.myth/` with rule templates and a SQLite state database.

Verify with `myth doctor`. Expected: 4–5 `[ok]` lines and possibly one `[warn]` about the embed socket — that warning is normal (daemon auto-spawns on first use).

**1.4 Pick a test project.** Ask the user which Claude Code project to use for testing.

⚠️ **Strongly recommend a sandbox project, not a production codebase.** Shadow mode is safe-by-design, but v0.1.1 is experimental and the never-list will block catastrophic commands regardless of mode. If they need a sandbox:

```bash
mkdir -p ~/myth-test && cd ~/myth-test && git init
```

Also run `myth init` in the sandbox project if it doesn't yet have a `.claude/` directory:

```bash
myth init
```

**1.5 Wire hooks into the project.** From the chosen project directory:

```bash
myth link
```

This registers six myth hooks into the project's `.claude/settings.json` (or `.claude/settings.local.json` if that file already exists — `myth link` prefers the local file).

Properties to reassure the user about:
- **Non-destructive** — existing non-myth hook entries are preserved
- **Idempotent** — safe to re-run
- **Backed up** — original settings file copied to `.pre-myth-{timestamp}` before modification

After it completes, show the user the updated settings file so they understand what was added:

```bash
cat .claude/settings.local.json 2>/dev/null || cat .claude/settings.json
```

---

### Phase 2 — First run

**2.1** Have the user start a normal Claude Code session in the test project and perform a simple task (read a file, run a small command). The goal is to generate hook events, not to accomplish real work.

**2.2** After a few tool calls, inspect captured data:

```bash
myth status
myth lesson list
```

In v0.1.1 shadow mode, most entries will be in a "collecting" state rather than promoted to active precedents. That's expected.

**2.3 (optional)** Trigger a deliberate non-catastrophic failure — ask Claude Code to read a file that doesn't exist, or run a typo'd command — to exercise the failure-capture path. Then check `myth status` again.

---

### Phase 3 — Feedback

Explain the channels clearly:

- 🐛 **Bugs** → GitHub Issues, label `bug`
- 🎭 **False positives** (something blocked that shouldn't be) → label `false-positive` with a minimal repro
- 🧠 **Precedent quality** (weird or wrong extracted lessons) → label `precedent-quality` — this is the highest-signal feedback
- 💡 **Feature ideas** → label `enhancement`
- 💬 **Open-ended discussion** → GitHub Discussions (if enabled on the repo)

If the user has already observed something worth reporting, offer to help draft their first issue.

---

### Troubleshooting

| Symptom | Likely cause | Fix |
|---------|--------------|-----|
| Rust build fails | Missing system deps | `sudo apt install clang mold build-essential` |
| `myth: command not found` | `~/.local/bin` not on PATH | Add `export PATH="$HOME/.local/bin:$PATH"` to shell rc |
| `myth doctor` warns: embed socket missing | Daemon not spawned yet | Run `myth embed probe "hello"` once |
| `myth link` says project has no `.claude/` | Project hasn't been initialized | Run `myth init` first, then `myth link` |
| Hooks don't fire in Claude Code | Settings file wasn't updated | Re-run `myth link`; verify the settings file (local variant preferred) has 6 hook entries with `myth-hook-` commands |
| User wants to unwire a project | — | `myth unlink [PATH]` removes only the myth-authored entries, preserving everything else |
| User wants to fully uninstall | — | Run `myth unlink` in each linked project, then `myth uninstall` to remove binaries from `~/.local/bin` |

---

### Session end-state check

Before wrapping up, verify with the user:

- [ ] `which myth` returns a path
- [ ] At least one project has myth hooks wired (6 entries in `.claude/settings.json` or `.claude/settings.local.json`)
- [ ] They've run at least one Claude Code session with myth active
- [ ] They know which GitHub issue label to use for each kind of feedback

Thank them for testing and stop. Do not continue into unrelated tasks.
