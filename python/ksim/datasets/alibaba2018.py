"""Alibaba cluster-trace-v2018 (github.com/alibaba/clusterdata, cluster-trace-v2018/schema.txt).

CSV files have no header. `container_meta`: container_id, machine_id, time_stamp, app_du,
status, cpu_request (100 = 1 core), cpu_limit, mem_size (normalised [0, 100]).
`container_usage`: container_id, machine_id, time_stamp (s), cpu_util_percent,
mem_util_percent, cpi, mem_gps, mpki, net_in, net_out, disk_io_percent.
"""

from pathlib import Path

import polars as pl


def load(
    container_meta: Path,
    container_usage: Path,
    *,
    memory_bytes_per_unit: float = float(2**30),
    cpu_millis_per_request: float = 50.0,
) -> pl.DataFrame:
    """Long frame (see `normalize`): service = app_du, instance = container_id,
    `cpu_millis = cpu_util_percent / 100 * cpu_request * 10` (utilisation of the requested
    cores), `memory_bytes = mem_util_percent / 100 * mem_size * memory_bytes_per_unit`
    (mem_size is normalised, so the byte scale is an assumption), and
    `request_rate = cpu_millis / cpu_millis_per_request` because the trace has no request
    counts. Each container uses its latest meta row; usage rows without meta, or with
    utilisation outside [0, 100], are dropped."""
    raise NotImplementedError
