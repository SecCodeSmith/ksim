mod common;

use approx::assert_abs_diff_eq;
use ksim_core::time::{SimDuration, SimTime};
use ksim_metrics::{
    metric::{Slo, SloPercentile, standard_metrics},
    report::{EpisodeReport, evaluate},
    trace::TickSample,
};
use std::collections::BTreeMap;

const EPSILON: f64 = 0.001;

#[test]
fn evaluate_returns_all_metrics_keyed_by_name() {
    // Arrange: simple trace with one tick
    let tick_duration = SimDuration::from_secs(15);
    let ticks = vec![TickSample {
        time: SimTime::from_secs(0),
        services: vec![ksim_metrics::trace::ServiceSample::default()],
        nodes: 1,
        hourly_cost: 0.1,
        replica_changes: 0,
        evictions: 0,
        node_changes: 0,
    }];

    let trace = common::episode_trace(tick_duration, ticks);
    let slo = Slo {
        latency_ms: 200.0,
        percentile: SloPercentile::P95,
        max_error_rate: 0.01,
        availability_target: 0.999,
    };
    let metrics = standard_metrics(slo);

    // Act
    let result = evaluate(&trace, &metrics);

    // Assert: result is a BTreeMap with string keys
    assert!(!result.is_empty());
    for (key, value) in &result {
        assert!(!key.is_empty());
        assert!(value.is_finite() || value.is_nan());
    }
}

#[test]
fn standard_metrics_includes_all_required_metrics() {
    // Arrange: create standard metrics
    let slo = Slo {
        latency_ms: 200.0,
        percentile: SloPercentile::P95,
        max_error_rate: 0.01,
        availability_target: 0.999,
    };

    let metrics = standard_metrics(slo);
    let metric_names: Vec<&str> = metrics.iter().map(|m| m.name()).collect();

    // Assert: all required metric names are present
    // Availability metrics
    assert!(metric_names.contains(&"availability"));
    assert!(metric_names.contains(&"slo_compliance"));
    assert!(metric_names.contains(&"availability_nines"));
    assert!(metric_names.contains(&"error_budget_burn"));
    assert!(metric_names.contains(&"longest_slo_violation_s"));

    // Elasticity metrics (6 total)
    assert!(metric_names.contains(&"under_provisioning_accuracy"));
    assert!(metric_names.contains(&"over_provisioning_accuracy"));
    assert!(metric_names.contains(&"under_provisioning_timeshare"));
    assert!(metric_names.contains(&"over_provisioning_timeshare"));
    assert!(metric_names.contains(&"jitter_per_hour"));
    assert!(metric_names.contains(&"instability"));

    // Cost metrics
    assert!(metric_names.contains(&"cost"));
    assert!(metric_names.contains(&"node_hours"));
    assert!(metric_names.contains(&"cpu_request_utilization"));

    // Operational metrics (5 total)
    assert!(metric_names.contains(&"scaling_actions"));
    assert!(metric_names.contains(&"pending_pod_seconds"));
    assert!(metric_names.contains(&"oom_events"));
    assert!(metric_names.contains(&"evictions"));
    assert!(metric_names.contains(&"mean_replicas"));
}

#[test]
fn standard_metrics_names_are_unique() {
    // Arrange: create standard metrics
    let slo = Slo {
        latency_ms: 200.0,
        percentile: SloPercentile::P95,
        max_error_rate: 0.01,
        availability_target: 0.999,
    };

    let metrics = standard_metrics(slo);
    let metric_names: Vec<&str> = metrics.iter().map(|m| m.name()).collect();

    // Assert: no duplicate names
    let mut seen = std::collections::HashSet::new();
    for name in metric_names {
        assert!(seen.insert(name), "Duplicate metric name: {name}");
    }
}

#[test]
fn episode_report_json_roundtrip() {
    // Arrange: create a report
    let mut metrics_map = BTreeMap::new();
    metrics_map.insert("availability".to_string(), 0.99);
    metrics_map.insert("cost".to_string(), 15.5);
    metrics_map.insert("mean_replicas".to_string(), 5.0);

    let report = EpisodeReport {
        scenario: "test_scenario".to_string(),
        policy: "test_policy".to_string(),
        seed: 12345,
        metrics: metrics_map,
    };

    // Act: serialize and deserialize
    let json = serde_json::to_string(&report).expect("Failed to serialize");
    let deserialized: EpisodeReport = serde_json::from_str(&json).expect("Failed to deserialize");

    // Assert: values are preserved
    assert_eq!(deserialized.scenario, "test_scenario");
    assert_eq!(deserialized.policy, "test_policy");
    assert_eq!(deserialized.seed, 12345);
    assert_eq!(deserialized.metrics.len(), 3);
    assert_abs_diff_eq!(
        deserialized.metrics["availability"],
        0.99,
        epsilon = EPSILON
    );
    assert_abs_diff_eq!(deserialized.metrics["cost"], 15.5, epsilon = EPSILON);
    assert_abs_diff_eq!(
        deserialized.metrics["mean_replicas"],
        5.0,
        epsilon = EPSILON
    );
}

#[test]
fn episode_report_with_p99_slo() {
    // Arrange: create standard metrics with P99 percentile
    let slo = Slo {
        latency_ms: 500.0,
        percentile: SloPercentile::P99,
        max_error_rate: 0.001,
        availability_target: 0.9999,
    };

    let metrics = standard_metrics(slo);
    let tick_duration = SimDuration::from_secs(15);
    let ticks = vec![TickSample {
        time: SimTime::from_secs(0),
        services: vec![ksim_metrics::trace::ServiceSample::default()],
        nodes: 1,
        hourly_cost: 0.1,
        replica_changes: 0,
        evictions: 0,
        node_changes: 0,
    }];

    let trace = common::episode_trace(tick_duration, ticks);

    // Act: evaluate all metrics
    let result = evaluate(&trace, &metrics);

    // Assert: should have all metrics including those that vary by percentile
    assert!(result.contains_key("slo_compliance"));
    assert!(result.contains_key("error_budget_burn"));
    assert!(result.contains_key("longest_slo_violation_s"));
}
