mod common;

use approx::assert_abs_diff_eq;
use ksim_core::time::{SimDuration, SimTime};
use ksim_metrics::{
    cost::{Cost, CpuRequestUtilization, NodeHours},
    metric::Metric,
    operational::Operational,
    trace::{ServiceSample, TickSample},
};

const EPSILON: f64 = 0.0001;

#[test]
fn cost_calculation() {
    // Arrange: 3 ticks of 15 seconds each
    // Tick 0: hourly_cost = 0.5
    // Tick 1: hourly_cost = 0.3
    // Tick 2: hourly_cost = 0.2
    // cost = Σ hourly_cost * (15/3600) = (0.5 + 0.3 + 0.2) * (15/3600) = 1.0 * (1/240) ≈ 0.00417
    let tick_duration = SimDuration::from_secs(15);
    let ticks = vec![
        TickSample {
            time: SimTime::from_secs(0),
            services: vec![ServiceSample::default()],
            nodes: 5,
            hourly_cost: 0.5,
            replica_changes: 0,
            evictions: 0,
            node_changes: 0,
        },
        TickSample {
            time: SimTime::from_secs(15000),
            services: vec![ServiceSample::default()],
            nodes: 3,
            hourly_cost: 0.3,
            replica_changes: 0,
            evictions: 0,
            node_changes: 0,
        },
        TickSample {
            time: SimTime::from_secs(30000),
            services: vec![ServiceSample::default()],
            nodes: 2,
            hourly_cost: 0.2,
            replica_changes: 0,
            evictions: 0,
            node_changes: 0,
        },
    ];

    let trace = common::episode_trace(tick_duration, ticks);
    let metric = Cost;

    // Act
    let cost = metric.compute(&trace);

    // Assert: (0.5 + 0.3 + 0.2) * (15/3600) ≈ 0.00417
    assert_abs_diff_eq!(cost, 1.0 * 15.0 / 3600.0, epsilon = 0.0001);
}

#[test]
fn node_hours_calculation() {
    // Arrange: 3 ticks of 15 seconds each
    // Tick 0: 5 nodes => 5 * 15 / 3600 = 75 / 3600
    // Tick 1: 3 nodes => 3 * 15 / 3600 = 45 / 3600
    // Tick 2: 2 nodes => 2 * 15 / 3600 = 30 / 3600
    // total = (75 + 45 + 30) / 3600 = 150 / 3600 ≈ 0.04167
    let tick_duration = SimDuration::from_secs(15);
    let ticks = vec![
        TickSample {
            time: SimTime::from_secs(0),
            services: vec![ServiceSample::default()],
            nodes: 5,
            hourly_cost: 0.1,
            replica_changes: 0,
            evictions: 0,
            node_changes: 0,
        },
        TickSample {
            time: SimTime::from_secs(15000),
            services: vec![ServiceSample::default()],
            nodes: 3,
            hourly_cost: 0.1,
            replica_changes: 0,
            evictions: 0,
            node_changes: 0,
        },
        TickSample {
            time: SimTime::from_secs(30000),
            services: vec![ServiceSample::default()],
            nodes: 2,
            hourly_cost: 0.1,
            replica_changes: 0,
            evictions: 0,
            node_changes: 0,
        },
    ];

    let trace = common::episode_trace(tick_duration, ticks);
    let metric = NodeHours;

    // Act
    let node_hours = metric.compute(&trace);

    // Assert: (5 + 3 + 2) * 15 / 3600 = 10 * 15 / 3600
    assert_abs_diff_eq!(node_hours, 10.0 * 15.0 / 3600.0, epsilon = 0.0001);
}

#[test]
fn cpu_request_utilization_calculation() {
    // Arrange: 2 services across 2 ticks
    // Tick 0: svc0 usage=500, supply=1000; svc1 usage=200, supply=500
    // Tick 1: svc0 usage=800, supply=1000; svc1 usage=300, supply=500
    // Total usage = 500 + 200 + 800 + 300 = 1800
    // Total supply = 1000 + 500 + 1000 + 500 = 3000
    // utilization = 1800 / 3000 = 0.6
    let tick_duration = SimDuration::from_secs(15);
    let ticks = vec![
        TickSample {
            time: SimTime::from_secs(0),
            services: vec![
                ServiceSample {
                    cpu_usage_millis: 500.0,
                    supply_cpu_millis: 1000.0,
                    ..ServiceSample::default()
                },
                ServiceSample {
                    cpu_usage_millis: 200.0,
                    supply_cpu_millis: 500.0,
                    ..ServiceSample::default()
                },
            ],
            nodes: 2,
            hourly_cost: 0.2,
            replica_changes: 0,
            evictions: 0,
            node_changes: 0,
        },
        TickSample {
            time: SimTime::from_secs(15000),
            services: vec![
                ServiceSample {
                    cpu_usage_millis: 800.0,
                    supply_cpu_millis: 1000.0,
                    ..ServiceSample::default()
                },
                ServiceSample {
                    cpu_usage_millis: 300.0,
                    supply_cpu_millis: 500.0,
                    ..ServiceSample::default()
                },
            ],
            nodes: 2,
            hourly_cost: 0.2,
            replica_changes: 0,
            evictions: 0,
            node_changes: 0,
        },
    ];

    let trace = common::episode_trace(tick_duration, ticks);
    let metric = CpuRequestUtilization;

    // Act
    let utilization = metric.compute(&trace);

    // Assert: (500 + 200 + 800 + 300) / (1000 + 500 + 1000 + 500) = 1800/3000 = 0.6
    assert_abs_diff_eq!(utilization, 0.6, epsilon = 0.001);
}

#[test]
fn cpu_request_utilization_zero_when_no_supply() {
    // Arrange: trace with no supply
    let tick_duration = SimDuration::from_secs(15);
    let ticks = vec![TickSample {
        time: SimTime::from_secs(0),
        services: vec![ServiceSample {
            cpu_usage_millis: 100.0,
            supply_cpu_millis: 0.0,
            ..ServiceSample::default()
        }],
        nodes: 0,
        hourly_cost: 0.0,
        replica_changes: 0,
        evictions: 0,
        node_changes: 0,
    }];

    let trace = common::episode_trace(tick_duration, ticks);
    let metric = CpuRequestUtilization;

    // Act
    let utilization = metric.compute(&trace);

    // Assert: 0 when no supply
    assert_abs_diff_eq!(utilization, 0.0, epsilon = EPSILON);
}

#[test]
fn scaling_actions_count() {
    // Arrange: 3 ticks
    // Tick 0: replica_changes = 2, node_changes = 1 => 3
    // Tick 1: replica_changes = 0, node_changes = 2 => 2
    // Tick 2: replica_changes = 1, node_changes = 0 => 1
    // Total = 6
    let tick_duration = SimDuration::from_secs(15);
    let ticks = vec![
        TickSample {
            time: SimTime::from_secs(0),
            services: vec![ServiceSample::default()],
            nodes: 5,
            hourly_cost: 0.1,
            replica_changes: 2,
            evictions: 0,
            node_changes: 1,
        },
        TickSample {
            time: SimTime::from_secs(15000),
            services: vec![ServiceSample::default()],
            nodes: 3,
            hourly_cost: 0.1,
            replica_changes: 0,
            evictions: 0,
            node_changes: 2,
        },
        TickSample {
            time: SimTime::from_secs(30000),
            services: vec![ServiceSample::default()],
            nodes: 2,
            hourly_cost: 0.1,
            replica_changes: 1,
            evictions: 0,
            node_changes: 0,
        },
    ];

    let trace = common::episode_trace(tick_duration, ticks);
    let metric = Operational::ScalingActions;

    // Act
    let actions = metric.compute(&trace);

    // Assert: (2+1) + (0+2) + (1+0) = 6
    assert_abs_diff_eq!(actions, 6.0, epsilon = EPSILON);
}

#[test]
fn pending_pod_seconds() {
    // Arrange: 3 ticks of 15 seconds each
    // Tick 0: pending = 2 => 2 * 15 = 30
    // Tick 1: pending = 2 => 2 * 15 = 30
    // Tick 2: pending = 2 => 2 * 15 = 30
    // Total = 90
    let tick_duration = SimDuration::from_secs(15);
    let ticks = vec![
        TickSample {
            time: SimTime::from_secs(0),
            services: vec![ServiceSample {
                pending: 2,
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
                pending: 2,
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
                pending: 2,
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
    let metric = Operational::PendingPodSeconds;

    // Act
    let pending_seconds = metric.compute(&trace);

    // Assert: 2 * 15 * 3 = 90
    assert_abs_diff_eq!(pending_seconds, 90.0, epsilon = EPSILON);
}

#[test]
fn oom_events_count() {
    // Arrange: 4 samples, 2 with OOM
    let tick_duration = SimDuration::from_secs(15);
    let ticks = vec![
        TickSample {
            time: SimTime::from_secs(0),
            services: vec![
                ServiceSample {
                    oom: true,
                    ..ServiceSample::default()
                },
                ServiceSample {
                    oom: false,
                    ..ServiceSample::default()
                },
            ],
            nodes: 1,
            hourly_cost: 0.1,
            replica_changes: 0,
            evictions: 0,
            node_changes: 0,
        },
        TickSample {
            time: SimTime::from_secs(15000),
            services: vec![
                ServiceSample {
                    oom: true,
                    ..ServiceSample::default()
                },
                ServiceSample {
                    oom: false,
                    ..ServiceSample::default()
                },
            ],
            nodes: 1,
            hourly_cost: 0.1,
            replica_changes: 0,
            evictions: 0,
            node_changes: 0,
        },
    ];

    let trace = common::episode_trace(tick_duration, ticks);
    let metric = Operational::OomEvents;

    // Act
    let oom_count = metric.compute(&trace);

    // Assert: 2 samples with oom=true
    assert_abs_diff_eq!(oom_count, 2.0, epsilon = EPSILON);
}

#[test]
fn evictions_count() {
    // Arrange: 3 ticks with evictions
    // Tick 0: evictions = 3
    // Tick 1: evictions = 1
    // Tick 2: evictions = 2
    // Total = 6
    let tick_duration = SimDuration::from_secs(15);
    let ticks = vec![
        TickSample {
            time: SimTime::from_secs(0),
            services: vec![ServiceSample::default()],
            nodes: 1,
            hourly_cost: 0.1,
            replica_changes: 0,
            evictions: 3,
            node_changes: 0,
        },
        TickSample {
            time: SimTime::from_secs(15000),
            services: vec![ServiceSample::default()],
            nodes: 1,
            hourly_cost: 0.1,
            replica_changes: 0,
            evictions: 1,
            node_changes: 0,
        },
        TickSample {
            time: SimTime::from_secs(30000),
            services: vec![ServiceSample::default()],
            nodes: 1,
            hourly_cost: 0.1,
            replica_changes: 0,
            evictions: 2,
            node_changes: 0,
        },
    ];

    let trace = common::episode_trace(tick_duration, ticks);
    let metric = Operational::Evictions;

    // Act
    let evictions = metric.compute(&trace);

    // Assert: 3 + 1 + 2 = 6
    assert_abs_diff_eq!(evictions, 6.0, epsilon = EPSILON);
}

#[test]
fn mean_replicas() {
    // Arrange: 4 samples with ready counts [2, 3, 1, 2]
    // mean = (2 + 3 + 1 + 2) / 4 = 8 / 4 = 2.0
    let tick_duration = SimDuration::from_secs(15);
    let ticks = vec![
        TickSample {
            time: SimTime::from_secs(0),
            services: vec![
                ServiceSample {
                    ready: 2,
                    ..ServiceSample::default()
                },
                ServiceSample {
                    ready: 3,
                    ..ServiceSample::default()
                },
            ],
            nodes: 1,
            hourly_cost: 0.1,
            replica_changes: 0,
            evictions: 0,
            node_changes: 0,
        },
        TickSample {
            time: SimTime::from_secs(15000),
            services: vec![
                ServiceSample {
                    ready: 1,
                    ..ServiceSample::default()
                },
                ServiceSample {
                    ready: 2,
                    ..ServiceSample::default()
                },
            ],
            nodes: 1,
            hourly_cost: 0.1,
            replica_changes: 0,
            evictions: 0,
            node_changes: 0,
        },
    ];

    let trace = common::episode_trace(tick_duration, ticks);
    let metric = Operational::MeanReplicas;

    // Act
    let mean = metric.compute(&trace);

    // Assert: (2 + 3 + 1 + 2) / 4 = 2.0
    assert_abs_diff_eq!(mean, 2.0, epsilon = 0.001);
}
