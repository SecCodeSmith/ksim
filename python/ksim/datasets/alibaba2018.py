"""Alibaba cluster-trace-v2018 (github.com/alibaba/clusterdata, cluster-trace-v2018/schema.txt).

CSV files have no header. `container_meta`: container_id, machine_id, time_stamp, app_du,
status, cpu_request (100 = 1 core), cpu_limit, mem_size (normalised [0, 100]).
`container_usage`: container_id, machine_id, time_stamp (s), cpu_util_percent,
mem_util_percent, cpi, mem_gps, mpki, net_in, net_out, disk_io_percent.
"""

from pathlib import Path

import polars as pl

LONG_COLUMNS = ("time_s", "service", "instance", "request_rate", "cpu_millis", "memory_bytes")

META_SCHEMA: dict[str, pl.DataType | type[pl.DataType]] = {
    "container_id": pl.String,
    "machine_id": pl.String,
    "time_stamp": pl.Float64,
    "app_du": pl.String,
    "status": pl.String,
    "cpu_request": pl.Float64,
    "cpu_limit": pl.Float64,
    "mem_size": pl.Float64,
}

USAGE_SCHEMA: dict[str, pl.DataType | type[pl.DataType]] = {
    "container_id": pl.String,
    "machine_id": pl.String,
    "time_stamp": pl.Float64,
    "cpu_util_percent": pl.Float64,
    "mem_util_percent": pl.Float64,
    "cpi": pl.Float64,
    "mem_gps": pl.Float64,
    "mpki": pl.Float64,
    "net_in": pl.Float64,
    "net_out": pl.Float64,
    "disk_io_percent": pl.Float64,
}


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
    latest_meta = (
        pl.scan_csv(container_meta, has_header=False, schema=META_SCHEMA)
        .sort("time_stamp")
        .group_by("container_id")
        .agg(pl.col("app_du", "cpu_request", "mem_size").last())
    )
    usage = pl.scan_csv(container_usage, has_header=False, schema=USAGE_SCHEMA).filter(
        pl.col("cpu_util_percent").is_between(0, 100)
        & pl.col("mem_util_percent").is_between(0, 100)
    )
    cpu_millis = pl.col("cpu_util_percent") / 100 * pl.col("cpu_request") * 10
    return (
        usage.join(latest_meta, on="container_id", how="inner")
        .select(
            pl.col("time_stamp").alias("time_s"),
            pl.col("app_du").alias("service"),
            pl.col("container_id").alias("instance"),
            cpu_millis.alias("cpu_millis"),
            (pl.col("mem_util_percent") / 100 * pl.col("mem_size") * memory_bytes_per_unit).alias(
                "memory_bytes"
            ),
        )
        .with_columns((pl.col("cpu_millis") / cpu_millis_per_request).alias("request_rate"))
        .select(*LONG_COLUMNS)
        .collect()
    )
