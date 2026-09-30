"""Per-tick replica and latency series shared by the simulated and the observed side."""

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
    raise NotImplementedError
