use ksim_bench::output::write_trajectory_csv;
use ksim_bench::policy::Policy;
use ksim_bench::runner::{episode_trace, run_episode};
use ksim_bench::scenario::Scenario;
use ksim_core::time::{SimDuration, SimTime};
use ksim_metrics::metric::standard_metrics;
use ksim_metrics::report::evaluate;
use ksim_metrics::trace::{EpisodeTrace, ServiceSample, TickSample};
use std::io;

#[test]
fn episode_trace_returns_trace_with_scenario_steps_ticks() {
    let text = include_str!("fixtures/tiny.toml");
    let scenario = Scenario::from_toml_str(text).expect("failed to parse tiny.toml");
    let trace = episode_trace(&scenario, Policy::Hpa, 3).expect("failed to get episode trace");

    assert_eq!(
        trace.ticks.len() as u64,
        scenario.steps(),
        "trace should have scenario.steps() ticks"
    );
}

#[test]
fn write_trajectory_csv_empty_trace_writes_header_only() {
    let trace = EpisodeTrace {
        tick: SimDuration::from_secs(15),
        ticks: Vec::new(),
    };
    let service_names = vec!["web".to_string()];

    let mut output = Vec::new();
    write_trajectory_csv(&trace, &service_names, &mut output).expect("failed to write csv");

    let output_str = String::from_utf8(output).expect("invalid utf8");
    let lines: Vec<&str> = output_str.lines().collect();

    assert_eq!(lines.len(), 1, "empty trace should have only header line");
    assert_eq!(
        lines[0],
        "time_s,service,demand_rps,desired,ready,cpu_usage_millis,latency_p95_ms,error_rate",
        "header should have expected columns"
    );
}

#[test]
#[allow(clippy::too_many_lines)]
fn write_trajectory_csv_two_ticks_two_services_produces_correct_output() {
    // Build a trace with 2 ticks and 2 services with concrete values
    let trace = EpisodeTrace {
        tick: SimDuration::from_secs(15),
        ticks: vec![
            TickSample {
                time: SimTime::from_secs(0),
                services: vec![
                    ServiceSample {
                        demand_rps: 10.0,
                        demand_cpu_millis: 500.0,
                        supply_cpu_millis: 1000.0,
                        cpu_usage_millis: 450.0,
                        served_rps: 10.0,
                        error_rate: 0.0,
                        latency_p50_ms: 50.0,
                        latency_p95_ms: 95.5,
                        latency_p99_ms: 99.0,
                        desired: 2,
                        ready: 2,
                        pending: 0,
                        oom: false,
                    },
                    ServiceSample {
                        demand_rps: 5.0,
                        demand_cpu_millis: 250.0,
                        supply_cpu_millis: 500.0,
                        cpu_usage_millis: 200.0,
                        served_rps: 5.0,
                        error_rate: 0.01,
                        latency_p50_ms: 30.0,
                        latency_p95_ms: 75.25,
                        latency_p99_ms: 99.5,
                        desired: 1,
                        ready: 1,
                        pending: 0,
                        oom: false,
                    },
                ],
                nodes: 2,
                hourly_cost: 0.4,
                replica_changes: 0,
                evictions: 0,
                node_changes: 0,
            },
            TickSample {
                time: SimTime::from_secs(15),
                services: vec![
                    ServiceSample {
                        demand_rps: 12.5,
                        demand_cpu_millis: 625.0,
                        supply_cpu_millis: 1500.0,
                        cpu_usage_millis: 550.0,
                        served_rps: 12.5,
                        error_rate: 0.0,
                        latency_p50_ms: 60.0,
                        latency_p95_ms: 100.0,
                        latency_p99_ms: 120.5,
                        desired: 3,
                        ready: 3,
                        pending: 0,
                        oom: false,
                    },
                    ServiceSample {
                        demand_rps: 3.0,
                        demand_cpu_millis: 150.0,
                        supply_cpu_millis: 500.0,
                        cpu_usage_millis: 125.0,
                        served_rps: 3.0,
                        error_rate: 0.005,
                        latency_p50_ms: 25.0,
                        latency_p95_ms: 70.0,
                        latency_p99_ms: 95.25,
                        desired: 1,
                        ready: 1,
                        pending: 0,
                        oom: false,
                    },
                ],
                nodes: 3,
                hourly_cost: 0.6,
                replica_changes: 1,
                evictions: 0,
                node_changes: 1,
            },
        ],
    };

    let service_names = vec!["web".to_string(), "api".to_string()];

    let mut output = Vec::new();
    write_trajectory_csv(&trace, &service_names, &mut output).expect("failed to write csv");

    let output_str = String::from_utf8(output).expect("invalid utf8");
    let lines: Vec<&str> = output_str.lines().collect();

    // Expect header + 4 data rows (2 ticks × 2 services)
    assert_eq!(lines.len(), 5, "should have header + 4 data rows");

    // Check header
    assert_eq!(
        lines[0],
        "time_s,service,demand_rps,desired,ready,cpu_usage_millis,latency_p95_ms,error_rate",
        "header should match"
    );

    // Check tick 0, service 0 (web)
    assert_eq!(
        lines[1], "0,web,10,2,2,450,95.5,0",
        "first row should match expected values"
    );

    // Check tick 0, service 1 (api)
    assert_eq!(
        lines[2], "0,api,5,1,1,200,75.25,0.01",
        "second row should match expected values"
    );

    // Check tick 1, service 0 (web)
    assert_eq!(
        lines[3], "15,web,12.5,3,3,550,100,0",
        "third row should match expected values"
    );

    // Check tick 1, service 1 (api)
    assert_eq!(
        lines[4], "15,api,3,1,1,125,70,0.005",
        "fourth row should match expected values"
    );
}

#[test]
fn write_trajectory_csv_service_count_mismatch_returns_error() {
    let trace = EpisodeTrace {
        tick: SimDuration::from_secs(15),
        ticks: vec![TickSample {
            time: SimTime::from_secs(0),
            services: vec![ServiceSample::default(), ServiceSample::default()],
            nodes: 1,
            hourly_cost: 0.0,
            replica_changes: 0,
            evictions: 0,
            node_changes: 0,
        }],
    };

    // Provide only one service name but trace has two services
    let service_names = vec!["web".to_string()];

    let mut output = Vec::new();
    let result = write_trajectory_csv(&trace, &service_names, &mut output);

    assert!(
        result.is_err(),
        "should return error for service count mismatch"
    );
    if let Err(e) = result {
        assert_eq!(
            e.kind(),
            io::ErrorKind::InvalidData,
            "should be InvalidData error"
        );
    }
}

#[test]
fn write_trajectory_csv_f64_formatting_integer_values() {
    // Test that f64 values format as integers when they have no fractional part
    let trace = EpisodeTrace {
        tick: SimDuration::from_secs(15),
        ticks: vec![TickSample {
            time: SimTime::from_secs(0),
            services: vec![ServiceSample {
                demand_rps: 3.0,
                cpu_usage_millis: 100.0,
                latency_p95_ms: 50.0,
                error_rate: 0.0,
                ..ServiceSample::default()
            }],
            nodes: 1,
            hourly_cost: 0.0,
            replica_changes: 0,
            evictions: 0,
            node_changes: 0,
        }],
    };

    let service_names = vec!["web".to_string()];
    let mut output = Vec::new();
    write_trajectory_csv(&trace, &service_names, &mut output).expect("failed to write csv");

    let output_str = String::from_utf8(output).expect("invalid utf8");
    let lines: Vec<&str> = output_str.lines().collect();

    // Check that values like 3.0, 100.0, 50.0, 0.0 format as "3", "100", "50", "0"
    assert_eq!(lines[1], "0,web,3,0,0,100,50,0");
}

#[test]
fn episode_trace_evaluated_metrics_match_run_episode() {
    let text = include_str!("fixtures/tiny.toml");
    let scenario = Scenario::from_toml_str(text).expect("failed to parse tiny.toml");

    let trace = episode_trace(&scenario, Policy::Hpa, 3)
        .expect("failed to get episode trace from episode_trace");
    let evaluated_metrics = evaluate(&trace, &standard_metrics(scenario.slo));

    let episode_report =
        run_episode(&scenario, Policy::Hpa, 3).expect("failed to run episode with run_episode");
    let run_metrics = &episode_report.metrics;

    // Compare metrics with NaN-aware equality
    assert_eq!(
        evaluated_metrics.len(),
        run_metrics.len(),
        "evaluated and run metrics should have same number of entries"
    );

    for (key, evaluated_val) in &evaluated_metrics {
        let run_val = run_metrics
            .get(key)
            .unwrap_or_else(|| panic!("metric {key} missing from run_episode"));
        if evaluated_val.is_nan() && run_val.is_nan() {
            // Both NaN is considered equal
        } else {
            #[allow(clippy::float_cmp)]
            {
                assert_eq!(
                    evaluated_val, run_val,
                    "metric {key} should match between episode_trace and run_episode"
                );
            }
        }
    }
}
