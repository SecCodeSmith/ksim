# ksim

ksim is a lightweight, deterministic, trace-driven simulator of a Kubernetes cluster for training and benchmarking autoscalers. It replays real public traces, provides the default Kubernetes autoscalers (HPA, VPA, Cluster Autoscaler) as baselines, and scores policies with availability, SLO and elasticity metrics from the literature.

Design goals: low CPU and RAM cost (compute is reserved for training), reproducibility (seeded, same seed gives identical metrics), and extension by traits rather than edits to the engine.

## Status legend

Every wiki page marks content as **Implemented** (exists in the repository now) or **Planned** (a later milestone in [docs/PLAN.md](../PLAN.md)).

| Area | Status |
|---|---|
| Cargo workspace skeleton (8 crates), pinned toolchain | Implemented (Milestone 1) |
| `ksim-bench --version` | Implemented |
| Python package `ksim`, `ksim.native_version()` | Implemented |
| C ABI `ksim_version()` | Implemented |
| CI helper `ksim_ci` (nightly version bumps) | Implemented |
| GitHub Actions pipelines, Docker/Podman images on GHCR | Implemented |
| Simulation core, perf model | Implemented (Milestone 2) |
| HPA / VPA / CA baselines, `KubernetesStack` | Implemented (Milestone 3) |
| Metrics (`ksim-metrics`), `ksim-bench run`, `ksim.analysis` leaderboard | Implemented (Milestone 4) |
| Dataset converters (Alibaba 2018/2021, Azure 2019, Borg 2019), `.ksimtrace`, trace-driven workloads | Implemented (Milestone 5) |
| Gymnasium env, vector env, PPO example | Planned (Milestone 6) |
| Calibration against a real `kind` cluster | Planned (Milestone 7) |

## Quick start

```sh
cargo run -p ksim-bench -- --version
cargo nextest run --workspace
uv run maturin develop && uv run pytest
docker run --rm ghcr.io/seccodesmith/ksim:nightly ksim-bench --version
```

## Navigation

- Understand it: [Architecture](Architecture), [Simulation Model](Simulation-Model), [Datasets](Datasets)
- Compare against it: [Baseline Autoscalers](Baseline-Autoscalers), [Benchmark Metrics](Benchmark-Metrics), [Benchmark Protocol](Benchmark-Protocol)
- Train with it: [Training Guide](Training-Guide)
- Contribute: [Extending](Extending), [Development](Development), [CI/CD](CI-CD), [Containers](Containers)
- Sources: [References](References)
