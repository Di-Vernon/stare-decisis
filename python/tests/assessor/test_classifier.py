"""Tier 0 pattern matching tests."""

from __future__ import annotations

from myth_py.assessor.classifier import Category, Level, classify


def test_timeout_matches() -> None:
    result = classify({}, "Error: operation timed out after 30s")
    assert result is not None
    assert result.level == Level.LOW
    assert result.category == Category.PROCESS
    assert result.rationale == "transient_network"


def test_rate_limit_matches() -> None:
    result = classify({}, "HTTP 429 Too Many Requests")
    assert result is not None
    assert result.rationale == "rate_limit"
    assert result.confidence >= 0.9


def test_file_not_found_matches() -> None:
    result = classify({}, "ENOENT: no such file or directory: /tmp/x")
    assert result is not None
    assert result.category == Category.CORRECTNESS


def test_merge_conflict_critical() -> None:
    diff = "<<<<<<< HEAD\nfoo\n=======\nbar\n>>>>>>> feature"
    result = classify({}, diff)
    assert result is not None
    assert result.level == Level.HIGH
    assert result.category == Category.DATA_SAFETY


def test_destructive_operation_critical() -> None:
    result = classify({}, "$ rm -rf /var/data")
    assert result is not None
    assert result.level == Level.CRITICAL


def test_no_match_returns_none() -> None:
    assert classify({}, "the flux capacitor fluxed") is None


def test_empty_error_returns_none() -> None:
    assert classify({}, "") is None
