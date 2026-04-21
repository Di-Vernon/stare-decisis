"""Milestone Readiness evaluation — A, B, C, D, E.

Milestone C (Gavel daemon) uses production hook-latency.ndjson only, per
Wave 3 기반점 2 (bench results are excluded from trigger evaluation).
Other milestones are Day-1 pending placeholders.
"""

from __future__ import annotations

import json
from collections.abc import Iterator
from dataclasses import dataclass, field
from datetime import datetime, timedelta, timezone
from pathlib import Path


@dataclass
class MilestoneStatus:
    id: str  # "A" .. "E"
    title: str
    triggered: bool
    current_value: str
    threshold: str
    notes: list[str] = field(default_factory=list)


def compute_all_milestones(
    *,
    hook_latency_path: Path | None = None,
) -> list[MilestoneStatus]:
    hook_latency_path = hook_latency_path or (
        Path.home() / ".local" / "state" / "myth" / "hook-latency.ndjson"
    )
    return [
        _milestone_a(),
        _milestone_b(),
        _milestone_c(hook_latency_path),
        _milestone_d(),
        _milestone_e(),
    ]


def _milestone_a() -> MilestoneStatus:
    return MilestoneStatus(
        id="A",
        title="Assessor Tier review (3 weeks elapsed)",
        triggered=False,
        current_value="pending",
        threshold="3 weeks + tier-1 compliance < 70%",
    )


def _milestone_b() -> MilestoneStatus:
    return MilestoneStatus(
        id="B",
        title="Vector store migration",
        triggered=False,
        current_value="pending",
        threshold="N/A (design work)",
    )


def _milestone_c(latency_path: Path) -> MilestoneStatus:
    """Gavel daemon migration — production-only data, per 기반점 2."""
    if not latency_path.exists():
        return MilestoneStatus(
            id="C",
            title="The Gavel daemon migration",
            triggered=False,
            current_value="no data",
            threshold="P99 > 15ms (sustained 14d)",
            notes=[f"source: {latency_path} (production only)"],
        )

    cutoff = datetime.now(timezone.utc) - timedelta(days=14)
    latencies: list[float] = []
    for entry in _iter_ndjson(latency_path):
        if entry.get("event") != "pre_tool":
            continue
        ts = _parse_ts(entry.get("ts"))
        if ts is None or ts < cutoff:
            continue
        raw = entry.get("latency_ms")
        if isinstance(raw, (int, float)):
            latencies.append(float(raw))

    if not latencies:
        return MilestoneStatus(
            id="C",
            title="The Gavel daemon migration",
            triggered=False,
            current_value="insufficient data",
            threshold="P99 > 15ms (sustained 14d)",
            notes=[
                f"source: {latency_path} (production only)",
                "no pre_tool events in last 14 days",
            ],
        )

    latencies.sort()
    p99_index = min(int(len(latencies) * 0.99), len(latencies) - 1)
    p99 = latencies[p99_index]

    # Require minimum sample density so we do not trip on a handful of
    # startup outliers.
    min_sample = 14 * 10
    triggered = p99 > 15.0 and len(latencies) >= min_sample

    return MilestoneStatus(
        id="C",
        title="The Gavel daemon migration",
        triggered=triggered,
        current_value=f"P99: {p99:.1f}ms",
        threshold="P99 > 15ms (sustained 14d)",
        notes=[
            f"source: {latency_path} (production only)",
            f"samples: {len(latencies)} pre_tool events in 14d",
            f"min density: {min_sample}",
        ],
    )


def _milestone_d() -> MilestoneStatus:
    return MilestoneStatus(
        id="D",
        title="Semantic detection",
        triggered=False,
        current_value="pending",
        threshold="N/A (design work)",
    )


def _milestone_e() -> MilestoneStatus:
    return MilestoneStatus(
        id="E",
        title="AST validation",
        triggered=False,
        current_value="pending",
        threshold="N/A (design work)",
    )


def _iter_ndjson(path: Path) -> "Iterator[dict[str, object]]":
    with path.open(encoding="utf-8") as f:
        for line in f:
            line = line.strip()
            if not line:
                continue
            try:
                yield json.loads(line)
            except json.JSONDecodeError:
                continue


def _parse_ts(value: object) -> datetime | None:
    if not isinstance(value, str):
        return None
    try:
        return datetime.fromisoformat(value.replace("Z", "+00:00"))
    except ValueError:
        return None
