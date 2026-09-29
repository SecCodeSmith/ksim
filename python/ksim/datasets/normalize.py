"""Turn long-format samples into fixed-tick per-service series.

Loaders produce a long frame with columns `time_s` (float, seconds from trace start),
`service`, `instance`, `request_rate`, `cpu_millis`, `memory_bytes`.
"""

from dataclasses import dataclass

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
    raise NotImplementedError


def top_services(series: list[ServiceSeries], count: int) -> list[ServiceSeries]:
    """The `count` services with the largest total CPU, ties by name, in descending CPU
    order."""
    raise NotImplementedError
