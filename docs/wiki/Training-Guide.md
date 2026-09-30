# Training guide

Status: **Planned** (Milestone 6). Today only `ksim.native_version()` exists (**Implemented**).

## Environments

- `KsimEnv`: Gymnasium `Env` over the Rust core (PyO3, abi3 wheel).
- `KsimVectorEnv`: N environments stepped in parallel in Rust with rayon; observations are written directly into a shared NumPy buffer, so there is no Python loop per environment. Compatible with Stable-Baselines3, RLlib and CleanRL.
- A C header `ksim.h` (cbindgen) serves C/C++ trainers.

## Observation, action, reward

- Observation: fixed-size f32 tensor per service (windowed CPU/memory utilization, RPS, p95 latency, replicas, pending pods, requests) plus cluster features; a mask handles variable service counts.
- Action: `MultiDiscrete` (replica delta, request scale step, node delta) or a continuous variant.
- Reward: `-w_slo * SLO_violation - w_cost * resource_cost - w_act * scaling_actions`, replaceable from Python via the `RewardFn` trait.

## Example (Planned)

A short PPO run with Stable-Baselines3 (optional extra `train`: stable-baselines3, sb3-contrib, torch) compared against the HPA baseline report. Train on the train split, select on validation, report on test.

## Performance targets

At least 100k env-steps/s per core on a 50-service scenario; under 100 MB RAM per environment, verified by criterion benchmarks.
