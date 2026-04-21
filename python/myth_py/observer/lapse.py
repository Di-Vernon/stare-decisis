"""Lapse computation — transition active lessons to lapsed/archived.

Drift 6 (docs/05 internal contradiction): docs/05 §lapse.py imports
`myth_py.db.SqliteLessonStore` which is not defined anywhere. We inline
sqlite3 usage here instead of creating a myth_py.db wrapper module.
Algorithmic intent (threshold + mark_status) is preserved.
"""

from __future__ import annotations

import sqlite3
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path

# Level → lapse threshold (lower means easier to lapse).
# Level 5 (Bedrock) is immune — threshold is None means "never lapse".
_LAPSE_THRESHOLD: dict[int, int | None] = {
    1: 50,
    2: 50,
    3: 200,
    4: 200,
    5: None,
}

# After this many idle days on lapsed status, archive the lesson.
_ARCHIVE_IDLE_DAYS = 180


@dataclass
class LapseResult:
    new_lapsed_count: int
    revived_count: int
    archived_count: int


def update_lapse_scores(*, state_db_path: Path | None = None) -> LapseResult:
    path = state_db_path or (Path.home() / ".myth" / "state.db")
    result = LapseResult(new_lapsed_count=0, revived_count=0, archived_count=0)

    if not path.exists():
        return result

    conn = sqlite3.connect(str(path), isolation_level=None)
    try:
        conn.execute("PRAGMA busy_timeout = 5000")
        now = datetime.now(timezone.utc)
        active = conn.execute(
            "SELECT id, level, missed_hook_count, last_seen FROM lessons "
            "WHERE status = 'active'"
        ).fetchall()

        for lesson_id, level, missed, last_seen_secs in active:
            threshold = _LAPSE_THRESHOLD.get(int(level))
            if threshold is None:
                continue

            last_seen = datetime.fromtimestamp(int(last_seen_secs), tz=timezone.utc)
            idle_days = max(0, (now - last_seen).days)
            score = float(missed) * 1.0 + float(idle_days) * 10.0

            if score >= threshold:
                conn.execute(
                    "UPDATE lessons SET status = 'lapsed' WHERE id = ?",
                    (lesson_id,),
                )
                result.new_lapsed_count += 1
                if idle_days >= _ARCHIVE_IDLE_DAYS:
                    conn.execute(
                        "UPDATE lessons SET status = 'archived' WHERE id = ?",
                        (lesson_id,),
                    )
                    result.archived_count += 1
    finally:
        conn.close()

    return result
