use approx::assert_abs_diff_eq;
use ksim_autoscalers::hpa::{desired_replicas, HpaBehavior, HpaSpec, ScalingPolicy, SelectPolicy};
use ksim_core::{
    ids::ServiceId,
    observation::{Observation, ServiceObservation},
    performance::ServiceSignals,
    resources::Resources,
    time::{SimDuration, SimTime},
};

// ============================================================================
// HpaBehavior defaults
// ============================================================================

#[test]
fn hpa_behavior_default_tolerance() {
    // upstream autoscaling/v2 defaults
    let behavior = HpaBehavior::default();
    assert_abs_diff_eq!(behavior.tolerance, 0.1);
}

#[test]
fn hpa_behavior_default_scale_up_window() {
    // upstream autoscaling/v2 defaults: scale up with a 0 s window
    let behavior = HpaBehavior::default();
    assert_eq!(behavior.scale_up.stabilization_window, SimDuration::ZERO);
}

#[test]
fn hpa_behavior_default_scale_up_policies() {
    // upstream autoscaling/v2 defaults: Max of 100% or 4 pods per 15s
    let behavior = HpaBehavior::default();
    assert_eq!(behavior.scale_up.select, SelectPolicy::Max);
    assert_eq!(behavior.scale_up.policies.len(), 2);
    // Order-insensitive check: ensure both policies are present
    let has_percent_100 = behavior.scale_up.policies.iter().any(|p| {
        matches!(p, ScalingPolicy::Percent { value: 100, period } if *period == SimDuration::from_secs(15))
    });
    let has_pods_4 = behavior.scale_up.policies.iter().any(|p| {
        matches!(p, ScalingPolicy::Pods { value: 4, period } if *period == SimDuration::from_secs(15))
    });
    assert!(has_percent_100, "scale_up should have Percent{{100, 15s}}");
    assert!(has_pods_4, "scale_up should have Pods{{4, 15s}}");
}

#[test]
fn hpa_behavior_default_scale_down_window() {
    // upstream autoscaling/v2 defaults: scale down with a 300 s window
    let behavior = HpaBehavior::default();
    assert_eq!(behavior.scale_down.stabilization_window, SimDuration::from_secs(300));
}

#[test]
fn hpa_behavior_default_scale_down_policies() {
    // upstream autoscaling/v2 defaults: 100% per 15s
    let behavior = HpaBehavior::default();
    assert_eq!(behavior.scale_down.select, SelectPolicy::Max);
    assert_eq!(behavior.scale_down.policies.len(), 1);
    assert_eq!(
        behavior.scale_down.policies[0],
        ScalingPolicy::Percent { value: 100, period: SimDuration::from_secs(15) }
    );
}

// ============================================================================
// desired_replicas
// ============================================================================

#[test]
fn desired_replicas_scale_up_from_3_to_6() {
    // current 3, ready 3, usage 1500 → 6
    // upstream replica_calculator: utilization = 1500 / (3 * 500) = 1.0, ratio = 1.0 / 0.5 = 2.0
    // desired = ceil(2.0 * 3) = 6
    let result = desired_replicas(
        3,    // current
        3,    // ready
        0,    // not_ready
        1500.0, // usage
        500,  // request (500m)
        0.5,  // target
        0.1,  // tolerance
    );
    assert_eq!(result, 6);
}

#[test]
fn desired_replicas_within_tolerance() {
    // usage 787.5 (ratio 1.05) → 3
    // utilization = 787.5 / (3 * 500) = 0.525, ratio = 0.525 / 0.5 = 1.05
    // Within tolerance [0.9, 1.1], keep current
    let result = desired_replicas(
        3,    // current
        3,    // ready
        0,    // not_ready
        787.5, // usage
        500,  // request
        0.5,  // target
        0.1,  // tolerance
    );
    assert_eq!(result, 3);
}

#[test]
fn desired_replicas_scale_down_to_2() {
    // usage 375 → 2
    // utilization = 375 / (3 * 500) = 0.25, ratio = 0.25 / 0.5 = 0.5
    // desired = ceil(0.5 * 3) = ceil(1.5) = 2
    let result = desired_replicas(
        3,    // current
        3,    // ready
        0,    // not_ready
        375.0, // usage
        500,  // request
        0.5,  // target
        0.1,  // tolerance
    );
    assert_eq!(result, 2);
}

#[test]
fn desired_replicas_with_not_ready_within_tolerance_scale_up_direction() {
    // ready 2, not_ready 2, current 4, usage 1000 → 4
    // First compute with ready only: ratio = 1000 / (2 * 500) / 0.5 = 2.0 (outside tolerance, scale-up direction)
    // For scale-up, recompute with ready+not_ready at 0% usage: ratio = 1000 / (4 * 500) / 0.5 = 1.0
    // Within tolerance [0.9, 1.1], keep current 4
    let result = desired_replicas(
        4,    // current
        2,    // ready
        2,    // not_ready
        1000.0, // usage
        500,  // request
        0.5,  // target
        0.1,  // tolerance
    );
    assert_eq!(result, 4);
}

#[test]
fn desired_replicas_with_not_ready_scale_up() {
    // ready 2, not_ready 1, current 3, usage 2000 → 8
    // First compute with ready only: ratio = 2000 / (2 * 500) / 0.5 = 4.0 (scale-up)
    // Recompute with ready+not_ready: utilization = 2000 / (3 * 500) = 2.667, ratio = 2.667 / 0.5 = 5.333
    // Wait, that's not matching. Let me recalculate.
    // utilization = 2000 / (3 * 500) = 2000 / 1500 = 1.333...
    // ratio = 1.333... / 0.5 = 2.667
    // desired = ceil(2.667 * 3) = ceil(8.0) = 8
    let result = desired_replicas(
        3,    // current
        2,    // ready
        1,    // not_ready
        2000.0, // usage
        500,  // request
        0.5,  // target
        0.1,  // tolerance
    );
    assert_eq!(result, 8);
}

#[test]
fn desired_replicas_scale_down_ignores_not_ready() {
    // ready 2, not_ready 2, current 4, usage 250 → 1
    // First compute with ready only: ratio = 250 / (2 * 500) / 0.5 = 0.5 (outside tolerance, scale-down direction)
    // For scale-down, ignore not-ready: desired = ceil(0.5 * 2) = 1
    let result = desired_replicas(
        4,    // current
        2,    // ready
        2,    // not_ready
        250.0, // usage
        500,  // request
        0.5,  // target
        0.1,  // tolerance
    );
    assert_eq!(result, 1);
}

#[test]
fn desired_replicas_no_ready_pods() {
    // ready 0 → current
    // No ready pods to compute on, return current
    let result = desired_replicas(
        5,    // current
        0,    // ready
        2,    // not_ready
        1000.0, // usage
        500,  // request
        0.5,  // target
        0.1,  // tolerance
    );
    assert_eq!(result, 5);
}

#[test]
fn desired_replicas_zero_usage() {
    // usage 0 with ready 2, current 2 → 0
    // utilization = 0 / (2 * 500) = 0, ratio = 0 / 0.5 = 0
    // desired = ceil(0 * 2) = 0
    let result = desired_replicas(
        2,   // current
        2,   // ready
        0,   // not_ready
        0.0, // usage
        500, // request
        0.5, // target
        0.1, // tolerance
    );
    assert_eq!(result, 0);
}

// ============================================================================
// Hpa::recommend with scaling policies and stabilization windows
// ============================================================================

#[test]
fn hpa_recommend_scale_up_limited_by_policies() {
    // scale-up limited by policies: current 1, ready 1, usage 5000 (raw 20) → 5
    // Pods 4 beats Percent 100 → Max
    use ksim_autoscalers::hpa::Hpa;

    let spec = HpaSpec {
        min_replicas: 1,
        max_replicas: 10,
        target_cpu_utilization: 0.5,
        behavior: HpaBehavior::default(),
    };
    let mut hpa = Hpa::new(vec![Some(spec)]);
    let service_id = ServiceId(0);

    let observation = ServiceObservation {
        desired: 1,
        ready: 1,
        pending: 0,
        unschedulable: 0,
        outdated: 0,
        requests: Resources::new(500, 0),
        limits: Resources::new(500, 0),
        signals: ServiceSignals {
            cpu_usage_millis: 5000.0,
            ..Default::default()
        },
    };

    let recommendation = hpa.recommend(service_id, &observation, SimTime::ZERO);
    // Raw desired: 20 replicas (5000 / (1 * 500) / 0.5 = 20)
    // Scale-up policies at t=0: Percent 100 allows 1+ceil(1*100%)=2, Pods 4 allows 1+4=5
    // Max selects 5
    assert_eq!(recommendation, Some(5));
}

#[test]
fn hpa_recommend_scale_up_continued_high_load() {
    // next call at +15s with current 5 and still high load → 10
    // period start 5; Percent gives 5+ceil(5*100%)=10, Pods gives 5+4=9 → Max 10; clamp max 10
    use ksim_autoscalers::hpa::Hpa;

    let spec = HpaSpec {
        min_replicas: 1,
        max_replicas: 10,
        target_cpu_utilization: 0.5,
        behavior: HpaBehavior::default(),
    };
    let mut hpa = Hpa::new(vec![Some(spec)]);
    let service_id = ServiceId(0);

    // First call at t=0
    let observation = ServiceObservation {
        desired: 1,
        ready: 1,
        pending: 0,
        unschedulable: 0,
        outdated: 0,
        requests: Resources::new(500, 0),
        limits: Resources::new(500, 0),
        signals: ServiceSignals {
            cpu_usage_millis: 5000.0,
            ..Default::default()
        },
    };
    let _ = hpa.recommend(service_id, &observation, SimTime::ZERO);

    // Second call at t=15s with current=5, still high load
    let observation2 = ServiceObservation {
        desired: 5,
        ready: 5,
        pending: 0,
        unschedulable: 0,
        outdated: 0,
        requests: Resources::new(500, 0),
        limits: Resources::new(500, 0),
        signals: ServiceSignals {
            cpu_usage_millis: 5000.0,
            ..Default::default()
        },
    };
    let recommendation = hpa.recommend(service_id, &observation2, SimTime::from_secs(15));
    // Raw desired still 20
    // Scale-up policies: Percent 100 gives 5+ceil(5*100%)=10, Pods 4 gives 5+4=9
    // Max selects 10, clamped to max 10
    assert_eq!(recommendation, Some(10));
}

#[test]
fn hpa_recommend_scale_down_stabilization_window() {
    // scale-down stabilisation: at t=0 current 10 high load recommendation 10;
    // from t=15s on load drops (raw 2); returns 10 at t=15..285s;
    // at t=300s the t=0 record is outside the window (a record counts only if its timestamp is
    // strictly after now-300s) so returns 2
    use ksim_autoscalers::hpa::Hpa;

    let spec = HpaSpec {
        min_replicas: 1,
        max_replicas: 10,
        target_cpu_utilization: 0.5,
        behavior: HpaBehavior::default(),
    };
    let mut hpa = Hpa::new(vec![Some(spec)]);
    let service_id = ServiceId(0);

    // t=0, high load, current 10
    let observation = ServiceObservation {
        desired: 10,
        ready: 10,
        pending: 0,
        unschedulable: 0,
        outdated: 0,
        requests: Resources::new(500, 0),
        limits: Resources::new(500, 0),
        signals: ServiceSignals {
            cpu_usage_millis: 5000.0, // high load → recommendation 10
            ..Default::default()
        },
    };
    let rec_t0 = hpa.recommend(service_id, &observation, SimTime::ZERO);
    assert_eq!(rec_t0, Some(10));

    // t=15s, load drops, but within stabilization window
    let observation_low = ServiceObservation {
        desired: 10,
        ready: 10,
        pending: 0,
        unschedulable: 0,
        outdated: 0,
        requests: Resources::new(500, 0),
        limits: Resources::new(500, 0),
        signals: ServiceSignals {
            cpu_usage_millis: 500.0, // utilisation 0.1, ratio 0.2 → raw ceil(0.2 * 10) = 2
            ..Default::default()
        },
    };
    let rec_t15 = hpa.recommend(service_id, &observation_low, SimTime::from_secs(15));
    // Scale-down takes MAX of records within window, t=0 record (10) is still within window
    assert_eq!(rec_t15, Some(10));

    // t=300s, t=0 record is outside window (strictly after now-300s)
    let rec_t300 = hpa.recommend(service_id, &observation_low, SimTime::from_secs(300));
    // Only low load records are in window now → recommendation 2
    assert_eq!(rec_t300, Some(2));
}

#[test]
fn hpa_recommend_clamp_to_min() {
    // clamp: raw 0 → min_replicas 1
    use ksim_autoscalers::hpa::Hpa;

    let spec = HpaSpec {
        min_replicas: 1,
        max_replicas: 10,
        target_cpu_utilization: 0.5,
        behavior: HpaBehavior::default(),
    };
    let mut hpa = Hpa::new(vec![Some(spec)]);
    let service_id = ServiceId(0);

    let observation = ServiceObservation {
        desired: 5,
        ready: 5,
        pending: 0,
        unschedulable: 0,
        outdated: 0,
        requests: Resources::new(500, 0),
        limits: Resources::new(500, 0),
        signals: ServiceSignals {
            cpu_usage_millis: 0.0, // very low usage
            ..Default::default()
        },
    };
    let recommendation = hpa.recommend(service_id, &observation, SimTime::ZERO);
    // Raw desired would be 0, but clamped to min_replicas 1
    assert_eq!(recommendation, Some(1));
}

#[test]
fn hpa_recommend_clamp_to_max() {
    // clamp: raw 50 at current 10 → 10
    use ksim_autoscalers::hpa::Hpa;

    let spec = HpaSpec {
        min_replicas: 1,
        max_replicas: 10,
        target_cpu_utilization: 0.5,
        behavior: HpaBehavior::default(),
    };
    let mut hpa = Hpa::new(vec![Some(spec)]);
    let service_id = ServiceId(0);

    let observation = ServiceObservation {
        desired: 10,
        ready: 10,
        pending: 0,
        unschedulable: 0,
        outdated: 0,
        requests: Resources::new(500, 0),
        limits: Resources::new(500, 0),
        signals: ServiceSignals {
            cpu_usage_millis: 250_000.0, // extremely high usage → would give 50
            ..Default::default()
        },
    };
    let recommendation = hpa.recommend(service_id, &observation, SimTime::ZERO);
    // Raw desired would be 50, clamped to max_replicas 10
    assert_eq!(recommendation, Some(10));
}

// ============================================================================
// Hpa::recommend_replicas for multiple services
// ============================================================================

#[test]
fn hpa_recommend_replicas_single_service_with_spec() {
    // Single service with spec should return one entry
    use ksim_autoscalers::hpa::Hpa;
    use ksim_autoscalers::traits::ReplicaRecommender;

    let spec = HpaSpec {
        min_replicas: 1,
        max_replicas: 10,
        target_cpu_utilization: 0.5,
        behavior: HpaBehavior::default(),
    };
    let mut hpa = Hpa::new(vec![Some(spec)]);

    let observation = Observation {
        now: SimTime::ZERO,
        services: vec![ServiceObservation {
            desired: 3,
            ready: 3,
            pending: 0,
            unschedulable: 0,
            outdated: 0,
            requests: Resources::new(500, 0),
            limits: Resources::new(500, 0),
            signals: ServiceSignals {
                cpu_usage_millis: 1500.0,
                ..Default::default()
            },
        }],
        pools: vec![],
        nodes: vec![],
        node_pods: vec![],
    };

    let recommendations = hpa.recommend_replicas(&observation);
    assert_eq!(recommendations.len(), 1);
    assert_eq!(recommendations[0], Some(6)); // Should scale up to 6
}

#[test]
fn hpa_recommend_replicas_service_without_spec() {
    // Service without spec should return None
    use ksim_autoscalers::hpa::Hpa;
    use ksim_autoscalers::traits::ReplicaRecommender;

    let mut hpa = Hpa::new(vec![None]);

    let observation = Observation {
        now: SimTime::ZERO,
        services: vec![ServiceObservation {
            desired: 3,
            ready: 3,
            pending: 0,
            unschedulable: 0,
            outdated: 0,
            requests: Resources::new(500, 0),
            limits: Resources::new(500, 0),
            signals: ServiceSignals {
                cpu_usage_millis: 1500.0,
                ..Default::default()
            },
        }],
        pools: vec![],
        nodes: vec![],
        node_pods: vec![],
    };

    let recommendations = hpa.recommend_replicas(&observation);
    assert_eq!(recommendations.len(), 1);
    assert_eq!(recommendations[0], None); // No spec, no recommendation
}

#[test]
fn hpa_recommend_replicas_multiple_services() {
    // Multiple services should return one entry per service in observation order
    use ksim_autoscalers::hpa::Hpa;
    use ksim_autoscalers::traits::ReplicaRecommender;

    let spec1 = HpaSpec {
        min_replicas: 1,
        max_replicas: 10,
        target_cpu_utilization: 0.5,
        behavior: HpaBehavior::default(),
    };
    let spec2 = HpaSpec {
        min_replicas: 2,
        max_replicas: 20,
        target_cpu_utilization: 0.8,
        behavior: HpaBehavior::default(),
    };
    let mut hpa = Hpa::new(vec![Some(spec1), Some(spec2)]);

    let observation = Observation {
        now: SimTime::ZERO,
        services: vec![
            ServiceObservation {
                desired: 3,
                ready: 3,
                pending: 0,
                unschedulable: 0,
                outdated: 0,
                requests: Resources::new(500, 0),
                limits: Resources::new(500, 0),
                signals: ServiceSignals {
                    cpu_usage_millis: 1500.0, // high for target 0.5
                    ..Default::default()
                },
            },
            ServiceObservation {
                desired: 5,
                ready: 5,
                pending: 0,
                unschedulable: 0,
                outdated: 0,
                requests: Resources::new(500, 0),
                limits: Resources::new(500, 0),
                signals: ServiceSignals {
                    cpu_usage_millis: 2000.0, // normal for target 0.8
                    ..Default::default()
                },
            },
        ],
        pools: vec![],
        nodes: vec![],
        node_pods: vec![],
    };

    let recommendations = hpa.recommend_replicas(&observation);
    assert_eq!(recommendations.len(), 2);
    assert!(recommendations[0].is_some());
    assert!(recommendations[1].is_some());
}

// ============================================================================
// Custom behavior: scale_up select Disabled
// ============================================================================

#[test]
fn hpa_recommend_scale_up_disabled() {
    // Custom behavior: scale_up select Disabled → never above current
    use ksim_autoscalers::hpa::Hpa;

    let mut behavior = HpaBehavior::default();
    behavior.scale_up.select = SelectPolicy::Disabled;

    let spec = HpaSpec {
        min_replicas: 1,
        max_replicas: 10,
        target_cpu_utilization: 0.5,
        behavior,
    };
    let mut hpa = Hpa::new(vec![Some(spec)]);
    let service_id = ServiceId(0);

    let observation = ServiceObservation {
        desired: 2,
        ready: 2,
        pending: 0,
        unschedulable: 0,
        outdated: 0,
        requests: Resources::new(500, 0),
        limits: Resources::new(500, 0),
        signals: ServiceSignals {
            cpu_usage_millis: 5000.0, // high load, would normally scale up to 20
            ..Default::default()
        },
    };

    let recommendation = hpa.recommend(service_id, &observation, SimTime::ZERO);
    // Scale-up is disabled, so should not go above current (2)
    // Raw desired would be 20, but Disabled policy prevents scaling up
    assert_eq!(recommendation, Some(2));
}

#[test]
fn hpa_recommend_scale_down_disabled_allows_scale_up() {
    // Custom behavior: scale_down select Disabled but scale_up enabled
    // Should allow scale-up and block scale-down
    use ksim_autoscalers::hpa::Hpa;

    let mut behavior = HpaBehavior::default();
    behavior.scale_down.select = SelectPolicy::Disabled;

    let spec = HpaSpec {
        min_replicas: 1,
        max_replicas: 10,
        target_cpu_utilization: 0.5,
        behavior,
    };
    let mut hpa = Hpa::new(vec![Some(spec)]);
    let service_id = ServiceId(0);

    // First: establish high load state
    let observation_high = ServiceObservation {
        desired: 5,
        ready: 5,
        pending: 0,
        unschedulable: 0,
        outdated: 0,
        requests: Resources::new(500, 0),
        limits: Resources::new(500, 0),
        signals: ServiceSignals {
            cpu_usage_millis: 2500.0, // utilization 1.0, ratio 2.0 → recommend 10
            ..Default::default()
        },
    };
    let rec_high = hpa.recommend(service_id, &observation_high, SimTime::ZERO);
    assert_eq!(rec_high, Some(10)); // Can scale up

    // Then: low load, should stay at current due to disabled scale-down
    let observation_low = ServiceObservation {
        desired: 10,
        ready: 10,
        pending: 0,
        unschedulable: 0,
        outdated: 0,
        requests: Resources::new(500, 0),
        limits: Resources::new(500, 0),
        signals: ServiceSignals {
            cpu_usage_millis: 500.0, // very low
            ..Default::default()
        },
    };
    let rec_low = hpa.recommend(service_id, &observation_low, SimTime::from_secs(15));
    // Scale-down is disabled, so should not scale down
    assert_eq!(rec_low, Some(10)); // Stays at current
}
