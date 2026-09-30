use ksim_bench::policy::Policy;
use ksim_bench::runner::run_episode;
use ksim_bench::scenario::Scenario;

#[test]
fn static_peak_under_provisioning_timeshare_below_threshold() {
    let text = include_str!("../../../scenarios/synthetic_step.toml");
    let scenario = Scenario::from_toml_str(text).expect("failed to parse synthetic_step.toml");
    let report = run_episode(&scenario, Policy::StaticPeak, 0).expect("failed to run episode");

    let under_provisioning_timeshare = report
        .metrics
        .get("under_provisioning_timeshare")
        .copied()
        .unwrap_or(0.0);
    assert!(
        under_provisioning_timeshare < 5.0,
        "under_provisioning_timeshare {under_provisioning_timeshare} should be < 5.0"
    );
}

#[test]
fn static_peak_slo_compliance_above_threshold() {
    let text = include_str!("../../../scenarios/synthetic_step.toml");
    let scenario = Scenario::from_toml_str(text).expect("failed to parse synthetic_step.toml");
    let report = run_episode(&scenario, Policy::StaticPeak, 0).expect("failed to run episode");

    let slo_compliance = report.metrics.get("slo_compliance").copied().unwrap_or(0.0);
    assert!(
        slo_compliance > 0.95,
        "slo_compliance {slo_compliance} should be > 0.95"
    );
}

#[test]
fn oracle_slo_compliance_above_threshold() {
    let text = include_str!("../../../scenarios/synthetic_step.toml");
    let scenario = Scenario::from_toml_str(text).expect("failed to parse synthetic_step.toml");
    let report = run_episode(&scenario, Policy::Oracle, 0).expect("failed to run episode");

    let slo_compliance = report.metrics.get("slo_compliance").copied().unwrap_or(0.0);
    assert!(
        slo_compliance > 0.95,
        "slo_compliance {slo_compliance} should be > 0.95"
    );
}

#[test]
fn static_peak_cost_greater_than_static_average() {
    let text = include_str!("../../../scenarios/synthetic_step.toml");
    let scenario = Scenario::from_toml_str(text).expect("failed to parse synthetic_step.toml");

    let peak_report = run_episode(&scenario, Policy::StaticPeak, 0).expect("failed to run episode");
    let average_report =
        run_episode(&scenario, Policy::StaticAverage, 0).expect("failed to run episode");

    let peak_cost = peak_report.metrics.get("cost").copied().unwrap_or(0.0);
    let average_cost = average_report.metrics.get("cost").copied().unwrap_or(0.0);

    assert!(
        peak_cost > average_cost,
        "peak_cost {peak_cost} should be > average_cost {average_cost}"
    );
}
