"""Observer analyzer tests."""

from __future__ import annotations

import json
from datetime import datetime, timedelta, timezone
from pathlib import Path

from myth_py.observer.analyzer import run_analysis


def test_empty_paths_yields_zero_counts(tmp_path: Path) -> None:
    result = run_analysis(
        caselog_path=tmp_path / "caselog.jsonl",
        shadow_path=tmp_path / "shadow.jsonl",
        tier3_path=tmp_path / "tier3.jsonl",
        state_db_path=tmp_path / "state.db",
    )
    assert result.total_caselog_entries == 0
    assert result.bedrock_matches == 0
    assert result.tier_3_cost_usd == 0.0


def test_caselog_within_7d_counted(tmp_path: Path) -> None:
    caselog = tmp_path / "caselog.jsonl"
    now = datetime.now(timezone.utc)
    rows = [
        {
            "ts": (now - timedelta(days=1)).isoformat().replace("+00:00", "Z"),
            "level": 3,
            "category": "correctness",
            "bedrock_match": False,
        },
        {
            "ts": (now - timedelta(days=10)).isoformat().replace("+00:00", "Z"),
            "level": 4,
            "category": "security",
        },
        {
            "ts": (now - timedelta(hours=2)).isoformat().replace("+00:00", "Z"),
            "level": 5,
            "category": "data_safety",
            "bedrock_match": True,
        },
    ]
    caselog.write_text(
        "\n".join(json.dumps(r) for r in rows) + "\n", encoding="utf-8"
    )

    result = run_analysis(
        caselog_path=caselog,
        shadow_path=tmp_path / "no.jsonl",
        tier3_path=tmp_path / "no.jsonl",
        state_db_path=tmp_path / "no.db",
    )
    assert result.total_caselog_entries == 2  # 10-day-old entry excluded
    assert result.level_distribution[3] == 1
    assert result.level_distribution[5] == 1
    assert result.bedrock_matches == 1


def test_tier3_cost_estimated(tmp_path: Path) -> None:
    tier3 = tmp_path / "tier3.jsonl"
    tier3.write_text(
        json.dumps({"tokens_in": 500_000, "tokens_out": 500_000}) + "\n",
        encoding="utf-8",
    )
    result = run_analysis(
        caselog_path=tmp_path / "none.jsonl",
        shadow_path=tmp_path / "none.jsonl",
        tier3_path=tier3,
        state_db_path=tmp_path / "none.db",
    )
    # 1M tokens * $2.4/1M = $2.4
    assert result.tier_3_cost_usd == 2.4
