# Architecture

## Crates (Implemented as skeleton; behavior Planned)

| Crate | Responsibility | Status |
|---|---|---|
| `ksim-core` | cluster state machine, clock, event queue, scheduler | skeleton |
| `ksim-workload` | `.ksimtrace` format, mmap reader, replay, time-warp/scale | skeleton |
| `ksim-perf` | performance model: latency, throttling, OOM | skeleton |
| `ksim-autoscalers` | `Autoscaler` trait, HPA, VPA, CA baselines | skeleton |
| `ksim-metrics` | benchmark metrics and reports | skeleton |
| `ksim-bench` | CLI: scenarios (TOML) to results (JSON/CSV) | `--version` only |
| `ksim-py` | PyO3 bindings, Gymnasium and vector env | `native_version()` only |
| `ksim-capi` | C ABI, header `ksim.h` via cbindgen | `ksim_version()` only |

Python lives in `python/ksim/` (datasets, gym wrappers, analysis) and stays out of the step loop.

## Data flow (Planned)

```
trace (Alibaba / Borg / Azure)
  -> python/ksim/datasets (polars) -> .ksimtrace (columnar, mmap)
  -> ksim-workload (TraceSource) -> demand per service per tick
  -> ksim-perf (PerformanceModel) -> usage, latency, errors
  -> Observation -> Autoscaler (baseline or RL agent) -> Actions
  -> ksim-core applies actions (scheduler, events) -> next tick
  -> ksim-metrics (Metric) -> report
```

## Traits and SOLID boundaries

- `Autoscaler::act(&mut self, obs: &Observation, now: SimTime) -> Actions`: baselines and RL agents share this path.
- `TraceSource`, `PerformanceModel`, `RewardFn`, `Metric`, `SchedulerPlugin`: extension points.
- Small interfaces: `ReplicaRecommender` (HPA), `ResourceRecommender` (VPA), `NodeProvisioner` (CA).
- The engine depends only on traits, injected through `SimulationBuilder`; tests inject fakes.
- Every implementation of a trait must pass a shared contract test suite (e.g. `autoscaler_contract_tests!`), which enforces Liskov substitutability.

The workspace forbids `unsafe_code` by default and enables selected `clippy::pedantic` lints.
