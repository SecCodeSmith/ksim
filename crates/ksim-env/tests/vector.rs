mod common;

use approx::assert_abs_diff_eq;
use ksim_env::{EnvError, env::Env, layout::EnvConfig, vector::VectorEnv};

#[test]
fn no_environments_error() {
    let scenario = common::make_scenario();
    let config = EnvConfig::default();

    let result = VectorEnv::new(&scenario, &config, 0);
    assert!(matches!(result, Err(EnvError::NoEnvironments)));
}

#[test]
fn reset_buffer_length_error() {
    let scenario = common::make_scenario();
    let config = EnvConfig::default();
    let mut vec_env = VectorEnv::new(&scenario, &config, 2).unwrap();

    let layout = vec_env.first().layout();
    let obs_len = layout.observation_len();
    let expected_len = 2 * obs_len;

    let mut obs = vec![0.0; expected_len - 1];
    let result = vec_env.reset(42, &mut obs);

    assert!(matches!(result, Err(EnvError::BufferLength { .. })));
}

#[test]
fn step_obs_buffer_length_error() {
    let scenario = common::make_scenario();
    let config = EnvConfig::default();
    let mut vec_env = VectorEnv::new(&scenario, &config, 2).unwrap();

    let layout = vec_env.first().layout();
    let obs_len = layout.observation_len();
    let action_nvec = layout.action_nvec();

    let mut obs = vec![0.0; 2 * obs_len];
    vec_env.reset(42, &mut obs).unwrap();

    let mut obs_too_short = vec![0.0; 2 * obs_len - 1];
    let action = vec![3; 2 * action_nvec.len()];
    let mut rewards = vec![0.0; 2];
    let mut terminated = vec![false; 2];
    let mut truncated = vec![false; 2];

    let result = vec_env.step(
        &action,
        &mut obs_too_short,
        &mut rewards,
        &mut terminated,
        &mut truncated,
    );
    assert!(matches!(result, Err(EnvError::BufferLength { .. })));
}

#[test]
fn step_action_buffer_length_error() {
    let scenario = common::make_scenario();
    let config = EnvConfig::default();
    let mut vec_env = VectorEnv::new(&scenario, &config, 2).unwrap();

    let layout = vec_env.first().layout();
    let obs_len = layout.observation_len();
    let action_nvec = layout.action_nvec();

    let mut obs = vec![0.0; 2 * obs_len];
    vec_env.reset(42, &mut obs).unwrap();

    let action = vec![3; 2 * action_nvec.len() - 1];
    let mut rewards = vec![0.0; 2];
    let mut terminated = vec![false; 2];
    let mut truncated = vec![false; 2];

    let result = vec_env.step(
        &action,
        &mut obs,
        &mut rewards,
        &mut terminated,
        &mut truncated,
    );
    assert!(matches!(result, Err(EnvError::BufferLength { .. })));
}

#[test]
fn reset_seeds_env_i_with_seed_plus_i() {
    let scenario = common::make_scenario();
    let config = EnvConfig::default();
    let mut vec_env = VectorEnv::new(&scenario, &config, 3).unwrap();

    let layout = vec_env.first().layout();
    let obs_len = layout.observation_len();

    let mut vec_obs = vec![0.0; 3 * obs_len];
    vec_env.reset(100, &mut vec_obs).unwrap();

    for i in 0..3 {
        let mut single_env = Env::new(&scenario, &config).unwrap();
        let mut single_obs = vec![0.0; obs_len];
        single_env.reset(100 + i as u64, &mut single_obs).unwrap();

        for j in 0..obs_len {
            assert_abs_diff_eq!(vec_obs[i * obs_len + j], single_obs[j], epsilon = 1e-5);
        }
    }
}

#[test]
fn step_matches_independent_envs() {
    let scenario = common::make_scenario();
    let config = EnvConfig::default();
    let mut vec_env = VectorEnv::new(&scenario, &config, 2).unwrap();

    let layout = vec_env.first().layout();
    let obs_len = layout.observation_len();
    let action_nvec = layout.action_nvec();

    let mut vec_obs = vec![0.0; 2 * obs_len];
    vec_env.reset(50, &mut vec_obs).unwrap();

    let action = vec![3; 2 * action_nvec.len()];
    let mut rewards = vec![0.0; 2];
    let mut terminated = vec![false; 2];
    let mut truncated = vec![false; 2];

    vec_env
        .step(
            &action,
            &mut vec_obs,
            &mut rewards,
            &mut terminated,
            &mut truncated,
        )
        .unwrap();

    for i in 0..2 {
        let mut single_env = Env::new(&scenario, &config).unwrap();
        let mut single_obs = vec![0.0; obs_len];
        single_env.reset(50 + i as u64, &mut single_obs).unwrap();

        let single_action = vec![3; action_nvec.len()];
        let outcome = single_env.step(&single_action, &mut single_obs).unwrap();

        for j in 0..obs_len {
            assert_abs_diff_eq!(vec_obs[i * obs_len + j], single_obs[j], epsilon = 1e-5);
        }

        assert_abs_diff_eq!(f64::from(rewards[i]), outcome.reward, epsilon = 1e-5);
        assert_eq!(terminated[i], outcome.terminated);
        assert_eq!(truncated[i], outcome.truncated);
    }
}

#[test]
fn num_envs_accessor() {
    let scenario = common::make_scenario();
    let config = EnvConfig::default();
    let vec_env = VectorEnv::new(&scenario, &config, 5).unwrap();

    assert_eq!(vec_env.num_envs(), 5);
}

#[test]
fn first_accessor() {
    let scenario = common::make_scenario();
    let config = EnvConfig::default();
    let vec_env = VectorEnv::new(&scenario, &config, 3).unwrap();

    let _first = vec_env.first();
}

#[test]
fn next_step_autoreset_after_truncation() {
    let scenario = common::make_scenario();
    let config = EnvConfig::default();
    let mut vec_env = VectorEnv::new(&scenario, &config, 1).unwrap();

    let layout = vec_env.first().layout();
    let obs_len = layout.observation_len();
    let action_nvec = layout.action_nvec();
    let episode_steps = vec_env.first().episode_steps();

    let mut obs = vec![0.0; obs_len];
    vec_env.reset(42, &mut obs).unwrap();

    let action = vec![3; action_nvec.len()];
    let mut rewards = vec![0.0; 1];
    let mut terminated = vec![false; 1];
    let mut truncated = vec![false; 1];

    for _ in 0..episode_steps {
        vec_env
            .step(
                &action,
                &mut obs,
                &mut rewards,
                &mut terminated,
                &mut truncated,
            )
            .unwrap();
    }

    assert!(truncated[0]);

    let mut obs_ep2 = vec![0.0; obs_len];
    let mut rewards_ep2 = vec![0.0; 1];
    let mut terminated_ep2 = vec![false; 1];
    let mut truncated_ep2 = vec![false; 1];

    vec_env
        .step(
            &action,
            &mut obs_ep2,
            &mut rewards_ep2,
            &mut terminated_ep2,
            &mut truncated_ep2,
        )
        .unwrap();

    assert_abs_diff_eq!(rewards_ep2[0], 0.0);
    assert!(!terminated_ep2[0]);
    assert!(!truncated_ep2[0]);

    let mut single_env = Env::new(&scenario, &config).unwrap();
    let mut single_obs = vec![0.0; obs_len];
    single_env.reset(42 + 1, &mut single_obs).unwrap();

    for j in 0..obs_len {
        assert_abs_diff_eq!(obs_ep2[j], single_obs[j], epsilon = 1e-5);
    }
}

#[test]
fn step_rewards_buffer_length_error() {
    let scenario = common::make_scenario();
    let config = EnvConfig::default();
    let mut vec_env = VectorEnv::new(&scenario, &config, 2).unwrap();

    let layout = vec_env.first().layout();
    let obs_len = layout.observation_len();
    let action_nvec = layout.action_nvec();

    let mut obs = vec![0.0; 2 * obs_len];
    vec_env.reset(42, &mut obs).unwrap();

    let action = vec![3; 2 * action_nvec.len()];
    let mut rewards = vec![0.0; 1];
    let mut terminated = vec![false; 2];
    let mut truncated = vec![false; 2];

    let result = vec_env.step(
        &action,
        &mut obs,
        &mut rewards,
        &mut terminated,
        &mut truncated,
    );
    assert!(matches!(result, Err(EnvError::BufferLength { .. })));
}

#[test]
fn step_terminated_buffer_length_error() {
    let scenario = common::make_scenario();
    let config = EnvConfig::default();
    let mut vec_env = VectorEnv::new(&scenario, &config, 2).unwrap();

    let layout = vec_env.first().layout();
    let obs_len = layout.observation_len();
    let action_nvec = layout.action_nvec();

    let mut obs = vec![0.0; 2 * obs_len];
    vec_env.reset(42, &mut obs).unwrap();

    let action = vec![3; 2 * action_nvec.len()];
    let mut rewards = vec![0.0; 2];
    let mut terminated = vec![false; 1];
    let mut truncated = vec![false; 2];

    let result = vec_env.step(
        &action,
        &mut obs,
        &mut rewards,
        &mut terminated,
        &mut truncated,
    );
    assert!(matches!(result, Err(EnvError::BufferLength { .. })));
}

#[test]
fn step_truncated_buffer_length_error() {
    let scenario = common::make_scenario();
    let config = EnvConfig::default();
    let mut vec_env = VectorEnv::new(&scenario, &config, 2).unwrap();

    let layout = vec_env.first().layout();
    let obs_len = layout.observation_len();
    let action_nvec = layout.action_nvec();

    let mut obs = vec![0.0; 2 * obs_len];
    vec_env.reset(42, &mut obs).unwrap();

    let action = vec![3; 2 * action_nvec.len()];
    let mut rewards = vec![0.0; 2];
    let mut terminated = vec![false; 2];
    let mut truncated = vec![false; 1];

    let result = vec_env.step(
        &action,
        &mut obs,
        &mut rewards,
        &mut terminated,
        &mut truncated,
    );
    assert!(matches!(result, Err(EnvError::BufferLength { .. })));
}
