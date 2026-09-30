"""Error between a simulated and an observed trajectory over their common ticks.

Ticks are aligned by index and truncated to the shorter trajectory; ticks where either side
is NaN are skipped per metric. `ready_mae`, `ready_rmse` and `ready_max_error` compare ready
replicas; `desired_mae` compares desired replicas; `scale_up_lag_s` is the observed minus the
simulated time (tick index times tick_s) of the first tick whose desired count exceeds that
side's first desired count, `None` when either side never scales up; `latency_p95_relative_error`
is the median of `|sim - real| / real` over ticks where both have a positive latency, `None`
when there is no such tick; `error_rate_mae` compares error rates.
"""

from dataclasses import dataclass

from ksim.validation.trajectory import Trajectory


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
    raise NotImplementedError


def render_markdown(report: ValidationReport) -> str:
    """A `## Simulator vs. kind cluster` heading and a two-column markdown table
    (`| Metric | Value |`) with one row per report field in declaration order; floats with
    two decimals, `None` as `n/a`."""
    raise NotImplementedError
