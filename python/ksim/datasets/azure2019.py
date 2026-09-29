"""Azure Functions 2019 (github.com/Azure/AzurePublicDataset, AzureFunctionsDataset2019.md).

Per day dNN: `invocations_per_function_md.anon.dNN.csv` (HashOwner, HashApp, HashFunction,
Trigger, 1..1440 invocations per minute), `function_durations_percentiles.anon.dNN.csv`
(HashOwner, HashApp, HashFunction, Average ms, Count, ...), `app_memory_percentiles.anon.dNN.csv`
(HashOwner, HashApp, SampleCount, AverageAllocatedMb, ...; days 13-14 are missing).
"""

from pathlib import Path

import polars as pl


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
    raise NotImplementedError
