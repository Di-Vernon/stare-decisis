"""Migration Readiness tests — Milestone C logic primarily."""

from __future__ import annotations

import json
from datetime import datetime, timedelta, timezone
from pathlib import Path

from myth_py.observer.migration import compute_all_milestones


def test_no_latency_file_c_is_no_data(tmp_path: Path) -> None:
    milestones = compute_all_milestones(
        hook_latency_path=tmp_path / "missing.ndjson"
    )
    c = next(m for m in milestones if m.id == "C")
    assert not c.triggered
    assert c.current_value == "no data"


def test_c_insufficient_data_not_triggered(tmp_path: Path) -> None:
    path = tmp_path / "latency.ndjson"
    path.write_text("", encoding="utf-8")
    milestones = compute_all_milestones(hook_latency_path=path)
    c = next(m for m in milestones if m.id == "C")
    assert not c.triggered


def test_c_high_latency_triggers(tmp_path: Path) -> None:
    path = tmp_path / "latency.ndjson"
    now = datetime.now(timezone.utc)
    lines = []
    # 150 events, latency 1-150ms — P99 ~ 149ms, well above 15ms.
    for i in range(1, 151):
        ts = (now - timedelta(days=1)).isoformat().replace("+00:00", "Z")
        lines.append(
            json.dumps({"ts": ts, "event": "pre_tool", "latency_ms": float(i)})
        )
    path.write_text("\n".join(lines) + "\n", encoding="utf-8")

    milestones = compute_all_milestones(hook_latency_path=path)
    c = next(m for m in milestones if m.id == "C")
    assert c.triggered, f"expected triggered, got current_value={c.current_value}"


def test_c_low_latency_not_triggered(tmp_path: Path) -> None:
    path = tmp_path / "latency.ndjson"
    now = datetime.now(timezone.utc)
    lines = []
    for i in range(1, 151):
        ts = (now - timedelta(days=1)).isoformat().replace("+00:00", "Z")
        lines.append(
            json.dumps({"ts": ts, "event": "pre_tool", "latency_ms": 5.0})
        )
    path.write_text("\n".join(lines) + "\n", encoding="utf-8")

    milestones = compute_all_milestones(hook_latency_path=path)
    c = next(m for m in milestones if m.id == "C")
    assert not c.triggered


def test_all_five_milestones_returned(tmp_path: Path) -> None:
    milestones = compute_all_milestones(hook_latency_path=tmp_path / "n.ndjson")
    ids = [m.id for m in milestones]
    assert ids == ["A", "B", "C", "D", "E"]
