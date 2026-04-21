"""Lapse computation tests — work on a disposable sqlite3 DB."""

from __future__ import annotations

import sqlite3
from datetime import datetime, timedelta, timezone
from pathlib import Path

import pytest

from myth_py.observer.lapse import update_lapse_scores


_SCHEMA = """
CREATE TABLE lessons (
    id BLOB PRIMARY KEY,
    identity_hash_tier1 BLOB NOT NULL,
    level INTEGER NOT NULL,
    category TEXT NOT NULL,
    recurrence_count REAL NOT NULL DEFAULT 0,
    missed_hook_count INTEGER NOT NULL DEFAULT 0,
    first_seen INTEGER NOT NULL,
    last_seen INTEGER NOT NULL,
    lapse_score REAL NOT NULL DEFAULT 0,
    appeals INTEGER NOT NULL DEFAULT 0,
    status TEXT NOT NULL DEFAULT 'active',
    description TEXT NOT NULL DEFAULT '',
    rationale TEXT NOT NULL DEFAULT '',
    meta_json TEXT
);
"""


@pytest.fixture
def db_path(tmp_path: Path) -> Path:
    path = tmp_path / "state.db"
    conn = sqlite3.connect(str(path), isolation_level=None)
    conn.execute(_SCHEMA)
    conn.close()
    return path


def _insert(
    db: Path,
    lesson_id: bytes,
    *,
    level: int,
    missed: int,
    last_seen_days_ago: int,
    status: str = "active",
) -> None:
    now = int(datetime.now(timezone.utc).timestamp())
    last_seen = int(
        (datetime.now(timezone.utc) - timedelta(days=last_seen_days_ago)).timestamp()
    )
    conn = sqlite3.connect(str(db), isolation_level=None)
    conn.execute(
        "INSERT INTO lessons (id, identity_hash_tier1, level, category, "
        "first_seen, last_seen, missed_hook_count, status) "
        "VALUES (?, ?, ?, 'correctness', ?, ?, ?, ?)",
        (lesson_id, b"\x00" * 20, level, now, last_seen, missed, status),
    )
    conn.close()


def _status(db: Path, lesson_id: bytes) -> str:
    conn = sqlite3.connect(str(db), isolation_level=None)
    row = conn.execute(
        "SELECT status FROM lessons WHERE id = ?", (lesson_id,)
    ).fetchone()
    conn.close()
    return row[0]


def test_no_db_returns_zero(tmp_path: Path) -> None:
    result = update_lapse_scores(state_db_path=tmp_path / "dne.db")
    assert result.new_lapsed_count == 0


def test_low_score_stays_active(db_path: Path) -> None:
    lid = b"\x01" * 16
    _insert(db_path, lid, level=3, missed=0, last_seen_days_ago=1)
    update_lapse_scores(state_db_path=db_path)
    assert _status(db_path, lid) == "active"


def test_level1_lapses_on_idle(db_path: Path) -> None:
    # Level 1 threshold = 50. 6 idle days * 10 = 60 → lapses.
    lid = b"\x02" * 16
    _insert(db_path, lid, level=1, missed=0, last_seen_days_ago=6)
    result = update_lapse_scores(state_db_path=db_path)
    assert result.new_lapsed_count == 1
    assert _status(db_path, lid) == "lapsed"


def test_bedrock_level5_immune(db_path: Path) -> None:
    lid = b"\x03" * 16
    _insert(db_path, lid, level=5, missed=9999, last_seen_days_ago=365)
    result = update_lapse_scores(state_db_path=db_path)
    assert result.new_lapsed_count == 0
    assert _status(db_path, lid) == "active"


def test_very_idle_auto_archives(db_path: Path) -> None:
    # Level 1 threshold = 50. 200 idle days * 10 = 2000 → lapses, and
    # 200 >= 180 archive-idle → also archives.
    lid = b"\x04" * 16
    _insert(db_path, lid, level=1, missed=0, last_seen_days_ago=200)
    result = update_lapse_scores(state_db_path=db_path)
    assert result.new_lapsed_count == 1
    assert result.archived_count == 1
    assert _status(db_path, lid) == "archived"
