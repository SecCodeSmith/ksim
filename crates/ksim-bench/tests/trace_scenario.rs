use ksim_bench::policy::Policy;
use ksim_bench::runner::run_episode;
use ksim_bench::scenario::{Scenario, ScenarioError};
use ksim_core::ids::ServiceId;
use ksim_workload::trace::{ServiceSeries, TraceError, encode};
use std::fs;
use std::path::PathBuf;
use std::process;

fn temp_trace_dir(test_name: &str) -> PathBuf {
    let pid = process::id();
    let temp_dir = std::env::temp_dir();
    temp_dir.join(format!("ksim-trace-{pid}-{test_name}"))
}

fn create_test_trace() -> Result<Vec<u8>, TraceError> {
    // Create a 40-tick trace with tick = 15 seconds
    // api: cpu rising from 250 to 2000, rate = cpu / 50
    // batch: constant cpu 1000, rate = cpu / 50
    let tick_seconds = 15u32;
    let tick_count = 40usize;

    // API CPU: rising from 250 to 2000 over 40 ticks
    let mut api_cpu = Vec::new();
    for i in 0..tick_count {
        #[allow(clippy::cast_precision_loss)]
        let progress = i as f32 / (tick_count - 1) as f32;
        let cpu = 250.0 + progress * (2000.0 - 250.0);
        api_cpu.push(cpu);
    }

    // API rate: cpu / 50
    let api_rate: Vec<f32> = api_cpu.iter().map(|c| c / 50.0).collect();

    // API memory: constant 256 MiB * 1024 * 1024 bytes
    let api_memory: Vec<f32> = vec![256.0 * 1024.0 * 1024.0; tick_count];

    // Batch CPU: constant 1000
    let batch_cpu: Vec<f32> = vec![1000.0; tick_count];

    // Batch rate: cpu / 50
    let batch_rate: Vec<f32> = batch_cpu.iter().map(|c| c / 50.0).collect();

    // Batch memory: constant 256 MiB
    let batch_memory: Vec<f32> = vec![256.0 * 1024.0 * 1024.0; tick_count];

    let services = vec![
        ServiceSeries {
            name: "api".to_string(),
            request_rate: api_rate,
            cpu_millis: api_cpu,
            memory_bytes: api_memory,
        },
        ServiceSeries {
            name: "batch".to_string(),
            request_rate: batch_rate,
            cpu_millis: batch_cpu,
            memory_bytes: batch_memory,
        },
    ];

    encode(tick_seconds, &services)
}

fn create_scenario_toml(trace_file_name: &str) -> String {
    format!(
        r#"name = "trace_test"
duration_s = 600
tick_s = 15

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
name = "api"
requests_cpu_millis = 500
requests_memory_mib = 512
limits_cpu_millis = 1000
limits_memory_mib = 1024
initial_replicas = 2
cpu_millis_per_request = 50.0
memory_base_mib = 256.0
memory_mib_per_rps = 0.0

[services.curve]
kind = "trace"
service = "api"

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

[[services]]
name = "batch"
requests_cpu_millis = 500
requests_memory_mib = 512
limits_cpu_millis = 1000
limits_memory_mib = 1024
initial_replicas = 1
cpu_millis_per_request = 50.0
memory_base_mib = 256.0
memory_mib_per_rps = 0.0

[services.curve]
kind = "trace"
service = "batch"

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

[trace]
path = "{trace_file_name}"
"#
    )
}

#[test]
fn from_toml_file_with_relative_trace_path() {
    let test_name = "from_toml_file_relative";
    let dir = temp_trace_dir(test_name);
    fs::create_dir_all(&dir).expect("failed to create temp dir");

    let trace_data = create_test_trace().expect("failed to create trace");
    let trace_file = dir.join("test.ksimtrace");
    fs::write(&trace_file, trace_data).expect("failed to write trace file");

    let scenario_toml = create_scenario_toml("test.ksimtrace");
    let scenario_file = dir.join("scenario.toml");
    fs::write(&scenario_file, scenario_toml).expect("failed to write scenario file");

    let result = Scenario::from_toml_file(&scenario_file);
    assert!(result.is_ok());
    let scenario = result.expect("failed to load scenario");
    assert_eq!(scenario.name, "trace_test");
}

#[test]
fn from_toml_str_with_absolute_trace_path() {
    let test_name = "from_toml_str_absolute";
    let dir = temp_trace_dir(test_name);
    fs::create_dir_all(&dir).expect("failed to create temp dir");

    let trace_data = create_test_trace().expect("failed to create trace");
    let trace_file = dir.join("test.ksimtrace");
    fs::write(&trace_file, &trace_data).expect("failed to write trace file");

    let absolute_path = trace_file
        .to_str()
        .expect("failed to convert path to string");
    let scenario_toml = create_scenario_toml(absolute_path);

    let result = Scenario::from_toml_str(&scenario_toml);
    assert!(result.is_ok());
}

#[test]
fn workload_demand_matches_trace_api_at_tick_zero() {
    let test_name = "workload_api_tick_zero";
    let dir = temp_trace_dir(test_name);
    fs::create_dir_all(&dir).expect("failed to create temp dir");

    let trace_data = create_test_trace().expect("failed to create trace");
    let trace_file = dir.join("test.ksimtrace");
    fs::write(&trace_file, &trace_data).expect("failed to write trace file");

    let scenario_toml = create_scenario_toml("test.ksimtrace");
    let scenario_file = dir.join("scenario.toml");
    fs::write(&scenario_file, scenario_toml).expect("failed to write scenario file");

    let scenario = Scenario::from_toml_file(&scenario_file).expect("failed to load scenario");
    let workload = scenario.workload(0);

    // At tick 0, API CPU should be 250 (start of the ramp)
    let demand = workload.demand(ServiceId(0), ksim_core::time::SimTime::from_secs(0));
    // CPU 250 -> rate of 250/50 = 5.0
    assert!((demand.cpu_millis - 250.0).abs() < 0.01);
}

#[test]
fn missing_trace_file_returns_io_error() {
    let test_name = "missing_trace_file";
    let dir = temp_trace_dir(test_name);
    fs::create_dir_all(&dir).expect("failed to create temp dir");

    let scenario_toml = create_scenario_toml("/nonexistent/trace.ksimtrace");
    let scenario_file = dir.join("scenario.toml");
    fs::write(&scenario_file, scenario_toml).expect("failed to write scenario file");

    let result = Scenario::from_toml_file(&scenario_file);
    assert!(result.is_err());
    match result {
        Err(ScenarioError::Trace(TraceError::Io(_))) => {}
        Err(e) => panic!("expected TraceError::Io, got {e:?}"),
        Ok(_) => panic!("expected error but got success"),
    }
}

#[test]
fn unknown_service_in_trace_returns_error() {
    let test_name = "unknown_service";
    let dir = temp_trace_dir(test_name);
    fs::create_dir_all(&dir).expect("failed to create temp dir");

    let trace_data = create_test_trace().expect("failed to create trace");
    let trace_file = dir.join("test.ksimtrace");
    fs::write(&trace_file, trace_data).expect("failed to write trace file");

    let unknown_service_toml = r#"name = "trace_test"
duration_s = 600
tick_s = 15

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
name = "unknown_service"
requests_cpu_millis = 500
requests_memory_mib = 512
limits_cpu_millis = 1000
limits_memory_mib = 1024
initial_replicas = 2
cpu_millis_per_request = 50.0
memory_base_mib = 256.0
memory_mib_per_rps = 0.0

[services.curve]
kind = "trace"
service = "unknown_service"

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

[trace]
path = "test.ksimtrace"
"#;

    let scenario_file = dir.join("scenario.toml");
    fs::write(&scenario_file, unknown_service_toml).expect("failed to write scenario file");

    let result = Scenario::from_toml_file(&scenario_file);
    assert!(result.is_err());
    match result {
        Err(ScenarioError::Trace(TraceError::UnknownService(_)) | ScenarioError::Invalid(_)) => {
        }
        Err(e) => panic!("expected UnknownService or Invalid error, got {e:?}"),
        Ok(_) => panic!("expected error but got success"),
    }
}

#[test]
fn mixing_trace_and_constant_curves_returns_invalid_error() {
    let test_name = "mixed_curves";
    let dir = temp_trace_dir(test_name);
    fs::create_dir_all(&dir).expect("failed to create temp dir");

    let trace_data = create_test_trace().expect("failed to create trace");
    let trace_file = dir.join("test.ksimtrace");
    fs::write(&trace_file, trace_data).expect("failed to write trace file");

    let mixed_toml = r#"name = "trace_test"
duration_s = 600
tick_s = 15

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
name = "api"
requests_cpu_millis = 500
requests_memory_mib = 512
limits_cpu_millis = 1000
limits_memory_mib = 1024
initial_replicas = 2
cpu_millis_per_request = 50.0
memory_base_mib = 256.0
memory_mib_per_rps = 0.0

[services.curve]
kind = "trace"
service = "api"

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

[[services]]
name = "constant_service"
requests_cpu_millis = 500
requests_memory_mib = 512
limits_cpu_millis = 1000
limits_memory_mib = 1024
initial_replicas = 1
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

[trace]
path = "test.ksimtrace"
"#;

    let scenario_file = dir.join("scenario.toml");
    fs::write(&scenario_file, mixed_toml).expect("failed to write scenario file");

    let result = Scenario::from_toml_file(&scenario_file);
    assert!(result.is_err());
    match result {
        Err(ScenarioError::Invalid(_)) => {}
        Err(e) => panic!("expected Invalid error, got {e:?}"),
        Ok(_) => panic!("expected error but got success"),
    }
}

#[test]
fn trace_curve_without_trace_table_returns_invalid_error() {
    let trace_curve_no_table_toml = r#"name = "trace_test"
duration_s = 600
tick_s = 15

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
name = "api"
requests_cpu_millis = 500
requests_memory_mib = 512
limits_cpu_millis = 1000
limits_memory_mib = 1024
initial_replicas = 2
cpu_millis_per_request = 50.0
memory_base_mib = 256.0
memory_mib_per_rps = 0.0

[services.curve]
kind = "trace"
service = "api"

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

    let result = Scenario::from_toml_str(trace_curve_no_table_toml);
    assert!(result.is_err());
    match result {
        Err(ScenarioError::Invalid(_)) => {}
        Err(e) => panic!("expected Invalid error, got {e:?}"),
        Ok(_) => panic!("expected error but got success"),
    }
}

#[test]
fn trace_compress_zero_returns_invalid_error() {
    let test_name = "compress_zero";
    let dir = temp_trace_dir(test_name);
    fs::create_dir_all(&dir).expect("failed to create temp dir");

    let trace_data = create_test_trace().expect("failed to create trace");
    let trace_file = dir.join("test.ksimtrace");
    fs::write(&trace_file, trace_data).expect("failed to write trace file");

    let compress_zero_toml = r#"name = "trace_test"
duration_s = 600
tick_s = 15

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
name = "api"
requests_cpu_millis = 500
requests_memory_mib = 512
limits_cpu_millis = 1000
limits_memory_mib = 1024
initial_replicas = 2
cpu_millis_per_request = 50.0
memory_base_mib = 256.0
memory_mib_per_rps = 0.0

[services.curve]
kind = "trace"
service = "api"

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

[trace]
path = "test.ksimtrace"
compress = 0.0
"#;

    let scenario_file = dir.join("scenario.toml");
    fs::write(&scenario_file, compress_zero_toml).expect("failed to write scenario file");

    let result = Scenario::from_toml_file(&scenario_file);
    assert!(result.is_err());
}

#[test]
fn run_episode_returns_availability_in_valid_range() {
    let test_name = "episode_availability";
    let dir = temp_trace_dir(test_name);
    fs::create_dir_all(&dir).expect("failed to create temp dir");

    let trace_data = create_test_trace().expect("failed to create trace");
    let trace_file = dir.join("test.ksimtrace");
    fs::write(&trace_file, trace_data).expect("failed to write trace file");

    let scenario_toml = create_scenario_toml("test.ksimtrace");
    let scenario_file = dir.join("scenario.toml");
    fs::write(&scenario_file, scenario_toml).expect("failed to write scenario file");

    let scenario = Scenario::from_toml_file(&scenario_file).expect("failed to load scenario");
    let report = run_episode(&scenario, Policy::K8s, 0).expect("failed to run episode");

    let availability = report
        .metrics
        .get("availability")
        .expect("availability metric not found");
    assert!(*availability >= 0.0 && *availability <= 1.0);
}

#[test]
fn same_seed_twice_produces_identical_report() {
    let test_name = "same_seed_deterministic";
    let dir = temp_trace_dir(test_name);
    fs::create_dir_all(&dir).expect("failed to create temp dir");

    let trace_data = create_test_trace().expect("failed to create trace");
    let trace_file = dir.join("test.ksimtrace");
    fs::write(&trace_file, &trace_data).expect("failed to write trace file");

    let scenario_toml = create_scenario_toml("test.ksimtrace");
    let scenario_file = dir.join("scenario.toml");
    fs::write(&scenario_file, scenario_toml).expect("failed to write scenario file");

    let scenario = Scenario::from_toml_file(&scenario_file).expect("failed to load scenario");

    let report1 = run_episode(&scenario, Policy::K8s, 42).expect("failed to run episode");
    let report2 = run_episode(&scenario, Policy::K8s, 42).expect("failed to run episode");

    assert_eq!(report1.metrics, report2.metrics);
}

#[test]
fn different_seeds_same_workload_demand_for_trace() {
    let test_name = "trace_ignores_seed";
    let dir = temp_trace_dir(test_name);
    fs::create_dir_all(&dir).expect("failed to create temp dir");

    let trace_data = create_test_trace().expect("failed to create trace");
    let trace_file = dir.join("test.ksimtrace");
    fs::write(&trace_file, &trace_data).expect("failed to write trace file");

    let scenario_toml = create_scenario_toml("test.ksimtrace");
    let scenario_file = dir.join("scenario.toml");
    fs::write(&scenario_file, scenario_toml).expect("failed to write scenario file");

    let scenario = Scenario::from_toml_file(&scenario_file).expect("failed to load scenario");

    let workload_seed1 = scenario.workload(1);
    let workload_seed2 = scenario.workload(999);

    let service_id = ServiceId(0);
    let t = ksim_core::time::SimTime::from_secs(0);

    let demand1 = workload_seed1.demand(service_id, t);
    let demand2 = workload_seed2.demand(service_id, t);

    assert!((demand1.cpu_millis - demand2.cpu_millis).abs() < 0.01);
}
