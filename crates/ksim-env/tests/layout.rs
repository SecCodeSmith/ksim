mod common;

use approx::assert_abs_diff_eq;
use ksim_bench::scenario::Scenario;
use ksim_env::{
    EnvError,
    layout::{Control, EnvConfig, Layout},
};

#[test]
fn observation_len_basic() {
    let scenario = common::make_scenario();
    let config = EnvConfig::default();
    let layout = Layout::new(&scenario, &config).unwrap();

    let expected = scenario.services.len() * 10 + scenario.pools.len() * 5 + 2;
    assert_eq!(layout.observation_len(), expected);
}

#[test]
fn observation_len_with_service_slots() {
    let scenario = common::make_scenario();
    let config = EnvConfig {
        service_slots: Some(5),
        ..Default::default()
    };
    let layout = Layout::new(&scenario, &config).unwrap();

    let expected = 5 * 10 + scenario.pools.len() * 5 + 2;
    assert_eq!(layout.observation_len(), expected);
}

#[test]
fn action_nvec_replicas() {
    let scenario = common::make_scenario();
    let config = EnvConfig {
        max_replica_step: 3,
        control: Control::Replicas,
        ..Default::default()
    };
    let layout = Layout::new(&scenario, &config).unwrap();

    let action_nvec = layout.action_nvec();
    assert_eq!(action_nvec.len(), scenario.services.len());

    for &size in &action_nvec {
        assert_eq!(size, 2 * 3 + 1);
    }
}

#[test]
fn action_nvec_replicas_with_slots() {
    let scenario = common::make_scenario();
    let config = EnvConfig {
        max_replica_step: 2,
        control: Control::Replicas,
        service_slots: Some(4),
        ..Default::default()
    };
    let layout = Layout::new(&scenario, &config).unwrap();

    let action_nvec = layout.action_nvec();
    assert_eq!(action_nvec.len(), 4);

    for &size in &action_nvec {
        assert_eq!(size, 2 * 2 + 1);
    }
}

#[test]
fn action_nvec_replicas_and_nodes() {
    let scenario = common::make_scenario();
    let config = EnvConfig {
        max_replica_step: 3,
        control: Control::ReplicasAndNodes,
        ..Default::default()
    };
    let layout = Layout::new(&scenario, &config).unwrap();

    let action_nvec = layout.action_nvec();
    let expected_len = scenario.services.len() + scenario.pools.len();
    assert_eq!(action_nvec.len(), expected_len);

    for &size in action_nvec.iter().take(scenario.services.len()) {
        assert_eq!(size, 2 * 3 + 1);
    }

    for &size in action_nvec.iter().skip(scenario.services.len()) {
        assert_eq!(size, 3);
    }
}

#[test]
fn service_slots_error_too_small() {
    let scenario = common::make_scenario();
    let config = EnvConfig {
        service_slots: Some(1),
        ..Default::default()
    };

    let result = Layout::new(&scenario, &config);
    assert!(matches!(result, Err(EnvError::ServiceSlots { .. })));

    if let Err(EnvError::ServiceSlots { services, slots }) = result {
        assert_eq!(services, scenario.services.len());
        assert_eq!(slots, 1);
    }
}

#[test]
fn replica_bounds_from_hpa() {
    let scenario = common::make_scenario();
    let config = EnvConfig::default();
    let layout = Layout::new(&scenario, &config).unwrap();

    assert_eq!(layout.services.len(), scenario.services.len());

    for (i, service) in scenario.services.iter().enumerate() {
        assert_eq!(layout.services[i].min_replicas, service.hpa.min_replicas);
        assert_eq!(layout.services[i].max_replicas, service.hpa.max_replicas);
    }
}

#[test]
fn slo_latency_from_scenario() {
    let scenario = common::make_scenario();
    let config = EnvConfig::default();
    let layout = Layout::new(&scenario, &config).unwrap();

    assert_abs_diff_eq!(layout.slo_latency_ms, scenario.slo.latency_ms);
}

#[test]
fn slo_uses_p95_by_default() {
    let scenario = common::make_scenario();
    let config = EnvConfig::default();
    let layout = Layout::new(&scenario, &config).unwrap();

    assert!(!layout.slo_uses_p99);
}

#[test]
fn slo_uses_p99_when_configured() {
    let toml = r#"
name = "test_p99"
duration_s = 300
tick_s = 15

[slo]
latency_ms = 200.0
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
}

#[test]
fn pools_count() {
    let scenario = common::make_scenario();
    let config = EnvConfig::default();
    let layout = Layout::new(&scenario, &config).unwrap();

    assert_eq!(layout.pools, scenario.pools.len());
}

#[test]
fn service_slots_preserved() {
    let scenario = common::make_scenario();
    let config = EnvConfig {
        service_slots: Some(8),
        ..Default::default()
    };
    let layout = Layout::new(&scenario, &config).unwrap();

    assert_eq!(layout.service_slots, 8);
}

#[test]
fn max_replica_step_preserved() {
    let scenario = common::make_scenario();
    let config = EnvConfig {
        max_replica_step: 5,
        ..Default::default()
    };
    let layout = Layout::new(&scenario, &config).unwrap();

    assert_eq!(layout.max_replica_step, 5);
}

#[test]
fn control_preserved() {
    let scenario = common::make_scenario();
    let config = EnvConfig {
        control: Control::ReplicasAndNodes,
        ..Default::default()
    };
    let layout = Layout::new(&scenario, &config).unwrap();

    assert_eq!(layout.control, Control::ReplicasAndNodes);
}
