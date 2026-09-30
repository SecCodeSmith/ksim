mod common;

use ksim_bench::policy::Policy;
use ksim_bench::runner;
use ksim_env::layout::EnvConfig;

fn are_maps_equal_or_nan(
    env_metrics: &std::collections::BTreeMap<String, f64>,
    ref_metrics: &std::collections::BTreeMap<String, f64>,
) -> bool {
    if env_metrics.len() != ref_metrics.len() {
        return false;
    }

    for (key, &env_val) in env_metrics {
        match ref_metrics.get(key) {
            Some(&ref_val) => {
                if env_val.is_nan() && ref_val.is_nan() {
                    continue;
                }
                if (env_val - ref_val).abs() > 1e-9 {
                    return false;
                }
            }
            None => return false,
        }
    }

    true
}

#[test]
fn env_parity_with_ca_seed_1() {
    let scenario = common::make_scenario();
    let config = EnvConfig::default();
    let mut env = ksim_env::Env::new(&scenario, &config).unwrap();

    let mut obs = vec![0.0; env.layout().observation_len()];
    env.reset(1, &mut obs).unwrap();

    let k = env.layout().max_replica_step;
    let zero_delta_action: Vec<u32> = (0..env.layout().action_nvec().len()).map(|_| k).collect();

    for _ in 0..env.episode_steps() {
        let _ = env.step(&zero_delta_action, &mut obs);
    }

    let env_metrics = env.episode_metrics();

    let reference = runner::run_episode(&scenario, Policy::Ca, 1)
        .expect("reference episode should run")
        .metrics;

    assert!(are_maps_equal_or_nan(&env_metrics, &reference));
}

#[test]
fn env_parity_with_ca_seed_7() {
    let scenario = common::make_scenario();
    let config = EnvConfig::default();
    let mut env = ksim_env::Env::new(&scenario, &config).unwrap();

    let mut obs = vec![0.0; env.layout().observation_len()];
    env.reset(7, &mut obs).unwrap();

    let k = env.layout().max_replica_step;
    let zero_delta_action: Vec<u32> = (0..env.layout().action_nvec().len()).map(|_| k).collect();

    for _ in 0..env.episode_steps() {
        let _ = env.step(&zero_delta_action, &mut obs);
    }

    let env_metrics = env.episode_metrics();

    let reference = runner::run_episode(&scenario, Policy::Ca, 7)
        .expect("reference episode should run")
        .metrics;

    assert!(are_maps_equal_or_nan(&env_metrics, &reference));
}
