mod common;

use approx::assert_abs_diff_eq;
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
fn service_features_ten_per_slot() {
    let scenario = common::make_scenario();
    let config = EnvConfig::default();
    let layout = Layout::new(&scenario, &config).unwrap();

    let obs = Observation::default();
    let mut out = vec![0.0; layout.observation_len()];
    encode(&obs, &layout, &mut out);

    assert_eq!(out.len(), layout.observation_len());
}

#[test]
fn utilization_zero_with_no_ready_pods() {
    let scenario = common::make_scenario();
    let config = EnvConfig::default();
    let layout = Layout::new(&scenario, &config).unwrap();

    let obs = Observation {
        now: SimTime::from_secs(0),
        services: vec![ServiceObservation {
            desired: 5,
            ready: 0,
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
                cpu_usage_millis: 0.0,
                memory_usage_bytes: 0.0,
                served_rps: 0.0,
                latency_p50_ms: 0.0,
                latency_p95_ms: 0.0,
                latency_p99_ms: 0.0,
                error_rate: 0.0,
                throttled_fraction: 0.0,
                oom: false,
            },
        }],
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

    assert_abs_diff_eq!(out[0], 0.0);
}

#[test]
fn rps_encoded_as_ln_one_plus_rps() {
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
                cpu_usage_millis: 1000.0,
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

    let expected = (1.0_f32 + 100.0).ln();
    assert_abs_diff_eq!(out[2], expected, epsilon = 1e-5);
}

#[test]
fn time_features_sin_cos_at_21600() {
    let scenario = common::make_scenario();
    let config = EnvConfig::default();
    let layout = Layout::new(&scenario, &config).unwrap();

    let obs = Observation {
        now: SimTime::from_secs(21600),
        services: vec![],
        pools: vec![],
        nodes: vec![],
        node_pods: vec![],
    };

    let mut out = vec![0.0; layout.observation_len()];
    encode(&obs, &layout, &mut out);

    let service_features_end = layout.service_slots * 10;
    let pool_features_end = service_features_end + layout.pools * 5;

    let sin_value = out[pool_features_end];
    let cos_value = out[pool_features_end + 1];

    assert_abs_diff_eq!(sin_value, 1.0, epsilon = 1e-5);
    assert_abs_diff_eq!(cos_value, 0.0, epsilon = 1e-2);
}

#[test]
fn padding_slots_all_zero() {
    let scenario = common::make_scenario();
    let config = EnvConfig {
        service_slots: Some(5),
        ..Default::default()
    };
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

    for i in 1..5 {
        for j in 0..10 {
            assert_abs_diff_eq!(out[i * 10 + j], 0.0);
        }
    }
}

#[test]
fn mask_one_for_real_service_zero_for_padding() {
    let scenario = common::make_scenario();
    let config = EnvConfig {
        service_slots: Some(5),
        ..Default::default()
    };
    let layout = Layout::new(&scenario, &config).unwrap();

    let obs = Observation {
        now: SimTime::from_secs(0),
        services: vec![ServiceObservation::default(), ServiceObservation::default()],
        pools: vec![PoolObservation::default()],
        nodes: vec![],
        node_pods: vec![],
    };

    let mut out = vec![0.0; layout.observation_len()];
    encode(&obs, &layout, &mut out);

    assert_abs_diff_eq!(out[10 - 1], 1.0);
    assert_abs_diff_eq!(out[2 * 10 - 1], 1.0);
    assert_abs_diff_eq!(out[3 * 10 - 1], 0.0);
    assert_abs_diff_eq!(out[4 * 10 - 1], 0.0);
}

#[test]
fn clipping_at_high_value() {
    let scenario = common::make_scenario();
    let config = EnvConfig::default();
    let layout = Layout::new(&scenario, &config).unwrap();

    let obs = Observation {
        now: SimTime::from_secs(0),
        services: vec![ServiceObservation {
            desired: 100,
            ready: 100,
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
                cpu_usage_millis: 1_000_000.0,
                memory_usage_bytes: 1_000_000.0,
                served_rps: 100_000.0,
                latency_p50_ms: 50.0,
                latency_p95_ms: 1500.0,
                latency_p99_ms: 2000.0,
                error_rate: 1.0,
                throttled_fraction: 0.0,
                oom: false,
            },
        }],
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

    for &value in &out {
        assert!(value <= 10.0);
    }
}
