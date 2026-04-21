"""myth Python layer — Assessor (Tier 0/3) and Observer (weekly analysis).

See ~/myth/docs/05-PYTHON.md for architecture. Rust side is authoritative for
hook-path deterministic checks; Python handles structured failure analysis
(assessor) and periodic roll-ups (observer).
"""

__version__ = "0.1.0"
