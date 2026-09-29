mod common;

use approx::assert_abs_diff_eq;
use ksim_core::{
    actions::Actions,
    time::{SimDuration, SimTime},
    workload::ServiceDemand,
};
use ksim_metrics::trace::TraceRecorder;

const EPSILON: f64 = 0.001;

#[test]
fn record_single_observation_with_two_services() {
    // Arrange: setup for 2 services, 1 pool, 15 second ticks, pool price $0.10/hour
    let tick_duration = SimDuration::from_secs(15);
    let pool_prices = vec![0.1]; // One pool, $0.10 per hour
    let mut recorder = TraceRecorder::new(tick_duration, pool_prices);

    let observation = common::simple_observation(SimTime::from_secs(0));
    let actions = Actions::default(); // No scaling actions
    let demands = vec![
        ServiceDemand {
            request_rate: 100.0, // demand_rps
            cpu_millis: 500.0,  // demand_cpu_millis = 100 rps
            memory_bytes: 0.0,
        },
        ServiceDemand {
            request_rate: 50.0,
            cpu_millis: 250.0,
            memory_bytes: 0.0,
        },
    ];

    // Act
    recorder.record(&observation, &actions, &demands);
    let trace = recorder.finish();

    // Assert: one tick recorded
    assert_eq!(trace.ticks.len(), 1);
    assert_eq!(trace.tick, tick_duration);

    let tick = &trace.ticks[0];
    assert_eq!(tick.time, SimTime::from_secs(0));
    assert_eq!(tick.services.len(), 2);

    // Service 0: ready=2, requests.cpu=500 => supply = 2*500 = 1000
    // demand_rps = 100, demand_cpu_millis = 500
    let svc0 = &tick.services[0];
    assert_abs_diff_eq!(svc0.demand_rps, 100.0, epsilon = EPSILON); // from ServiceDemand.request_rate
    assert_abs_diff_eq!(svc0.demand_cpu_millis, 500.0, epsilon = EPSILON); // from ServiceDemand.cpu_millis
    assert_abs_diff_eq!(svc0.supply_cpu_millis, 1000.0, epsilon = EPSILON); // 2 ready * 500 cpu_millis

    // Service 1: ready=1, requests.cpu=250 => supply = 1*250 = 250
    // demand_rps = 50, demand_cpu_millis = 250
    let svc1 = &tick.services[1];
    assert_abs_diff_eq!(svc1.demand_rps, 50.0, epsilon = EPSILON);
    assert_abs_diff_eq!(svc1.demand_cpu_millis, 250.0, epsilon = EPSILON);
    assert_abs_diff_eq!(svc1.supply_cpu_millis, 250.0, epsilon = EPSILON);

    // hourly_cost = 2 nodes * $0.10/hour = $0.20
    assert_abs_diff_eq!(tick.hourly_cost, 0.2, epsilon = EPSILON);

    // No scaling actions
    assert_eq!(tick.replica_changes, 0);
    assert_eq!(tick.evictions, 0);
    assert_eq!(tick.node_changes, 0);
}

#[test]
fn record_replica_changes() {
    // Arrange: replica change from desired=2 to actual=3
    let tick_duration = SimDuration::from_secs(15);
    let pool_prices = vec![0.1];
    let mut recorder = TraceRecorder::new(tick_duration, pool_prices);

    let observation = common::simple_observation(SimTime::from_secs(0));
    let actions = Actions {
        replicas: vec![Some(3)],
        ..Default::default()
    };
    let demands = vec![
        ServiceDemand {
            request_rate: 100.0,
            cpu_millis: 500.0,
            memory_bytes: 0.0,
        },
        ServiceDemand {
            request_rate: 50.0,
            cpu_millis: 250.0,
            memory_bytes: 0.0,
        },
    ];

    // Act
    recorder.record(&observation, &actions, &demands);
    let trace = recorder.finish();

    // Assert: one service has a replica change (desired=2, action=Some(3))
    assert_eq!(trace.ticks[0].replica_changes, 1);
}

#[test]
fn record_node_scaling_actions() {
    // Arrange: scale pool by +2 nodes
    let tick_duration = SimDuration::from_secs(15);
    let pool_prices = vec![0.1];
    let mut recorder = TraceRecorder::new(tick_duration, pool_prices);

    let observation = common::simple_observation(SimTime::from_secs(0));
    let actions = Actions {
        pool_delta: vec![2],
        ..Default::default()
    };
    let demands = vec![
        ServiceDemand {
            request_rate: 100.0,
            cpu_millis: 500.0,
            memory_bytes: 0.0,
        },
        ServiceDemand {
            request_rate: 50.0,
            cpu_millis: 250.0,
            memory_bytes: 0.0,
        },
    ];

    // Act
    recorder.record(&observation, &actions, &demands);
    let trace = recorder.finish();

    // Assert: node_changes = |2| = 2
    assert_eq!(trace.ticks[0].node_changes, 2);
}

#[test]
fn record_evictions() {
    // Arrange: evict 3 pods total
    let tick_duration = SimDuration::from_secs(15);
    let pool_prices = vec![0.1];
    let mut recorder = TraceRecorder::new(tick_duration, pool_prices);

    let observation = common::simple_observation(SimTime::from_secs(0));
    let actions = Actions {
        evict: vec![2, 1],
        ..Default::default()
    };
    let demands = vec![
        ServiceDemand {
            request_rate: 100.0,
            cpu_millis: 500.0,
            memory_bytes: 0.0,
        },
        ServiceDemand {
            request_rate: 50.0,
            cpu_millis: 250.0,
            memory_bytes: 0.0,
        },
    ];

    // Act
    recorder.record(&observation, &actions, &demands);
    let trace = recorder.finish();

    // Assert: evictions = sum(evict) = 2 + 1 = 3
    assert_eq!(trace.ticks[0].evictions, 3);
}

#[test]
fn record_signals_copied_from_observation() {
    // Arrange: signals from observation should be copied to tick sample
    let tick_duration = SimDuration::from_secs(15);
    let pool_prices = vec![0.1];
    let mut recorder = TraceRecorder::new(tick_duration, pool_prices);

    let mut observation = common::simple_observation(SimTime::from_secs(0));
    observation.services[0].signals = ksim_core::performance::ServiceSignals {
        cpu_usage_millis: 123.45,
        memory_usage_bytes: 456_789.0,
        served_rps: 99.5,
        latency_p50_ms: 10.0,
        latency_p95_ms: 25.5,
        latency_p99_ms: 45.0,
        error_rate: 0.001,
        throttled_fraction: 0.0,
        oom: false,
    };

    let actions = Actions::default();
    let demands = vec![
        ServiceDemand {
            request_rate: 100.0,
            cpu_millis: 500.0,
            memory_bytes: 0.0,
        },
        ServiceDemand {
            request_rate: 50.0,
            cpu_millis: 250.0,
            memory_bytes: 0.0,
        },
    ];

    // Act
    recorder.record(&observation, &actions, &demands);
    let trace = recorder.finish();

    // Assert: signals copied to service sample
    let svc0 = &trace.ticks[0].services[0];
    assert_abs_diff_eq!(svc0.cpu_usage_millis, 123.45, epsilon = EPSILON);
    assert_abs_diff_eq!(svc0.served_rps, 99.5, epsilon = EPSILON);
    assert_abs_diff_eq!(svc0.error_rate, 0.001, epsilon = EPSILON);
    assert_abs_diff_eq!(svc0.latency_p50_ms, 10.0, epsilon = EPSILON);
    assert_abs_diff_eq!(svc0.latency_p95_ms, 25.5, epsilon = EPSILON);
    assert_abs_diff_eq!(svc0.latency_p99_ms, 45.0, epsilon = EPSILON);
}

#[test]
fn finish_returns_ticks_in_order_with_tick_duration() {
    // Arrange: record multiple ticks
    let tick_duration = SimDuration::from_secs(15);
    let pool_prices = vec![0.1];
    let mut recorder = TraceRecorder::new(tick_duration, pool_prices);

    let observation1 = common::simple_observation(SimTime::from_secs(0));
    let observation2 = common::simple_observation(SimTime::from_secs(15000));
    let observation3 = common::simple_observation(SimTime::from_secs(30000));
    let actions = Actions::default();
    let demands = vec![
        ServiceDemand {
            request_rate: 100.0,
            cpu_millis: 500.0,
            memory_bytes: 0.0,
        },
        ServiceDemand {
            request_rate: 50.0,
            cpu_millis: 250.0,
            memory_bytes: 0.0,
        },
    ];

    // Act
    recorder.record(&observation1, &actions, &demands);
    recorder.record(&observation2, &actions, &demands);
    recorder.record(&observation3, &actions, &demands);
    let trace = recorder.finish();

    // Assert: ticks in order with correct duration
    assert_eq!(trace.tick, tick_duration);
    assert_eq!(trace.ticks.len(), 3);
    assert_eq!(trace.ticks[0].time, SimTime::from_secs(0));
    assert_eq!(trace.ticks[1].time, SimTime::from_secs(15000));
    assert_eq!(trace.ticks[2].time, SimTime::from_secs(30000));
}
