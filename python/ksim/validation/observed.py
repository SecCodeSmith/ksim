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

from pathlib import Path

from ksim.validation.trajectory import Trajectory


def load_observed(
    recorder_csv: Path, k6_csv: Path, start_epoch: float, tick_s: int, ticks: int
) -> Trajectory:
    """`ticks` ticks starting at `start_epoch`. Raises ValueError for a non-positive tick or
    tick count, or a CSV missing its required columns."""
    raise NotImplementedError
