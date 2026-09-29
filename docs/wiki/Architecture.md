# Architecture

## Crates

| Crate | Responsibility | Status |
|---|---|---|
| `ksim-core` | cluster state machine, clock, event queue, scheduler, lifecycle, scrape pipeline; owns the extension traits | Implemented (Milestone 2) |
| `ksim-workload` | synthetic demand curves (Implemented, Milestone 2); `.ksimtrace` format, mmap reader, replay, time-warp/scale (Planned) | partial |
| `ksim-perf` | queueing (Erlang C, Allen-Cunneen) and utilisation performance models | Implemented (Milestone 2) |
| `ksim-autoscalers` | HPA, VPA, Cluster Autoscaler baselines, `KubernetesStack`, small ISP traits (the `Autoscaler` trait itself lives in `ksim-core`) | Implemented (Milestone 3) |
| `ksim-metrics` | benchmark metrics (availability, SLO, SPEC RG elasticity, cost, operational) and reports | Implemented (Milestone 4) |
| `ksim-bench` | CLI: `run` (scenario TOML x policy x seeds to JSONL/CSV), `list-policies` | Implemented (Milestone 4) |
| `ksim-py` | PyO3 bindings, Gymnasium and vector env | `native_version()` only |
| `ksim-capi` | C ABI, header `ksim.h` via cbindgen | `ksim_version()` only |

Python lives in `python/ksim/` (datasets, gym wrappers, `ksim.analysis` leaderboard and statistics, Implemented in Milestone 4) and stays out of the step loop.

## Dependency direction

`ksim-core` owns the traits (`WorkloadSource`, `PerformanceModel`, `LatencyModel`, `ScoringPolicy`) and depends on no other workspace crate. `ksim-workload` and `ksim-perf` depend on `ksim-core` and implement its traits; they never depend on each other. Core tests and benches use in-file fakes.

## Data flow (trace replay Planned)

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

- `Autoscaler::act(&mut self, observation: &Observation) -> Actions` (in `ksim_core::autoscaler`): baselines and RL agents share this path; the observation carries `now`.
- `WorkloadSource` (a `TraceSource`), `PerformanceModel`, `RewardFn`, `Metric`, `SchedulerPlugin`: extension points.
- Small interfaces (interface segregation, in `ksim-autoscalers::traits`): `ReplicaRecommender` (HPA), `ResourceRecommender` (VPA), `NodeProvisioner` (CA). `KubernetesStack` composes any of them into one `Autoscaler`.
- The engine depends only on traits, injected through `SimulationBuilder`; tests inject fakes.
- Contract test macros live in `ksim-core` behind the `contract-tests` feature (e.g. `workload_source_contract_tests!`).
- Every implementation of a trait must pass a shared contract test suite (e.g. `autoscaler_contract_tests!`), which enforces Liskov substitutability.

The workspace forbids `unsafe_code` by default and enables selected `clippy::pedantic` lints.
