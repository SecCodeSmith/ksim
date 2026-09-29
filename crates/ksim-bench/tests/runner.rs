use ksim_bench::policy::Policy;
use ksim_bench::runner::{run_episode, run_seeds};
use ksim_bench::scenario::Scenario;

#[test]
fn run_episode_k8s_has_correct_metadata() {
    let text = include_str!("fixtures/tiny.toml");
    let scenario = Scenario::from_toml_str(text).expect("failed to parse tiny.toml");
    let report = run_episode(&scenario, Policy::K8s, 1).expect("failed to run episode");
    assert_eq!(report.scenario, "tiny");
    assert_eq!(report.policy, "k8s");
    assert_eq!(report.seed, 1);
}

#[test]
fn run_episode_contains_all_standard_metric_names() {
    let text = include_str!("fixtures/tiny.toml");
    let scenario = Scenario::from_toml_str(text).expect("failed to parse tiny.toml");
    let report = run_episode(&scenario, Policy::K8s, 1).expect("failed to run episode");
    assert!(!report.metrics.is_empty());
}

#[test]
fn run_episode_availability_in_valid_range() {
    let text = include_str!("fixtures/tiny.toml");
    let scenario = Scenario::from_toml_str(text).expect("failed to parse tiny.toml");
    let report = run_episode(&scenario, Policy::K8s, 1).expect("failed to run episode");
    if let Some(&availability) = report.metrics.get("availability") {
        assert!(availability >= 0.0);
        assert!(availability <= 1.0);
    }
}

#[test]
fn same_seed_produces_identical_report() {
    let text = include_str!("fixtures/tiny.toml");
    let scenario = Scenario::from_toml_str(text).expect("failed to parse tiny.toml");
    let report1 = run_episode(&scenario, Policy::K8s, 1).expect("failed to run episode");
    let report2 = run_episode(&scenario, Policy::K8s, 1).expect("failed to run episode");
    assert_eq!(report1, report2);
}

#[test]
fn run_seeds_returns_correct_seed_sequence() {
    let text = include_str!("fixtures/tiny.toml");
    let scenario = Scenario::from_toml_str(text).expect("failed to parse tiny.toml");
    let reports = run_seeds(&scenario, Policy::K8s, 3, 2).expect("failed to run seeds");
    assert_eq!(reports.len(), 2);
    assert_eq!(reports[0].seed, 3);
    assert_eq!(reports[1].seed, 4);
}

#[test]
fn oracle_availability_greater_or_equal_static_average_on_tiny() {
    let text = include_str!("fixtures/tiny.toml");
    let scenario = Scenario::from_toml_str(text).expect("failed to parse tiny.toml");
    let oracle_report =
        run_episode(&scenario, Policy::Oracle, 1).expect("failed to run oracle episode");
    let static_avg_report = run_episode(&scenario, Policy::StaticAverage, 1)
        .expect("failed to run static-average episode");

    let oracle_avail = oracle_report
        .metrics
        .get("availability")
        .copied()
        .unwrap_or(0.0);
    let static_avg_avail = static_avg_report
        .metrics
        .get("availability")
        .copied()
        .unwrap_or(0.0);

    assert!(oracle_avail >= static_avg_avail);
}
