# Plan: Kubernetes Autoscaler Training Simulator (trace-driven, Rust core)

## Context
The repository is empty (only a README). Goal: a lightweight, fast Kubernetes cluster simulator for **training and benchmarking autoscalers** (RL or heuristic), driven by **real public cluster traces** (Alibaba 2018, Alibaba 2021/2022 microservices, Google Borg 2019, Azure Functions 2019). It must include the **default Kubernetes autoscalers as baselines** (HPA, VPA, Cluster Autoscaler, with upstream default parameters) and a **benchmark suite** reporting availability/SLO/elasticity metrics taken from the scientific literature.

User decisions:
- Scope: **HPA + VPA + Cluster Autoscaler** from the start.
- Language: **Rust** core (low CPU/RAM so compute goes to AI training). Python is used only at the edges (dataset preprocessing, Gymnasium bindings for training frameworks). A C ABI is exported too.
- Datasets: all four above.

## Engineering rules (apply to every milestone)
1. **Test-Driven Development, always test first**: for every feature the commit sequence is *failing test → minimal implementation → refactor*. No production code without a failing test that demands it. Baseline autoscalers start from tests ported from upstream Kubernetes test cases; metrics start from hand-computed examples from the cited papers. CI blocks merges on test failure and on coverage regression (`cargo-llvm-cov`, target > 80 % for core/autoscalers/metrics).
2. **SOLID**:
   - *S*: one responsibility per module/type (scheduler ≠ perf model ≠ metrics ≠ trace IO).
   - *O*: new autoscalers, datasets, perf models, rewards and metrics are added by implementing a trait, never by editing the engine (`Autoscaler`, `TraceSource`, `PerformanceModel`, `RewardFn`, `Metric`, `SchedulerPlugin`).
   - *L*: every trait implementation passes the same shared contract test suite (e.g. `autoscaler_contract_tests!`, `trace_source_contract_tests!`).
   - *I*: small traits (`ReplicaRecommender`, `ResourceRecommender`, `NodeProvisioner`) instead of one fat interface; HPA/VPA/CA each implement only what they need.
   - *D*: the engine depends on traits injected via a builder (`SimulationBuilder`), not on concrete types; tests inject fakes.
3. **Clean code, no noise comments**: self-explanatory names, small functions; comments only where they add information the code cannot (a *why*, a formula source/citation, an upstream-parity note). No comments that restate the code, no commented-out code, no TODO without an issue link. Enforced by review + `clippy::pedantic` (selected lints) and `ruff`.
4. **Latest stable versions only**: every dependency pinned to the newest stable release at implementation time (no pre-releases, no yanked, no unmaintained crates). Rust toolchain pinned via `rust-toolchain.toml`. `cargo-deny` (advisories, licenses, bans), `cargo-outdated` and Dependabot/Renovate in CI keep it current; Python deps managed with `uv` and a lockfile.

## Multi-agent development workflow (cheaper model writes tests)
TDD is executed with specialised agent roles so expensive reasoning is spent only where needed:
- **Orchestrator — the main session**: splits a milestone into features with disjoint file ownership, dispatches agents, resolves conflicts, and integrates results. At most **5 agents run concurrently**.
- **Test author — Haiku 4.5** (`claude-haiku-4-5-20251001`, `.claude/agents/test-writer.md`): given the wiki spec / upstream reference for a feature, writes the failing tests first (unit, property, contract, snapshot) and commits them as `test: …`.
- **Implementer — Sonnet 5.5 subagents** (`.claude/agents/implementer.md`): several run in parallel, each owning a disjoint set of files/crates. They make the tests pass with minimal code, then refactor. They **never modify tests**; a needed test change goes back to the orchestrator and lands as a separate, justified `test:` commit.
- **Reviewer — Haiku 4.5** (`.claude/agents/reviewer.md`, read-only): checks the diff against SOLID, clean-code, noise-comment and test-first-ordering rules.
- Agents **communicate via messages** through the orchestrator (test paths and failures in, changed files and results out); no shared mutable state beyond owned files.

## CI/CD pipelines (GitHub Actions, actions pinned to latest stable major / commit SHA)
Many small, single-purpose workflows (SRP applied to CI). Shared steps live in **reusable workflows** (`workflow_call`, prefixed `_`) and a composite action, so each file stays short (target < ~60 lines) and can fail/re-run independently.

**Reusable building blocks** (`.github/workflows/_*.yml`, `.github/actions/setup-rust-python/`):
| File | Does one thing |
|---|---|
| `actions/setup-rust-python` | install pinned toolchain, uv, caches |
| `_rust-test.yml` | `cargo nextest` + doc tests |
| `_python-test.yml` | maturin develop + pytest |
| `_coverage.yml` | llvm-cov + pytest-cov, threshold gate, upload report |
| `_build-binaries.yml` | `ksim-bench` + C libs/`ksim.h`, per target (matrix) |
| `_build-wheels.yml` | abi3 wheels via maturin |
| `_build-image.yml` | build one image target with Docker buildx *or* Podman (input), optional push |
| `_scan-image.yml` | Trivy scan of an image ref |
| `_sign-image.yml` | SBOM (syft) + cosign sign/attest |
| `_bump-version.yml` | compute + apply version for a branch, commit, tag |

**Push / PR triggers** (each its own workflow, run in parallel):
| Workflow | Trigger | Content |
|---|---|---|
| `lint.yml` | push, PR | fmt, clippy `-D warnings`, ruff, mypy |
| `test.yml` | push, PR | calls `_rust-test` + `_python-test` |
| `coverage.yml` | push, PR | calls `_coverage` |
| `security-deps.yml` | push, PR, weekly | cargo-deny, cargo-audit, pip-audit |
| `security-codeql.yml` | push, PR, weekly | CodeQL Rust + Python |
| `security-secrets.yml` | push, PR | gitleaks |
| `container-check.yml` | PR touching `Containerfile`/src | `_build-image` with Docker **and** Podman (no push) + `_scan-image` |

**Nightly** (chain of small workflows linked by `workflow_run` / `workflow_call`):
| Workflow | Content |
|---|---|
| `nightly-detect.yml` | cron daily + manual; finds `main` / `release/*` branches with commits or merges in last 24 h; outputs branch list; exits if empty |
| `nightly-bump.yml` | per branch: `_bump-version` (`main` → `X.Y.Z-nightly.YYYYMMDD`, `release/X.Y` → patch bump) via `cargo set-version` + `pyproject.toml` sync |
| `nightly-build.yml` | per branch: `_build-binaries` + `_build-wheels`, attach to pre-release |
| `nightly-images.yml` | `_build-image` (push, multi-arch) → `_scan-image` → `_sign-image` |

**Release** (on `v*` tag): `release-artifacts.yml` (binaries, wheels, GitHub release) and `release-images.yml` (images tagged `X.Y.Z`, `X.Y`, `latest`), both reusing the same `_*` workflows.

Dependabot/Renovate for Cargo, uv, GitHub Actions and base images.

## Containers: Docker + Podman
- One multi-stage **`Containerfile`** (symlinked as `Dockerfile`): Rust builder stage (pinned latest stable `rust` image) → minimal runtime (distroless/`debian-slim`, non-root user) with `ksim-bench`; a second target `ksim-train` with Python + wheel + gymnasium for training jobs.
- Rootless-friendly (works under rootless Podman), OCI labels, SBOM (`syft`) and provenance attestations, images signed with **cosign**.
- Published to **GitHub Container Registry** `ghcr.io/seccodesmith/ksim` and `ghcr.io/seccodesmith/ksim-train` (pullable by both `docker pull` and `podman pull`), tags: `nightly`, `nightly-YYYYMMDD`, `X.Y.Z`, `latest`.
- `compose.yaml` usable by `docker compose` and `podman compose`, plus a `podman kube play` manifest; documented in wiki `Containers.md`.

## Library research (latest stable versions, verified 2026-09-29 on crates.io / PyPI / rust-lang)

| Area | Choice | Version | Why |
|---|---|---|---|
| Toolchain | Rust stable | 1.98.1 | pinned in `rust-toolchain.toml`, edition 2024 |
| Python bindings | `pyo3` | 0.29.2 | Rust↔Python, abi3 wheels |
| NumPy zero-copy | `numpy` (crate) | 0.29.0 | matches pyo3 0.29 |
| Build wheels | `maturin` | 1.15.0 | PyO3 packaging |
| Parallel vector env | `rayon` | 1.12.0 | data-parallel stepping |
| Trace IO | `memmap2` | 0.9.11 | zero-copy mmap of `.ksimtrace` |
| RNG | `rand` / `rand_chacha` | 0.10.3 / 0.10.0 | reproducible seeded streams |
| Config | `serde` / `toml` / `serde_json` | 1.0.229 / 1.1.6 / 1.0.151 | scenario files, reports |
| CLI | `clap` | 4.6.7 | `ksim-bench` |
| Errors | `thiserror` (libs) / `anyhow` (bins) | 2.0.21 / 1.0.104 | typed errors in libraries |
| Stats in Rust | `statrs` | 0.19.1 | distributions for latencies/queueing |
| Logging | `tracing` | 0.1.44 | structured, zero-cost when off |
| C ABI | `cbindgen` | 0.29.4 | generate `ksim.h` |
| Testing | `proptest` 1.11.0, `insta` 1.48.0, `criterion` 0.8.2 | | property tests, snapshot reports, benchmarks |
| Preprocessing | `polars` (Py) / `pyarrow` | 1.44.2 / 25.0.1 | fast out-of-core trace normalization (Arrow IPC intermediate) |
| RL API | `gymnasium` / `pettingzoo` | 1.3.0 / 1.27.0 | single- and multi-agent envs |
| Example training | `stable-baselines3` / `sb3-contrib` / `torch` | 2.9.0 / 2.9.0 / 2.14.0 | PPO/RecurrentPPO example only (optional extra) |
| Analysis | `numpy` 2.5.3, `scipy` 1.18.1, `matplotlib` 3.11.2 | | bootstrap CI, Wilcoxon, plots |
| Borg 2019 access | `google-cloud-bigquery` | 3.45.2 | optional extra |
| Py quality | `pytest` 9.1.1, `hypothesis` 6.168.3, `ruff` 0.16.9, `mypy` 2.3.1 | | TDD + lint + types |

Deliberately *not* used: Rust `polars`/`arrow` in the simulator hot path (heavy compile/RAM; the custom mmap format is smaller and faster), Python in the step loop. Upstream Kubernetes / kubernetes-autoscaler reference versions for HPA/VPA/CA parity are pinned to the newest stable release tag at implementation time and recorded in the wiki.

## Wiki (`docs/wiki/`, mirrored to the GitHub Wiki)
Dedicated explanation-oriented wiki, written alongside code (each milestone's PR updates its pages):
- `Home.md` – purpose, quick start, navigation.
- `Architecture.md` – crates, traits, data flow diagram, SOLID boundaries.
- `Simulation-Model.md` – time model, scheduler, lifecycle latencies, perf/queueing model with formulas and assumptions/limitations.
- `Datasets.md` – per dataset: source, license, schema, download, normalization, train/val/test splits.
- `Baseline-Autoscalers.md` – HPA/VPA/CA algorithms, default parameters with upstream source references and version tag.
- `Benchmark-Metrics.md` – every metric's definition, formula, citation, interpretation.
- `Benchmark-Protocol.md` – scenarios, seeds, statistics, how to submit a result.
- `Training-Guide.md` – Gymnasium/vector env usage, observation/action/reward spec, PPO example.
- `Extending.md` – adding an autoscaler/dataset/metric by implementing a trait (TDD walkthrough).
- `Development.md` – TDD workflow, multi-agent roles, clean-code rules, dependency update policy.
- `CI-CD.md` – pipelines, nightly version-bump rules, security scanning, release process.
- `Containers.md` – Docker/Podman usage, registry, tags, verification of signatures.
- `References.md` – bibliography.

## Architecture (Cargo workspace + thin Python package)

```
Cargo.toml                      # workspace
crates/
  ksim-core/        # cluster state machine, clock, events, scheduler
  ksim-workload/    # trace format, mmap reader, replay, time-warp/scale
  ksim-perf/        # performance model (latency, throttling, OOM)
  ksim-autoscalers/ # Autoscaler trait + HPA, VPA, CA baselines
  ksim-metrics/     # benchmark metrics + reports
  ksim-bench/       # CLI: scenarios (TOML) -> results (JSON/CSV)
  ksim-py/          # PyO3 bindings: Gymnasium env + Rust-side vector env
  ksim-capi/        # C ABI via cbindgen (ksim.h)
python/ksim/
  datasets/         # downloaders + normalizers (polars) -> .ksimtrace
  gym.py            # gymnasium.Env / VectorEnv wrappers over ksim-py
  analysis/         # stats: bootstrap CI, Wilcoxon, plots
scenarios/          # benchmark TOML definitions
docs/               # model assumptions, metric definitions, citations
```

### 1. Simulation core (`ksim-core`)
- **Hybrid time model**: fixed control tick (default 15 s = HPA sync period) + a binary-heap event queue for delayed events (pod start, node provision, eviction, image pull). Deterministic, seeded (`rand_chacha`).
- **Objects** (struct-of-arrays, preallocated, no per-tick allocation): `Node{cpu_alloc, mem_alloc, pool, state}`, `Pod{service, requests, limits, node, phase, start_at}`, `Service/Deployment{replicas, requests, limits, hpa/vpa cfg}`, `NodePool{instance type, min/max, price/h, provision delay dist}`.
- **Scheduler**: simplified kube-scheduler — filter `NodeResourcesFit`, score `LeastAllocated` (default) / `MostAllocated` (bin-packing option), pending queue with backoff. Unschedulable pods feed Cluster Autoscaler.
- **Lifecycle latencies** (configurable distributions, defaults from literature/K8s SIG-scalability SLO: pod startup p99 ≤ 5 s excl. image pull): image pull, container start, readiness delay; node provisioning 60–180 s; graceful termination 30 s.

### 2. Workload from traces (`ksim-workload` + `python/ksim/datasets`)
Offline Python/polars pipeline downloads and normalizes each dataset into one compact binary format `.ksimtrace` (header + columnar f32/u32 arrays, per-service time series at fixed resolution), which Rust **mmaps** (zero-copy, memmap2).

| Dataset | Files used | Mapping to simulator |
|---|---|---|
| Alibaba 2018 (8 days, ~4k machines) | `container_meta`, `container_usage`, `machine_meta`, `batch_task/instance` | Group containers by `app_du` → service; demand = Σ cpu_util×cpu_request; batch tasks as background load/node pressure |
| Alibaba 2021/2022 microservices | `MSRTMCR`/`MSMetrics`, `MSCallGraph`, `NodeMetrics` | Per-microservice RPS + response time → request-rate driven services; call graph → dependency fan-out (v2) |
| Google Borg 2019 | `instance_usage`, `instance_events`, `collection_events` | Collections → services; 5-min CPU/mem usage (NCU-normalized); priority tiers |
| Azure Functions 2019 | invocations/min, duration percentiles, memory | Function → service; bursty RPS, per-request cost from duration |

- Replay features: service selection (top-K by volume, random sample with seed), time windows, **time-compression and amplitude scaling**, train/validation/test **temporal splits** (e.g. Alibaba 2018 days 1–5 / 6 / 7–8) to avoid leakage.
- Synthetic generators (step, ramp, sinusoid+noise, flash crowd) for unit tests and sanity scenarios.

### 3. Performance model (`ksim-perf`)
Converts demand vs. allocated resources into observable signals per tick, cheap enough for millions of steps:
- Request-driven services: each pod = server with capacity μ derived from CPU limit/usage-per-request; latency via **M/M/c (Erlang-C)** with Allen–Cunneen G/G/c correction (trace-derived CV); requests beyond queue bound/timeout → **dropped/errored**.
- Utilization-driven services (Alibaba 2018 / Borg): CPU demand > limit → CFS **throttling** → latency inflation; memory usage > limit → **OOMKill** + restart delay; node memory pressure → eviction.
- Outputs: CPU/mem usage per pod, RPS, p50/p95/p99 latency, error rate, pending pods — the same metrics a Metrics Server/Prometheus would expose, with configurable **metric scrape delay/window** (Metrics Server default 15 s resolution).

### 4. Autoscaler interface + baselines (`ksim-autoscalers`)
`trait Autoscaler { fn act(&mut self, obs: &Observation, now: SimTime) -> Actions; }` — actions: per-service replica target, per-service CPU/mem requests, per-pool node delta. RL agents and baselines use the same path.

Baselines, reimplementing upstream algorithms with their **default parameters** (documented with source links in `docs/`):
- **HPA** (k8s `replica_calculator.go`/`horizontal.go`): `desired = ceil(current × metric/target)`, tolerance 0.1, sync 15 s, scale-down stabilization 300 s, scale-up stabilization 0 s, default policies (up: max(100 %, 4 pods)/15 s; down: 100 %/15 s), missing-metric and not-ready-pod handling, CPU init period 5 min.
- **VPA** (recommender + updater, `Auto` mode): decaying CPU histogram half-life 24 h, target = P90 CPU, lower/upper P50/P95, memory = peak per 24 h interval, safety margin +15 %, min 25m CPU / 250 Mi; updater evicts when outside bounds, respects PDB-like limit; HPA+VPA-on-CPU conflict flagged as in upstream docs.
- **Cluster Autoscaler**: scan interval 10 s, scale-up on unschedulable pods with expander `random` (+`least-waste`), scale-down-utilization-threshold 0.5, scale-down-unneeded-time 10 min, scale-down-delay-after-add 10 min, max-graceful-termination 600 s.
- Extra reference points: static over-provisioning (peak), static average, and a **clairvoyant oracle** (knows future demand) as upper bound.
- Unit tests port key cases from upstream HPA/VPA/CA test files to prove fidelity.

### 5. Training interface (`ksim-py`, `ksim-capi`)
- PyO3 + maturin wheel exposing `KsimEnv` implementing **Gymnasium** API and a **Rust-side `KsimVectorEnv`** (N envs stepped in parallel with rayon, observations written directly into a shared NumPy buffer — no Python loop per env). Compatible with Stable-Baselines3 / RLlib / CleanRL.
- Observation: fixed-size f32 tensor per service (windowed CPU/mem util, RPS, p95 latency, replicas, pending, requests) + cluster features; masking for variable service count. Action: MultiDiscrete (replica delta, request scale step, node delta) or continuous variant.
- Reward: configurable weighted sum `−w_slo·SLO_violation − w_cost·resource_cost − w_act·scaling_actions`, defaults documented; also pluggable from Python.
- C header `ksim.h` (cbindgen) for C/C++ trainers.
- Performance targets: ≥100k env-steps/s per core on a 50-service scenario; <100 MB RAM per env (verified by criterion benchmarks).

### 6. Benchmark & metrics (`ksim-metrics`, `ksim-bench`, `python/ksim/analysis`)
Metrics grounded in literature (definitions + citations in `docs/metrics.md`):
- **Availability / SLO** (Google SRE book): request success ratio (non-error AND latency ≤ SLO), expressed in nines; time-based availability; SLO violation rate for p95/p99 latency; error-budget burn.
- **SPEC RG elasticity metrics** (Herbst et al., "Elasticity in Cloud Computing: What It Is, and What It Is Not", ICAC 2013; Herbst et al., "Ready for Rain?", ACM TOMPECS 2018): under/over-provisioning accuracy θ_U, θ_O; timeshare τ_U, τ_O; **jitter**; instability; elastic speedup — computed from supplied vs. demanded resources.
- **Cost/efficiency**: node-hours × price, CPU/mem request utilization, allocated-vs-used waste.
- **Operational**: scaling actions count, oscillations, time-to-recover after a surge, pending-pod time, OOMKills, evictions, cold starts.
- **Protocol**: fixed scenario suite (per dataset: steady, diurnal, burst/flash-crowd, mixed batch+online, node-failure), held-out test windows, ≥10 seeds, mean ± 95 % bootstrap CI, paired **Wilcoxon signed-rank** vs. default HPA+VPA+CA baseline, and normalized score (baseline = 1.0). Output: JSON + CSV + leaderboard markdown/plots.
- `ksim-bench run scenarios/alibaba2018_diurnal.toml --policy hpa` and `--policy python:module:Policy`.

## Milestones
Every milestone follows red → green → refactor and ends with its wiki pages updated.

1. **Workspace skeleton**: Cargo workspace, `rust-toolchain.toml`, the small workflows + reusable `_*` blocks listed above, `Containerfile` + GHCR publishing, agent definitions, maturin build, `docs/wiki/` scaffold, contract-test macros.
2. **Core + synthetic workload**: cluster state, scheduler, events, perf model; deterministic tests.
3. **Baselines**: HPA → VPA → CA with ported upstream test cases.
4. **Metrics + bench CLI**: SLO/availability, SPEC elasticity, cost; scenario TOML; reports.
5. **Datasets**: `.ksimtrace` format + normalizers in order Alibaba 2018 → Alibaba 2021/22 → Azure 2019 → Borg 2019 (BigQuery export needs user GCP credentials; document it).
6. **Python/Gym bindings + vector env**, example PPO training script (SB3) vs. HPA baseline.
7. **Validation & calibration**: compare simulated HPA behavior against a real `kind` cluster run of the same scaled trace (k6 load generator) — report replica-count and latency error; document model limitations.

## Verification
- Git history shows test commits preceding implementation commits per feature; `cargo deny check` and `cargo outdated --exit-code 1` pass (latest stable deps).
- `actionlint` passes on all workflows; each push shows separate lint/test/coverage/security checks, all green; nightly chain dispatched manually: no-op with no new commits, builds + bumps version + pushes images when there are; `docker run` and `podman run ghcr.io/seccodesmith/ksim:nightly ksim-bench --version` both work; `cosign verify` succeeds.
- `cargo fmt --check && cargo clippy -- -D warnings && cargo test --workspace` (unit tests incl. ported upstream HPA/VPA/CA cases, determinism test: same seed → identical metrics).
- `cargo bench` (criterion) for steps/s and memory targets.
- `maturin develop && pytest python/tests` — Gymnasium `check_env`, vector env shape/dtype tests.
- End-to-end: `ksim-bench run scenarios/synthetic_step.toml --policy hpa` produces expected replica trajectory; run on a small Alibaba 2018 sample (downloaded subset) and check report contents; train a short PPO run and compare against baseline report.
- Commit and push to branch `claude/great-shannon-vtzs1g`.
