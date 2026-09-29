use ksim_bench::scenario::{Scenario, ScenarioError};

const TINY_TOML: &str = include_str!("fixtures/tiny.toml");

#[test]
fn valid_scenario_from_tiny_toml() {
    let result = Scenario::from_toml_str(TINY_TOML);
    assert!(result.is_ok(), "tiny.toml should parse successfully");
}

#[test]
fn pool_cpu_millis_zero_is_invalid() {
    let invalid = TINY_TOML.replace("cpu_millis = 4000", "cpu_millis = 0");
    let result = Scenario::from_toml_str(&invalid);
    assert!(
        matches!(result, Err(ScenarioError::Invalid(_))),
        "pool cpu_millis = 0 should be invalid"
    );
}

#[test]
fn pool_memory_mib_zero_is_invalid() {
    let invalid = TINY_TOML.replace("memory_mib = 16384", "memory_mib = 0");
    let result = Scenario::from_toml_str(&invalid);
    assert!(
        matches!(result, Err(ScenarioError::Invalid(_))),
        "pool memory_mib = 0 should be invalid"
    );
}

#[test]
fn pool_min_nodes_greater_than_max_nodes_is_invalid() {
    let invalid = TINY_TOML.replace("min_nodes = 1", "min_nodes = 10");
    let result = Scenario::from_toml_str(&invalid);
    assert!(
        matches!(result, Err(ScenarioError::Invalid(_))),
        "pool min_nodes > max_nodes should be invalid"
    );
}

#[test]
fn pool_initial_nodes_greater_than_max_nodes_is_invalid() {
    let invalid = TINY_TOML.replace("initial_nodes = 1", "initial_nodes = 10");
    let result = Scenario::from_toml_str(&invalid);
    assert!(
        matches!(result, Err(ScenarioError::Invalid(_))),
        "pool initial_nodes > max_nodes should be invalid"
    );
}

#[test]
fn service_requests_cpu_millis_zero_is_invalid() {
    let invalid = TINY_TOML.replace("requests_cpu_millis = 500", "requests_cpu_millis = 0");
    let result = Scenario::from_toml_str(&invalid);
    assert!(
        matches!(result, Err(ScenarioError::Invalid(_))),
        "service requests_cpu_millis = 0 should be invalid"
    );
}

#[test]
fn service_limits_cpu_millis_less_than_requests_cpu_millis_is_invalid() {
    let invalid = TINY_TOML.replace("limits_cpu_millis = 1000", "limits_cpu_millis = 250");
    let result = Scenario::from_toml_str(&invalid);
    assert!(
        matches!(result, Err(ScenarioError::Invalid(_))),
        "service limits_cpu_millis < requests_cpu_millis should be invalid"
    );
}

#[test]
fn hpa_target_cpu_utilization_zero_is_invalid() {
    let invalid = TINY_TOML.replace(
        "target_cpu_utilization = 0.6",
        "target_cpu_utilization = 0.0",
    );
    let result = Scenario::from_toml_str(&invalid);
    assert!(
        matches!(result, Err(ScenarioError::Invalid(_))),
        "hpa target_cpu_utilization = 0.0 should be invalid"
    );
}

#[test]
fn hpa_target_cpu_utilization_greater_than_one_is_invalid() {
    let invalid = TINY_TOML.replace(
        "target_cpu_utilization = 0.6",
        "target_cpu_utilization = 1.5",
    );
    let result = Scenario::from_toml_str(&invalid);
    assert!(
        matches!(result, Err(ScenarioError::Invalid(_))),
        "hpa target_cpu_utilization = 1.5 should be invalid"
    );
}

#[test]
fn hpa_min_replicas_greater_than_max_replicas_is_invalid() {
    let invalid = TINY_TOML.replace("min_replicas = 1", "min_replicas = 15");
    let result = Scenario::from_toml_str(&invalid);
    assert!(
        matches!(result, Err(ScenarioError::Invalid(_))),
        "hpa min_replicas > max_replicas should be invalid"
    );
}

#[test]
fn slo_latency_ms_zero_is_invalid() {
    let invalid = TINY_TOML.replace("latency_ms = 250.0", "latency_ms = 0.0");
    let result = Scenario::from_toml_str(&invalid);
    assert!(
        matches!(result, Err(ScenarioError::Invalid(_))),
        "slo latency_ms = 0.0 should be invalid"
    );
}

#[test]
fn slo_availability_target_one_is_invalid() {
    let invalid = TINY_TOML.replace("availability_target = 0.999", "availability_target = 1.0");
    let result = Scenario::from_toml_str(&invalid);
    assert!(
        matches!(result, Err(ScenarioError::Invalid(_))),
        "slo availability_target = 1.0 should be invalid"
    );
}

#[test]
fn queueing_cpu_millis_per_request_zero_is_invalid() {
    let invalid = TINY_TOML.replace(
        "[services.performance]\nkind = \"queueing\"\ncpu_millis_per_request = 50.0",
        "[services.performance]\nkind = \"queueing\"\ncpu_millis_per_request = 0.0",
    );
    let result = Scenario::from_toml_str(&invalid);
    assert!(
        matches!(result, Err(ScenarioError::Invalid(_))),
        "queueing cpu_millis_per_request = 0.0 should be invalid"
    );
}
