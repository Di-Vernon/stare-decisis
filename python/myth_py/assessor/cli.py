"""Reserved CLI for direct Assessor invocation — stub until Wave 8.

Day-1 invocation paths (per docs/05 §역할 분담):
- Task subagent (Claude Code-internal, Python uninvolved)
- Rust hook → classifier.py (Tier 0 fallback)
- Tier 3 dispatcher (Milestone A gate)

`myth-assessor` console script (pyproject.toml [project.scripts]) points
here so hatch build and `uv sync` succeed, but the commands are stubs.
"""

from __future__ import annotations

import typer

app = typer.Typer(
    no_args_is_help=True, help="myth Assessor (stub; direct CLI reserved for Wave 8)"
)


@app.callback()
def _main() -> None:
    """Empty callback — forces typer into multi-command mode so `run` parses
    as a subcommand (mirrors observer.cli for consistency).
    """


@app.command()
def run() -> None:
    """Reserved — not yet implemented.

    Direct assessor invocation from the command line is scheduled for Wave 8
    integration work.
    """
    typer.echo(
        "myth-assessor: direct CLI invocation is reserved for Wave 8 integration."
    )
    raise typer.Exit(code=0)


if __name__ == "__main__":
    app()
