"""Remand feedback generator (RESERVED — not active in v0.2).

Activated on Milestone A (Tier 3 Assessor + subtleness classifier).

See ``experiment/remand-prototype/results/FINAL_REPORT.md`` Section 7
for the experiment that justified this scaffold.

Reference implementation (for Milestone A port):
``experiment/remand-prototype/src/feedback_templates.py``
``experiment/remand-prototype/src/issue_context.py``
"""

from __future__ import annotations


def generate_feedback(
    issue_type: str,
    initial_response: str,
    feedback_level: int = 5,
) -> str:
    """Generate Remand feedback at the given level (1-5).

    Not active until Milestone A. Tier 3 LLM-judge assessor must be
    online to compute subtleness_score before this fires.
    """
    raise NotImplementedError(
        "Remand feedback generator reserved for Milestone A activation. "
        "See experiment/remand-prototype/results/FINAL_REPORT.md Section 7."
    )


def compute_subtleness_score(
    issue_type: str,
    code_patch: str,
    historical_caselog_path: str,
) -> float:
    """Compute subtleness score [0, 1] for selective Remand trigger.

    Not active until Milestone A. Requires Tier 3 LLM-judge assessor.
    """
    raise NotImplementedError(
        "Subtleness classifier reserved for Milestone A activation. "
        "Requires Tier 3 LLM-judge assessor."
    )
