"""Generate the weekly brief.md.

Takes precomputed WeeklyAnalysis / LapseResult / milestone list and formats
them as Markdown. Pure formatting — no I/O, no database work.
"""

from __future__ import annotations

from datetime import datetime, timezone

from .analyzer import WeeklyAnalysis
from .lapse import LapseResult
from .migration import MilestoneStatus

_LEVEL_NAMES = {1: "INFO", 2: "LOW", 3: "MEDIUM", 4: "HIGH", 5: "CRITICAL"}


def generate_brief(
    analysis: WeeklyAnalysis,
    lapse_result: LapseResult,
    milestones: list[MilestoneStatus],
) -> str:
    now = datetime.now(timezone.utc)
    week = now.strftime("%Y-W%V")
    sections: list[str] = []

    sections.append(f"# myth Brief — {week}\n")
    sections.append(
        f"_Generated: {now.isoformat().replace('+00:00', 'Z')}_\n\n"
    )

    sections.append("## Summary\n\n")
    sections.append(f"- Analyzed {analysis.total_caselog_entries} events\n")
    sections.append(f"- {len(analysis.new_lessons)} new lessons\n")
    sections.append(f"- {analysis.recurrence_increments} recurrence increments\n")
    sections.append(f"- {lapse_result.new_lapsed_count} lessons lapsed\n")
    sections.append(f"- {lapse_result.archived_count} lessons archived\n")
    sections.append(f"- Bedrock matches: {analysis.bedrock_matches}\n\n")

    sections.append("## Active Lessons (top 10 by recurrence)\n\n")
    if analysis.top_active_lessons:
        for lesson in analysis.top_active_lessons:
            name = _LEVEL_NAMES.get(lesson.level, "?")
            sections.append(
                f"- **{lesson.lesson_id}** "
                f"(L{lesson.level} {name}, recurrence {lesson.recurrence_count:.1f})\n"
            )
            sections.append(f"  {lesson.rationale}\n")
    else:
        sections.append("_(no active lessons)_\n")
    sections.append("\n")

    sections.append("## Migration Readiness\n\n")
    for m in milestones:
        mark = "⚠" if m.triggered else " "
        sections.append(f"- [{mark}] **Milestone {m.id}** — {m.title}\n")
        sections.append(
            f"  Current: {m.current_value} / Threshold: {m.threshold}\n"
        )
        for note in m.notes:
            sections.append(f"  - {note}\n")
    sections.append("\n")

    sections.append("## Assessor Tier 1 Compliance\n\n")
    sections.append(f"Rate: {analysis.tier_1_compliance_rate:.1%}\n\n")
    if analysis.tier_1_compliance_rate < 0.70:
        sections.append(
            "⚠ Below 70% — consider enabling Tier 2/3 (see Milestone A).\n\n"
        )

    if analysis.tier_3_cost_usd > 0:
        sections.append("## Tier 3 Cost (this week)\n\n")
        sections.append(f"${analysis.tier_3_cost_usd:.2f}\n\n")

    sections.append("## Observer Recommendations\n\n")
    recs = _generate_recommendations(analysis, lapse_result)
    if recs:
        for r in recs:
            sections.append(f"- {r}\n")
    else:
        sections.append("_(no recommendations this week)_\n")

    return "".join(sections)


def _generate_recommendations(
    analysis: WeeklyAnalysis, lapse_result: LapseResult
) -> list[str]:
    recs: list[str] = []
    if analysis.tier_1_compliance_rate > 0.0 and analysis.tier_1_compliance_rate < 0.70:
        recs.append(
            "Tier 1 compliance below 70% — evaluate Milestone A activation."
        )
    if analysis.bedrock_matches > 0:
        recs.append(
            f"{analysis.bedrock_matches} Bedrock Rule match(es) this week — "
            f"review caselog for unusual patterns."
        )
    if lapse_result.new_lapsed_count > 0:
        recs.append(
            f"{lapse_result.new_lapsed_count} lessons newly lapsed — consider "
            f"Grid adjustment if recurrence pattern shifts."
        )
    return recs
