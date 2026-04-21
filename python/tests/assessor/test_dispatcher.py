"""Dispatcher tests — Milestone A gate semantics only.

Real Anthropic API calls are never made (Milestone A stub enforcement).
"""

from __future__ import annotations

from pathlib import Path

import pytest

from myth_py.assessor import dispatcher


def test_load_api_key_from_env(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setenv("MYTH_ANTHROPIC_API_KEY", "sk-ant-test-0001")
    monkeypatch.setattr(
        dispatcher, "API_KEY_PATH", Path("/nonexistent/api_key_dne")
    )
    assert dispatcher.load_api_key() == "sk-ant-test-0001"


def test_load_api_key_from_file(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path
) -> None:
    monkeypatch.delenv("MYTH_ANTHROPIC_API_KEY", raising=False)
    key_file = tmp_path / "api_key"
    key_file.write_text("sk-ant-test-0002\n", encoding="utf-8")
    monkeypatch.setattr(dispatcher, "API_KEY_PATH", key_file)
    assert dispatcher.load_api_key() == "sk-ant-test-0002"


def test_load_api_key_missing_raises(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path
) -> None:
    monkeypatch.delenv("MYTH_ANTHROPIC_API_KEY", raising=False)
    monkeypatch.setattr(dispatcher, "API_KEY_PATH", tmp_path / "not_there")
    with pytest.raises(RuntimeError, match="No API key configured"):
        dispatcher.load_api_key()
