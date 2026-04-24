"""Dispatcher tests — Milestone A gate semantics only.

Real Anthropic API calls are never made (Milestone A stub enforcement).
"""

from __future__ import annotations

from pathlib import Path

import pytest

from myth_py.assessor import dispatcher


def _isolate_env(monkeypatch: pytest.MonkeyPatch, tmp_path: Path) -> None:
    """Strip every key-resolution input so each test sets exactly one source."""
    monkeypatch.delenv("ANTHROPIC_API_KEY", raising=False)
    monkeypatch.delenv("MYTH_ANTHROPIC_API_KEY", raising=False)
    monkeypatch.setattr(dispatcher, "API_KEY_PATH", tmp_path / "legacy_api_key")
    monkeypatch.setattr(dispatcher, "CREDENTIALS_PATH", tmp_path / "credentials")


def test_load_api_key_anthropic_env_takes_precedence(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path
) -> None:
    _isolate_env(monkeypatch, tmp_path)
    monkeypatch.setenv("ANTHROPIC_API_KEY", "sk-ant-canonical-env")
    monkeypatch.setenv("MYTH_ANTHROPIC_API_KEY", "sk-ant-legacy-env")  # should not win
    assert dispatcher.load_api_key() == "sk-ant-canonical-env"


def test_load_api_key_from_legacy_env(monkeypatch: pytest.MonkeyPatch, tmp_path: Path) -> None:
    _isolate_env(monkeypatch, tmp_path)
    monkeypatch.setenv("MYTH_ANTHROPIC_API_KEY", "sk-ant-test-0001")
    assert dispatcher.load_api_key() == "sk-ant-test-0001"


def test_load_api_key_from_credentials_file(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path
) -> None:
    _isolate_env(monkeypatch, tmp_path)
    cred = tmp_path / "credentials"
    cred.write_text("# header\n\nANTHROPIC_API_KEY=sk-ant-creds-0003\n", encoding="utf-8")
    monkeypatch.setattr(dispatcher, "CREDENTIALS_PATH", cred)
    assert dispatcher.load_api_key() == "sk-ant-creds-0003"


def test_load_api_key_from_legacy_file(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path
) -> None:
    _isolate_env(monkeypatch, tmp_path)
    legacy = tmp_path / "legacy_api_key"
    legacy.write_text("sk-ant-test-0002\n", encoding="utf-8")
    monkeypatch.setattr(dispatcher, "API_KEY_PATH", legacy)
    assert dispatcher.load_api_key() == "sk-ant-test-0002"


def test_load_api_key_missing_raises(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path
) -> None:
    _isolate_env(monkeypatch, tmp_path)
    with pytest.raises(RuntimeError, match="No API key configured"):
        dispatcher.load_api_key()


def test_load_api_key_credentials_skips_unrelated_keys(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path
) -> None:
    _isolate_env(monkeypatch, tmp_path)
    cred = tmp_path / "credentials"
    cred.write_text(
        "OTHER_KEY=ignored\nANTHROPIC_API_KEY=sk-ant-extracted\n",
        encoding="utf-8",
    )
    monkeypatch.setattr(dispatcher, "CREDENTIALS_PATH", cred)
    assert dispatcher.load_api_key() == "sk-ant-extracted"
