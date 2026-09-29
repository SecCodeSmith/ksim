use std::process::Command;

#[test]
fn test_version_flag_succeeds() {
    let output = Command::new(env!("CARGO_BIN_EXE_ksim-bench"))
        .arg("--version")
        .output()
        .expect("Failed to execute binary");

    assert!(
        output.status.success(),
        "Expected --version to exit successfully"
    );

    let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8 in stdout");
    let expected = format!("ksim-bench {}", env!("CARGO_PKG_VERSION"));
    assert_eq!(stdout.trim(), expected, "Version output mismatch");
}

#[test]
fn test_help_flag_succeeds() {
    let output = Command::new(env!("CARGO_BIN_EXE_ksim-bench"))
        .arg("--help")
        .output()
        .expect("Failed to execute binary");

    assert!(
        output.status.success(),
        "Expected --help to exit successfully"
    );

    let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8 in stdout");
    assert!(
        stdout.contains("Usage"),
        "Help output should contain 'Usage'"
    );
}

#[test]
fn test_unknown_subcommand_fails() {
    let output = Command::new(env!("CARGO_BIN_EXE_ksim-bench"))
        .arg("definitely-not-a-command")
        .output()
        .expect("Failed to execute binary");

    assert!(
        !output.status.success(),
        "Expected unknown subcommand to exit with failure"
    );
}
