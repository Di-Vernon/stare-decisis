"""Assessor — tool-failure classification.

Day-1 surfaces (per docs/05 §역할 분담):
- Tier 0 classifier.py — deterministic regex-based classification
- Tier 3 dispatcher.py — Anthropic SDK (Milestone A, unreachable until `myth key set`)
- templates.py — Variant B prompt rendering
- schema.py — Pydantic verdict validation
- cli.py — reserved stub (Wave 8 direct-invocation integration)

Explicitly excluded (docs/05 §본문 부재, Day-1 scope 외):
- subagent_runner.py (§역할 분담 경로 1: Claude Code 내부 완결)
- state.py (lesson-state.jsonl read path not used in Day-1 observer pipeline)
"""
