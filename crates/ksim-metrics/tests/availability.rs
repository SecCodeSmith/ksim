mod common;

use approx::assert_abs_diff_eq;
use ksim_core::time::{SimDuration, SimTime};
use ksim_metrics::{
    availability::{
        AvailabilityNines, ErrorBudgetBurn, LongestSloViolation, RequestAvailability, SloCompliance,
    },
    metric::{Metric, SloPercentile},
    trace::{ServiceSample, TickSample},
};

const EPSILON: f64 = 0.001;

#[test]
fn request_availability_with_errors() {
    // Arrange: 2 services across 2 ticks
    // Tick 0: svc0 demand_rps=100 error=0, svc1 demand_rps=50 error=0
    // Tick 1: svc0 demand_rps=100 error=0.1, svc1 demand_rps=50 error=0.05
    // Availability = Σ(demand_rps * (1 - error)) / Σ(demand_rps)
    //              = (100*1 + 50*1 + 100*0.9 + 50*0.95) / (100+50+100+50)
    //              = (100 + 50 + 90 + 47.5) / 300 = 287.5/300 ≈ 0.958333
    let tick_duration = SimDuration::from_secs(15);
    let tick0 = TickSample {
        time: SimTime::from_secs(0),
        services: vec![
            ServiceSample {
                demand_rps: 100.0,
                demand_cpu_millis: 500.0,
                supply_cpu_millis: 1000.0,
                cpu_usage_millis: 750.0,
                served_rps: 100.0,
                error_rate: 0.0,
                latency_p50_ms: 10.0,
                latency_p95_ms: 25.0,
                latency_p99_ms: 50.0,
                desired: 2,
                ready: 2,
                pending: 0,
                oom: false,
            },
            ServiceSample {
                demand_rps: 50.0,
                demand_cpu_millis: 250.0,
                supply_cpu_millis: 250.0,
                cpu_usage_millis: 200.0,
                served_rps: 50.0,
                error_rate: 0.0,
                latency_p50_ms: 8.0,
                latency_p95_ms: 20.0,
                latency_p99_ms: 40.0,
                desired: 1,
                ready: 1,
                pending: 0,
                oom: false,
            },
        ],
        nodes: 2,
        hourly_cost: 0.2,
        replica_changes: 0,
        evictions: 0,
        node_changes: 0,
    };

    let tick1 = TickSample {
        time: SimTime::from_secs(15000),
        services: vec![
            ServiceSample {
                demand_rps: 100.0,
                error_rate: 0.1,
                served_rps: 90.0,
                ..tick0.services[0]
            },
            ServiceSample {
                demand_rps: 50.0,
                error_rate: 0.05,
                served_rps: 47.5,
                ..tick0.services[1]
            },
        ],
        ..tick0
    };

    let trace = common::episode_trace(tick_duration, vec![tick0, tick1]);
    let metric = RequestAvailability;

    // Act
    let availability = metric.compute(&trace);

    // Assert: (100 + 50 + 90 + 47.5) / 300 ≈ 0.958333
    assert_abs_diff_eq!(availability, 0.958_333, epsilon = 0.001);
}

#[test]
fn request_availability_no_demand_returns_one() {
    // Arrange: single tick with zero demand
    let tick_duration = SimDuration::from_secs(15);
    let tick = TickSample {
        time: SimTime::from_secs(0),
        services: vec![ServiceSample {
            demand_rps: 0.0,
            demand_cpu_millis: 0.0,
            supply_cpu_millis: 0.0,
            cpu_usage_millis: 0.0,
            served_rps: 0.0,
            error_rate: 0.0,
            latency_p50_ms: 0.0,
            latency_p95_ms: 0.0,
            latency_p99_ms: 0.0,
            desired: 0,
            ready: 0,
            pending: 0,
            oom: false,
        }],
        nodes: 0,
        hourly_cost: 0.0,
        replica_changes: 0,
        evictions: 0,
        node_changes: 0,
    };

    let trace = common::episode_trace(tick_duration, vec![tick]);
    let metric = RequestAvailability;

    // Act
    let availability = metric.compute(&trace);

    // Assert: no demand => 1.0
    assert_abs_diff_eq!(availability, 1.0, epsilon = EPSILON);
}

#[test]
fn availability_nines_of_0_95() {
    // Arrange: create a trace that yields availability 0.95
    // -log10(1 - 0.95) = -log10(0.05) ≈ 1.30103
    let tick_duration = SimDuration::from_secs(15);
    let tick0 = TickSample {
        time: SimTime::from_secs(0),
        services: vec![ServiceSample {
            demand_rps: 100.0,
            demand_cpu_millis: 500.0,
            supply_cpu_millis: 1000.0,
            cpu_usage_millis: 750.0,
            served_rps: 100.0,
            error_rate: 0.0,
            latency_p50_ms: 10.0,
            latency_p95_ms: 25.0,
            latency_p99_ms: 50.0,
            desired: 2,
            ready: 2,
            pending: 0,
            oom: false,
        }],
        nodes: 1,
        hourly_cost: 0.1,
        replica_changes: 0,
        evictions: 0,
        node_changes: 0,
    };

    let tick1 = TickSample {
        time: SimTime::from_secs(15000),
        services: vec![ServiceSample {
            error_rate: 0.1,
            served_rps: 90.0,
            ..tick0.services[0]
        }],
        ..tick0
    };

    let trace = common::episode_trace(tick_duration, vec![tick0, tick1]);
    let metric = AvailabilityNines;

    // Act
    let nines = metric.compute(&trace);

    // Assert: -log10(0.05) ≈ 1.30103
    assert_abs_diff_eq!(nines, 1.30103, epsilon = 0.001);
}

#[test]
fn availability_nines_capped_at_9() {
    // Arrange: perfect availability => nines should cap at 9.0
    let tick_duration = SimDuration::from_secs(15);
    let tick = TickSample {
        time: SimTime::from_secs(0),
        services: vec![ServiceSample {
            demand_rps: 100.0,
            demand_cpu_millis: 500.0,
            supply_cpu_millis: 1000.0,
            cpu_usage_millis: 750.0,
            served_rps: 100.0,
            error_rate: 0.0,
            latency_p50_ms: 10.0,
            latency_p95_ms: 25.0,
            latency_p99_ms: 50.0,
            desired: 2,
            ready: 2,
            pending: 0,
            oom: false,
        }],
        nodes: 1,
        hourly_cost: 0.1,
        replica_changes: 0,
        evictions: 0,
        node_changes: 0,
    };

    let trace = common::episode_trace(tick_duration, vec![tick]);
    let metric = AvailabilityNines;

    // Act
    let nines = metric.compute(&trace);

    // Assert: perfect availability capped at 9.0
    assert_abs_diff_eq!(nines, 9.0, epsilon = EPSILON);
}

#[test]
fn slo_compliance_with_mixed_samples() {
    // Arrange: SLO latency_ms=200, P95, max_error_rate=0.01
    // Sample 0: p95=150, error=0 => complies (within SLO)
    // Sample 1: p95=250, error=0 => violates (latency > 200)
    // Sample 2: p95=0, error=0.02 => violates (error > 0.01)
    // Sample 3: demand_rps=0 => complies (zero demand always complies)
    // Result: 2/4 = 0.5
    let tick_duration = SimDuration::from_secs(15);
    let tick = TickSample {
        time: SimTime::from_secs(0),
        services: vec![
            ServiceSample {
                demand_rps: 100.0,
                demand_cpu_millis: 500.0,
                supply_cpu_millis: 1000.0,
                cpu_usage_millis: 750.0,
                served_rps: 100.0,
                error_rate: 0.0,
                latency_p95_ms: 150.0,
                latency_p50_ms: 10.0,
                latency_p99_ms: 50.0,
                desired: 2,
                ready: 2,
                pending: 0,
                oom: false,
            },
            ServiceSample {
                demand_rps: 100.0,
                latency_p95_ms: 250.0,
                error_rate: 0.0,
                ..ServiceSample::default()
            },
            ServiceSample {
                demand_rps: 100.0,
                latency_p95_ms: 50.0,
                error_rate: 0.02,
                ..ServiceSample::default()
            },
            ServiceSample {
                demand_rps: 0.0,
                latency_p95_ms: 150.0,
                error_rate: 0.5,
                ..ServiceSample::default()
            },
        ],
        nodes: 1,
        hourly_cost: 0.1,
        replica_changes: 0,
        evictions: 0,
        node_changes: 0,
    };

    let slo = ksim_metrics::metric::Slo {
        latency_ms: 200.0,
        percentile: SloPercentile::P95,
        max_error_rate: 0.01,
        availability_target: 0.999,
    };

    let trace = common::episode_trace(tick_duration, vec![tick]);
    let metric = SloCompliance { slo };

    // Act
    let compliance = metric.compute(&trace);

    // Assert: 2/4 = 0.5
    assert_abs_diff_eq!(compliance, 0.5, epsilon = 0.001);
}

#[test]
fn slo_compliance_uses_p99_when_specified() {
    // Arrange: SLO with P99 percentile
    let tick_duration = SimDuration::from_secs(15);
    let tick = TickSample {
        time: SimTime::from_secs(0),
        services: vec![ServiceSample {
            demand_rps: 100.0,
            demand_cpu_millis: 500.0,
            supply_cpu_millis: 1000.0,
            cpu_usage_millis: 750.0,
            served_rps: 100.0,
            error_rate: 0.0,
            latency_p95_ms: 250.0, // violates P95 but not P99
            latency_p99_ms: 150.0, // complies with P99
            latency_p50_ms: 10.0,
            desired: 2,
            ready: 2,
            pending: 0,
            oom: false,
        }],
        nodes: 1,
        hourly_cost: 0.1,
        replica_changes: 0,
        evictions: 0,
        node_changes: 0,
    };

    let slo = ksim_metrics::metric::Slo {
        latency_ms: 200.0,
        percentile: SloPercentile::P99,
        max_error_rate: 0.01,
        availability_target: 0.999,
    };

    let trace = common::episode_trace(tick_duration, vec![tick]);
    let metric = SloCompliance { slo };

    // Act
    let compliance = metric.compute(&trace);

    // Assert: should use P99 latency (150 ms) which complies
    assert_abs_diff_eq!(compliance, 1.0, epsilon = 0.001);
}

#[test]
fn error_budget_burn_calculation() {
    // Arrange: availability = 0.95, availability_target = 0.999
    // error_budget_burn = (1 - 0.95) / (1 - 0.999) = 0.05 / 0.001 = 50
    let tick_duration = SimDuration::from_secs(15);
    let tick0 = TickSample {
        time: SimTime::from_secs(0),
        services: vec![ServiceSample {
            demand_rps: 100.0,
            error_rate: 0.0,
            served_rps: 100.0,
            ..ServiceSample::default()
        }],
        nodes: 1,
        hourly_cost: 0.1,
        replica_changes: 0,
        evictions: 0,
        node_changes: 0,
    };

    let tick1 = TickSample {
        time: SimTime::from_secs(15000),
        services: vec![ServiceSample {
            demand_rps: 100.0,
            error_rate: 0.1,
            served_rps: 90.0,
            ..tick0.services[0]
        }],
        ..tick0
    };

    let slo = ksim_metrics::metric::Slo {
        latency_ms: 200.0,
        percentile: SloPercentile::P95,
        max_error_rate: 0.01,
        availability_target: 0.999,
    };

    let trace = common::episode_trace(tick_duration, vec![tick0, tick1]);
    let metric = ErrorBudgetBurn { slo };

    // Act
    let burn = metric.compute(&trace);

    // Assert: (1 - 0.95) / (1 - 0.999) = 50
    assert_abs_diff_eq!(burn, 50.0, epsilon = 0.001);
}

#[test]
fn longest_slo_violation_with_pattern() {
    // Arrange: 15 second ticks with violation pattern [ok, bad, bad, ok, bad]
    // Longest violation: ticks 1-2 = 2 * 15 = 30 seconds
    let tick_duration = SimDuration::from_secs(15);
    let slo = ksim_metrics::metric::Slo {
        latency_ms: 200.0,
        percentile: SloPercentile::P95,
        max_error_rate: 0.01,
        availability_target: 0.999,
    };

    let ticks = vec![
        // Tick 0: ok (p95=150, error=0)
        TickSample {
            time: SimTime::from_secs(0),
            services: vec![ServiceSample {
                demand_rps: 100.0,
                latency_p95_ms: 150.0,
                error_rate: 0.0,
                ..ServiceSample::default()
            }],
            nodes: 1,
            hourly_cost: 0.1,
            replica_changes: 0,
            evictions: 0,
            node_changes: 0,
        },
        // Tick 1: bad (p95=250)
        TickSample {
            time: SimTime::from_secs(15000),
            services: vec![ServiceSample {
                demand_rps: 100.0,
                latency_p95_ms: 250.0,
                error_rate: 0.0,
                ..ServiceSample::default()
            }],
            nodes: 1,
            hourly_cost: 0.1,
            replica_changes: 0,
            evictions: 0,
            node_changes: 0,
        },
        // Tick 2: bad (error=0.02)
        TickSample {
            time: SimTime::from_secs(30000),
            services: vec![ServiceSample {
                demand_rps: 100.0,
                latency_p95_ms: 150.0,
                error_rate: 0.02,
                ..ServiceSample::default()
            }],
            nodes: 1,
            hourly_cost: 0.1,
            replica_changes: 0,
            evictions: 0,
            node_changes: 0,
        },
        // Tick 3: ok
        TickSample {
            time: SimTime::from_secs(45000),
            services: vec![ServiceSample {
                demand_rps: 100.0,
                latency_p95_ms: 150.0,
                error_rate: 0.0,
                ..ServiceSample::default()
            }],
            nodes: 1,
            hourly_cost: 0.1,
            replica_changes: 0,
            evictions: 0,
            node_changes: 0,
        },
        // Tick 4: bad
        TickSample {
            time: SimTime::from_secs(60000),
            services: vec![ServiceSample {
                demand_rps: 100.0,
                latency_p95_ms: 250.0,
                error_rate: 0.0,
                ..ServiceSample::default()
            }],
            nodes: 1,
            hourly_cost: 0.1,
            replica_changes: 0,
            evictions: 0,
            node_changes: 0,
        },
    ];

    let trace = common::episode_trace(tick_duration, ticks);
    let metric = LongestSloViolation { slo };

    // Act
    let longest_violation_s = metric.compute(&trace);

    // Assert: ticks 1-2 = 2 * 15 = 30 seconds
    assert_abs_diff_eq!(longest_violation_s, 30.0, epsilon = 0.001);
}
