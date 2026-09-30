mod common;

use approx::assert_abs_diff_eq;
use ksim_env::{
    EnvError,
    env::Env,
    layout::EnvConfig,
    reward::{RewardContext, RewardFn},
};

#[test]
fn step_before_reset_needs_reset() {
    let scenario = common::make_scenario();
    let config = EnvConfig::default();
    let mut env = Env::new(&scenario, &config).unwrap();

    let mut obs = vec![0.0; env.layout().observation_len()];
    let action = vec![3; env.layout().action_nvec().len()];

    let result = env.step(&action, &mut obs);
    assert!(matches!(result, Err(EnvError::NeedsReset)));
}

#[test]
fn reset_writes_finite_observation() {
    let scenario = common::make_scenario();
    let config = EnvConfig::default();
    let mut env = Env::new(&scenario, &config).unwrap();

    let mut obs = vec![0.0; env.layout().observation_len()];
    env.reset(42, &mut obs).unwrap();

    for &value in &obs {
        assert!(value.is_finite());
        assert!(value >= -1.0);
        assert!(value <= 10.0);
    }
}

#[test]
fn reset_buffer_length_error() {
    let scenario = common::make_scenario();
    let config = EnvConfig::default();
    let mut env = Env::new(&scenario, &config).unwrap();

    let mut obs = vec![0.0; env.layout().observation_len() - 1];
    let result = env.reset(42, &mut obs);

    assert!(matches!(result, Err(EnvError::BufferLength { .. })));
}

#[test]
fn step_buffer_length_error() {
    let scenario = common::make_scenario();
    let config = EnvConfig::default();
    let mut env = Env::new(&scenario, &config).unwrap();

    let mut obs = vec![0.0; env.layout().observation_len()];
    env.reset(42, &mut obs).unwrap();

    let mut obs_too_short = vec![0.0; env.layout().observation_len() - 1];
    let action = vec![3; env.layout().action_nvec().len()];

    let result = env.step(&action, &mut obs_too_short);
    assert!(matches!(result, Err(EnvError::BufferLength { .. })));
}

#[test]
fn determinism_same_seed_same_actions() {
    let scenario = common::make_scenario();
    let config = EnvConfig::default();
    let mut env1 = Env::new(&scenario, &config).unwrap();
    let mut env2 = Env::new(&scenario, &config).unwrap();

    let mut obs1 = vec![0.0; env1.layout().observation_len()];
    let mut obs2 = vec![0.0; env2.layout().observation_len()];

    env1.reset(42, &mut obs1).unwrap();
    env2.reset(42, &mut obs2).unwrap();

    for i in 0..obs1.len() {
        assert_abs_diff_eq!(obs1[i], obs2[i], epsilon = 1e-5);
    }

    let action = vec![3; env1.layout().action_nvec().len()];

    let outcome1 = env1.step(&action, &mut obs1).unwrap();
    let outcome2 = env2.step(&action, &mut obs2).unwrap();

    for i in 0..obs1.len() {
        assert_abs_diff_eq!(obs1[i], obs2[i], epsilon = 1e-5);
    }

    assert_abs_diff_eq!(outcome1.reward, outcome2.reward, epsilon = 1e-5);
}

#[test]
fn episode_steps_matches_scenario() {
    let scenario = common::make_scenario();
    let config = EnvConfig::default();
    let env = Env::new(&scenario, &config).unwrap();

    assert_eq!(env.episode_steps(), scenario.steps());
}

#[test]
fn truncated_at_episode_steps() {
    let scenario = common::make_scenario();
    let config = EnvConfig::default();
    let mut env = Env::new(&scenario, &config).unwrap();

    let mut obs = vec![0.0; env.layout().observation_len()];
    env.reset(42, &mut obs).unwrap();

    let action = vec![3; env.layout().action_nvec().len()];
    let episode_steps = env.episode_steps();

    for step in 0..episode_steps {
        let outcome = env.step(&action, &mut obs).unwrap();

        if step < episode_steps - 1 {
            assert!(!outcome.truncated);
        }
    }

    let outcome = env.step(&action, &mut obs).unwrap();
    assert!(outcome.truncated);
}

#[test]
fn never_terminated() {
    let scenario = common::make_scenario();
    let config = EnvConfig::default();
    let mut env = Env::new(&scenario, &config).unwrap();

    let mut obs = vec![0.0; env.layout().observation_len()];
    env.reset(42, &mut obs).unwrap();

    let action = vec![3; env.layout().action_nvec().len()];
    let episode_steps = env.episode_steps();

    for _ in 0..=episode_steps {
        let outcome = env.step(&action, &mut obs).unwrap();
        assert!(!outcome.terminated);
    }
}

#[test]
fn step_after_truncation_needs_reset() {
    let scenario = common::make_scenario();
    let config = EnvConfig::default();
    let mut env = Env::new(&scenario, &config).unwrap();

    let mut obs = vec![0.0; env.layout().observation_len()];
    env.reset(42, &mut obs).unwrap();

    let action = vec![3; env.layout().action_nvec().len()];
    let episode_steps = env.episode_steps();

    for _ in 0..=episode_steps {
        let _ = env.step(&action, &mut obs);
    }

    let result = env.step(&action, &mut obs);
    assert!(matches!(result, Err(EnvError::NeedsReset)));
}

#[test]
fn reset_after_truncation_works() {
    let scenario = common::make_scenario();
    let config = EnvConfig::default();
    let mut env = Env::new(&scenario, &config).unwrap();

    let mut obs = vec![0.0; env.layout().observation_len()];
    env.reset(42, &mut obs).unwrap();

    let action = vec![3; env.layout().action_nvec().len()];
    let episode_steps = env.episode_steps();

    for _ in 0..=episode_steps {
        let _ = env.step(&action, &mut obs);
    }

    let result = env.reset(43, &mut obs);
    assert!(result.is_ok());
}

#[test]
fn episode_metrics_empty_before_reset() {
    let scenario = common::make_scenario();
    let config = EnvConfig::default();
    let env = Env::new(&scenario, &config).unwrap();

    let metrics = env.episode_metrics();
    assert!(metrics.is_empty());
}

#[test]
fn episode_metrics_contains_keys_after_steps() {
    let scenario = common::make_scenario();
    let config = EnvConfig::default();
    let mut env = Env::new(&scenario, &config).unwrap();

    let mut obs = vec![0.0; env.layout().observation_len()];
    env.reset(42, &mut obs).unwrap();

    let action = vec![3; env.layout().action_nvec().len()];
    let _ = env.step(&action, &mut obs);

    let metrics = env.episode_metrics();
    assert!(!metrics.is_empty());
    assert!(metrics.contains_key("Cost") || metrics.contains_key("cost"));
}

#[test]
fn with_reward_constant_reward() {
    #[derive(Debug, Clone)]
    struct ConstantReward(f64);

    impl RewardFn for ConstantReward {
        fn reward(
            &mut self,
            _sample: &ksim_metrics::trace::TickSample,
            _context: &RewardContext,
        ) -> f64 {
            self.0
        }
    }

    let scenario = common::make_scenario();
    let config = EnvConfig::default();
    let env = Env::new(&scenario, &config).unwrap();
    let _env = env.with_reward(Box::new(ConstantReward(7.0)));
}

#[test]
fn action_out_of_range_error() {
    let scenario = common::make_scenario();
    let config = EnvConfig::default();
    let mut env = Env::new(&scenario, &config).unwrap();

    let mut obs = vec![0.0; env.layout().observation_len()];
    env.reset(42, &mut obs).unwrap();

    let mut action = vec![3; env.layout().action_nvec().len()];
    action[0] = env.layout().action_nvec()[0];

    let result = env.step(&action, &mut obs);
    assert!(matches!(result, Err(EnvError::ActionOutOfRange { .. })));
}

#[test]
fn layout_accessor() {
    let scenario = common::make_scenario();
    let config = EnvConfig::default();
    let env = Env::new(&scenario, &config).unwrap();

    let layout = env.layout();
    assert_abs_diff_eq!(layout.slo_latency_ms, scenario.slo.latency_ms);
}
