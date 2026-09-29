# Datasets

Status: **Implemented** (Milestone 5): Python converters for the four sources, the `.ksimtrace` format (Python writer, Rust reader) and `TraceWorkload` replay. Nothing is redistributed by this repository: `/data/` and `*.ksimtrace` are git-ignored and you download the raw traces yourself. Each dataset keeps its own licence; check its terms before use.

## Implemented (Milestone 5)

Converter: `python/ksim/datasets`. Every loader yields a long frame `time_s, service, instance, request_rate, cpu_millis, memory_bytes`; `normalize.to_series` turns it into fixed-tick per-service series.

```sh
python -m ksim.datasets convert --source alibaba2018 --input DIR \
  --output data/alibaba2018-train.ksimtrace --tick 15 --top 50 --split train
```

`--source` is `alibaba2018`, `alibaba2021`, `azure2019` or `borg2019`; `--tick` seconds (default 15), `--top` services (default 50), `--split` `train` (default), `validation` or `test`. Input files are found by name inside `DIR`. Exit code 2 for a missing input, unknown source or an empty window.

### Normalisation

1. Keep samples inside the split window; tick = `floor((time_s - start) / tick)`.
2. Average each instance's samples within a tick, then sum instances per service.
3. Forward fill ticks without samples (0 before the first sample).
4. Keep the top-K services by total CPU (ties by name), in descending CPU order.

### Splits (`normalize.SPLITS`, time from trace start)

| Source | train | validation | test |
|---|---|---|---|
| alibaba2018 | days 0-5 | day 5-6 | days 6-8 |
| alibaba2021 | 0-8 h | 8-10 h | 10-12 h |
| azure2019 | days 0-10 | days 10-12 | days 12-14 |
| borg2019 | days 0-21 | days 21-24 | days 24-31 |

Splits are temporal to avoid leakage; validation and test are never used for tuning or training.

### Column mapping and unit assumptions

Where a trace publishes only normalised units or no request counts, the converter needs an assumption. These are keyword defaults of the loader functions (`python/ksim/datasets/*.py`); the `convert` CLI does not expose flags for them, so change them by calling the loader from Python.

| Source | Files and columns | request_rate | cpu_millis | memory_bytes |
|---|---|---|---|---|
| Alibaba cluster-trace-v2018 | `container_meta.csv` (no header): container_id, machine_id, time_stamp, app_du, status, cpu_request (100 = 1 core), cpu_limit, mem_size (normalised 0-100). `container_usage.csv`: container_id, machine_id, time_stamp (s), cpu_util_percent, mem_util_percent, cpi, mem_gps, mpki, net_in, net_out, disk_io_percent | `cpu_millis / cpu_millis_per_request` (default 50; trace has no request counts) | `cpu_util_percent / 100 * cpu_request * 10` | `mem_util_percent / 100 * mem_size * memory_bytes_per_unit` (default 2^30 bytes per normalised unit) |
| Alibaba cluster-trace-microservices-v2021 | `MSResource*.csv` (header): timestamp (ms, 60 s), msname, msinstanceid, nodeid, cpu_utilization, memory_utilization. `MSRTQps*.csv`: timestamp (ms), msname, msinstanceid, metric, value | sum of `providerRPC_MCR` and `HTTP_MCR` (calls/s) for the same timestamp and instance, else 0 | `cpu_utilization * container_cpu_millis` (default 1000) | `memory_utilization * container_memory_bytes` (default 2^31); container sizes are not published |
| Azure Functions 2019 | per day NN: `invocations_per_function_md.anon.dNN.csv` (HashOwner, HashApp, HashFunction, Trigger, 1..1440), `function_durations_percentiles.anon.dNN.csv` (uses `Average`, ms), `app_memory_percentiles.anon.dNN.csv` (uses `AverageAllocatedMb`; days 13-14 missing) | `invocations / 60` per minute | `request_rate * Average` (one core busy for the mean duration); functions without a duration row are dropped | `AverageAllocatedMb * 2^20` of the function's app, 0 without memory data |
| Google cluster-data 2019 (Borg) | BigQuery `instance_usage` exported to `instance_usage*.csv`: start_time (us), collection_id, instance_index, average_usage_cpus, average_usage_memory (normalised) | `cpu_millis / cpu_millis_per_request` (default 50) | `average_usage_cpus * cpu_millis_per_unit` (default 1000 per normalised unit) | `average_usage_memory * memory_bytes_per_unit` (default 2^30) |

Service key: Alibaba 2018 `app_du`; Alibaba 2021 `msname`; Azure `HashApp:HashFunction` (minute m of day d at `(d-1)*86400 + (m-1)*60` s); Borg `collection_id`. Alibaba 2018 usage rows without meta or with utilisation outside [0, 100] are dropped; each container uses its latest meta row.

### Sources, size and licence

- Alibaba 2018 and 2021: https://github.com/alibaba/clusterdata (`cluster-trace-v2018`, `cluster-trace-microservices-v2021`). The 2018 archive is about 49 GB compressed; the converter reads only the two container files.
- Azure Functions 2019: https://github.com/Azure/AzurePublicDataset (`AzureFunctionsDataset2019`), 14 days at 1-minute resolution.
- Google cluster-data 2019: https://github.com/google/cluster-data; the data lives in BigQuery. Export `instance_usage` to CSV from your own GCP project (costs are yours).
- Licence: this repository grants none for the data. Check each dataset's own terms in its repository before use or redistribution.

### `.ksimtrace` format

All integers and floats little-endian. Writer: `ksim.datasets.ksimtrace`; reader: `ksim_workload::trace::TraceFile`.

| Part | Content |
|---|---|
| header (48 bytes) | magic `KSIMTRC1`, `u32` version (1), `u32` tick seconds (> 0), `u32` service count, `u32` tick count, `u64` names offset (48), `u64` names length, `u64` data offset |
| names | per service `u16` byte length + UTF-8 bytes |
| padding | zeros up to the data offset (next multiple of 4) |
| data | columns `request_rate`, `cpu_millis`, `memory_bytes`, each `f32[service][tick]` |

Validation on open: magic, version, non-zero tick, consistent offsets, UTF-8 names, and file length exactly `data offset + 3 * services * ticks * 4`. The Python encoder and the Rust encoder produce byte-identical files (tested). The file is read into memory rather than memory-mapped, because mmap needs `unsafe` and the workspace forbids `unsafe_code`; a top-50 trace is small.

### Using a trace in a scenario

```toml
[trace]
path = "../data/alibaba2018-train.ksimtrace"  # relative to the scenario file
start_s = 0
compress = 1.0
scale = 1.0
interpolation = "step"   # or "linear"
wrap = true

[[services]]
name = "web"
# ...
curve = { kind = "trace", service = "<a service name in the file>" }
```

Either every service uses a `trace` curve and `[trace]` exists, or neither. See `scenarios/alibaba2018_fixture.toml` and [Simulation Model](Simulation-Model#trace-workload) for the replay semantics.

## Source notes

## Alibaba cluster-trace-v2018

- Source: https://github.com/alibaba/clusterdata, directory `cluster-trace-v2018`. Data is fetched via links in the repository (it may require a short access form).
- Files: `machine_meta`, `machine_usage`, `container_meta`, `container_usage`, `batch_task`, `batch_instance`.
- Key columns: container_meta (`container_id`, `machine_id`, `time_stamp`, `app_du`, `status`, `cpu_request`, `cpu_limit`, `mem_size`); container_usage (`container_id`, `time_stamp`, `cpu_util_percent`, `mem_util_percent`, `cpi`, `mem_gps`, `mpki`, `net_in`, `net_out`, `disk_io_percent`).
- Size/duration: 8 days, about 4,000 machines; the uncompressed data is very large (hundreds of GB), so the normalizer streams and can work on a downloaded subset.
- License: see the repository `LICENSE` (Creative Commons style attribution license as stated by the repository).
- Mapping: group containers by `app_du` to a service; demand = sum of `cpu_util_percent` x `cpu_request`; batch tasks become background load and node pressure.
- Split: see the table above.

## Alibaba cluster-trace-microservices-v2021 / v2022

- Source: https://github.com/alibaba/clusterdata, directories `cluster-trace-microservices-v2021` and `cluster-trace-microservices-v2022`.
- Files: `MSMetrics` / `MSRTMCR` (per-microservice call rate and response time, CPU/memory), `MSCallGraph` (call graph), `NodeMetrics`.
- Key columns: timestamp, microservice id, RPC/HTTP call rate, response time, CPU and memory utilization; call graph edges (upstream, downstream, rpc type). Exact column names follow the repository schema documents.
- Size/duration: v2021 covers a window of hours, v2022 a longer multi-day window; check the repository README for current figures.
- License: see repository `LICENSE`.
- Mapping: microservice becomes a request-rate-driven service with response time; the call graph gives dependency fan-out (v2 of the simulator).
- Split: chronological 60 % / 20 % / 20 % of the available window (train / validation / test).
- Analysis reference: Luo et al., SoCC 2021.

## Google cluster-data 2019 (Borg)

- Source: https://github.com/google/cluster-data (ClusterData2019); data is in Google BigQuery, not downloadable files.
- Tables: `instance_usage`, `instance_events`, `collection_events`, `machine_events`, `machine_attributes`.
- Key columns: usage windows (`start_time`, `end_time`, `collection_id`, `instance_index`, `machine_id`, `average_usage` cpus/memory, `maximum_usage`, `random_sample_usage`, `cpu_usage_distribution`), resource requests and priority in events tables; CPU and memory are normalized (NCU / NMU).
- Size/duration: eight cells, one month (May 2019), 5-minute usage windows; very large, so the export is filtered per cell and time range in BigQuery.
- Access and cost: requires the user's own Google Cloud project and credentials (`google-cloud-bigquery`, optional extra); query costs are the user's. Terms and license are stated by the repository and the trace paper.
- Mapping: collections become services; 5-minute CPU/memory usage as demand; priority tiers for eviction behavior.
- Split: days 1-20 train, 21-24 validation, 25-31 test, applied within a chosen cell.
- Reference: Tirmazi et al., EuroSys 2020.

## Azure Functions 2019

- Source: https://github.com/Azure/AzurePublicDataset, `AzureFunctionsDataset2019`.
- Files: `invocations_per_function_md.anon.dNN.csv` (per-minute invocation counts, columns `HashOwner`, `HashApp`, `HashFunction`, `Trigger`, `1`..`1440`), `function_durations_percentiles.anon.dNN.csv` (average, count, min, max, percentiles), `app_memory_percentiles.anon.dNN.csv`.
- Size/duration: 14 days (July 2019), one file per day, 1-minute resolution.
- License: see repository `LICENSE` (attribution license as stated by the repository).
- Mapping: function becomes a service; invocations per minute become bursty RPS; per-request cost from duration; memory from app percentiles.
- Split: days 1-10 train, 11-12 validation, 13-14 test.
- Reference: Shahrad et al., USENIX ATC 2020.
