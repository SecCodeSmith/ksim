mod common;

use approx::assert_abs_diff_eq;
use ksim_bench::scenario::Scenario;
use ksim_core::{
    observation::{Observation, PoolObservation, ServiceObservation},
    performance::ServiceSignals,
    resources::Resources,
    time::SimTime,
};
use ksim_env::{
    layout::{EnvConfig, Layout},
    observation::encode,
};

#[test]
#[allow(clippy::cast_possible_truncation)]
fn cpu_utilisation_concrete_numbers() {
    let scenario = common::make_scenario();
    let config = EnvConfig::default();
    let layout = Layout::new(&scenario, &config).unwrap();

    let obs = Observation {
        now: SimTime::from_secs(0),
        services: vec![ServiceObservation {
            desired: 10,
            ready: 5,
            pending: 0,
            unschedulable: 0,
            outdated: 0,
            requests: Resources {
                cpu_millis: 1000,
                memory_bytes: 1024 * 1024 * 1024,
            },
            limits: Resources {
                cpu_millis: 2000,
                memory_bytes: 2048 * 1024 * 1024,
            },
            signals: ServiceSignals {
                cpu_usage_millis: 2000.0,
                memory_usage_bytes: 1536.0 * 1024.0 * 1024.0,
                served_rps: 50.0,
                latency_p50_ms: 10.0,
                latency_p95_ms: 50.0,
                latency_p99_ms: 100.0,
                error_rate: 0.0,
                throttled_fraction: 0.0,
                oom: false,
            },
        }],
        pools: vec![PoolObservation::default()],
        nodes: vec![],
        node_pods: vec![],
    };

    let mut out = vec![0.0; layout.observation_len()];
    encode(&obs, &layout, &mut out);

    let expected_cpu = (2000.0 / (5.0 * 1000.0)) as f32;
    assert_abs_diff_eq!(out[0], expected_cpu, epsilon = 1e-5);
}

#[test]
#[allow(clippy::cast_possible_truncation)]
fn memory_utilisation_concrete_numbers() {
    let scenario = common::make_scenario();
    let config = EnvConfig::default();
    let layout = Layout::new(&scenario, &config).unwrap();

    let obs = Observation {
        now: SimTime::from_secs(0),
        services: vec![ServiceObservation {
            desired: 10,
            ready: 4,
            pending: 0,
            unschedulable: 0,
            outdated: 0,
            requests: Resources {
                cpu_millis: 500,
                memory_bytes: 512 * 1024 * 1024,
            },
            limits: Resources {
                cpu_millis: 1000,
                memory_bytes: 1024 * 1024 * 1024,
            },
            signals: ServiceSignals {
                cpu_usage_millis: 500.0,
                memory_usage_bytes: 1024.0 * 1024.0 * 1024.0,
                served_rps: 50.0,
                latency_p50_ms: 10.0,
                latency_p95_ms: 50.0,
                latency_p99_ms: 100.0,
                error_rate: 0.0,
                throttled_fraction: 0.0,
                oom: false,
            },
        }],
        pools: vec![PoolObservation::default()],
        nodes: vec![],
        node_pods: vec![],
    };

    let mut out = vec![0.0; layout.observation_len()];
    encode(&obs, &layout, &mut out);

    let expected_memory = (1024.0 * 1024.0 * 1024.0 / (4.0 * 512.0 * 1024.0 * 1024.0)) as f32;
    assert_abs_diff_eq!(out[1], expected_memory, epsilon = 1e-5);
}

#[test]
#[allow(clippy::cast_possible_truncation)]
fn latency_ratio_feature_index_3_p95() {
    let scenario = common::make_scenario();
    let config = EnvConfig::default();
    let layout = Layout::new(&scenario, &config).unwrap();

    let slo_latency_ms = layout.slo_latency_ms;
    let obs = Observation {
        now: SimTime::from_secs(0),
        services: vec![ServiceObservation {
            desired: 5,
            ready: 5,
            pending: 0,
            unschedulable: 0,
            outdated: 0,
            requests: Resources {
                cpu_millis: 500,
                memory_bytes: 512 * 1024 * 1024,
            },
            limits: Resources {
                cpu_millis: 1000,
                memory_bytes: 1024 * 1024 * 1024,
            },
            signals: ServiceSignals {
                cpu_usage_millis: 500.0,
                memory_usage_bytes: 256.0 * 1024.0 * 1024.0,
                served_rps: 100.0,
                latency_p50_ms: 50.0,
                latency_p95_ms: 150.0,
                latency_p99_ms: 200.0,
                error_rate: 0.0,
                throttled_fraction: 0.0,
                oom: false,
            },
        }],
        pools: vec![PoolObservation::default()],
        nodes: vec![],
        node_pods: vec![],
    };

    let mut out = vec![0.0; layout.observation_len()];
    encode(&obs, &layout, &mut out);

    let expected_latency_ratio = (150.0 / slo_latency_ms) as f32;
    assert_abs_diff_eq!(out[3], expected_latency_ratio, epsilon = 1e-5);
}

#[test]
#[allow(clippy::cast_possible_truncation)]
fn latency_ratio_feature_uses_p99_when_configured() {
    let toml = r#"
name = "test_p99"
duration_s = 300
tick_s = 15

[slo]
latency_ms = 150.0
percentile = "P99"
max_error_rate = 0.01
availability_target = 0.999

[[pools]]
name = "standard"
cpu_millis = 4000
memory_mib = 16384
min_nodes = 1
max_nodes = 10
initial_nodes = 2
price_per_hour = 0.25

[[services]]
name = "api"
requests_cpu_millis = 500
requests_memory_mib = 512
limits_cpu_millis = 1000
limits_memory_mib = 1024
initial_replicas = 4
cpu_millis_per_request = 40.0
memory_base_mib = 200.0
memory_mib_per_rps = 0.1

[services.curve]
kind = "constant"
rate = 100.0

[services.performance]
kind = "queueing"
cpu_millis_per_request = 40.0
timeout_ms = 15000.0
ca2 = 1.2
cs2 = 0.8

[services.hpa]
min_replicas = 2
max_replicas = 20
target_cpu_utilization = 0.7
"#;
    let scenario = Scenario::from_toml_str(toml).unwrap();
    let config = EnvConfig::default();
    let layout = Layout::new(&scenario, &config).unwrap();

    assert!(layout.slo_uses_p99);

    let slo_latency_ms = layout.slo_latency_ms;
    let obs = Observation {
        now: SimTime::from_secs(0),
        services: vec![ServiceObservation {
            desired: 5,
            ready: 5,
            pending: 0,
            unschedulable: 0,
            outdated: 0,
            requests: Resources {
                cpu_millis: 500,
                memory_bytes: 512 * 1024 * 1024,
            },
            limits: Resources {
                cpu_millis: 1000,
                memory_bytes: 1024 * 1024 * 1024,
            },
            signals: ServiceSignals {
                cpu_usage_millis: 500.0,
                memory_usage_bytes: 256.0 * 1024.0 * 1024.0,
                served_rps: 100.0,
                latency_p50_ms: 50.0,
                latency_p95_ms: 140.0,
                latency_p99_ms: 200.0,
                error_rate: 0.0,
                throttled_fraction: 0.0,
                oom: false,
            },
        }],
        pools: vec![PoolObservation::default()],
        nodes: vec![],
        node_pods: vec![],
    };

    let mut out = vec![0.0; layout.observation_len()];
    encode(&obs, &layout, &mut out);

    let expected_latency_ratio = (200.0 / slo_latency_ms) as f32;
    assert_abs_diff_eq!(out[3], expected_latency_ratio, epsilon = 1e-5);
}

#[test]
fn error_rate_feature_index_4() {
    let scenario = common::make_scenario();
    let config = EnvConfig::default();
    let layout = Layout::new(&scenario, &config).unwrap();

    let obs = Observation {
        now: SimTime::from_secs(0),
        services: vec![ServiceObservation {
            desired: 5,
            ready: 5,
            pending: 0,
            unschedulable: 0,
            outdated: 0,
            requests: Resources {
                cpu_millis: 500,
                memory_bytes: 512 * 1024 * 1024,
            },
            limits: Resources {
                cpu_millis: 1000,
                memory_bytes: 1024 * 1024 * 1024,
            },
            signals: ServiceSignals {
                cpu_usage_millis: 500.0,
                memory_usage_bytes: 256.0 * 1024.0 * 1024.0,
                served_rps: 100.0,
                latency_p50_ms: 50.0,
                latency_p95_ms: 100.0,
                latency_p99_ms: 150.0,
                error_rate: 0.035,
                throttled_fraction: 0.0,
                oom: false,
            },
        }],
        pools: vec![PoolObservation::default()],
        nodes: vec![],
        node_pods: vec![],
    };

    let mut out = vec![0.0; layout.observation_len()];
    encode(&obs, &layout, &mut out);

    assert_abs_diff_eq!(out[4], 0.035, epsilon = 1e-5);
}

#[test]
#[allow(clippy::cast_precision_loss)]
fn ready_desired_pending_outdated_divided_by_max_replicas() {
    let scenario = common::make_scenario();
    let config = EnvConfig::default();
    let layout = Layout::new(&scenario, &config).unwrap();

    let obs = Observation {
        now: SimTime::from_secs(0),
        services: vec![ServiceObservation {
            desired: 8,
            ready: 6,
            pending: 1,
            unschedulable: 0,
            outdated: 2,
            requests: Resources {
                cpu_millis: 500,
                memory_bytes: 512 * 1024 * 1024,
            },
            limits: Resources {
                cpu_millis: 1000,
                memory_bytes: 1024 * 1024 * 1024,
            },
            signals: ServiceSignals {
                cpu_usage_millis: 500.0,
                memory_usage_bytes: 256.0 * 1024.0 * 1024.0,
                served_rps: 100.0,
                latency_p50_ms: 50.0,
                latency_p95_ms: 100.0,
                latency_p99_ms: 150.0,
                error_rate: 0.0,
                throttled_fraction: 0.0,
                oom: false,
            },
        }],
        pools: vec![PoolObservation::default()],
        nodes: vec![],
        node_pods: vec![],
    };

    let mut out = vec![0.0; layout.observation_len()];
    encode(&obs, &layout, &mut out);

    let max_replicas = layout.services[0].max_replicas as f32;
    assert_abs_diff_eq!(out[5], 6.0 / max_replicas, epsilon = 1e-5);
    assert_abs_diff_eq!(out[6], 8.0 / max_replicas, epsilon = 1e-5);
    assert_abs_diff_eq!(out[7], 1.0 / max_replicas, epsilon = 1e-5);
    assert_abs_diff_eq!(out[8], 2.0 / max_replicas, epsilon = 1e-5);
}

#[test]
fn pool_features_ready_provisioning_draining_over_max_nodes() {
    let scenario = common::make_scenario();
    let config = EnvConfig::default();
    let layout = Layout::new(&scenario, &config).unwrap();

    let obs = Observation {
        now: SimTime::from_secs(0),
        services: vec![ServiceObservation::default()],
        pools: vec![PoolObservation {
            ready: 3,
            provisioning: 2,
            draining: 1,
            allocatable: Resources {
                cpu_millis: 8000,
                memory_bytes: 32768 * 1024 * 1024,
            },
            min_nodes: 1,
            max_nodes: 10,
        }],
        nodes: vec![],
        node_pods: vec![],
    };

    let mut out = vec![0.0; layout.observation_len()];
    encode(&obs, &layout, &mut out);

    let service_features_end = layout.service_slots * 10;
    let max_nodes = 10.0;

    assert_abs_diff_eq!(out[service_features_end], 3.0 / max_nodes, epsilon = 1e-5);
    assert_abs_diff_eq!(
        out[service_features_end + 1],
        2.0 / max_nodes,
        epsilon = 1e-5
    );
    assert_abs_diff_eq!(
        out[service_features_end + 2],
        1.0 / max_nodes,
        epsilon = 1e-5
    );
}

#[test]
fn pool_features_cpu_requested_over_allocatable() {
    let scenario = common::make_scenario();
    let config = EnvConfig::default();
    let layout = Layout::new(&scenario, &config).unwrap();

    let obs = Observation {
        now: SimTime::from_secs(0),
        services: vec![ServiceObservation::default()],
        pools: vec![PoolObservation {
            ready: 2,
            provisioning: 0,
            draining: 0,
            allocatable: Resources {
                cpu_millis: 8000,
                memory_bytes: 32768 * 1024 * 1024,
            },
            min_nodes: 1,
            max_nodes: 10,
        }],
        nodes: vec![],
        node_pods: vec![],
    };

    let mut out = vec![0.0; layout.observation_len()];
    encode(&obs, &layout, &mut out);

    let service_features_end = layout.service_slots * 10;

    assert_abs_diff_eq!(out[service_features_end + 3], 0.0, epsilon = 1e-5);
}

#[test]
fn pool_features_mask_one() {
    let scenario = common::make_scenario();
    let config = EnvConfig::default();
    let layout = Layout::new(&scenario, &config).unwrap();

    let obs = Observation {
        now: SimTime::from_secs(0),
        services: vec![ServiceObservation::default()],
        pools: vec![PoolObservation::default()],
        nodes: vec![],
        node_pods: vec![],
    };

    let mut out = vec![0.0; layout.observation_len()];
    encode(&obs, &layout, &mut out);

    let service_features_end = layout.service_slots * 10;
    assert_abs_diff_eq!(out[service_features_end + 4], 1.0, epsilon = 1e-5);
}

#[test]
fn max_action_increases_service_0_desired_over_steps() {
    let scenario = common::make_scenario();
    let config = EnvConfig::default();
    let mut env = ksim_env::Env::new(&scenario, &config).unwrap();

    let mut obs_initial = vec![0.0; env.layout().observation_len()];
    env.reset(42, &mut obs_initial).unwrap();

    let initial_desired = obs_initial[6];

    let mut obs_after_steps = vec![0.0; env.layout().observation_len()];
    for _ in 0..3 {
        let max_actions: Vec<u32> = env.layout().action_nvec().iter().map(|&b| b - 1).collect();
        let _ = env.step(&max_actions, &mut obs_after_steps);
    }

    assert!(obs_after_steps[6] > initial_desired);
}

#[test]
fn max_action_service_0_desired_stays_clamped_at_max() {
    let scenario = common::make_scenario();
    let config = EnvConfig::default();
    let mut env = ksim_env::Env::new(&scenario, &config).unwrap();

    let mut obs = vec![0.0; env.layout().observation_len()];
    env.reset(42, &mut obs).unwrap();

    #[allow(clippy::cast_precision_loss)]
    let max_replicas = env.layout().services[0].max_replicas as f32;

    let max_actions: Vec<u32> = env.layout().action_nvec().iter().map(|&b| b - 1).collect();

    for _ in 0..20 {
        let _ = env.step(&max_actions, &mut obs);
    }

    assert!(obs[6] <= max_replicas);
}
