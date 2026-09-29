mod common;

use approx::assert_abs_diff_eq;
use ksim_core::time::{SimDuration, SimTime};
use ksim_metrics::{
    elasticity::Elasticity,
    metric::Metric,
    trace::{ServiceSample, TickSample},
};

const EPSILON: f64 = 0.0001;

#[test]
fn under_provisioning_accuracy() {
    // Arrange: d=[100,100,100,100], s=[50,100,150,100]
    // theta_u = 100/4 * Σ max(d-s, 0)/d = 100/4 * (50/100 + 0 + 0 + 0) = 12.5
    let tick_duration = SimDuration::from_secs(15);
    let ticks = vec![
        TickSample {
            time: SimTime::from_secs(0),
            services: vec![ServiceSample {
                demand_cpu_millis: 100.0,
                supply_cpu_millis: 50.0,
                ..ServiceSample::default()
            }],
            nodes: 1,
            hourly_cost: 0.1,
            replica_changes: 0,
            evictions: 0,
            node_changes: 0,
        },
        TickSample {
            time: SimTime::from_secs(15000),
            services: vec![ServiceSample {
                demand_cpu_millis: 100.0,
                supply_cpu_millis: 100.0,
                ..ServiceSample::default()
            }],
            nodes: 1,
            hourly_cost: 0.1,
            replica_changes: 0,
            evictions: 0,
            node_changes: 0,
        },
        TickSample {
            time: SimTime::from_secs(30000),
            services: vec![ServiceSample {
                demand_cpu_millis: 100.0,
                supply_cpu_millis: 150.0,
                ..ServiceSample::default()
            }],
            nodes: 1,
            hourly_cost: 0.1,
            replica_changes: 0,
            evictions: 0,
            node_changes: 0,
        },
        TickSample {
            time: SimTime::from_secs(45000),
            services: vec![ServiceSample {
                demand_cpu_millis: 100.0,
                supply_cpu_millis: 100.0,
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
    let metric = Elasticity::UnderProvisioningAccuracy;

    // Act
    let theta_u = metric.compute(&trace);

    // Assert: 100/4 * (0.5 + 0 + 0 + 0) = 12.5
    assert_abs_diff_eq!(theta_u, 12.5, epsilon = 0.001);
}

#[test]
fn over_provisioning_accuracy() {
    // Arrange: d=[100,100,100,100], s=[50,100,150,100]
    // theta_o = 100/4 * Σ max(s-d, 0)/d = 100/4 * (0 + 0 + 50/100 + 0) = 12.5
    let tick_duration = SimDuration::from_secs(15);
    let ticks = vec![
        TickSample {
            time: SimTime::from_secs(0),
            services: vec![ServiceSample {
                demand_cpu_millis: 100.0,
                supply_cpu_millis: 50.0,
                ..ServiceSample::default()
            }],
            nodes: 1,
            hourly_cost: 0.1,
            replica_changes: 0,
            evictions: 0,
            node_changes: 0,
        },
        TickSample {
            time: SimTime::from_secs(15000),
            services: vec![ServiceSample {
                demand_cpu_millis: 100.0,
                supply_cpu_millis: 100.0,
                ..ServiceSample::default()
            }],
            nodes: 1,
            hourly_cost: 0.1,
            replica_changes: 0,
            evictions: 0,
            node_changes: 0,
        },
        TickSample {
            time: SimTime::from_secs(30000),
            services: vec![ServiceSample {
                demand_cpu_millis: 100.0,
                supply_cpu_millis: 150.0,
                ..ServiceSample::default()
            }],
            nodes: 1,
            hourly_cost: 0.1,
            replica_changes: 0,
            evictions: 0,
            node_changes: 0,
        },
        TickSample {
            time: SimTime::from_secs(45000),
            services: vec![ServiceSample {
                demand_cpu_millis: 100.0,
                supply_cpu_millis: 100.0,
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
    let metric = Elasticity::OverProvisioningAccuracy;

    // Act
    let theta_o = metric.compute(&trace);

    // Assert: 100/4 * (0 + 0 + 0.5 + 0) = 12.5
    assert_abs_diff_eq!(theta_o, 12.5, epsilon = 0.001);
}

#[test]
fn under_provisioning_timeshare() {
    // Arrange: d=[100,100,100,100], s=[50,100,150,100]
    // tau_u = 100/4 * Σ[s < d] = 100/4 * (1 + 0 + 0 + 0) = 25
    let tick_duration = SimDuration::from_secs(15);
    let ticks = vec![
        TickSample {
            time: SimTime::from_secs(0),
            services: vec![ServiceSample {
                demand_cpu_millis: 100.0,
                supply_cpu_millis: 50.0,
                ..ServiceSample::default()
            }],
            nodes: 1,
            hourly_cost: 0.1,
            replica_changes: 0,
            evictions: 0,
            node_changes: 0,
        },
        TickSample {
            time: SimTime::from_secs(15000),
            services: vec![ServiceSample {
                demand_cpu_millis: 100.0,
                supply_cpu_millis: 100.0,
                ..ServiceSample::default()
            }],
            nodes: 1,
            hourly_cost: 0.1,
            replica_changes: 0,
            evictions: 0,
            node_changes: 0,
        },
        TickSample {
            time: SimTime::from_secs(30000),
            services: vec![ServiceSample {
                demand_cpu_millis: 100.0,
                supply_cpu_millis: 150.0,
                ..ServiceSample::default()
            }],
            nodes: 1,
            hourly_cost: 0.1,
            replica_changes: 0,
            evictions: 0,
            node_changes: 0,
        },
        TickSample {
            time: SimTime::from_secs(45000),
            services: vec![ServiceSample {
                demand_cpu_millis: 100.0,
                supply_cpu_millis: 100.0,
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
    let metric = Elasticity::UnderProvisioningTimeshare;

    // Act
    let tau_u = metric.compute(&trace);

    // Assert: 100/4 * (1 + 0 + 0 + 0) = 25
    assert_abs_diff_eq!(tau_u, 25.0, epsilon = 0.001);
}

#[test]
fn over_provisioning_timeshare() {
    // Arrange: d=[100,100,100,100], s=[50,100,150,100]
    // tau_o = 100/4 * Σ[s > d] = 100/4 * (0 + 0 + 1 + 0) = 25
    let tick_duration = SimDuration::from_secs(15);
    let ticks = vec![
        TickSample {
            time: SimTime::from_secs(0),
            services: vec![ServiceSample {
                demand_cpu_millis: 100.0,
                supply_cpu_millis: 50.0,
                ..ServiceSample::default()
            }],
            nodes: 1,
            hourly_cost: 0.1,
            replica_changes: 0,
            evictions: 0,
            node_changes: 0,
        },
        TickSample {
            time: SimTime::from_secs(15000),
            services: vec![ServiceSample {
                demand_cpu_millis: 100.0,
                supply_cpu_millis: 100.0,
                ..ServiceSample::default()
            }],
            nodes: 1,
            hourly_cost: 0.1,
            replica_changes: 0,
            evictions: 0,
            node_changes: 0,
        },
        TickSample {
            time: SimTime::from_secs(30000),
            services: vec![ServiceSample {
                demand_cpu_millis: 100.0,
                supply_cpu_millis: 150.0,
                ..ServiceSample::default()
            }],
            nodes: 1,
            hourly_cost: 0.1,
            replica_changes: 0,
            evictions: 0,
            node_changes: 0,
        },
        TickSample {
            time: SimTime::from_secs(45000),
            services: vec![ServiceSample {
                demand_cpu_millis: 100.0,
                supply_cpu_millis: 100.0,
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
    let metric = Elasticity::OverProvisioningTimeshare;

    // Act
    let tau_o = metric.compute(&trace);

    // Assert: 100/4 * (0 + 0 + 1 + 0) = 25
    assert_abs_diff_eq!(tau_o, 25.0, epsilon = 0.001);
}

#[test]
fn jitter_per_hour() {
    // Arrange: d=[100,100,100,100], s=[50,100,150,100]
    // Supply changes at t=1,2,3 (3 changes), demand changes 0 times
    // Total time = 4 * 15 / 3600 = 1/60 hour
    // jitter = (3 - 0) / (1/60) = 180 per hour
    let tick_duration = SimDuration::from_secs(15);
    let ticks = vec![
        TickSample {
            time: SimTime::from_secs(0),
            services: vec![ServiceSample {
                demand_cpu_millis: 100.0,
                supply_cpu_millis: 50.0,
                ..ServiceSample::default()
            }],
            nodes: 1,
            hourly_cost: 0.1,
            replica_changes: 0,
            evictions: 0,
            node_changes: 0,
        },
        TickSample {
            time: SimTime::from_secs(15000),
            services: vec![ServiceSample {
                demand_cpu_millis: 100.0,
                supply_cpu_millis: 100.0,
                ..ServiceSample::default()
            }],
            nodes: 1,
            hourly_cost: 0.1,
            replica_changes: 0,
            evictions: 0,
            node_changes: 0,
        },
        TickSample {
            time: SimTime::from_secs(30000),
            services: vec![ServiceSample {
                demand_cpu_millis: 100.0,
                supply_cpu_millis: 150.0,
                ..ServiceSample::default()
            }],
            nodes: 1,
            hourly_cost: 0.1,
            replica_changes: 0,
            evictions: 0,
            node_changes: 0,
        },
        TickSample {
            time: SimTime::from_secs(45000),
            services: vec![ServiceSample {
                demand_cpu_millis: 100.0,
                supply_cpu_millis: 100.0,
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
    let metric = Elasticity::Jitter;

    // Act
    let jitter = metric.compute(&trace);

    // Assert: (3 - 0) / (1/60 hour) = 180 per hour
    assert_abs_diff_eq!(jitter, 180.0, epsilon = 0.1);
}

#[test]
fn instability_metric() {
    // Arrange: d=[100,100,100,100], s=[50,100,150,100]
    // Δs = [+50, +50, -50], Δd = [0, 0, 0]
    // sgn(Δs) = [+1, +1, -1], sgn(Δd) = [0, 0, 0]
    // Count where signs differ: all 3
    // instability = 100/3 * 3 = 100
    let tick_duration = SimDuration::from_secs(15);
    let ticks = vec![
        TickSample {
            time: SimTime::from_secs(0),
            services: vec![ServiceSample {
                demand_cpu_millis: 100.0,
                supply_cpu_millis: 50.0,
                ..ServiceSample::default()
            }],
            nodes: 1,
            hourly_cost: 0.1,
            replica_changes: 0,
            evictions: 0,
            node_changes: 0,
        },
        TickSample {
            time: SimTime::from_secs(15000),
            services: vec![ServiceSample {
                demand_cpu_millis: 100.0,
                supply_cpu_millis: 100.0,
                ..ServiceSample::default()
            }],
            nodes: 1,
            hourly_cost: 0.1,
            replica_changes: 0,
            evictions: 0,
            node_changes: 0,
        },
        TickSample {
            time: SimTime::from_secs(30000),
            services: vec![ServiceSample {
                demand_cpu_millis: 100.0,
                supply_cpu_millis: 150.0,
                ..ServiceSample::default()
            }],
            nodes: 1,
            hourly_cost: 0.1,
            replica_changes: 0,
            evictions: 0,
            node_changes: 0,
        },
        TickSample {
            time: SimTime::from_secs(45000),
            services: vec![ServiceSample {
                demand_cpu_millis: 100.0,
                supply_cpu_millis: 100.0,
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
    let metric = Elasticity::Instability;

    // Act
    let instability = metric.compute(&trace);

    // Assert: 100/3 * 3 = 100
    assert_abs_diff_eq!(instability, 100.0, epsilon = 0.001);
}

#[test]
fn elasticity_with_zero_demand_ticks() {
    // Arrange: one service with demand = [100, 0, 100], supply = [100, 0, 100]
    // Ticks with demand=0 add 0 to theta calculations
    let tick_duration = SimDuration::from_secs(15);
    let ticks = vec![
        TickSample {
            time: SimTime::from_secs(0),
            services: vec![ServiceSample {
                demand_cpu_millis: 100.0,
                supply_cpu_millis: 100.0,
                ..ServiceSample::default()
            }],
            nodes: 1,
            hourly_cost: 0.1,
            replica_changes: 0,
            evictions: 0,
            node_changes: 0,
        },
        TickSample {
            time: SimTime::from_secs(15000),
            services: vec![ServiceSample {
                demand_cpu_millis: 0.0,
                supply_cpu_millis: 0.0,
                ..ServiceSample::default()
            }],
            nodes: 1,
            hourly_cost: 0.1,
            replica_changes: 0,
            evictions: 0,
            node_changes: 0,
        },
        TickSample {
            time: SimTime::from_secs(30000),
            services: vec![ServiceSample {
                demand_cpu_millis: 100.0,
                supply_cpu_millis: 100.0,
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
    let metric = Elasticity::UnderProvisioningAccuracy;

    // Act
    let theta_u = metric.compute(&trace);

    // Assert: only counts non-zero demand ticks
    // (0 + 0 + 0) / 2 = 0
    assert_abs_diff_eq!(theta_u, 0.0, epsilon = 0.001);
}

#[test]
fn elasticity_empty_trace_returns_zero() {
    // Arrange: empty trace
    let tick_duration = SimDuration::from_secs(15);
    let trace = common::episode_trace(tick_duration, vec![]);

    // Act & Assert: all elasticity metrics return 0 for empty traces
    assert_abs_diff_eq!(
        Elasticity::UnderProvisioningAccuracy.compute(&trace),
        0.0,
        epsilon = EPSILON
    );
    assert_abs_diff_eq!(
        Elasticity::OverProvisioningAccuracy.compute(&trace),
        0.0,
        epsilon = EPSILON
    );
    assert_abs_diff_eq!(
        Elasticity::UnderProvisioningTimeshare.compute(&trace),
        0.0,
        epsilon = EPSILON
    );
    assert_abs_diff_eq!(
        Elasticity::OverProvisioningTimeshare.compute(&trace),
        0.0,
        epsilon = EPSILON
    );
    assert_abs_diff_eq!(Elasticity::Jitter.compute(&trace), 0.0, epsilon = EPSILON);
    assert_abs_diff_eq!(
        Elasticity::Instability.compute(&trace),
        0.0,
        epsilon = EPSILON
    );
}

#[test]
fn elasticity_single_tick_for_jitter_returns_zero() {
    // Arrange: single tick (jitter needs 2+ ticks)
    let tick_duration = SimDuration::from_secs(15);
    let ticks = vec![TickSample {
        time: SimTime::from_secs(0),
        services: vec![ServiceSample {
            demand_cpu_millis: 100.0,
            supply_cpu_millis: 100.0,
            ..ServiceSample::default()
        }],
        nodes: 1,
        hourly_cost: 0.1,
        replica_changes: 0,
        evictions: 0,
        node_changes: 0,
    }];

    let trace = common::episode_trace(tick_duration, ticks);
    let metric = Elasticity::Jitter;

    // Act
    let jitter = metric.compute(&trace);

    // Assert: single tick returns 0 for jitter
    assert_abs_diff_eq!(jitter, 0.0, epsilon = EPSILON);
}
