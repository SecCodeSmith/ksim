"""Fail CI when a SARIF report contains findings at or above a severity level."""

import sys
from typing import Any, TextIO

LEVELS = ("none", "note", "warning", "error")


def effective_level(result: dict[str, Any], rules: dict[str, dict[str, Any]]) -> str:
    """SARIF 2.1.0 §3.27.10: a result's `level` defaults to its rule's
    `defaultConfiguration.level`, and to "warning" when neither is present. `rules` maps
    rule id to the rule object of the run's tool driver."""
    raise NotImplementedError


def findings_at_or_above(sarif: dict[str, Any], threshold: str) -> list[dict[str, Any]]:
    """Results of every run whose effective level ranks at or above `threshold` in
    LEVELS order; an unknown threshold raises ValueError."""
    raise NotImplementedError


def main(argv: list[str], stdout: TextIO) -> int:
    """`python -m ksim_ci.sarif_gate FILE... [--level error]`: prints one line per finding
    (`<file>: <ruleId> <level> <first location uri>:<start line>`) and returns 1 when any
    finding reaches the level, 0 otherwise; a missing file returns 2."""
    raise NotImplementedError


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:], sys.stdout))
