use std::fs;
use std::process::Command;

#[test]
fn list_policies_prints_all_eight_names() {
    let output = Command::new(env!("CARGO_BIN_EXE_ksim-bench"))
        .arg("list-policies")
        .output()
        .expect("Failed to execute list-policies");

    assert!(
        output.status.success(),
        "list-policies should exit with success"
    );

    let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8 in stdout");
    let lines: Vec<&str> = stdout.trim().lines().collect();
    assert_eq!(lines.len(), 8);

    let mut names: Vec<&str> = lines;
    names.sort_unstable();
    let expected = vec![
        "ca",
        "hpa",
        "k8s",
        "k8s-full",
        "oracle",
        "static-average",
        "static-peak",
        "vpa",
    ];
    assert_eq!(names, expected);
}

#[test]
fn run_scenario_with_hpa_policy_succeeds() {
    let temp_dir = std::env::temp_dir().join(format!(
        "ksim_bench_test_{}",
        std::process::id()
    ));
    fs::create_dir_all(&temp_dir).expect("Failed to create temp dir");

    let scenario_path = std::path::absolute("crates/ksim-bench/tests/fixtures/tiny.toml")
        .expect("Failed to resolve scenario path");

    let output = Command::new(env!("CARGO_BIN_EXE_ksim-bench"))
        .arg("run")
        .arg("--scenario")
        .arg(&scenario_path)
        .arg("--policy")
        .arg("hpa")
        .arg("--seeds")
        .arg("2")
        .arg("--out")
        .arg(&temp_dir)
        .output()
        .expect("Failed to execute run command");

    assert!(
        output.status.success(),
        "run command should succeed, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let jsonl_path = temp_dir.join("tiny-hpa.jsonl");
    assert!(jsonl_path.exists(), "tiny-hpa.jsonl should exist");

    let csv_path = temp_dir.join("tiny-hpa.csv");
    assert!(csv_path.exists(), "tiny-hpa.csv should exist");

    let jsonl_content =
        fs::read_to_string(&jsonl_path).expect("Failed to read jsonl file");
    let jsonl_lines = jsonl_content.lines().count();
    assert_eq!(jsonl_lines, 2, "jsonl should have 2 lines");

    fs::remove_dir_all(&temp_dir).expect("Failed to clean up temp dir");
}

#[test]
fn run_with_bad_policy_exits_nonzero() {
    let temp_dir = std::env::temp_dir().join(format!(
        "ksim_bench_test_bad_policy_{}",
        std::process::id()
    ));
    fs::create_dir_all(&temp_dir).expect("Failed to create temp dir");

    let scenario_path = std::path::absolute("crates/ksim-bench/tests/fixtures/tiny.toml")
        .expect("Failed to resolve scenario path");

    let output = Command::new(env!("CARGO_BIN_EXE_ksim-bench"))
        .arg("run")
        .arg("--scenario")
        .arg(&scenario_path)
        .arg("--policy")
        .arg("nonexistent-policy")
        .arg("--seeds")
        .arg("2")
        .arg("--out")
        .arg(&temp_dir)
        .output()
        .expect("Failed to execute run command");

    assert!(
        !output.status.success(),
        "run command with bad policy should fail"
    );

    fs::remove_dir_all(&temp_dir).ok();
}
