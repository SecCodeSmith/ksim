"""Google cluster-data 2019 (Borg), exported from BigQuery `instance_usage` to CSV.

Expected columns: start_time (µs), collection_id, instance_index, average_usage_cpus and
average_usage_memory (both normalised units). Exporting needs the user's GCP project.
"""

from pathlib import Path

import polars as pl


def load(
    instance_usage: list[Path],
    *,
    cpu_millis_per_unit: float = 1000.0,
    memory_bytes_per_unit: float = float(2**30),
    cpu_millis_per_request: float = 50.0,
) -> pl.DataFrame:
    """Long frame (see `normalize`): service = collection_id, instance = instance_index,
    `time_s = start_time / 1e6`, `cpu_millis = average_usage_cpus * cpu_millis_per_unit`,
    `memory_bytes = average_usage_memory * memory_bytes_per_unit`,
    `request_rate = cpu_millis / cpu_millis_per_request`. The unit scales are assumptions
    because the trace normalises resources."""
    cpu_millis = pl.col("average_usage_cpus") * cpu_millis_per_unit
    return (
        pl.scan_csv(
            instance_usage,
            schema_overrides={"collection_id": pl.String, "instance_index": pl.String},
        )
        .select(
            (pl.col("start_time") / 1e6).alias("time_s"),
            pl.col("collection_id").alias("service"),
            pl.col("instance_index").alias("instance"),
            (cpu_millis / cpu_millis_per_request).alias("request_rate"),
            cpu_millis.alias("cpu_millis"),
            (pl.col("average_usage_memory") * memory_bytes_per_unit).alias("memory_bytes"),
        )
        .collect()
    )
