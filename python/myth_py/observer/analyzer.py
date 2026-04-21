"""Weekly caselog analysis.

Reads ~/.myth/caselog.jsonl (Rust hook output) + optional shadow metrics +
Tier 3 dispatch log, aggregating into WeeklyAnalysis for brief_gen.

SQLite-based top-lesson pull is inlined here (drift 6: no shared myth_py.db
wrapper per docs/05 internal contradiction; we use sqlite3 directly).
"""

from __future__ import annotations

import json
import sqlite3
from collections import defaultdict
from dataclasses import dataclass, field
from datetime import datetime, timedelta, timezone
from pathlib import Path


@dataclass
class LessonRow:
    lesson_id: str
    level: int
    rationale: str
    recurrence_count: float


@dataclass
class WeeklyAnalysis:
    total_caselog_entries: int = 0
    new_lessons: list[str] = field(default_factory=list)
    recurrence_increments: int = 0
    level_distribution: dict[int, int] = field(
        default_factory=lambda: defaultdict(int)
    )
    category_distribution: dict[str, int] = field(
        default_factory=lambda: defaultdict(int)
    )
    bedrock_matches: int = 0
    tier_1_compliance_rate: float = 0.0
    tier_3_cost_usd: float = 0.0
    top_active_lessons: list[LessonRow] = field(default_factory=list)


def run_analysis(
    *,
    caselog_path: Path | None = None,
    shadow_path: Path | None = None,
    tier3_path: Path | None = None,
    state_db_path: Path | None = None,
) -> WeeklyAnalysis:
    """Aggregate the last 7 days of data.

    All paths are overrideable for tests; defaults match the myth layout.
    Missing files are tolerated (contribute zero to the result).
    """
    caselog_path = caselog_path or (Path.home() / ".myth" / "caselog.jsonl")
    shadow_path = shadow_path or (
        Path.home() / ".myth" / "metrics" / "reflector-shadow.jsonl"
    )
    tier3_path = tier3_path or (
        Path.home() / ".local" / "state" / "myth" / "tier3-dispatch.jsonl"
    )
    state_db_path = state_db_path or (Path.home() / ".myth" / "state.db")

    cutoff = datetime.now(timezone.utc) - timedelta(days=7)
    result = WeeklyAnalysis()

    if caselog_path.exists():
        with caselog_path.open(encoding="utf-8") as f:
            for line in f:
                line = line.strip()
                if not line:
                    continue
                try:
                    entry = json.loads(line)
                except json.JSONDecodeError:
                    continue
                ts = _parse_ts(entry.get("ts"))
                if ts is None or ts < cutoff:
                    continue
                result.total_caselog_entries += 1
                result.level_distribution[int(entry.get("level", 1))] += 1
                result.category_distribution[str(entry.get("category", "unknown"))] += 1
                if entry.get("bedrock_match"):
                    result.bedrock_matches += 1

    if shadow_path.exists():
        result.tier_1_compliance_rate = _compute_tier1_compliance(shadow_path, cutoff)

    if tier3_path.exists():
        result.tier_3_cost_usd = _sum_tier3_costs(tier3_path)

    if state_db_path.exists():
        result.top_active_lessons = _top_active_lessons(state_db_path, limit=10)

    return result


def _parse_ts(value: object) -> datetime | None:
    if not isinstance(value, str):
        return None
    try:
        return datetime.fromisoformat(value.replace("Z", "+00:00"))
    except ValueError:
        return None


def _compute_tier1_compliance(path: Path, cutoff: datetime) -> float:
    total = 0
    compliant = 0
    with path.open(encoding="utf-8") as f:
        for line in f:
            line = line.strip()
            if not line:
                continue
            try:
                entry = json.loads(line)
            except json.JSONDecodeError:
                continue
            ts = _parse_ts(entry.get("ts"))
            if ts is None or ts < cutoff:
                continue
            total += 1
            if entry.get("compliant"):
                compliant += 1
    if total == 0:
        return 0.0
    return compliant / total


def _sum_tier3_costs(path: Path) -> float:
    # Day-1: cost is not directly logged (tokens only). Observer estimates
    # via a placeholder rate; Milestone A will replace this with actual
    # price-per-token tracking.
    total_tokens = 0
    with path.open(encoding="utf-8") as f:
        for line in f:
            line = line.strip()
            if not line:
                continue
            try:
                entry = json.loads(line)
            except json.JSONDecodeError:
                continue
            total_tokens += int(entry.get("tokens_in", 0)) + int(
                entry.get("tokens_out", 0)
            )
    # Haiku approx $0.80 per 1M input + $4 per 1M output; average ~$2.4/1M.
    return round(total_tokens * 2.4 / 1_000_000, 4)


def _top_active_lessons(db_path: Path, limit: int) -> list[LessonRow]:
    conn = sqlite3.connect(str(db_path), isolation_level=None)
    try:
        conn.execute("PRAGMA busy_timeout = 5000")
        cursor = conn.execute(
            "SELECT id, level, rationale, recurrence_count "
            "FROM lessons WHERE status = 'active' "
            "ORDER BY recurrence_count DESC LIMIT ?",
            (limit,),
        )
        rows: list[LessonRow] = []
        for row in cursor.fetchall():
            lesson_id = _uuid_from_blob(row[0])
            rows.append(
                LessonRow(
                    lesson_id=lesson_id,
                    level=int(row[1]),
                    rationale=str(row[2]),
                    recurrence_count=float(row[3]),
                )
            )
        return rows
    finally:
        conn.close()


def _uuid_from_blob(blob: bytes | str) -> str:
    """Render LessonId short form (first 8 chars) from DB's raw 16-byte BLOB."""
    if isinstance(blob, str):
        return blob[:8]
    if len(blob) == 16:
        # Format as hyphenated UUID, then take the first 8 hex chars.
        hex_str = blob.hex()
        return f"{hex_str[0:8]}"
    return "????????"
