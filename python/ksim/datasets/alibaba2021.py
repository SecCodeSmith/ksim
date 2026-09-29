"""Alibaba cluster-trace-microservices-v2021 (github.com/alibaba/clusterdata).

`MSResource` (header): timestamp (ms, 60 s interval), msname, msinstanceid, nodeid,
cpu_utilization, memory_utilization. `MSRTQps` (header): timestamp (ms), msname,
msinstanceid, metric, value; `providerRPC_MCR` and `HTTP_MCR` are calls per second.
"""

from pathlib import Path

import polars as pl

RATE_METRICS = ("providerRPC_MCR", "HTTP_MCR")


def load(
    ms_resource: list[Path],
    ms_rt_qps: list[Path],
    *,
    container_cpu_millis: float = 1000.0,
    container_memory_bytes: float = float(2**31),
) -> pl.DataFrame:
    """Long frame (see `normalize`): service = msname, instance = msinstanceid,
    `time_s = timestamp / 1000`, `cpu_millis = cpu_utilization * container_cpu_millis`,
    `memory_bytes = memory_utilization * container_memory_bytes` (container sizes are not
    published, so both are assumptions), `request_rate` = sum of RATE_METRICS values for the
    same (timestamp, instance), 0 when absent. Rows are the outer join of both tables on
    (timestamp, msname, msinstanceid)."""
    raise NotImplementedError
