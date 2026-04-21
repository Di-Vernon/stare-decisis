"""Variant B prompt rendering.

Reads the shared template file (~/myth/templates/assessor-variants/variant_b.md)
so Rust (include_str!) and Python (file read) produce identical prompts.
"""

from __future__ import annotations

import json
import os
from pathlib import Path


def _default_template_dir() -> Path:
    # Override for tests / non-standard layouts.
    override = os.environ.get("MYTH_TEMPLATE_DIR")
    if override:
        return Path(override)
    # ~/.myth/templates/ is where `myth install` copies repo templates/.
    return Path.home() / ".myth" / "templates" / "assessor-variants"


def render_variant_b(
    tool_name: str,
    compact_json: dict[str, object],
    reminder_id: str,
    template_dir: Path | None = None,
) -> str:
    """Render the Variant B prompt with tool-name / compact-json / RID slots."""
    base = template_dir or _default_template_dir()
    path = base / "variant_b.md"
    template = path.read_text(encoding="utf-8")
    return (
        template.replace("{tool_name}", tool_name)
        .replace("{compact_json}", _compact(compact_json))
        .replace("{rid}", reminder_id)
    )


def _compact(obj: dict[str, object]) -> str:
    """Compact JSON, no whitespace, first 500 chars."""
    return json.dumps(obj, separators=(",", ":"), ensure_ascii=False)[:500]
