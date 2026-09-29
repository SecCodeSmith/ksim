"""Aggregate episode reports into a comparison against a baseline policy."""

from collections.abc import Sequence
from dataclasses import dataclass
from pathlib import Path
from typing import Any

from ksim.analysis.stats import Interval


@dataclass(frozen=True)
class MetricSummary:
    interval: Interval
    normalized: float
    p_value: float


@dataclass(frozen=True)
class PolicyRow:
    scenario: str
    policy: str
    seeds: int
    metrics: dict[str, MetricSummary]


def load_reports(directory: Path) -> list[dict[str, Any]]:
    """Every JSON line of every `*.jsonl` file in `directory`, files in name order."""
    raise NotImplementedError


def summarize(reports: Sequence[dict[str, Any]], baseline: str) -> list[PolicyRow]:
    """One row per (scenario, policy), sorted by scenario then policy name. For each metric:
    bootstrap CI of the mean, `normalized = mean / baseline mean` (1.0 when both are 0,
    inf when only the baseline is 0) and the paired Wilcoxon p-value against the baseline on
    seeds both ran (1.0 for the baseline itself). A scenario without the baseline policy
    raises ValueError."""
    raise NotImplementedError


def render_markdown(rows: Sequence[PolicyRow], metrics: Sequence[str]) -> str:
    """A table per scenario: `| policy | seeds | <metric> ... |` with cells
    `mean [low, high] x<normalized>` formatted to 4 significant digits, plus `*` when
    p < 0.05."""
    raise NotImplementedError
