mod common;

use ksim_env::layout::EnvConfig;

#[test]
fn default_reward_non_positive_and_finite() {
    let scenario = common::make_scenario();
    let config = EnvConfig::default();
    let mut env = ksim_env::Env::new(&scenario, &config).unwrap();

    let mut obs = vec![0.0; env.layout().observation_len()];
    env.reset(42, &mut obs).unwrap();

    let action = vec![3; env.layout().action_nvec().len()];

    let outcome = env.step(&action, &mut obs).unwrap();

    assert!(outcome.reward.is_finite());
    assert!(outcome.reward <= 0.0);
}
