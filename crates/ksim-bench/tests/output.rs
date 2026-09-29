use std::collections::BTreeMap;

use ksim_bench::output::{write_csv, write_json_lines};
use ksim_bench::policy::Policy;
use ksim_bench::runner::run_episode;
use ksim_bench::scenario::Scenario;
use ksim_metrics::report::EpisodeReport;

#[test]
fn write_json_lines_outputs_one_per_report() {
    let text = include_str!("fixtures/tiny.toml");
    let scenario = Scenario::from_toml_str(text).expect("failed to parse tiny.toml");
    let report_seed_1 = run_episode(&scenario, Policy::K8s, 1).expect("failed to run episode");
    let report_seed_2 = run_episode(&scenario, Policy::K8s, 2).expect("failed to run episode");

    let reports = vec![report_seed_1, report_seed_2];
    let mut output = Vec::new();
    write_json_lines(&reports, &mut output).expect("failed to write json lines");

    let output_str = String::from_utf8(output).expect("invalid utf8");
    let line_count = output_str.lines().count();
    assert_eq!(line_count, 2);
}

#[test]
fn write_json_lines_output_is_valid_json() {
    let text = include_str!("fixtures/tiny.toml");
    let scenario = Scenario::from_toml_str(text).expect("failed to parse tiny.toml");
    let report = run_episode(&scenario, Policy::K8s, 1).expect("failed to run episode");

    let reports = vec![report.clone()];
    let mut output = Vec::new();
    write_json_lines(&reports, &mut output).expect("failed to write json lines");

    let output_str = String::from_utf8(output).expect("invalid utf8");
    let line = output_str.lines().next().expect("no output lines");
    let parsed: EpisodeReport = serde_json::from_str(line).expect("failed to parse json line");
    assert_eq!(parsed.seed, 1);
}

#[test]
fn write_json_lines_round_trips_through_serde() {
    let text = include_str!("fixtures/tiny.toml");
    let scenario = Scenario::from_toml_str(text).expect("failed to parse tiny.toml");
    let original = run_episode(&scenario, Policy::K8s, 1).expect("failed to run episode");

    let reports = vec![original.clone()];
    let mut output = Vec::new();
    write_json_lines(&reports, &mut output).expect("failed to write json lines");

    let output_str = String::from_utf8(output).expect("invalid utf8");
    let line = output_str.lines().next().expect("no output lines");
    let parsed: EpisodeReport = serde_json::from_str(line).expect("failed to parse json line");

    assert_eq!(parsed.scenario, original.scenario);
    assert_eq!(parsed.policy, original.policy);
    assert_eq!(parsed.seed, original.seed);
    assert_eq!(parsed.metrics, original.metrics);
}

#[test]
fn write_csv_has_correct_header() {
    let text = include_str!("fixtures/tiny.toml");
    let scenario = Scenario::from_toml_str(text).expect("failed to parse tiny.toml");
    let report = run_episode(&scenario, Policy::K8s, 1).expect("failed to run episode");

    let reports = vec![report];
    let mut output = Vec::new();
    write_csv(&reports, &mut output).expect("failed to write csv");

    let output_str = String::from_utf8(output).expect("invalid utf8");
    let lines: Vec<&str> = output_str.lines().collect();
    assert!(!lines.is_empty());

    let header = lines[0];
    assert!(header.starts_with("scenario,policy,seed,"));
}

#[test]
fn write_csv_metric_names_are_sorted() {
    let text = include_str!("fixtures/tiny.toml");
    let scenario = Scenario::from_toml_str(text).expect("failed to parse tiny.toml");
    let report = run_episode(&scenario, Policy::K8s, 1).expect("failed to run episode");

    let reports = vec![report];
    let mut output = Vec::new();
    write_csv(&reports, &mut output).expect("failed to write csv");

    let output_str = String::from_utf8(output).expect("invalid utf8");
    let header = output_str.lines().next().expect("no header");

    let parts: Vec<&str> = header.split(',').collect();
    let metric_names = &parts[3..];
    let mut sorted_names = metric_names.to_vec();
    sorted_names.sort_unstable();
    assert_eq!(metric_names, &sorted_names[..]);
}

#[test]
fn write_csv_mismatched_metric_names_error() {
    let report_with_a = EpisodeReport {
        scenario: "test".to_string(),
        policy: "k8s".to_string(),
        seed: 1,
        metrics: BTreeMap::from([("metric_a".to_string(), 1.0)]),
    };

    let report_with_b = EpisodeReport {
        scenario: "test".to_string(),
        policy: "k8s".to_string(),
        seed: 2,
        metrics: BTreeMap::from([("metric_b".to_string(), 2.0)]),
    };

    let reports = vec![report_with_a, report_with_b];
    let mut output = Vec::new();
    let result = write_csv(&reports, &mut output);
    assert!(result.is_err());
}
