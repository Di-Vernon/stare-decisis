"""Pydantic verdict schema tests."""

from __future__ import annotations

import pytest
from pydantic import ValidationError

from myth_py.assessor.schema import AssessorAxes, AssessorVerdict, parse_verdict


def _valid_payload() -> dict:
    return {
        "reminder_id": "rid-123",
        "level": 3,
        "category": "correctness",
        "axes": {
            "blast_radius": "file",
            "reversibility": "possible",
            "trigger_likelihood": "medium",
        },
        "uplift_applied": False,
        "rationale": "ENOENT on required config",
        "description": (
            "The tool attempted to open a configuration file that does not "
            "exist in the project root, which points to a missing scaffold "
            "step or an incorrect path assumption upstream in the pipeline."
        ),
        "recommended_action": "Check file path; scaffold if absent.",
    }


def test_valid_payload_parses() -> None:
    import json

    verdict = parse_verdict(json.dumps(_valid_payload()))
    assert isinstance(verdict, AssessorVerdict)
    assert verdict.level == 3
    assert verdict.axes.blast_radius == "file"


def test_level_range_enforced() -> None:
    import json

    payload = _valid_payload()
    payload["level"] = 6
    with pytest.raises(ValidationError):
        parse_verdict(json.dumps(payload))


def test_category_literal_enforced() -> None:
    import json

    payload = _valid_payload()
    payload["category"] = "not-a-valid-category"
    with pytest.raises(ValidationError):
        parse_verdict(json.dumps(payload))


def test_description_min_length() -> None:
    import json

    payload = _valid_payload()
    payload["description"] = "too short"
    with pytest.raises(ValidationError):
        parse_verdict(json.dumps(payload))


def test_axes_literal_enforced() -> None:
    axes = AssessorAxes(
        blast_radius="local",
        reversibility="trivial",
        trigger_likelihood="low",
    )
    assert axes.blast_radius == "local"
