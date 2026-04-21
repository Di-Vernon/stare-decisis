"""Tier 0 deterministic classifier — regex-based pattern matching.

Rust `myth-hooks` handles the hot path. Python classifier is invoked when
Rust defers (low confidence or out-of-pattern) for a richer but still
LLM-free decision.

Patterns mirror Rust's Tier 0 rules (docs/05 §classifier.py line 172-188).
"""

from __future__ import annotations

import re
from dataclasses import dataclass
from enum import Enum


class Level(Enum):
    INFO = 1
    LOW = 2
    MEDIUM = 3
    HIGH = 4
    CRITICAL = 5


class Category(Enum):
    SECURITY = "security"
    CORRECTNESS = "correctness"
    PROCESS = "process"
    DATA_SAFETY = "data_safety"
    TEMPORAL = "temporal"


@dataclass
class Classification:
    level: Level
    category: Category
    rationale: str
    confidence: float  # 0.0 .. 1.0


# (regex, level, category, rationale, confidence). Order = priority.
PATTERNS: list[tuple[re.Pattern[str], Level, Category, str, float]] = [
    (
        re.compile(r"(timeout|timed out|ETIMEDOUT)", re.I),
        Level.LOW,
        Category.PROCESS,
        "transient_network",
        0.9,
    ),
    (
        re.compile(r"(429|rate limit|too many requests)", re.I),
        Level.LOW,
        Category.PROCESS,
        "rate_limit",
        0.95,
    ),
    (
        re.compile(r"(ENOENT|no such file|file not found)", re.I),
        Level.MEDIUM,
        Category.CORRECTNESS,
        "file_not_found",
        0.85,
    ),
    (
        re.compile(r"(EACCES|permission denied)", re.I),
        Level.MEDIUM,
        Category.SECURITY,
        "permission_denied",
        0.8,
    ),
    (
        re.compile(r"(SyntaxError|ParseError)"),
        Level.MEDIUM,
        Category.CORRECTNESS,
        "syntax_error",
        0.9,
    ),
    (
        re.compile(r"(<<<<<<< HEAD|=======|>>>>>>> )", re.M),
        Level.HIGH,
        Category.DATA_SAFETY,
        "merge_conflict_artifact",
        0.95,
    ),
    (
        re.compile(r"(rm -rf /|DROP TABLE|DELETE FROM \w+(?! WHERE))", re.I),
        Level.CRITICAL,
        Category.DATA_SAFETY,
        "destructive_operation",
        0.98,
    ),
    (
        re.compile(r"(api.?key|secret.?token|password).{0,30}[=:]", re.I),
        Level.HIGH,
        Category.SECURITY,
        "credential_exposure",
        0.8,
    ),
]


def classify(tool_input: dict[str, object], error: str) -> Classification | None:
    """Classify a tool failure deterministically.

    `tool_input` is reserved for future structural inspection (e.g., inspecting
    the `Bash` command string separately from stderr). Day-1 uses stderr only.

    Returns `None` when no pattern matches with sufficient confidence — caller
    should escalate to Tier 1 subagent.
    """
    del tool_input  # reserved for future use
    for regex, level, category, rationale, confidence in PATTERNS:
        if regex.search(error):
            return Classification(level, category, rationale, confidence)
    return None
