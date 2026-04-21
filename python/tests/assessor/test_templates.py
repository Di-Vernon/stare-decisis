"""Variant B rendering tests."""

from __future__ import annotations

from pathlib import Path

import pytest

from myth_py.assessor.templates import render_variant_b


@pytest.fixture
def template_dir(tmp_path: Path) -> Path:
    variant = tmp_path / "variant_b.md"
    variant.write_text(
        "Tool: {tool_name}\nInput: {compact_json}\nRID: {rid}\n",
        encoding="utf-8",
    )
    return tmp_path


def test_render_substitutes_slots(template_dir: Path) -> None:
    out = render_variant_b(
        tool_name="Bash",
        compact_json={"cmd": "ls -la"},
        reminder_id="rid-42",
        template_dir=template_dir,
    )
    assert "Tool: Bash" in out
    assert '"cmd":"ls -la"' in out
    assert "RID: rid-42" in out


def test_compact_json_truncated_to_500(template_dir: Path) -> None:
    big = {"key": "x" * 1000}
    out = render_variant_b("Bash", big, "rid", template_dir=template_dir)
    # The replaced {compact_json} segment should be at most 500 chars.
    input_line = [line for line in out.splitlines() if line.startswith("Input:")][0]
    compact = input_line.removeprefix("Input: ")
    assert len(compact) <= 500
