use ksim_bench::scenario::Scenario;

#[test]
fn parses_tiny_fixture() {
    let text = include_str!("fixtures/tiny.toml");
    let scenario = Scenario::from_toml_str(text).expect("failed to parse tiny.toml");
    assert_eq!(scenario.name, "tiny");
    assert_eq!(scenario.duration_s, 600);
    assert_eq!(scenario.tick_s, 15);
    assert_eq!(scenario.pools.len(), 1);
    assert_eq!(scenario.services.len(), 1);
}

#[test]
fn tiny_fixture_steps_is_40() {
    let text = include_str!("fixtures/tiny.toml");
    let scenario = Scenario::from_toml_str(text).expect("failed to parse tiny.toml");
    assert_eq!(scenario.steps(), 40);
}

#[test]
fn unknown_field_returns_error() {
    let text = r#"
name = "test"
duration_s = 100
tick_s = 10
unknown_field = "value"

[slo]
latency_ms = 250.0
percentile = "P95"
max_error_rate = 0.01
availability_target = 0.999

[[pools]]
name = "default"
cpu_millis = 4000
memory_mib = 16384
min_nodes = 1
max_nodes = 5
initial_nodes = 1
price_per_hour = 0.2

[[services]]
name = "web"
requests_cpu_millis = 500
requests_memory_mib = 512
limits_cpu_millis = 1000
limits_memory_mib = 1024
initial_replicas = 2
cpu_millis_per_request = 50.0
memory_base_mib = 256.0
memory_mib_per_rps = 0.0

[services.curve]
kind = "constant"
rate = 5.0

[services.performance]
kind = "queueing"
cpu_millis_per_request = 50.0
timeout_ms = 10000.0
ca2 = 1.0
cs2 = 1.0

[services.hpa]
min_replicas = 1
max_replicas = 10
target_cpu_utilization = 0.6
"#;
    assert!(Scenario::from_toml_str(text).is_err());
}

#[test]
fn zero_tick_returns_invalid_error() {
    let text = r#"
name = "test"
duration_s = 100
tick_s = 0

[slo]
latency_ms = 250.0
percentile = "P95"
max_error_rate = 0.01
availability_target = 0.999

[[pools]]
name = "default"
cpu_millis = 4000
memory_mib = 16384
min_nodes = 1
max_nodes = 5
initial_nodes = 1
price_per_hour = 0.2

[[services]]
name = "web"
requests_cpu_millis = 500
requests_memory_mib = 512
limits_cpu_millis = 1000
limits_memory_mib = 1024
initial_replicas = 2
cpu_millis_per_request = 50.0
memory_base_mib = 256.0
memory_mib_per_rps = 0.0

[services.curve]
kind = "constant"
rate = 5.0

[services.performance]
kind = "queueing"
cpu_millis_per_request = 50.0
timeout_ms = 10000.0
ca2 = 1.0
cs2 = 1.0

[services.hpa]
min_replicas = 1
max_replicas = 10
target_cpu_utilization = 0.6
"#;
    let err = Scenario::from_toml_str(text).expect_err("expected error for zero tick");
    assert!(matches!(
        err,
        ksim_bench::scenario::ScenarioError::Invalid(_)
    ));
}

#[test]
fn no_services_returns_error() {
    let text = r#"
name = "test"
duration_s = 100
tick_s = 10

[slo]
latency_ms = 250.0
percentile = "P95"
max_error_rate = 0.01
availability_target = 0.999

[[pools]]
name = "default"
cpu_millis = 4000
memory_mib = 16384
min_nodes = 1
max_nodes = 5
initial_nodes = 1
price_per_hour = 0.2
"#;
    assert!(Scenario::from_toml_str(text).is_err());
}

#[test]
fn build_simulation_succeeds_with_seed_7() {
    let text = include_str!("fixtures/tiny.toml");
    let scenario = Scenario::from_toml_str(text).expect("failed to parse tiny.toml");
    let _ = scenario
        .build_simulation(7)
        .expect("failed to build simulation");
}

#[test]
fn workload_services_count_is_1() {
    let text = include_str!("fixtures/tiny.toml");
    let scenario = Scenario::from_toml_str(text).expect("failed to parse tiny.toml");
    let workload = scenario.workload(7);
    assert_eq!(workload.services(), 1);
}

#[test]
fn workload_demand_at_t0_for_seed_7() {
    use approx::assert_abs_diff_eq;
    let text = include_str!("fixtures/tiny.toml");
    let scenario = Scenario::from_toml_str(text).expect("failed to parse tiny.toml");
    let workload = scenario.workload(7);
    let demand = workload.demand(ksim_core::ids::ServiceId(0), ksim_core::time::SimTime::ZERO);
    assert_abs_diff_eq!(demand.request_rate, 5.0);
    assert_abs_diff_eq!(demand.cpu_millis, 250.0);
}
