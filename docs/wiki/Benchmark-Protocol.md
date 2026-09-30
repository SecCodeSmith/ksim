# Benchmark protocol

Status: **Implemented** (Milestone 4: harness, synthetic scenarios, policies, analysis). Trace-driven scenarios: **Implemented** (Milestone 5).

## Scenario TOML

Files in `scenarios/` (`synthetic_steady`, `synthetic_diurnal`, `synthetic_step`, `synthetic_flash_crowd`, and the trace-driven `alibaba2018_fixture` using `curve = { kind = "trace", service = "..." }`). Unknown fields are rejected. Durations are in seconds. A scenario needs at least one pool and one service, `tick_s > 0` and `duration_s >= tick_s`; the episode has `duration_s / tick_s` steps.

| Table | Fields |
|---|---|
| top level | `name`, `duration_s`, `tick_s` |
| `[slo]` | `latency_ms`, `percentile` (`"P95"` or `"P99"`), `max_error_rate`, `availability_target` |
| `[[pools]]` | `name`, `cpu_millis`, `memory_mib`, `min_nodes`, `max_nodes`, `initial_nodes`, `price_per_hour` |
| `[[services]]` | `name`, `requests_cpu_millis`, `requests_memory_mib`, `limits_cpu_millis`, `limits_memory_mib`, `initial_replicas`, `cpu_millis_per_request`, `memory_base_mib`, `memory_mib_per_rps` |
| `[trace]` | `path` (relative to the scenario file), `start_s` (0), `compress` (1), `scale` (1), `interpolation` (`"step"` or `"linear"`), `wrap` (false); see [Datasets](Datasets) |
| `[services.curve]` | `kind` plus the fields below |
| `[services.performance]` | `kind` plus the fields below |
| `[services.hpa]` | `min_replicas`, `max_replicas`, `target_cpu_utilization` |

Curve kinds (`ksim_workload::synthetic`, rates in requests/s):

- `constant`: `rate`
- `step`: `before`, `after`, `at_s`
- `ramp`: `from`, `to`, `start_s`, `end_s`
- `sinusoid`: `mean`, `amplitude`, `period_s`, `noise_sd`
- `flash_crowd`: `base`, `peak`, `start_s`, `decay_s`

Performance kinds (per-service models, so one scenario can mix them):

- `queueing`: `cpu_millis_per_request`, `timeout_ms`, `ca2`, `cs2` (Allen-Cunneen G/G/c)
- `utilization`: `base_latency_ms`

## Policies

`ksim-bench list-policies` prints them.

| Policy | Composition |
|---|---|
| `hpa` | HPA only, so it cannot add nodes and pods stay pending when the cluster is full |
| `vpa` | VPA only |
| `ca` | Cluster Autoscaler only (expander least-waste) |
| `k8s` | HPA + Cluster Autoscaler, the baseline stack |
| `k8s-full` | HPA + VPA + Cluster Autoscaler; upstream warns that VPA interferes with a CPU-based HPA |
| `static-peak` | fixed replicas sized for each service's peak CPU demand over the episode |
| `static-average` | fixed replicas sized for the mean CPU demand |
| `oracle` | clairvoyant: sizes each service for the highest CPU demand in `[now, now + lookahead]` |

Reference policies (`static-*`, `oracle`) only choose replicas, using `max(1, ceil(demand / (request * target)))` with the HPA target utilisation, and run together with the Cluster Autoscaler so their pods get nodes; the comparison is then about replica decisions alone. The oracle lookahead is **2 ticks + 5 s**: an action taken at `t` is applied at `t + tick`, the pod starts after its startup latency (at most 5 s by default) and is first observed at `t + 2 * tick`, so a smaller lookahead would size for demand that has already changed.

## Seeds and determinism

`--seeds N` runs seeds `first_seed .. first_seed + N` (default first seed 0, default N 10). The seed drives the workload noise, the latency model and the Cluster Autoscaler; the same scenario, policy and seed give identical metrics. Synthetic scenarios have little seed variance (latencies are the only randomness for noise-free curves), so their confidence intervals are narrow; trace scenarios carry real variability.

## Commands

```sh
ksim-bench list-policies
ksim-bench run --scenario scenarios/synthetic_step.toml --policy k8s --seeds 8 --out results/
python -m ksim.analysis results --baseline k8s --metrics availability,slo_compliance,cost
```

`run` writes `<scenario>-<policy>.jsonl` (one report per seed) and `.csv` (`scenario,policy,seed,<metrics sorted by name>`) into `--out` (default `results`). `ksim.analysis` reads every `*.jsonl`, needs the baseline policy in each scenario and defaults to all metrics.

## Statistics

- Mean per metric with a **percentile bootstrap 95 % CI**: 10 000 resamples of the per-seed values, generator seed 0, 2.5th and 97.5th percentiles of the resampled means (Efron and Tibshirani, 1993).
- Paired **Wilcoxon signed-rank** test (two-sided, `scipy.stats.wilcoxon`) against the baseline on the seeds both ran; a cell gets `*` when p < 0.05. No multiple-comparison correction is applied yet.
- **Normalised score** `x<n>` = policy mean / baseline mean (1 when both are 0, infinity when only the baseline is 0). It is not direction-aware; read it with the metric's direction (lower is better for cost, timeshares and accuracies).
- Cells render as `mean [low, high] x<normalised>` with 4 significant digits.

## Example leaderboard

Measured on `synthetic_step`, 8 seeds, 2026-09-29 (means):

| policy | availability | slo_compliance | under-prov. timeshare % | over-prov. accuracy % | cost | scaling actions |
|---|---|---|---|---|---|---|
| hpa | 0.9999 | 0.8316 | 17.29 | 32.62 | 6.80 | 15 |
| k8s | 0.9999 | 0.9946 | 1.30 | 42.26 | 10.06 | 22 |
| oracle | 1.0000 | 0.9976 | 0.43 | 50.69 | 10.32 | 13 |
| static-average | 1.0000 | 0.9997 | 0.14 | 91.58 | 7.80 | 4 |
| static-peak | 1.0000 | 0.9997 | 0.14 | 158.8 | 10.79 | 7 |

- `hpa` alone cannot add nodes: pods wait for capacity after the step, so it violates the SLO in about 17 % of samples but is the cheapest.
- `static-peak` buys availability with heavy over-provisioning (159 % accuracy gap, highest cost).
- `oracle` is the upper reference for a replica policy: it needs the fewest actions and near-zero under-provisioning at a cost close to `k8s`.

## Submitting a result

Open a pull request adding the result JSON, the exact commit hash of ksim, the policy code or a link, and the seed list. Results must be reproducible from those inputs.
