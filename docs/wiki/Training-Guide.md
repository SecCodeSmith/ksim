# Training guide

Status: **Implemented** (Milestone 6): the Rust environment (`ksim-env`), Gymnasium wrappers (`ksim.gym`), the C API (`ksim-capi`), benchmark-parity evaluation (`ksim.evaluate`) and a PPO example.

## Environments

- `ksim_env::Env`: one simulated cluster driven by an agent. Fixed-size `f32` observation, `MultiDiscrete` action, pluggable reward.
- `ksim_env::VectorEnv`: N independent `Env`s stepped in parallel with rayon into caller-provided buffers (env-major), so there is no Python loop per environment.
- `ksim.gym.KsimGymEnv` (`gymnasium.Env`) and `ksim.gym.KsimVectorGymEnv` (`gymnasium.vector.VectorEnv`, `autoreset_mode = NEXT_STEP`) wrap them; `pip install ksim[gym]`.
- `ksim.h` (cbindgen, crate `ksim-capi`) serves C/C++ trainers.

Options (`EnvConfig` in Rust, `ksim.gym.EnvOptions` or keyword arguments of `make` / `make_vec` in Python):

| Option | Default | Meaning |
|---|---|---|
| `max_replica_step` | 3 | `k` in the per-service action |
| `control` | `"replicas"` | `"replicas"` or `"replicas_and_nodes"` |
| `service_slots` | scenario's service count | pad observation and action to a fixed service count |
| `reward_weights` | `(1.0, 0.5, 0.01)` | SLO, cost, action weights of `WeightedReward` |

## Observation

A flat `Box(-1, 10, (observation_len,), float32)`: `service_slots * 10` service features, then `pools * 5` pool features, then 2 time features. Every feature is clipped to `[-1, 10]` (only the time features can be negative) and NaN becomes 0.

Per service, in order:

1. CPU utilisation of ready requests
2. memory utilisation of ready requests
3. `ln(1 + served_rps)`
4. SLO-percentile latency / SLO latency
5. error rate
6. ready pods / `max_replicas`
7. desired pods / `max_replicas`
8. pending pods / `max_replicas`
9. outdated pods / `max_replicas`
10. mask: 1 for a real service

Utilisations with no ready pods are 0. Slots beyond the scenario's services (`service_slots`) are all zero, including the mask, so one policy network can serve scenarios with different service counts.

Per pool, in order: ready, provisioning and draining nodes (each / `max_nodes`), requested CPU / allocatable CPU over the pool's existing nodes (0 without nodes), mask (1).

Time: `sin` and `cos` of the time of day (period 86 400 s).

## Action

`MultiDiscrete`, in this order:

- One entry of size `2k + 1` per service slot (`k = max_replica_step`). The entry `a` means a replica delta of `a - k` applied to the service's desired count, clamped to the HPA `[min_replicas, max_replicas]`. Padding slots are ignored.
- With `control="replicas_and_nodes"`, one 3-way entry per pool: node delta `a - 1`.
- With `control="replicas"` the Cluster Autoscaler manages nodes exactly as in the `k8s` benchmark policy, seeded with the episode seed.

Actions outside their bounds are rejected (`ActionOutOfRange`); nothing is stepped on error.

## Reward

`WeightedReward` (default) scores the tick sample recorded after the action took effect:

```
reward = -( w_slo * slo_violation
          + w_cost * hourly_cost / max_hourly_cost
          + w_act  * (replica_changes + node_changes) / services )
```

Defaults `w_slo = 1.0`, `w_cost = 0.5`, `w_act = 0.01`. `max_hourly_cost` is the cost of every pool at `max_nodes`. `slo_violation` is the demand-weighted share of services violating the SLO at this tick: a service with positive demand violates when its SLO-percentile latency exceeds `slo.latency_ms` or its error rate exceeds `slo.max_error_rate`; it is 0 without demand. Terms with a zero denominator drop out.

A custom reward implements `ksim_env::RewardFn` (`reward(&mut self, &TickSample, &RewardContext) -> f64`) and is injected with `Env::with_reward(Box<dyn RewardFn>)`; see [Extending](Extending#add-a-performance-model-or-reward). The Python wrappers use `WeightedReward`.

## Episodes, truncation and seeding

- Episodes never terminate; they are truncated after `scenario.steps()` steps (`episode_steps`). `step` then fails with `NeedsReset` until the next `reset`.
- On truncation `KsimGymEnv.step` returns `info["episode_metrics"]`, the benchmark metrics of the episode.
- `reset(seed=s)` seeds `np_random` and draws the simulation seed from it; later unseeded resets continue the sequence.
- `VectorEnv` follows Gymnasium's `NEXT_STEP` autoreset: the step after an env reports `truncated` resets that env, ignores its action and returns the reset observation, reward 0 and both flags false.
- Seeds: with reset seed `s`, env `i` uses `s + i`; its episode `e` (0-based) uses `s + i + e * n` for `n` envs. Per-env seed lists passed to `KsimVectorGymEnv.reset` must be consecutive.

## Python usage

```python
import ksim.gym
from gymnasium.utils.env_checker import check_env

env = ksim.gym.make("scenarios/synthetic_step.toml", control="replicas_and_nodes")
check_env(env.unwrapped, skip_render_check=True)

vec = ksim.gym.make_vec("scenarios/synthetic_step.toml", num_envs=8, service_slots=16)
observations, _ = vec.reset(seed=0)
```

## C usage

All functions are declared in `ksim.h`. The handle `KsimEnv*` is opaque. Functions returning `int32_t` return `KSIM_OK` (0), `KSIM_ERROR` (-1) or `KSIM_NULL_POINTER` (-2); after `KSIM_ERROR` (or a NULL return from `ksim_env_new`), `ksim_last_error()` returns the message for the calling thread, valid until the next failing call. Buffers belong to the caller; lengths are element counts.

| Function | Purpose |
|---|---|
| `ksim_env_new(scenario_path, max_replica_step)` | create with default options otherwise; NULL on error |
| `ksim_env_free(env)` | free; NULL ignored |
| `ksim_env_observation_len(env)`, `ksim_env_action_len(env)` | sizes (0 for NULL) |
| `ksim_env_action_nvec(env, out, len)` | write the `MultiDiscrete` sizes |
| `ksim_env_reset(env, seed, observation, len)` | start an episode |
| `ksim_env_step(env, action, action_len, observation, observation_len, &reward, &terminated, &truncated)` | advance one step; the flags are `uint8_t` 0 or 1 |

The C API exposes the single environment only.

## Evaluation parity

`ksim.evaluate.evaluate(scenario, policy, name, seeds=range(10))` runs one full episode per seed, resetting with exactly that seed (not one drawn by Gymnasium), and returns reports in the `ksim-bench` schema; `write_reports(reports, out)` writes `<scenario>-<policy>.jsonl`. The default seeds match `ksim-bench run --seeds 10`, so a trained policy is ranked next to the baselines by [`ksim.analysis`](Benchmark-Protocol). A zero-delta policy (every entry `k`) with `control="replicas"` reproduces the `ca` policy's metrics exactly; this is tested (`Env::step` records ticks like `ksim_bench::runner::run_episode`).

## Example: PPO with Stable-Baselines3

`examples/train_ppo.py` trains PPO (`MlpPolicy`) and benchmarks it. It needs the `train` extra (`uv sync --extra train`: stable-baselines3, sb3-contrib, torch).

```
uv run python examples/train_ppo.py --scenario scenarios/synthetic_step.toml \
    --timesteps 200000 --num-envs 8 --seeds 10 --out results --model models/ppo_ksim.zip
ksim-bench run --scenario scenarios/synthetic_step.toml --policy k8s --seeds 10 --out results
python -m ksim.analysis results --baseline k8s
```

Stable-Baselines3 has its own `VecEnv` API and does not accept a Gymnasium `VectorEnv`, so the script builds `DummyVecEnv` through `make_vec_env` over `ksim.gym.make`. Train on the train split, select on validation, report on test.

## Performance targets

`VectorEnv` steps its environments on the calling thread while a vector step is cheap and switches to rayon when the measured serial cost of a step reaches `PARALLEL_THRESHOLD` (200 µs; `with_parallel_threshold` overrides it). A single environment step on `synthetic_steady` takes about a microsecond, far less than waking a thread pool, so fanning out every step made 8 environments about 7× slower than stepping them in turn (about 106k env-steps/s); adaptive scheduling measured 430k to 690k env-steps/s on 4 cores. Serial and parallel schedules produce identical results (`tests/vector_schedule.rs`).

The release gate in `perf.yml` (`cargo test --release -p ksim-env --test perf -- --ignored`) requires at least 200k env-steps/s for 8 environments on `synthetic_steady` and a vector step no slower than 2× stepping the same environments serially; the margins leave room for shared CI runners.
