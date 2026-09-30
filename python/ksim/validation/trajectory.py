"""Per-tick replica and latency series shared by the simulated and the observed side."""

import csv
from dataclasses import dataclass
from pathlib import Path

import numpy as np
import numpy.typing as npt

Series = npt.NDArray[np.float64]


@dataclass(frozen=True)
class Trajectory:
    """Equal-length float64 series, one entry per tick of `tick_s` seconds. `latency_p95_ms` is
    NaN for ticks without requests; `demand_rps` is NaN where the demand is unknown."""

    tick_s: int
    demand_rps: Series
    desired: Series
    ready: Series
    latency_p95_ms: Series
    error_rate: Series


def load_simulated(path: Path, service: str) -> Trajectory:
    """The rows of `service` from a `ksim-bench trajectory` CSV, in file order; `tick_s` is the
    spacing of `time_s`. Raises ValueError when the service is absent or has fewer than two
    rows (the tick cannot be inferred)."""
    rows = _service_rows(path, service)
    if len(rows) < 2:
        raise ValueError(f"service {service!r} needs at least two rows in {path}")
    return Trajectory(
        tick_s=round(_column(rows, "time_s")[1] - _column(rows, "time_s")[0]),
        demand_rps=_column(rows, "demand_rps"),
        desired=_column(rows, "desired"),
        ready=_column(rows, "ready"),
        latency_p95_ms=_column(rows, "latency_p95_ms"),
        error_rate=_column(rows, "error_rate"),
    )


_REQUIRED_COLUMNS = (
    "time_s",
    "service",
    "demand_rps",
    "desired",
    "ready",
    "latency_p95_ms",
    "error_rate",
)


def _service_rows(path: Path, service: str) -> list[dict[str, str]]:
    with path.open(newline="") as handle:
        reader = csv.DictReader(handle)
        missing = [name for name in _REQUIRED_COLUMNS if name not in (reader.fieldnames or [])]
        if missing:
            raise ValueError(f"{path} lacks columns {missing}")
        return [row for row in reader if row["service"] == service]


def _column(rows: list[dict[str, str]], name: str) -> Series:
    return np.array([float(row[name]) for row in rows], dtype=np.float64)
