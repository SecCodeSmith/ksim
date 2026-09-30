use ksim_autoscalers::reference::{Oracle, StaticReplicas, replicas_for_demand};
use ksim_autoscalers::traits::ReplicaRecommender;
use ksim_core::{
    autoscaler::Autoscaler,
    ids::ServiceId,
    observation::{Observation, ServiceObservation},
    time::{SimDuration, SimTime},
    workload::{ServiceDemand, WorkloadSource},
};

// ============================================================================
// replicas_for_demand
// ============================================================================

#[test]
fn replicas_for_demand_four_replicas() {
    // ceil(1000 / (500 * 0.5)) = ceil(4.0) = 4
    let result = replicas_for_demand(1000.0, 500, 0.5);
    assert_eq!(result, 4);
}

#[test]
fn replicas_for_demand_zero_demand_minimum_one() {
    // ceil(0 / 250) = 0, but max(1, 0) = 1
    let result = replicas_for_demand(0.0, 500, 0.5);
    assert_eq!(result, 1);
}

#[test]
fn replicas_for_demand_ceil_rounds_up() {
    // ceil(1001 / 250) = ceil(4.004) = 5
    let result = replicas_for_demand(1001.0, 500, 0.5);
    assert_eq!(result, 5);
}

// ============================================================================
// StaticReplicas::new
// ============================================================================

#[test]
fn static_replicas_new() {
    let replicas = vec![2, 3, 1];
    let static_replicas = StaticReplicas::new(replicas.clone());
    assert_eq!(static_replicas.replicas(), &replicas);
}

// ============================================================================
// StaticReplicas::peak
// ============================================================================

#[test]
fn static_replicas_peak_two_services() {
    // Service 0: peak(100, 900, 300) = 900 → ceil(900 / (250 * 0.6)) = ceil(900 / 150) = 6
    // Service 1: peak(50) = 50 → ceil(50 / (100 * 0.6)) = ceil(50 / 60) = 1
    let samples = vec![vec![100.0, 900.0, 300.0], vec![50.0]];
    let cpu_requests = vec![250, 100];
    let target = 0.6;

    let static_replicas = StaticReplicas::peak(&samples, &cpu_requests, target);
    assert_eq!(static_replicas.replicas(), &[6, 1]);
}

// ============================================================================
// StaticReplicas::average
// ============================================================================

#[test]
fn static_replicas_average_two_services() {
    // Service 0: mean(100, 900, 300) = 433.33 → ceil(433.33 / (250 * 0.6)) = ceil(433.33 / 150) = 3
    // Service 1: mean(50) = 50 → ceil(50 / (100 * 0.6)) = ceil(50 / 60) = 1
    let samples = vec![vec![100.0, 900.0, 300.0], vec![50.0]];
    let cpu_requests = vec![250, 100];
    let target = 0.6;

    let static_replicas = StaticReplicas::average(&samples, &cpu_requests, target);
    assert_eq!(static_replicas.replicas(), &[3, 1]);
}

// ============================================================================
// StaticReplicas::act
// ============================================================================

#[test]
fn static_replicas_act_returns_consistent_actions() {
    let static_replicas_vec = vec![6, 1];
    let mut autoscaler = StaticReplicas::new(static_replicas_vec);

    let observation = Observation {
        now: SimTime::from_secs(0),
        services: vec![
            ServiceObservation {
                desired: 1,
                ready: 1,
                ..Default::default()
            },
            ServiceObservation {
                desired: 1,
                ready: 1,
                ..Default::default()
            },
        ],
        ..Default::default()
    };

    let actions1 = autoscaler.act(&observation);
    let actions2 = autoscaler.act(&observation);

    // Both calls should return the same actions
    assert_eq!(actions1.replicas, vec![Some(6), Some(1)]);
    assert_eq!(actions2.replicas, vec![Some(6), Some(1)]);
    assert_eq!(actions1, actions2);
}

// ============================================================================
// Oracle
// ============================================================================

/// Test `WorkloadSource` that returns constant 100 CPU before 60 s, then 1000 from 60 s onwards.
#[derive(Debug)]
struct ConstantAfterTimeWorkload {
    threshold_time: SimTime,
    before_demand: f64,
    after_demand: f64,
}

impl ConstantAfterTimeWorkload {
    fn new(threshold_time: SimTime, before: f64, after: f64) -> Self {
        Self {
            threshold_time,
            before_demand: before,
            after_demand: after,
        }
    }
}

impl WorkloadSource for ConstantAfterTimeWorkload {
    fn services(&self) -> usize {
        1
    }

    fn demand(&self, _service: ServiceId, now: SimTime) -> ServiceDemand {
        let cpu = if now < self.threshold_time {
            self.before_demand
        } else {
            self.after_demand
        };
        ServiceDemand {
            request_rate: 0.0,
            cpu_millis: cpu,
            memory_bytes: 0.0,
        }
    }
}

#[test]
fn oracle_at_now_zero_samples_within_lookahead() {
    // Demand: 100 before t=60s, 1000 from t=60s
    // At now=0, lookahead=30s, step=15s: samples at 0, 15, 30 → all < 60 → max=100
    // ceil(100 / (500 * 0.5)) = ceil(0.4) = 1
    let workload = Box::new(ConstantAfterTimeWorkload::new(
        SimTime::from_secs(60),
        100.0,
        1000.0,
    ));

    let mut oracle = Oracle::new(
        workload,
        vec![500],
        0.5,
        SimDuration::from_secs(30),
        SimDuration::from_secs(15),
    );

    let observation = Observation {
        now: SimTime::from_secs(0),
        services: vec![ServiceObservation {
            desired: 1,
            ready: 1,
            ..Default::default()
        }],
        ..Default::default()
    };

    let actions = oracle.act(&observation);

    // Should have one replica count for one service
    assert_eq!(actions.replicas.len(), 1);
    assert_eq!(actions.replicas[0], Some(1));
}

#[test]
fn oracle_at_now_thirty_samples_include_threshold() {
    // Demand: 100 before t=60s, 1000 from t=60s
    // At now=30s, lookahead=30s, step=15s: samples at 30, 45, 60 → 60 is at threshold → max=1000
    // ceil(1000 / (500 * 0.5)) = ceil(4.0) = 4
    let workload = Box::new(ConstantAfterTimeWorkload::new(
        SimTime::from_secs(60),
        100.0,
        1000.0,
    ));

    let mut oracle = Oracle::new(
        workload,
        vec![500],
        0.5,
        SimDuration::from_secs(30),
        SimDuration::from_secs(15),
    );

    let observation = Observation {
        now: SimTime::from_secs(30),
        services: vec![ServiceObservation {
            desired: 1,
            ready: 1,
            ..Default::default()
        }],
        ..Default::default()
    };

    let actions = oracle.act(&observation);

    // Should have one replica count for one service
    assert_eq!(actions.replicas.len(), 1);
    assert_eq!(actions.replicas[0], Some(4));
}

#[test]
fn oracle_act_returns_consistent_actions() {
    // Verify that Oracle returns the same actions on repeated calls (clairvoyant behavior)
    let workload = Box::new(ConstantAfterTimeWorkload::new(
        SimTime::from_secs(60),
        100.0,
        1000.0,
    ));

    let mut oracle = Oracle::new(
        workload,
        vec![500],
        0.5,
        SimDuration::from_secs(30),
        SimDuration::from_secs(15),
    );

    let observation = Observation {
        now: SimTime::from_secs(0),
        services: vec![ServiceObservation {
            desired: 1,
            ready: 1,
            ..Default::default()
        }],
        ..Default::default()
    };

    let actions1 = oracle.act(&observation);
    let actions2 = oracle.act(&observation);

    // Both calls should return the same actions
    assert_eq!(actions1, actions2);
    assert_eq!(actions1.replicas[0], Some(1));
}

// ============================================================================
// ReplicaRecommender trait
// ============================================================================

#[test]
fn static_replicas_recommend_replicas_matches_act() {
    let static_replicas_vec = vec![6, 1];
    let mut static_replicas = StaticReplicas::new(static_replicas_vec);

    let observation = Observation {
        now: SimTime::from_secs(0),
        services: vec![
            ServiceObservation {
                desired: 1,
                ready: 1,
                ..Default::default()
            },
            ServiceObservation {
                desired: 1,
                ready: 1,
                ..Default::default()
            },
        ],
        ..Default::default()
    };

    let recommendation = static_replicas.recommend_replicas(&observation);
    let actions = static_replicas.act(&observation);

    assert_eq!(recommendation, actions.replicas);
    assert_eq!(recommendation, vec![Some(6), Some(1)]);
}

#[test]
fn oracle_recommend_replicas_matches_act_at_now_zero() {
    let workload = Box::new(ConstantAfterTimeWorkload::new(
        SimTime::from_secs(60),
        100.0,
        1000.0,
    ));

    let mut oracle = Oracle::new(
        workload,
        vec![500],
        0.5,
        SimDuration::from_secs(30),
        SimDuration::from_secs(15),
    );

    let observation = Observation {
        now: SimTime::from_secs(0),
        services: vec![ServiceObservation {
            desired: 1,
            ready: 1,
            ..Default::default()
        }],
        ..Default::default()
    };

    let recommendation = oracle.recommend_replicas(&observation);
    let actions = oracle.act(&observation);

    assert_eq!(recommendation, actions.replicas);
    assert_eq!(recommendation, vec![Some(1)]);
}

#[test]
fn oracle_recommend_replicas_matches_act_at_now_thirty() {
    let workload = Box::new(ConstantAfterTimeWorkload::new(
        SimTime::from_secs(60),
        100.0,
        1000.0,
    ));

    let mut oracle = Oracle::new(
        workload,
        vec![500],
        0.5,
        SimDuration::from_secs(30),
        SimDuration::from_secs(15),
    );

    let observation = Observation {
        now: SimTime::from_secs(30),
        services: vec![ServiceObservation {
            desired: 1,
            ready: 1,
            ..Default::default()
        }],
        ..Default::default()
    };

    let recommendation = oracle.recommend_replicas(&observation);
    let actions = oracle.act(&observation);

    assert_eq!(recommendation, actions.replicas);
    assert_eq!(recommendation, vec![Some(4)]);
}
