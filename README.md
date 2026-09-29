# ksim: Kubernetes autoscaler training simulator

ksim is a fast, deterministic, trace-driven simulator of a Kubernetes cluster for training and benchmarking autoscalers (reinforcement learning or heuristic). It replays real public cluster traces (Alibaba 2018 and 2021/2022, Google Borg 2019, Azure Functions 2019), ships the default Kubernetes autoscalers (HPA, VPA, Cluster Autoscaler) as baselines, and reports availability, SLO and elasticity metrics from the literature. The core is Rust for low CPU and RAM overhead; Python is used only at the edges (dataset preprocessing, Gymnasium bindings). A C ABI is exported as well.

## Status: Milestones 1-3 (skeleton, simulation core, HPA/VPA/CA baselines)

Implemented:
- Cargo workspace with crates `ksim-core`, `ksim-workload`, `ksim-perf`, `ksim-autoscalers`, `ksim-metrics`, `ksim-bench`, `ksim-py`, `ksim-capi`
- `ksim-bench --version`
- Python package `ksim` with `native_version()` (maturin/PyO3, abi3)
- C ABI `ksim_version()`
- CI helper `ksim_ci` for nightly version bumps
- GitHub Actions pipelines (lint, test, coverage, security, nightly, release) and Docker/Podman container images
- Simulation core, synthetic workloads and performance models (Milestone 2)
- HPA, VPA and Cluster Autoscaler baselines with `KubernetesStack` (Milestone 3)

Planned (Milestones 4-7): metrics and benchmark CLI, dataset normalizers, Gymnasium/vector env, calibration against a real `kind` cluster. See [docs/PLAN.md](docs/PLAN.md).

## Quick start

Rust:
```sh
cargo run -p ksim-bench -- --version
cargo nextest run --workspace      # or: cargo test --workspace
```

Python (uv + maturin):
```sh
uv run maturin develop
uv run pytest
uv run python -c "import ksim; print(ksim.native_version())"
```

Containers (Docker or Podman):
```sh
docker pull ghcr.io/seccodesmith/ksim:nightly
docker run --rm ghcr.io/seccodesmith/ksim:nightly ksim-bench --version
podman run --rm ghcr.io/seccodesmith/ksim:nightly ksim-bench --version
```

## Documentation

- [Plan](docs/PLAN.md)
- Wiki: [Home](docs/wiki/Home.md), [Architecture](docs/wiki/Architecture.md), [Simulation model](docs/wiki/Simulation-Model.md), [Datasets](docs/wiki/Datasets.md), [Baseline autoscalers](docs/wiki/Baseline-Autoscalers.md), [Benchmark metrics](docs/wiki/Benchmark-Metrics.md), [Benchmark protocol](docs/wiki/Benchmark-Protocol.md), [Training guide](docs/wiki/Training-Guide.md), [Extending](docs/wiki/Extending.md), [Development](docs/wiki/Development.md), [CI/CD](docs/wiki/CI-CD.md), [Containers](docs/wiki/Containers.md), [References](docs/wiki/References.md)
- Project rules for contributors and agents: [CLAUDE.md](CLAUDE.md)

## License

Licensed under the SecCodeSmith Commercial License; see [LICENCE](LICENCE).
