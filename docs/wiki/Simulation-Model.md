# Simulation model

Status: **Implemented** (Milestone 2): core engine, scheduler, lifecycle, scrape pipeline, synthetic workloads, queueing and utilisation performance models. `TraceWorkload` trace replay: Implemented (Milestone 5).

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

## Trace workload

`TraceWorkload` (`ksim-workload::trace`) replays a `.ksimtrace` (see [Datasets](Datasets)) as a pure function of time. Trace time is `start + now * compress`; the sample position is `trace time / tick`.

- Interpolation: `step` uses the tick at `floor(position)`; `linear` blends it with the next tick by the fractional part.
- Past the last tick: `wrap` cycles back to tick 0 (the last tick blends toward tick 0 when linear); otherwise the last value is held.
- `scale` multiplies request rate and CPU only. Memory is a footprint and is not scaled.
- Services are selected and ordered by name from the scenario's `kind = "trace"` curves; unknown names, or non-positive or non-finite `compress` or `scale`, are rejected at load.

## Performance baseline

`cargo bench -p ksim-core --bench step` (criterion): 50 services on 20 nodes with changing replica actions each tick and trivial fake workload/performance models, so it measures the engine only. Measured over 20 consecutive steps from a fresh simulation.

| Date | Machine | Time per step | Steps/s |
|---|---|---|---|
| 2026-09-29 | CI container, single core | 1.29 ms | ~775 (before indexing) |
| 2026-09-29 | CI container, single core | 48 µs | ~20.9k (heavy churn: every service flips 6↔3 replicas each step) |

Release-mode gate (`cargo test --release -p ksim-core --test perf -- --ignored`, run by the `perf` workflow): 50 services on 20 nodes, replicas cycling 3–8, measured after 20,000 warm-up steps; must stay at or above 20,000 steps/s. Measured 2026-09-29: ~355,000 steps/s.

Per-step cost is proportional to live pods and nodes: pods are stored densely behind an id-to-slot table (ids stay stable and are never reused, so stale events resolve to `None`), with sorted per-service and per-node pod lists, cached per-node requested totals, per-service/phase and per-pool counters and a pending set; the observation and scrape buffers are reused across ticks.

## Actions added in Milestone 3

- `drain`: the node becomes `Draining` (unschedulable), its bound non-terminating pods are evicted (`Terminating`, removed after the termination latency) and recreated elsewhere by reconciliation; the node is removed after due events once nothing is bound. A drain is ignored unless the pool's ready plus provisioning nodes exceed `min_nodes`.
- `evict[s]`: up to `n` outdated pods of service `s` (requests differing from the service's current requests), oldest first, ties to the lower id.
- Every step reconciles each service to its desired count of non-terminating pods.
- `Observation.nodes` / `node_pods` expose each node's pool, phase, allocatable, requested (bound non-terminating pods) and pod placements; `ServiceObservation.outdated` and `limits`; `PoolObservation` bounds and draining count.

## Assumptions and limitations

- Homogeneous pods within a service; no network or noisy-neighbor model; steady-state queueing per tick rather than transient queue dynamics.
- Approximations are validated against a real `kind` cluster in Milestone 7 (Planned).
