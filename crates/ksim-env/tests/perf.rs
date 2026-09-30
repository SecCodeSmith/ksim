mod common;

use ksim_bench::scenario::Scenario;
use ksim_env::{env::Env, layout::EnvConfig, vector::VectorEnv};
use std::time::Instant;

#[test]
#[ignore = "release-mode throughput gate; run with --release -- --ignored"]
fn vector_env_throughput_with_eight_envs() {
    if cfg!(debug_assertions) {
        return;
    }

    let scenario_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../scenarios/synthetic_steady.toml"
    );
    let scenario = Scenario::from_toml_file(scenario_path).expect("scenario should load");

    let config = EnvConfig::default();
    let num_envs = 8;
    let mut vec_env =
        VectorEnv::new(&scenario, &config, num_envs).expect("VectorEnv should create");

    let (observation_len, action_len, max_replica_step) = {
        let layout = vec_env.first().layout();
        (
            layout.observation_len(),
            layout.action_len(),
            layout.max_replica_step,
        )
    };

    let mut observations = vec![0.0; num_envs * observation_len];
    let mut rewards = vec![0.0; num_envs];
    let mut terminated = vec![false; num_envs];
    let mut truncated = vec![false; num_envs];

    vec_env
        .reset(0, &mut observations)
        .expect("reset should succeed");
    let zero_delta_action = max_replica_step;
    let actions = vec![zero_delta_action; num_envs * action_len];

    for _ in 0..100 {
        vec_env
            .step(
                &actions,
                &mut observations,
                &mut rewards,
                &mut terminated,
                &mut truncated,
            )
            .expect("step should succeed");
    }

    let start = Instant::now();
    for _ in 0..3000 {
        vec_env
            .step(
                &actions,
                &mut observations,
                &mut rewards,
                &mut terminated,
                &mut truncated,
            )
            .expect("step should succeed");
    }
    let elapsed = start.elapsed();

    let env_steps = (num_envs as u64) * 3000u64;
    #[allow(clippy::cast_precision_loss)]
    let env_steps_per_second = env_steps as f64 / elapsed.as_secs_f64();

    println!("Throughput: {env_steps_per_second:.1} env-steps/s");
    assert!(
        env_steps_per_second >= 200_000.0,
        "throughput should be >= 200_000.0 env-steps/s, got {env_steps_per_second:.1}"
    );
}

#[test]
#[ignore = "release-mode throughput gate; run with --release -- --ignored"]
fn vector_env_is_not_slower_than_stepping_envs_serially() {
    if cfg!(debug_assertions) {
        return;
    }

    let scenario_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../scenarios/synthetic_steady.toml"
    );
    let scenario = Scenario::from_toml_file(scenario_path).expect("scenario should load");

    let config = EnvConfig::default();
    let num_envs = 8;
    let mut vec_env =
        VectorEnv::new(&scenario, &config, num_envs).expect("VectorEnv should create");

    let (observation_len, action_len, max_replica_step) = {
        let layout = vec_env.first().layout();
        (
            layout.observation_len(),
            layout.action_len(),
            layout.max_replica_step,
        )
    };

    let mut vec_observations = vec![0.0; num_envs * observation_len];
    let mut vec_rewards = vec![0.0; num_envs];
    let mut vec_terminated = vec![false; num_envs];
    let mut vec_truncated = vec![false; num_envs];

    vec_env
        .reset(0, &mut vec_observations)
        .expect("reset should succeed");

    let zero_delta_action = max_replica_step;
    let vec_actions = vec![zero_delta_action; num_envs * action_len];

    let vec_start = Instant::now();
    for _ in 0..3000 {
        vec_env
            .step(
                &vec_actions,
                &mut vec_observations,
                &mut vec_rewards,
                &mut vec_terminated,
                &mut vec_truncated,
            )
            .expect("step should succeed");
    }
    let vec_elapsed = vec_start.elapsed();

    let mut serial_envs = Vec::new();
    for _ in 0..num_envs {
        serial_envs.push(Env::new(&scenario, &config).expect("Env should create"));
    }

    let mut serial_observations = vec![vec![0.0; observation_len]; num_envs];
    let mut episodes = vec![0u64; num_envs];
    for (i, env) in serial_envs.iter_mut().enumerate() {
        env.reset(i as u64, &mut serial_observations[i])
            .expect("reset should succeed");
    }

    let single_action = vec![zero_delta_action; action_len];

    let serial_start = Instant::now();
    for _ in 0..3000 {
        for i in 0..num_envs {
            let outcome = serial_envs[i].step(&single_action, &mut serial_observations[i]);

            match outcome {
                Ok(out) if out.truncated => {
                    episodes[i] += 1;
                    let next_seed = (i as u64) + episodes[i] * (num_envs as u64);
                    serial_envs[i]
                        .reset(next_seed, &mut serial_observations[i])
                        .expect("reset should succeed");
                }
                Ok(_) => {}
                Err(e) => panic!("step should succeed: {e}"),
            }
        }
    }
    let serial_elapsed = serial_start.elapsed();

    let vec_secs = vec_elapsed.as_secs_f64();
    let serial_secs = serial_elapsed.as_secs_f64();
    let ratio = vec_secs / serial_secs;

    println!("Vector time: {vec_secs:.3}s, Serial time: {serial_secs:.3}s, Ratio: {ratio:.2}x");
    assert!(
        vec_elapsed <= serial_elapsed.mul_f64(2.0),
        "vector env time ({vec_secs:.3}s) should be <= 2x serial time ({serial_secs:.3}s), ratio: {ratio:.2}x"
    );
}
