# Datasets

Status: **Planned** (Milestone 5). Order of implementation: Alibaba 2018, Alibaba 2021/2022, Azure 2019, Borg 2019. Nothing is redistributed by this repository: downloaders fetch from the official sources, and each dataset keeps its own license. Verify license terms in the source repository before use.

## Normalization to `.ksimtrace`

An offline Python/polars pipeline (`python/ksim/datasets`) converts each dataset into one binary format: a header plus columnar f32/u32 arrays holding per-service time series at a fixed resolution. Rust mmaps it (`memmap2`), so replay is zero-copy. Arrow IPC is the intermediate. Replay supports service selection (top-K by volume or seeded random sample), time windows, time compression and amplitude scaling.

Splits are temporal to avoid leakage; validation and test windows are never used for tuning or training.

## Alibaba cluster-trace-v2018

- Source: https://github.com/alibaba/clusterdata, directory `cluster-trace-v2018`. Data is fetched via links in the repository (it may require a short access form).
- Files: `machine_meta`, `machine_usage`, `container_meta`, `container_usage`, `batch_task`, `batch_instance`.
- Key columns: container_meta (`container_id`, `machine_id`, `time_stamp`, `app_du`, `status`, `cpu_request`, `cpu_limit`, `mem_size`); container_usage (`container_id`, `time_stamp`, `cpu_util_percent`, `mem_util_percent`, `cpi`, `mem_gps`, `mpki`, `net_in`, `net_out`, `disk_io_percent`).
- Size/duration: 8 days, about 4,000 machines; the uncompressed data is very large (hundreds of GB), so the normalizer streams and can work on a downloaded subset.
- License: see the repository `LICENSE` (Creative Commons style attribution license as stated by the repository).
- Mapping: group containers by `app_du` to a service; demand = sum of `cpu_util_percent` x `cpu_request`; batch tasks become background load and node pressure.
- Split: days 1-5 train, day 6 validation, days 7-8 test.

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
