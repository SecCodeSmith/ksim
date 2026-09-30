use std::str::FromStr;

use ksim_bench::policy::{Policy, UnknownPolicy};
use ksim_bench::scenario::Scenario;

#[test]
fn all_policies_have_correct_names() {
    assert_eq!(Policy::Hpa.name(), "hpa");
    assert_eq!(Policy::Vpa.name(), "vpa");
    assert_eq!(Policy::Ca.name(), "ca");
    assert_eq!(Policy::K8s.name(), "k8s");
    assert_eq!(Policy::K8sFull.name(), "k8s-full");
    assert_eq!(Policy::StaticPeak.name(), "static-peak");
    assert_eq!(Policy::StaticAverage.name(), "static-average");
    assert_eq!(Policy::Oracle.name(), "oracle");
}

#[test]
fn policy_names_are_exactly_eight() {
    assert_eq!(Policy::ALL.len(), 8);
}

#[test]
fn all_policy_names_round_trip_through_from_str() {
    for policy in &Policy::ALL {
        let name = policy.name();
        let parsed = Policy::from_str(name).expect("failed to parse policy name");
        assert_eq!(parsed, *policy);
    }
}

#[test]
fn unknown_policy_returns_error() {
    let err = Policy::from_str("nope").expect_err("expected UnknownPolicy error");
    assert_eq!(err.0, "nope");
    assert!(matches!(err, UnknownPolicy(_)));
}

#[test]
fn build_hpa_on_tiny() {
    let text = include_str!("fixtures/tiny.toml");
    let scenario = Scenario::from_toml_str(text).expect("failed to parse tiny.toml");
    let _ = Policy::Hpa.build(&scenario, 7);
}

#[test]
fn build_vpa_on_tiny() {
    let text = include_str!("fixtures/tiny.toml");
    let scenario = Scenario::from_toml_str(text).expect("failed to parse tiny.toml");
    let _ = Policy::Vpa.build(&scenario, 7);
}

#[test]
fn build_ca_on_tiny() {
    let text = include_str!("fixtures/tiny.toml");
    let scenario = Scenario::from_toml_str(text).expect("failed to parse tiny.toml");
    let _ = Policy::Ca.build(&scenario, 7);
}

#[test]
fn build_k8s_on_tiny() {
    let text = include_str!("fixtures/tiny.toml");
    let scenario = Scenario::from_toml_str(text).expect("failed to parse tiny.toml");
    let _ = Policy::K8s.build(&scenario, 7);
}

#[test]
fn build_k8s_full_on_tiny() {
    let text = include_str!("fixtures/tiny.toml");
    let scenario = Scenario::from_toml_str(text).expect("failed to parse tiny.toml");
    let _ = Policy::K8sFull.build(&scenario, 7);
}

#[test]
fn build_static_peak_on_tiny() {
    let text = include_str!("fixtures/tiny.toml");
    let scenario = Scenario::from_toml_str(text).expect("failed to parse tiny.toml");
    let _ = Policy::StaticPeak.build(&scenario, 7);
}

#[test]
fn build_static_average_on_tiny() {
    let text = include_str!("fixtures/tiny.toml");
    let scenario = Scenario::from_toml_str(text).expect("failed to parse tiny.toml");
    let _ = Policy::StaticAverage.build(&scenario, 7);
}

#[test]
fn build_oracle_on_tiny() {
    let text = include_str!("fixtures/tiny.toml");
    let scenario = Scenario::from_toml_str(text).expect("failed to parse tiny.toml");
    let _ = Policy::Oracle.build(&scenario, 7);
}
