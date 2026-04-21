"""Observer — weekly roll-up analysis producing brief.md.

Pipeline: analyzer -> brief_gen -> migration -> lapse -> cli dispatch.
Called via `python3 -m myth_py.observer.cli run [--dry]` (Wave 5 myth-cli
observer subcmd), or directly as the `myth-observer` console script
(pyproject.toml [project.scripts]).

Explicitly excluded (docs/05 §본문 부재, Day-1 scope 외):
- report.py (pipeline absent in §cli.py and §실행 지점 table)
"""
