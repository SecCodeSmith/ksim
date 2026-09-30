use ksim_bench::scenario::Scenario;

#[allow(dead_code)]
pub fn make_scenario() -> Scenario {
    let toml = r#"
name = "test_scenario"
duration_s = 300
tick_s = 15

[slo]
latency_ms = 200.0
percentile = "P95"
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

[[services]]
name = "worker"
requests_cpu_millis = 1000
requests_memory_mib = 1024
limits_cpu_millis = 2000
limits_memory_mib = 2048
initial_replicas = 2
cpu_millis_per_request = 200.0
memory_base_mib = 512.0
memory_mib_per_rps = 0.5

[services.curve]
kind = "constant"
rate = 50.0

[services.performance]
kind = "queueing"
cpu_millis_per_request = 200.0
timeout_ms = 30000.0
ca2 = 1.5
cs2 = 1.0

[services.hpa]
min_replicas = 1
max_replicas = 15
target_cpu_utilization = 0.65
"#;
    Scenario::from_toml_str(toml).expect("scenario should be valid")
}
