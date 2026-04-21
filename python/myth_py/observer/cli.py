"""Observer CLI — `myth-observer` console script / `python -m myth_py.observer.cli`.

Called by Rust myth-cli (Wave 5 `subcmd/observer.rs`) via
`python3 -m myth_py.observer.cli run [--dry]`. The `if __name__ == "__main__":`
block ensures typer picks up subcommands when loaded as `-m`.
"""

from __future__ import annotations

from pathlib import Path

import typer
from rich.console import Console

from .analyzer import run_analysis
from .brief_gen import generate_brief
from .lapse import update_lapse_scores
from .migration import compute_all_milestones

app = typer.Typer(no_args_is_help=True, help="myth weekly Observer")
console = Console()


@app.callback()
def _main() -> None:
    """Empty callback — forces typer into multi-command mode so `run` is
    recognised as a subcommand (matches Wave 5 myth-cli observer.rs which
    invokes `python3 -m myth_py.observer.cli run [--dry]`).
    """


@app.command()
def run(
    dry: bool = typer.Option(
        False, "--dry", help="Run analysis but do not write brief.md"
    ),
) -> None:
    """Run the weekly Observer analysis and regenerate brief.md."""
    console.print("[cyan]Running Observer analysis...[/cyan]")

    analysis = run_analysis()
    console.print(
        f"  Analyzed {analysis.total_caselog_entries} caselog entries"
    )
    console.print(f"  Found {len(analysis.new_lessons)} new lessons")
    console.print(f"  Recurrence increments: {analysis.recurrence_increments}")

    lapse_result = update_lapse_scores()
    console.print(
        f"  Lapse transitions: {lapse_result.new_lapsed_count} "
        f"(archived {lapse_result.archived_count})"
    )

    milestones = compute_all_milestones()
    for m in milestones:
        status = "[red]TRIGGERED[/red]" if m.triggered else "[green]OK[/green]"
        console.print(f"  Milestone {m.id}: {status} ({m.current_value})")

    brief = generate_brief(analysis, lapse_result, milestones)

    if dry:
        console.print("[yellow]Dry run: brief not written[/yellow]")
        console.print(brief)
        return

    brief_path = Path.home() / ".myth" / "brief.md"
    brief_path.parent.mkdir(parents=True, exist_ok=True)
    brief_path.write_text(brief, encoding="utf-8")
    console.print(f"[green]Brief written to {brief_path}[/green]")


if __name__ == "__main__":
    app()
