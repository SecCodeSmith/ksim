use ksim_workload::trace::{ServiceSeries, TraceError, encode};
use std::fs;
use std::process::{Command, Stdio};

fn create_test_trace() -> Result<Vec<u8>, TraceError> {
	// Create a 40-tick trace with services "api" and "batch"
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
		r#"name = "cli_test"
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
fn run_resolves_trace_path_relative_to_scenario_file() {
	let test_name = "cli_trace_relative";
	let pid = std::process::id();
	let test_dir = std::env::temp_dir().join(format!("ksim-cli-{pid}-{test_name}"));
	fs::create_dir_all(&test_dir).expect("failed to create test directory");

	let trace_data = create_test_trace().expect("failed to create trace");
	let trace_file = test_dir.join("test.ksimtrace");
	fs::write(&trace_file, trace_data).expect("failed to write trace file");

	let scenario_toml = create_scenario_toml("test.ksimtrace");
	let scenario_file = test_dir.join("scenario.toml");
	fs::write(&scenario_file, scenario_toml).expect("failed to write scenario file");

	let out_dir = test_dir.join("output");
	fs::create_dir_all(&out_dir).expect("failed to create output directory");

	let binary_path = env!("CARGO_BIN_EXE_ksim-bench");

	let output = Command::new(binary_path)
		.args([
			"run",
			"--scenario",
			scenario_file.to_str().expect("scenario path is not UTF-8"),
			"--policy",
			"k8s",
			"--seeds",
			"1",
			"--out",
			out_dir.to_str().expect("output path is not UTF-8"),
		])
		.current_dir(std::env::temp_dir())
		.stdout(Stdio::piped())
		.stderr(Stdio::piped())
		.output()
		.expect("failed to execute binary");

	if !output.status.success() {
		let stderr = String::from_utf8_lossy(&output.stderr);
		panic!(
			"CLI command failed with status {}: {}",
			output.status, stderr
		);
	}

	let result_file = out_dir.join("cli_test-k8s.jsonl");
	assert!(
		result_file.exists(),
		"expected output file {} to exist",
		result_file.display()
	);

	fs::remove_dir_all(&test_dir).expect("failed to clean up test directory");
}
