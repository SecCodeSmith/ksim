"""Build the observed `Trajectory` from the cluster recorder CSV and k6's CSV output.

Recorder CSV (`tools/validation/record.sh`): header `epoch_s,desired,ready`, one sample per
poll. k6 CSV (`k6 run --out csv=...`): header with at least `metric_name,timestamp,
metric_value`; `timestamp` is Unix seconds, `http_req_duration` rows carry milliseconds and
`http_req_failed` rows carry 0 or 1.

Tick `i` covers `[start_epoch + i * tick_s, start_epoch + (i + 1) * tick_s)`. Per tick:
`desired`/`ready` are the last recorder sample inside the tick, carried forward from the
previous tick when none falls inside (NaN before the first sample); `latency_p95_ms` is the
95th percentile (numpy linear) of that tick's request durations, NaN without requests;
`error_rate` is the mean of that tick's `http_req_failed` values, 0 without requests;
`demand_rps` is NaN.
"""

import csv
from pathlib import Path

import numpy as np

from ksim.validation.trajectory import Series, Trajectory


def load_observed(
    recorder_csv: Path, k6_csv: Path, start_epoch: float, tick_s: int, ticks: int
) -> Trajectory:
    """`ticks` ticks starting at `start_epoch`. Raises ValueError for a non-positive tick or
    tick count, or a CSV missing its required columns."""
    if tick_s <= 0 or ticks <= 0:
        raise ValueError("tick_s and ticks must be positive")
    desired, ready = _recorder_series(recorder_csv, start_epoch, tick_s, ticks)
    durations, failures = _k6_buckets(k6_csv, start_epoch, tick_s, ticks)
    return Trajectory(
        tick_s=tick_s,
        demand_rps=np.full(ticks, np.nan),
        desired=desired,
        ready=ready,
        latency_p95_ms=np.array(
            [np.percentile(v, 95, method="linear") if v else np.nan for v in durations]
        ),
        error_rate=np.array([float(np.mean(v)) if v else 0.0 for v in failures]),
    )


def _read_rows(path: Path, required: tuple[str, ...]) -> list[dict[str, str]]:
    with path.open(newline="") as handle:
        reader = csv.DictReader(handle)
        missing = [name for name in required if name not in (reader.fieldnames or [])]
        if missing:
            raise ValueError(f"{path} lacks columns {missing}")
        return list(reader)


def _tick_index(timestamp: float, start_epoch: float, tick_s: int) -> int:
    return int((timestamp - start_epoch) // tick_s)


def _recorder_series(
    recorder_csv: Path, start_epoch: float, tick_s: int, ticks: int
) -> tuple[Series, Series]:
    desired = np.full(ticks, np.nan)
    ready = np.full(ticks, np.nan)
    samples = sorted(
        (float(row["epoch_s"]), float(row["desired"]), float(row["ready"]))
        for row in _read_rows(recorder_csv, ("epoch_s", "desired", "ready"))
    )
    for epoch, desired_value, ready_value in samples:
        index = _tick_index(epoch, start_epoch, tick_s)
        if 0 <= index < ticks:
            desired[index] = desired_value
            ready[index] = ready_value
    for index in range(1, ticks):
        if np.isnan(desired[index]):
            desired[index] = desired[index - 1]
            ready[index] = ready[index - 1]
    return desired, ready


def _k6_buckets(
    k6_csv: Path, start_epoch: float, tick_s: int, ticks: int
) -> tuple[list[list[float]], list[list[float]]]:
    buckets: dict[str, list[list[float]]] = {
        "http_req_duration": [[] for _ in range(ticks)],
        "http_req_failed": [[] for _ in range(ticks)],
    }
    for row in _read_rows(k6_csv, ("metric_name", "timestamp", "metric_value")):
        bucket = buckets.get(row["metric_name"])
        index = _tick_index(float(row["timestamp"]), start_epoch, tick_s)
        if bucket is not None and 0 <= index < ticks:
            bucket[index].append(float(row["metric_value"]))
    return buckets["http_req_duration"], buckets["http_req_failed"]
