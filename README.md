# myth

> Local AI agent orchestrator wrapping Claude Code.  
> A self-improving learning system that watches your tool failures, classifies them, and compiles the lessons into law.

```
완벽은 도달이 아니라 수렴이다.
수렴은 우연이 아니라 법이다.
```

---

## What myth does

myth is **three things** working at three time scales:

**The Gavel** (milliseconds). Before each tool call, 47 regex patterns check for catastrophic commands — `rm -rf /`, production secret leaks, auth bypasses. Catches these before they happen.

**Assessor** (seconds). When a tool fails, Claude Haiku analyzes the failure along 5 axes (blast radius, reversibility, likelihood, category, uplift) and produces a structured verdict: Level 1–5, severity-adjusted.

**Observer** (weekly). Claude Sonnet reviews every accumulated lesson, finds patterns, updates a brief that gets injected into each new session, and reports migration readiness for 5 system evolution thresholds.

All three work together in a **separation-of-powers** architecture. The Gavel acts but cannot interpret. Assessor interprets but cannot block. Observer sees the whole picture but cannot change anything without your approval.

## Why it exists

Using Claude Code over time accumulates three problems:

**Repeated mistakes**. The same heredoc quoting issue, the same missing Python venv, the same dangerous `rm -rf` — Claude forgets between sessions. You give the same feedback repeatedly.

**Missing catastrophic guards**. Claude Code itself has limited awareness of "this command causes irreversible damage." The commands need to be stopped *before* execution, not after.

**Unstructured learning**. Some failures are Level 1 (style). Some are Level 5 (data loss). Without structure, both end up as equal-weight notes — noise.

myth is the local answer to these problems.

## Status

**Version**: v0.1.0 (Day-1 release, pending build)  
**Platform**: WSL2 Ubuntu 24.04 + Claude Code 2.1.27+  
**Language**: Rust (60%) + Python (30%) + Shell (10%)  
**License**: MIT OR Apache-2.0

## Install

```bash
# Prerequisites (see WSL2-SETUP.md for full details):
# - WSL2 Ubuntu 24.04
# - Rust stable, mold linker, clang
# - Python 3.11+, tmux
# - Claude Code 2.1.27+

git clone https://github.com/Di-Vernon/myth.git ~/myth
cd ~/myth
bash scripts/install.sh

# First project:
cd ~/project/my-project
myth init       # scaffolds .claude/
myth link       # wires myth hooks into .claude/settings(.local).json
myth run        # wraps Claude Code with myth supervision
```

Verify:
```bash
myth doctor     # all checks green?
myth status
```

## Everyday use

```bash
# Wire / unwire myth hooks in a Claude Code project
myth link [PATH]          # wire myth hooks into a Claude Code project
myth unlink [PATH]        # remove myth hooks from a project

# Start a myth-supervised Claude Code session
myth run

# Check what's happening right now
myth status

# Full TUI dashboard
myth watch

# Lesson management
myth lesson list
myth lesson show <id>
myth lesson appeal <id> --reason "this was actually correct"

# Run weekly analysis (usually automated)
myth observer run

# Health check
myth doctor
myth doctor --migration    # Milestone readiness
```

## Directory layout

```
~/myth/                    # source (this repo)
~/.local/bin/myth*         # 8 installed binaries
~/.config/myth/            # user config + API key
~/.myth/                   # runtime data (rules, SQLite, JSONL logs)
~/.local/state/myth/       # ephemeral state (latency logs, daemon logs)
$XDG_RUNTIME_DIR/myth/     # Unix sockets (tmpfs)
```

Full details: [`docs/03-DIRECTORY.md`](docs/03-DIRECTORY.md).

## Key concepts

Read these in order if you're new:

1. [`docs/02-CONCEPTS.md`](docs/02-CONCEPTS.md) — terminology (The Gavel, Assessor, Observer, Bedrock/Foundation/Surface, Level, Recurrence, Enforcement, Lapse, Milestone, etc.)
2. [`docs/01-OVERVIEW.md`](docs/01-OVERVIEW.md) — what myth does and doesn't do
3. [`ARCHITECTURE.md`](ARCHITECTURE.md) — execution model, API contracts, Milestone transitions

## Philosophy

myth is built on three converging ideas:

**Legal theory.** Beccaria ("certainty over severity"), Montesquieu (separation of powers), Ayres-Braithwaite's responsive regulation pyramid. The system enforces gently first, harder only when patterns repeat.

**Rough Start.** No feature gets built on speculation. Everything on Day-1 is either actively used or has measurable conditions for activation (the Milestone system). Don't optimize what you can't measure.

**Convergence as law.** The master principle: perfection is not reached, it is converged upon. Convergence is not accidental, it is lawful. myth is the mechanism that makes the convergence lawful.

Full constitution: [`CONSTITUTION.md`](CONSTITUTION.md).

## What myth is not

- **Not a replacement for Claude Code.** myth wraps it, doesn't replace it. Claude Code's features work normally.
- **Not a code linter, refactorer, or formatter.** Those are Claude Code's job.
- **Not a network agent.** Runs entirely locally. The only external call is the optional Anthropic API (disabled by default).
- **Not a team tool.** v1 is single-user. Team sync is a separate project.
- **Not a replacement for your judgment.** Observer recommends. You decide.

## Milestones (system evolution)

myth uses measured conditions (not time) to decide when to evolve:

| Milestone | Condition | What happens |
|---|---|---|
| **A** | 3 weeks of use, analyze Tier 1 compliance | Activate Assessor Tier 2/3 if compliance < 85% |
| **B** | 20K lessons AND vector KNN P99 > 50ms | Migrate to sqlite-vec or usearch |
| **C** | Hook P99 > 15ms for 2 weeks + build profile applied + WSL2 green + PGO attempted | Switch The Gavel to daemon mode |
| **D** | Observer reports Bedrock misses | Activate semantic detection |
| **E** | Semantic FP > 5% or FN > 2% | Add AST-based validation |

Full details: [`ARCHITECTURE.md`](ARCHITECTURE.md) §4.

## Documentation

All design documents live in [`docs/`](docs/). Start at [`docs/00-INDEX.md`](docs/00-INDEX.md) for full navigation.

For implementers (Claude Code): start at [`docs/09-CLAUDE-PROMPTS.md`](docs/09-CLAUDE-PROMPTS.md).

## Attribution

myth builds on [gitleaks](https://github.com/gitleaks/gitleaks) (MIT) for Bedrock Rule regex patterns, [detect-secrets](https://github.com/Yelp/detect-secrets) (Apache-2.0) for keyword+entropy heuristics, and [multilingual-e5-small](https://huggingface.co/intfloat/multilingual-e5-small) (MIT, Microsoft Research) for identity embeddings. Full attribution: [`THIRD-PARTY.md`](THIRD-PARTY.md).

## License

MIT OR Apache-2.0 (dual, your choice).

---

**Author**: Jeffrey (Di-Vernon)  
**Repository**: https://github.com/Di-Vernon/myth  
**Constitution**: [`CONSTITUTION.md`](CONSTITUTION.md)
