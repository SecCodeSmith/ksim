"""Fail CI when a SARIF report contains findings at or above a severity level."""

import argparse
import json
import sys
from pathlib import Path
from typing import Any, TextIO

LEVELS = ("none", "note", "warning", "error")
DEFAULT_LEVEL = "warning"
UNKNOWN_LOCATION = "<unknown>"


def effective_level(result: dict[str, Any], rules: dict[str, dict[str, Any]]) -> str:
    """SARIF 2.1.0 §3.27.10: a result's `level` defaults to its rule's
    `defaultConfiguration.level`, and to "warning" when neither is present. `rules` maps
    rule id to the rule object of the run's tool driver."""
    if "level" in result:
        return str(result["level"])
    rule = rules.get(result.get("ruleId", ""), {})
    return str(rule.get("defaultConfiguration", {}).get("level", DEFAULT_LEVEL))


def _rules_by_id(run: dict[str, Any]) -> dict[str, dict[str, Any]]:
    rules = run.get("tool", {}).get("driver", {}).get("rules", [])
    return {rule["id"]: rule for rule in rules if "id" in rule}


def _leveled_findings(sarif: dict[str, Any], threshold: str) -> list[tuple[dict[str, Any], str]]:
    if threshold not in LEVELS:
        raise ValueError(f"unknown level {threshold!r}; expected one of {LEVELS}")
    minimum = LEVELS.index(threshold)
    findings = []
    for run in sarif.get("runs", []):
        rules = _rules_by_id(run)
        for result in run.get("results", []):
            level = effective_level(result, rules)
            if level in LEVELS and LEVELS.index(level) >= minimum:
                findings.append((result, level))
    return findings


def findings_at_or_above(sarif: dict[str, Any], threshold: str) -> list[dict[str, Any]]:
    """Results of every run whose effective level ranks at or above `threshold` in
    LEVELS order; an unknown threshold raises ValueError."""
    return [result for result, _ in _leveled_findings(sarif, threshold)]


def _first_location(result: dict[str, Any]) -> str:
    locations = result.get("locations") or [{}]
    physical = locations[0].get("physicalLocation", {})
    uri = physical.get("artifactLocation", {}).get("uri")
    if uri is None:
        return UNKNOWN_LOCATION
    line = physical.get("region", {}).get("startLine")
    return uri if line is None else f"{uri}:{line}"


def _format_finding(path: str, result: dict[str, Any], level: str) -> str:
    return f"{path}: {result.get('ruleId', '')} {level} {_first_location(result)}"


def main(argv: list[str], stdout: TextIO) -> int:
    """`python -m ksim_ci.sarif_gate FILE... [--level error]`: prints one line per finding
    (`<file>: <ruleId> <level> <first location uri>:<start line>`) and returns 1 when any
    finding reaches the level, 0 otherwise; a missing file returns 2."""
    parser = argparse.ArgumentParser(prog="sarif_gate")
    parser.add_argument("files", nargs="+")
    parser.add_argument("--level", default="error", choices=LEVELS)
    args = parser.parse_args(argv)

    reached = False
    for path in args.files:
        try:
            sarif = json.loads(Path(path).read_text(encoding="utf-8"))
        except FileNotFoundError:
            print(f"{path}: file not found", file=sys.stderr)
            return 2
        for result, level in _leveled_findings(sarif, args.level):
            print(_format_finding(path, result, level), file=stdout)
            reached = True
    return 1 if reached else 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:], sys.stdout))
