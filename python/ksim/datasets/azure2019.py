"""Azure Functions 2019 (github.com/Azure/AzurePublicDataset, AzureFunctionsDataset2019.md).

Per day dNN: `invocations_per_function_md.anon.dNN.csv` (HashOwner, HashApp, HashFunction,
Trigger, 1..1440 invocations per minute), `function_durations_percentiles.anon.dNN.csv`
(HashOwner, HashApp, HashFunction, Average ms, Count, ...), `app_memory_percentiles.anon.dNN.csv`
(HashOwner, HashApp, SampleCount, AverageAllocatedMb, ...; days 13-14 are missing).
"""

from pathlib import Path

import polars as pl

DAY_SECONDS = 86_400

_HASH_TYPES: dict[str, pl.DataType | type[pl.DataType]] = {
    "HashOwner": pl.String,
    "HashApp": pl.String,
    "HashFunction": pl.String,
}


def load_day(
    day: int,
    invocations: Path,
    durations: Path,
    memory: Path | None,
) -> pl.DataFrame:
    """Long frame (see `normalize`) for day `day` (1-based): service = instance =
    `HashApp:HashFunction`, minute m (1..1440) at `time_s = (day - 1) * 86400 + (m - 1) * 60`,
    `request_rate = invocations / 60`, `cpu_millis = request_rate * Average` (one core busy
    for the mean duration per invocation), `memory_bytes = AverageAllocatedMb * 2**20` of the
    function's app, 0 without memory data. Functions without a duration row are dropped."""
    function_key = ["HashApp", "HashFunction"]
    invocation_frame = pl.scan_csv(invocations, schema_overrides=_HASH_TYPES)
    minute_columns = [name for name in invocation_frame.collect_schema().names() if name.isdigit()]
    per_minute = invocation_frame.unpivot(
        on=minute_columns,
        index=function_key,
        variable_name="minute",
        value_name="invocations",
    )
    mean_durations = pl.scan_csv(durations, schema_overrides=_HASH_TYPES).select(
        *function_key, pl.col("Average").cast(pl.Float64).alias("duration_ms")
    )
    joined = per_minute.join(mean_durations.unique(function_key), on=function_key, how="inner")
    joined = _with_app_memory(joined, memory)
    request_rate = pl.col("invocations").cast(pl.Float64) / 60
    return joined.select(
        ((day - 1) * DAY_SECONDS + (pl.col("minute").cast(pl.Int64) - 1) * 60)
        .cast(pl.Float64)
        .alias("time_s"),
        pl.concat_str("HashApp", "HashFunction", separator=":").alias("service"),
        pl.concat_str("HashApp", "HashFunction", separator=":").alias("instance"),
        request_rate.alias("request_rate"),
        (request_rate * pl.col("duration_ms")).alias("cpu_millis"),
        pl.col("memory_bytes"),
    ).collect()


def _with_app_memory(functions: pl.LazyFrame, memory: Path | None) -> pl.LazyFrame:
    if memory is None:
        return functions.with_columns(pl.lit(0.0).alias("memory_bytes"))
    app_memory = (
        pl.scan_csv(memory, schema_overrides=_HASH_TYPES)
        .select(
            "HashApp", (pl.col("AverageAllocatedMb").cast(pl.Float64) * 2**20).alias("memory_bytes")
        )
        .unique("HashApp")
    )
    return functions.join(app_memory, on="HashApp", how="left").with_columns(
        pl.col("memory_bytes").fill_null(0.0)
    )
