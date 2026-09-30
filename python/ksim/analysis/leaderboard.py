"""Aggregate episode reports into a comparison against a baseline policy."""

import json
import math
from collections import defaultdict
from collections.abc import Sequence
from dataclasses import dataclass
from pathlib import Path
from typing import Any

from ksim.analysis.stats import Interval, bootstrap_ci, paired_wilcoxon

SIGNIFICANCE_LEVEL = 0.05

_SeedValues = dict[int, float]


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
    reports: list[dict[str, Any]] = []
    for path in sorted(directory.glob("*.jsonl")):
        lines = path.read_text().splitlines()
        reports.extend(json.loads(line) for line in lines if line.strip())
    return reports


def summarize(reports: Sequence[dict[str, Any]], baseline: str) -> list[PolicyRow]:
    """One row per (scenario, policy), sorted by scenario then policy name. For each metric:
    bootstrap CI of the mean, `normalized = mean / baseline mean` (1.0 when both are 0,
    inf when only the baseline is 0) and the paired Wilcoxon p-value against the baseline on
    seeds both ran (1.0 for the baseline itself). A scenario without the baseline policy
    raises ValueError."""
    scenarios = _group_by_scenario(reports)
    rows: list[PolicyRow] = []
    for scenario in sorted(scenarios):
        policies = scenarios[scenario]
        if baseline not in policies:
            raise ValueError(f"scenario {scenario!r} has no baseline policy {baseline!r}")
        rows.extend(
            _policy_row(scenario, policy, policies[policy], policies[baseline])
            for policy in sorted(policies)
        )
    return rows


def _group_by_scenario(
    reports: Sequence[dict[str, Any]],
) -> dict[str, dict[str, dict[str, _SeedValues]]]:
    grouped: dict[str, dict[str, dict[str, _SeedValues]]] = defaultdict(
        lambda: defaultdict(lambda: defaultdict(dict))
    )
    for report in reports:
        by_metric = grouped[report["scenario"]][report["policy"]]
        for metric, value in report["metrics"].items():
            by_metric[metric][report["seed"]] = float(value)
    return grouped


def _policy_row(
    scenario: str,
    policy: str,
    by_metric: dict[str, _SeedValues],
    baseline_by_metric: dict[str, _SeedValues],
) -> PolicyRow:
    metrics = {
        metric: _summarize_metric(
            values, baseline_by_metric.get(metric, {}), by_metric is baseline_by_metric
        )
        for metric, values in by_metric.items()
    }
    seeds = {seed for values in by_metric.values() for seed in values}
    return PolicyRow(scenario=scenario, policy=policy, seeds=len(seeds), metrics=metrics)


def _summarize_metric(
    values: _SeedValues, baseline: _SeedValues, is_baseline: bool
) -> MetricSummary:
    interval = bootstrap_ci(list(values.values()))
    baseline_mean = sum(baseline.values()) / len(baseline) if baseline else 0.0
    return MetricSummary(
        interval=interval,
        normalized=_normalize(interval.mean, baseline_mean),
        p_value=1.0 if is_baseline else _paired_p_value(values, baseline),
    )


def _normalize(mean: float, baseline_mean: float) -> float:
    if baseline_mean == 0.0:
        return 1.0 if mean == 0.0 else math.inf
    return mean / baseline_mean


def _paired_p_value(values: _SeedValues, baseline: _SeedValues) -> float:
    shared = sorted(values.keys() & baseline.keys())
    if not shared:
        return 1.0
    return paired_wilcoxon([values[s] for s in shared], [baseline[s] for s in shared])


def render_markdown(rows: Sequence[PolicyRow], metrics: Sequence[str]) -> str:
    """A table per scenario: `| policy | seeds | <metric> ... |` with cells
    `mean [low, high] x<normalized>` formatted to 4 significant digits, plus `*` when
    p < 0.05."""
    by_scenario: dict[str, list[PolicyRow]] = defaultdict(list)
    for row in rows:
        by_scenario[row.scenario].append(row)
    return "\n\n".join(
        _render_table(scenario, group, metrics) for scenario, group in by_scenario.items()
    )


def _render_table(scenario: str, rows: Sequence[PolicyRow], metrics: Sequence[str]) -> str:
    header = "| policy | seeds | " + " | ".join(metrics) + " |"
    divider = "|" + "---|" * (len(metrics) + 2)
    lines = [f"### {scenario}", "", header, divider]
    lines.extend(_render_row(row, metrics) for row in rows)
    return "\n".join(lines)


def _render_row(row: PolicyRow, metrics: Sequence[str]) -> str:
    cells = [_render_cell(row.metrics[m]) if m in row.metrics else "-" for m in metrics]
    return f"| {row.policy} | {row.seeds} | " + " | ".join(cells) + " |"


def _render_cell(summary: MetricSummary) -> str:
    interval = summary.interval
    cell = (
        f"{interval.mean:.4g} [{interval.low:.4g}, {interval.high:.4g}] x{summary.normalized:.4g}"
    )
    return cell + ("*" if summary.p_value < SIGNIFICANCE_LEVEL else "")
