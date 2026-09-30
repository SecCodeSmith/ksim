use std::fs;
use std::path::Path;
use std::process::Command;

#[test]
fn trajectory_scenario_with_hpa_policy_succeeds() {
    let temp_dir =
        std::env::temp_dir().join(format!("ksim_bench_trajectory_test_{}", std::process::id()));
    fs::create_dir_all(&temp_dir).expect("Failed to create temp dir");

    let nested_out = temp_dir.join("nested");
    fs::create_dir_all(&nested_out).expect("Failed to create nested dir");

    let scenario_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/tiny.toml");
    let out_file = nested_out.join("trajectory.csv");

    let output = Command::new(env!("CARGO_BIN_EXE_ksim-bench"))
        .arg("trajectory")
        .arg("--scenario")
        .arg(&scenario_path)
        .arg("--policy")
        .arg("hpa")
        .arg("--seed")
        .arg("1")
        .arg("--out")
        .arg(&out_file)
        .output()
        .expect("Failed to execute trajectory command");

    assert!(
        output.status.success(),
        "trajectory command should succeed, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    assert!(
        out_file.exists(),
        "trajectory output file should exist at {}",
        out_file.display()
    );

    let content = fs::read_to_string(&out_file).expect("Failed to read output file");
    let lines: Vec<&str> = content.lines().collect();

    // Should have header + (steps × services) rows
    // tiny.toml: 600s / 15s = 40 steps, 1 service (web)
    assert!(!lines.is_empty(), "should have at least header line");
    assert!(
        lines[0].starts_with("time_s,service,"),
        "first line should be header"
    );

    fs::remove_dir_all(&temp_dir).expect("Failed to clean up temp dir");
}

#[test]
fn trajectory_with_unknown_policy_exits_nonzero() {
    let temp_dir = std::env::temp_dir().join(format!(
        "ksim_bench_trajectory_bad_policy_{}",
        std::process::id()
    ));
    fs::create_dir_all(&temp_dir).expect("Failed to create temp dir");

    let scenario_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/tiny.toml");
    let out_file = temp_dir.join("trajectory.csv");

    let output = Command::new(env!("CARGO_BIN_EXE_ksim-bench"))
        .arg("trajectory")
        .arg("--scenario")
        .arg(&scenario_path)
        .arg("--policy")
        .arg("nonexistent-policy")
        .arg("--seed")
        .arg("1")
        .arg("--out")
        .arg(&out_file)
        .output()
        .expect("Failed to execute trajectory command");

    assert!(
        !output.status.success(),
        "trajectory command with unknown policy should fail"
    );

    fs::remove_dir_all(&temp_dir).ok();
}

#[test]
fn trajectory_output_file_row_count_matches_steps_times_services() {
    let temp_dir = std::env::temp_dir().join(format!(
        "ksim_bench_trajectory_rowcount_{}",
        std::process::id()
    ));
    fs::create_dir_all(&temp_dir).expect("Failed to create temp dir");

    let scenario_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/tiny.toml");
    let out_file = temp_dir.join("trajectory.csv");

    let output = Command::new(env!("CARGO_BIN_EXE_ksim-bench"))
        .arg("trajectory")
        .arg("--scenario")
        .arg(&scenario_path)
        .arg("--policy")
        .arg("k8s")
        .arg("--seed")
        .arg("0")
        .arg("--out")
        .arg(&out_file)
        .output()
        .expect("Failed to execute trajectory command");

    assert!(output.status.success(), "trajectory command should succeed");

    // Read the scenario to calculate expected rows
    let scenario_text = include_str!("fixtures/tiny.toml");
    let scenario = ksim_bench::scenario::Scenario::from_toml_str(scenario_text)
        .expect("failed to parse scenario");

    let expected_steps = scenario.steps();
    let expected_services = scenario.services.len();
    #[allow(clippy::cast_possible_truncation)]
    let expected_data_rows = expected_steps as usize * expected_services;

    let content = fs::read_to_string(&out_file).expect("Failed to read output file");
    let lines: Vec<&str> = content.lines().collect();

    // Header + data rows
    let expected_total_lines = 1 + expected_data_rows;
    assert_eq!(
        lines.len(),
        expected_total_lines,
        "row count should be header + (steps × services)"
    );

    fs::remove_dir_all(&temp_dir).expect("Failed to clean up temp dir");
}

#[test]
fn trajectory_header_line_has_expected_columns() {
    let temp_dir = std::env::temp_dir().join(format!(
        "ksim_bench_trajectory_header_{}",
        std::process::id()
    ));
    fs::create_dir_all(&temp_dir).expect("Failed to create temp dir");

    let scenario_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/tiny.toml");
    let out_file = temp_dir.join("trajectory.csv");

    let output = Command::new(env!("CARGO_BIN_EXE_ksim-bench"))
        .arg("trajectory")
        .arg("--scenario")
        .arg(&scenario_path)
        .arg("--policy")
        .arg("hpa")
        .arg("--seed")
        .arg("42")
        .arg("--out")
        .arg(&out_file)
        .output()
        .expect("Failed to execute trajectory command");

    assert!(output.status.success(), "trajectory command should succeed");

    let content = fs::read_to_string(&out_file).expect("Failed to read output file");
    let header = content.lines().next().expect("no header line");

    assert_eq!(
        header,
        "time_s,service,demand_rps,desired,ready,cpu_usage_millis,latency_p95_ms,error_rate",
        "header should match expected columns"
    );

    fs::remove_dir_all(&temp_dir).expect("Failed to clean up temp dir");
}
