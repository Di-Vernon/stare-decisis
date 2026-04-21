"""Pydantic models validating assessor verdict JSON.

Used by Tier 3 dispatcher to validate Claude Haiku's structured response.
Tier 0 classifier bypasses this — it emits Classification dataclasses directly.
"""

from __future__ import annotations

from typing import Literal

from pydantic import BaseModel, Field


class AssessorAxes(BaseModel):
    blast_radius: Literal["local", "file", "process", "system", "env"]
    reversibility: Literal["trivial", "possible", "difficult", "impossible"]
    trigger_likelihood: Literal["low", "medium", "high"]


class AssessorVerdict(BaseModel):
    reminder_id: str
    level: int = Field(ge=1, le=5)
    category: Literal[
        "security", "correctness", "process", "data_safety", "temporal"
    ]
    axes: AssessorAxes
    uplift_applied: bool
    rationale: str = Field(max_length=80)
    description: str = Field(min_length=100, max_length=300)
    recommended_action: str


def parse_verdict(json_str: str) -> AssessorVerdict:
    """Parse and validate a verdict JSON string.

    Raises pydantic.ValidationError on schema violation.
    """
    return AssessorVerdict.model_validate_json(json_str)
