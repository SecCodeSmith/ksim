"""Error between a simulated and an observed trajectory over their common ticks.

Ticks are aligned by index and truncated to the shorter trajectory; ticks where either side
is NaN are skipped per metric. `ready_mae`, `ready_rmse` and `ready_max_error` compare ready
replicas; `desired_mae` compares desired replicas; `scale_up_lag_s` is the observed minus the
simulated time (tick index times tick_s) of the first tick whose desired count exceeds that
side's first desired count, `None` when either side never scales up; `latency_p95_relative_error`
is the median of `|sim - real| / real` over ticks where both have a positive latency, `None`
when there is no such tick; `error_rate_mae` compares error rates.
"""

from dataclasses import dataclass, fields

import numpy as np

from ksim.validation.trajectory import Series, Trajectory


@dataclass(frozen=True)
class ValidationReport:
    ticks: int
    ready_mae: float
    ready_rmse: float
    ready_max_error: float
    desired_mae: float
    scale_up_lag_s: float | None
    latency_p95_relative_error: float | None
    error_rate_mae: float


def compare(simulated: Trajectory, observed: Trajectory) -> ValidationReport:
    """Raises ValueError when the tick lengths differ or there is no common tick."""
    if simulated.tick_s != observed.tick_s:
        raise ValueError("tick lengths differ")
    ticks = min(len(simulated.ready), len(observed.ready))
    if ticks == 0:
        raise ValueError("no common tick")
    ready = _errors(simulated.ready, observed.ready, ticks)
    latency = _relative_errors(simulated.latency_p95_ms, observed.latency_p95_ms, ticks)
    return ValidationReport(
        ticks=ticks,
        ready_mae=_mean(ready),
        ready_rmse=float(np.sqrt(_mean(ready**2))),
        ready_max_error=float(np.max(np.abs(ready))) if ready.size else float("nan"),
        desired_mae=_mean(_errors(simulated.desired, observed.desired, ticks)),
        scale_up_lag_s=_scale_up_lag(simulated, observed),
        latency_p95_relative_error=float(np.median(latency)) if latency.size else None,
        error_rate_mae=_mean(_errors(simulated.error_rate, observed.error_rate, ticks)),
    )


def render_markdown(report: ValidationReport) -> str:
    """A `## Simulator vs. kind cluster` heading and a two-column markdown table
    (`| Metric | Value |`) with one row per report field in declaration order; floats with
    two decimals, `None` as `n/a`."""
    rows = [
        f"| {field.name} | {_format(getattr(report, field.name))} |" for field in fields(report)
    ]
    return "\n".join(
        ["## Simulator vs. kind cluster", "", "| Metric | Value |", "| --- | --- |", *rows, ""]
    )


def _errors(simulated: Series, observed: Series, ticks: int) -> Series:
    """Signed error per tick, with ticks where either side is NaN dropped."""
    difference = simulated[:ticks] - observed[:ticks]
    return difference[~np.isnan(difference)]


def _relative_errors(simulated: Series, observed: Series, ticks: int) -> Series:
    sim, real = simulated[:ticks], observed[:ticks]
    usable = (sim > 0) & (real > 0)
    return np.abs(sim[usable] - real[usable]) / real[usable]


def _mean(values: Series) -> float:
    return float(np.mean(np.abs(values))) if values.size else float("nan")


def _first_scale_up(desired: Series) -> int | None:
    if desired.size == 0:
        return None
    rises = np.flatnonzero(desired > desired[0])
    return int(rises[0]) if rises.size else None


def _scale_up_lag(simulated: Trajectory, observed: Trajectory) -> float | None:
    sim_tick = _first_scale_up(simulated.desired)
    real_tick = _first_scale_up(observed.desired)
    if sim_tick is None or real_tick is None:
        return None
    return float((real_tick - sim_tick) * simulated.tick_s)


def _format(value: int | float | None) -> str:
    if value is None:
        return "n/a"
    return str(value) if isinstance(value, int) else f"{value:.2f}"
