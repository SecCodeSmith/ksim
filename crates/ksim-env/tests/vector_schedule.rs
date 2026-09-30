mod common;

use std::time::Duration;

use ksim_env::{env::Env, layout::EnvConfig, vector::VectorEnv};

const ENVS: usize = 3;
const STEPS: usize = 25;

fn vector_trajectory(threshold: Duration) -> (Vec<f32>, Vec<f32>) {
    let scenario = common::make_scenario();
    let mut vec_env = VectorEnv::new(&scenario, &EnvConfig::default(), ENVS)
        .unwrap()
        .with_parallel_threshold(threshold);
    let layout = vec_env.first().layout().clone();
    let mut observations = vec![0.0; ENVS * layout.observation_len()];
    let actions = vec![3; ENVS * layout.action_nvec().len()];
    let (mut rewards, mut terminated, mut truncated) =
        (vec![0.0; ENVS], vec![false; ENVS], vec![false; ENVS]);
    let mut all_rewards = Vec::new();
    vec_env.reset(9, &mut observations).unwrap();
    for _ in 0..STEPS {
        vec_env
            .step(
                &actions,
                &mut observations,
                &mut rewards,
                &mut terminated,
                &mut truncated,
            )
            .unwrap();
        all_rewards.extend_from_slice(&rewards);
    }
    (observations, all_rewards)
}

#[test]
fn parallel_and_serial_schedules_give_identical_results() {
    let serial = vector_trajectory(Duration::MAX);
    let parallel = vector_trajectory(Duration::ZERO);
    assert_eq!(serial, parallel);
}

#[test]
fn parallel_schedule_matches_independent_envs() {
    let (observations, _) = vector_trajectory(Duration::ZERO);
    let scenario = common::make_scenario();
    let config = EnvConfig::default();
    let observation_len = observations.len() / ENVS;
    for index in 0..ENVS {
        let mut env = Env::new(&scenario, &config).unwrap();
        let mut observation = vec![0.0; observation_len];
        env.reset(9 + index as u64, &mut observation).unwrap();
        let action = vec![3; env.layout().action_nvec().len()];
        for _ in 0..STEPS {
            if env.is_over() {
                env.reset(9 + index as u64 + ENVS as u64, &mut observation)
                    .unwrap();
            } else {
                env.step(&action, &mut observation).unwrap();
            }
        }
        assert_eq!(
            &observations[index * observation_len..(index + 1) * observation_len],
            observation.as_slice()
        );
    }
}
