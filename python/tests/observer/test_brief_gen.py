"""Brief generation tests — structural checks (no snapshot)."""

from __future__ import annotations

from myth_py.observer.analyzer import LessonRow, WeeklyAnalysis
from myth_py.observer.brief_gen import generate_brief
from myth_py.observer.lapse import LapseResult
from myth_py.observer.migration import MilestoneStatus


def _sample_analysis() -> WeeklyAnalysis:
    return WeeklyAnalysis(
        total_caselog_entries=42,
        new_lessons=["L1", "L2"],
        recurrence_increments=7,
        bedrock_matches=1,
        tier_1_compliance_rate=0.65,
        tier_3_cost_usd=1.23,
        top_active_lessons=[
            LessonRow(
                lesson_id="abcd1234",
                level=3,
                rationale="ENOENT on scaffold",
                recurrence_count=12.0,
            )
        ],
    )


def test_brief_contains_week_header() -> None:
    brief = generate_brief(
        _sample_analysis(),
        LapseResult(1, 0, 0),
        [
            MilestoneStatus("A", "Assessor review", False, "pending", "3w"),
            MilestoneStatus("C", "Gavel", True, "P99 18ms", "15ms"),
        ],
    )
    assert brief.startswith("# myth Brief —")
    assert "## Summary" in brief


def test_brief_flags_low_tier1() -> None:
    brief = generate_brief(
        _sample_analysis(), LapseResult(0, 0, 0), []
    )
    assert "Below 70%" in brief


def test_brief_shows_tier3_cost_when_nonzero() -> None:
    brief = generate_brief(
        _sample_analysis(), LapseResult(0, 0, 0), []
    )
    assert "Tier 3 Cost" in brief
    assert "$1.23" in brief


def test_brief_hides_tier3_cost_when_zero() -> None:
    analysis = _sample_analysis()
    analysis.tier_3_cost_usd = 0.0
    brief = generate_brief(analysis, LapseResult(0, 0, 0), [])
    assert "Tier 3 Cost" not in brief


def test_brief_recommendations_present() -> None:
    brief = generate_brief(
        _sample_analysis(),
        LapseResult(new_lapsed_count=3, revived_count=0, archived_count=0),
        [],
    )
    assert "Observer Recommendations" in brief
    assert "Milestone A activation" in brief
    assert "Bedrock Rule match" in brief


def test_milestone_triggered_mark() -> None:
    brief = generate_brief(
        _sample_analysis(),
        LapseResult(0, 0, 0),
        [MilestoneStatus("C", "Gavel", True, "P99 18ms", "15ms")],
    )
    # The triggered marker is ⚠ in brackets.
    assert "[⚠] **Milestone C**" in brief
