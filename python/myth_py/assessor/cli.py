"""CLI for direct Assessor invocation.

Day-1 invocation paths (per docs/05 §역할 분담):
- Task subagent (Claude Code-internal, Python uninvolved)
- Rust hook → classifier.py (Tier 0 fallback)
- Tier 3 dispatcher (Milestone A gate) — `classify` subcommand wired
  up in Wave 8 Task 8.3 as the structural endpoint. Not invoked on
  Day-1; activates only when Tier 1 compliance drops below 70% per
  Decision 3 / Decision 4.
"""

from __future__ import annotations

from pathlib import Path
from typing import Optional

import typer

app = typer.Typer(
    no_args_is_help=True, help="myth Assessor — CLI entry for Rust hook subprocess"
)


@app.callback()
def _main() -> None:
    """Empty callback — forces typer into multi-command mode so each
    command parses as a subcommand (mirrors observer.cli).
    """


@app.command()
def run() -> None:
    """Reserved — direct interactive invocation.

    Observer-style interactive mode is scheduled for Milestone A; on
    Day-1 this is a placeholder so `myth-assessor run` parses without
    a schema error.
    """
    typer.echo(
        "myth-assessor: direct CLI invocation is reserved for Milestone A."
    )
    raise typer.Exit(code=0)


@app.command()
def classify(
    input: Optional[Path] = typer.Option(
        None,
        "--input",
        "-i",
        help="Path to a JSON failure envelope from the Rust hook.",
    ),
) -> None:
    """Tier 3 dispatch endpoint (Wave 8 Task 8.3 — structural stub).

    The Rust `myth-hook-post-tool-failure` binary calls this
    subprocess when Tier 1 compliance falls below Decision 3's 70%
    threshold (Milestone A gate). On Day-1 the gate is hard-wired
    off in the Rust side, so this endpoint is exercised only by
    direct test invocation — it returns an "inactive" JSON payload
    so callers can parse the response uniformly across Day-1 /
    Milestone A.

    Real Tier 3 classification (Anthropic SDK dispatch, retry/
    backoff, spend cap enforcement) is implemented under Milestone A
    per Decision 4; this command will be updated in place once
    shadow-mode data justifies the upgrade.
    """
    # Day-1 behaviour: emit a stable JSON envelope so the Rust caller
    # can distinguish "Tier 3 disabled" (exit 0 + status=not_enabled)
    # from "Tier 3 error" (non-zero exit). The --input path is
    # accepted for future signature stability but not read.
    _ = input  # reserved for Milestone A implementation
    typer.echo(
        '{"status":"not_enabled",'
        '"reason":"Tier 3 dispatch is inactive on Day-1 '
        '(Milestone A gate, Decision 4)"}'
    )
    raise typer.Exit(code=0)


if __name__ == "__main__":
    app()
