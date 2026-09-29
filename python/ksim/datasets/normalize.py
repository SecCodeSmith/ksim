"""Turn long-format samples into fixed-tick per-service series.

Loaders produce a long frame with columns `time_s` (float, seconds from trace start),
`service`, `instance`, `request_rate`, `cpu_millis`, `memory_bytes`.
"""

import math
from dataclasses import dataclass

import numpy as np
import numpy.typing as npt
import polars as pl

from ksim.datasets.ksimtrace import ServiceSeries

DAY = 86_400
HOUR = 3_600


@dataclass(frozen=True)
class Window:
    start_s: int
    end_s: int


SPLITS: dict[str, dict[str, Window]] = {
    "alibaba2018": {
        "train": Window(0, 5 * DAY),
        "validation": Window(5 * DAY, 6 * DAY),
        "test": Window(6 * DAY, 8 * DAY),
    },
    "alibaba2021": {
        "train": Window(0, 8 * HOUR),
        "validation": Window(8 * HOUR, 10 * HOUR),
        "test": Window(10 * HOUR, 12 * HOUR),
    },
    "azure2019": {
        "train": Window(0, 10 * DAY),
        "validation": Window(10 * DAY, 12 * DAY),
        "test": Window(12 * DAY, 14 * DAY),
    },
    "borg2019": {
        "train": Window(0, 21 * DAY),
        "validation": Window(21 * DAY, 24 * DAY),
        "test": Window(24 * DAY, 31 * DAY),
    },
}


def to_series(frame: pl.DataFrame, tick_seconds: int, window: Window) -> list[ServiceSeries]:
    """Keep samples with `window.start_s <= time_s < window.end_s`; bucket them into ticks
    `floor((time_s - start) / tick)`; average each instance's samples within a tick, then sum
    instances per service. Ticks without samples carry the previous tick's value forward
    (0 before the first sample). Returns one series per service, sorted by name, each with
    `ceil((end - start) / tick)` ticks. Raises ValueError for a non-positive tick or an
    empty window."""
    if tick_seconds <= 0:
        raise ValueError("tick_seconds must be positive")
    if window.start_s >= window.end_s:
        raise ValueError("window must not be empty")
    tick_count = math.ceil((window.end_s - window.start_s) / tick_seconds)
    per_tick = _per_tick_totals(frame, tick_seconds, window)
    return [
        _series_for(name, part, tick_count)
        for (name,), part in per_tick.partition_by("service", as_dict=True).items()
    ]


def _per_tick_totals(frame: pl.DataFrame, tick_seconds: int, window: Window) -> pl.DataFrame:
    return (
        frame.lazy()
        .filter((pl.col("time_s") >= window.start_s) & (pl.col("time_s") < window.end_s))
        .with_columns(
            ((pl.col("time_s") - window.start_s) / tick_seconds)
            .floor()
            .cast(pl.Int64)
            .alias("tick")
        )
        .group_by("service", "instance", "tick")
        .agg(pl.col("request_rate", "cpu_millis", "memory_bytes").mean())
        .group_by("service", "tick")
        .agg(pl.col("request_rate", "cpu_millis", "memory_bytes").sum())
        .sort("service")
        .collect()
    )


def _series_for(name: str, part: pl.DataFrame, tick_count: int) -> ServiceSeries:
    ticks = part["tick"].to_numpy()
    return ServiceSeries(
        name=str(name),
        request_rate=_forward_filled(ticks, part["request_rate"].to_numpy(), tick_count),
        cpu_millis=_forward_filled(ticks, part["cpu_millis"].to_numpy(), tick_count),
        memory_bytes=_forward_filled(ticks, part["memory_bytes"].to_numpy(), tick_count),
    )


def _forward_filled(
    ticks: npt.NDArray[np.int64], values: npt.NDArray[np.float64], tick_count: int
) -> npt.NDArray[np.float32]:
    dense = np.zeros(tick_count, dtype=np.float64)
    dense[ticks] = values
    source = np.full(tick_count, -1, dtype=np.int64)
    source[ticks] = ticks
    latest = np.maximum.accumulate(source)
    filled = np.where(latest >= 0, dense[np.maximum(latest, 0)], 0.0)
    return filled.astype(np.float32)


def top_services(series: list[ServiceSeries], count: int) -> list[ServiceSeries]:
    """The `count` services with the largest total CPU, ties by name, in descending CPU
    order."""
    ranked = sorted(
        series, key=lambda item: (-float(item.cpu_millis.sum(dtype=np.float64)), item.name)
    )
    return ranked[:count]
