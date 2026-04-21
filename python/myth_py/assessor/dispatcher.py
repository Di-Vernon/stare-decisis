"""Tier 3 dispatcher — Anthropic SDK bridge (Milestone A, stub until activated).

The Anthropic client is constructed but never reached on Day-1 because
`load_api_key()` raises `RuntimeError` when no key file exists. Gate: a
user must run `myth key set` (Milestone A) before this path becomes live.

Telemetry records (tier3-dispatch.jsonl) are written per dispatch so Observer
can compute weekly Tier 3 spend.
"""

from __future__ import annotations

import json
import os
import uuid
from datetime import datetime, timezone
from pathlib import Path

from anthropic import Anthropic
from anthropic.types import TextBlock

API_KEY_PATH = Path.home() / ".config" / "myth" / "api_key"
DISPATCH_LOG = Path.home() / ".local" / "state" / "myth" / "tier3-dispatch.jsonl"
DEFAULT_MODEL = "claude-haiku-4-5-20251001"


def load_api_key() -> str:
    """Resolve API key: env var > config file. Raises if neither is set.

    The raise path is the Milestone A gate — on Day-1, no key is configured,
    so callers naturally bail out before any SDK call.
    """
    key = os.getenv("MYTH_ANTHROPIC_API_KEY")
    if key:
        return key
    if API_KEY_PATH.exists():
        return API_KEY_PATH.read_text(encoding="utf-8").strip()
    raise RuntimeError(
        "No API key configured. Run `myth key set` (Milestone A required)."
    )


def dispatch_haiku(prompt: str, max_tokens: int = 1000) -> str:
    """Call Haiku with the rendered prompt and return the text response.

    Intentionally thin — validation of the structured JSON output is the
    caller's responsibility (see `schema.parse_verdict`).
    """
    client = Anthropic(api_key=load_api_key())
    request_id = str(uuid.uuid4())
    response = client.messages.create(
        model=DEFAULT_MODEL,
        max_tokens=max_tokens,
        messages=[{"role": "user", "content": prompt}],
    )
    first_block = response.content[0]
    if not isinstance(first_block, TextBlock):
        raise RuntimeError(
            f"unexpected Claude response content type: {type(first_block).__name__}"
        )
    body = first_block.text
    _log_dispatch(
        request_id=request_id,
        tokens_in=response.usage.input_tokens,
        tokens_out=response.usage.output_tokens,
    )
    return body


def _log_dispatch(*, request_id: str, tokens_in: int, tokens_out: int) -> None:
    DISPATCH_LOG.parent.mkdir(parents=True, exist_ok=True)
    record = {
        "ts": datetime.now(timezone.utc).isoformat().replace("+00:00", "Z"),
        "request_id": request_id,
        "tokens_in": tokens_in,
        "tokens_out": tokens_out,
        "model": DEFAULT_MODEL,
    }
    with DISPATCH_LOG.open("a", encoding="utf-8") as f:
        f.write(json.dumps(record, ensure_ascii=False) + "\n")
