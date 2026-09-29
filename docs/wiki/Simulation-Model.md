# Simulation model

Status: **Implemented** (Milestone 2): core engine, scheduler, lifecycle, scrape pipeline, synthetic workloads, queueing and utilisation performance models. Trace replay is Milestone 3 (Planned).

## Time model

A fixed control tick (default 15 s, the HPA sync period) plus a binary-heap event queue for delayed events (pod running, node ready, pod removal). Deterministic: randomness comes from seeded `rand_chacha` streams, one per concern (`Concern::Lifecycle` for latency sampling). Ids are never reused.

## Step semantics

`Simulation::step(&Actions) -> &Observation` runs, per tick, in this order:

1. Apply actions. Requests first, so pods created by the same step get the new requests (existing pods keep theirs); then replica changes (create pods or delete victims); then node pool deltas.
2. Advance the clock by one tick and apply all events due at or before the new time (`NodeReady`, `PodRunning`, `PodRemoved`).
3. Schedule pending pods; each bound pod becomes `Starting` and a `PodRunning` event is queued after the sampled startup latency.
4. Evaluate the `PerformanceModel` per service from ready (`Running`) pods, the service requests/limits and the workload demand at the new time. If a service reports `oom`, every `Running` pod of it is restarted: restart count incremented, phase back to `Starting`, running again after the kubelet crash-loop backoff (10 s, doubling per restart, capped at 300 s).
5. Scrape: signals are snapshotted only when `now` is a multiple of `ScrapeConfig::resolution` (default 15 s); consumers see the snapshot `lag_scrapes` scrapes behind the newest (default 1). Until enough scrapes exist the signals are zeros.
6. Publish the observation.

Building starts warm: `initial_nodes` are `Ready` and `initial_replicas` pods are `Running` at time zero.

### Actions

`replicas`, `requests` (per service) and `pool_delta` (per pool) are indexed by id; missing entries mean no change. Positive pool deltas provision nodes (up to `max_nodes`, ready after the provisioning latency); negative deltas remove only empty `Ready` nodes and never go below `min_nodes`. Scale-down victims follow ReplicaSet `ActivePodsWithRanks`: unbound pods, then not-ready pods, then `Running` pods newest first. Deleted bound pods are `Terminating` until the termination latency elapses.

### Observation

Counts are the current API-server view; `signals` are the lagged scrape.

| Field | Meaning |
|---|---|
| `desired` | last requested replica count |
| `ready` | `Running` pods |
| `pending` | not-ready pods: `Pending` plus `Starting` |
| `unschedulable` | pods the scheduler could not place |
| `requests` | current per-pod requests of the service |
| `signals` | CPU, memory, served RPS, p50/p95/p99 latency, error rate, throttled fraction, `oom` |
| pool `ready`, `provisioning` | nodes in each phase |

## Scheduler

Simplified kube-scheduler: filter `NodeResourcesFit` on requests, score `LeastAllocated` (default) or `MostAllocated` via `ScoringPolicy`. Pods that fit nowhere are flagged `unschedulable`; this is the signal the Cluster Autoscaler consumes.

## Lifecycle latencies

`LatencyModel` (`DefaultLatencies`, `FixedLatencies`): pod startup = image pull + uniform(1 s, 5 s) (SIG-scalability SLO, p99 <= 5 s excluding image pull), node provisioning uniform(60 s, 180 s), termination 30 s.

## Performance models

### Queueing (`QueueingModel`, request-driven services: Alibaba 2021/22, Azure)

Each ready pod is a server with `mu = limits.cpu_millis / cpu_millis_per_request` requests per second. With `c` pods, arrival rate `lambda`, offered load `a = lambda/mu`:

```
Erlang C:  C(c, a) = [ a^c / c! * 1/(1-rho) ] / [ sum_{k=0}^{c-1} a^k/k! + a^c / c! * 1/(1-rho) ],  rho = a/c
P(Wq > t) = C * exp(-(c*mu - lambda) * t)                       (M/M/c)
Allen-Cunneen: waits scaled by k = (ca2 + cs2) / 2
```

- Latency quantile = scaled wait quantile + `1/mu`, each capped at `timeout_ms`.
- Admission: `admitted = min(lambda, 0.99 * c * mu)`; the rest is dropped and counted as errors. `served_rps` is the admitted rate.
- Timeout rule: of the admitted requests, the timed-out share is `C * exp(-(c*mu - lambda) * (timeout - 1/mu) / k)` and also counts as errors.
- No ready pods: every request errors and all latencies equal `timeout_ms`.
- `QueueingModel::mmc` sets `ca2 = cs2 = 1`; `allen_cunneen` takes both.

### Utilisation (`UtilizationModel`, Alibaba 2018 / Borg)

CPU usage is capped at `ready_pods * limits.cpu_millis`; the excess fraction is CFS-throttled and inflates every latency quantile of `base_latency_ms` by `demand / capacity`. Per-pod memory above `limits.memory_bytes` sets `oom`. No ready pods with non-zero demand: `error_rate` is 1.

## Synthetic workloads (`ksim-workload::synthetic`)

`CurveWorkload` holds one `(DemandCurve, DemandProfile)` per service. Curves give a request rate in rps as a pure function of time:

| Curve | Formula |
|---|---|
| `Constant` | `rate` |
| `Step` | `before` for `t < at`, else `after` |
| `Ramp` | linear `from` at `start` to `to` at `end`, flat outside |
| `Sinusoid` | `max(0, mean + amplitude * sin(2 pi t / period) + N(0, noise_sd))`, noise derived from `(seed, t)` |
| `FlashCrowd` | `base` before `start`; then `base + (peak - base) * exp(-(t - start) / decay)` |

`DemandProfile` maps rate to demand: `cpu = rate * cpu_millis_per_request`, `memory = memory_base_bytes + rate * memory_bytes_per_rps`.

## Performance baseline

`cargo bench -p ksim-core --bench step` (criterion): 50 services on 20 nodes with changing replica actions each tick and trivial fake workload/performance models, so it measures the engine only. Measured over 20 consecutive steps from a fresh simulation.

| Date | Machine | Time per step | Steps/s |
|---|---|---|---|
| 2026-09-29 | CI container, single core | 1.29 ms | ~775 |

Known cost driver: pod and node ids are never reused and the cluster scans all slots ever created, so per-step time grows with history (a 1000-step run in one simulation measured well above the baseline). Slot reuse or per-service indexes are a future optimisation.

## Assumptions and limitations

- Homogeneous pods within a service; no network or noisy-neighbor model; steady-state queueing per tick rather than transient queue dynamics.
- Approximations are validated against a real `kind` cluster in Milestone 7 (Planned).
