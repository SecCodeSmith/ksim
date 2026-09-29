#![allow(clippy::cast_precision_loss)]

use ksim_autoscalers::traits::ResourceRecommender;
use ksim_autoscalers::vpa::{UpdateMode, Vpa, VpaConfig};
use ksim_core::{
    ids::ServiceId,
    observation::{Observation, ServiceObservation},
    performance::ServiceSignals,
    resources::{MIB, Resources},
    time::{SimDuration, SimTime},
};

#[test]
fn vpa_config_default_values() {
    let cfg = VpaConfig::default();
    assert_eq!(cfg.recommender_interval, SimDuration::from_secs(60));
    assert_eq!(cfg.updater_interval, SimDuration::from_secs(60));
    assert_eq!(cfg.half_life, SimDuration::from_secs(24 * 3600));
    assert!((cfg.target_percentile - 0.9).abs() < f64::EPSILON);
    assert!((cfg.lower_percentile - 0.5).abs() < f64::EPSILON);
    assert!((cfg.upper_percentile - 0.95).abs() < f64::EPSILON);
    assert!((cfg.safety_margin - 0.15).abs() < f64::EPSILON);
    assert_eq!(cfg.min_cpu_millis, 25);
    assert_eq!(cfg.min_memory_bytes, 250 * MIB);
    assert_eq!(
        cfg.memory_aggregation_interval,
        SimDuration::from_secs(24 * 3600)
    );
    assert_eq!(cfg.min_replicas, 2);
    assert!((cfg.eviction_tolerance - 0.5).abs() < f64::EPSILON);
    assert_eq!(cfg.mode, UpdateMode::Auto);
}

#[test]
fn recommendation_none_before_samples() {
    let cfg = VpaConfig::default();
    let vpa = Vpa::new(cfg, vec![true]);
    let service = ServiceId(0);
    assert_eq!(vpa.recommendation(service), None);
}

#[test]
fn recommendation_none_for_disabled_services() {
    let cfg = VpaConfig::default();
    let vpa = Vpa::new(cfg, vec![false]);
    let service = ServiceId(0);
    assert_eq!(vpa.recommendation(service), None);
}

#[test]
fn constant_usage_produces_recommendation() {
    let cfg = VpaConfig::default();
    let mut vpa = Vpa::new(cfg, vec![true]);

    // Constant per-pod usage: 4 ready pods, 200m CPU each = 800m total
    // 300 MiB memory per pod
    let cpu_per_pod_millis = 200.0;
    let memory_per_pod_bytes = 300.0 * (MIB as f64);
    let ready_pods = 4u32;

    // Collect samples over 2 days
    let mut now = SimTime::from_secs(0);

    for _ in 0..48 {
        // 48 samples of 60s each = 2880s
        let obs = Observation {
            now,
            services: vec![ServiceObservation {
                desired: ready_pods,
                ready: ready_pods,
                pending: 0,
                unschedulable: 0,
                outdated: 0,
                requests: Resources::new(200, 256 * MIB),
                limits: Resources::new(500, 512 * MIB),
                signals: ServiceSignals {
                    cpu_usage_millis: cpu_per_pod_millis * f64::from(ready_pods),
                    memory_usage_bytes: memory_per_pod_bytes * f64::from(ready_pods),
                    ..Default::default()
                },
            }],
            pools: vec![],
            nodes: vec![],
            node_pods: vec![],
        };
        vpa.recommend_resources(&obs);
        now = now + SimDuration::from_secs(60);
    }

    // After samples, recommendation should exist
    let rec = vpa.recommendation(ServiceId(0));
    assert!(rec.is_some());
    let rec = rec.unwrap();

    // buckets near 200m are ~10 % wide (first 10m, ratio 1.05): target within [200*1.15, 200*1.15*1.10]
    let target_cpu_lower = 200.0 * 1.15;
    let target_cpu_upper = 200.0 * 1.15 * 1.10;
    assert!(
        rec.target.cpu_millis as f64 >= target_cpu_lower
            && rec.target.cpu_millis as f64 <= target_cpu_upper,
        "target cpu={} should be in [{}, {}]",
        rec.target.cpu_millis,
        target_cpu_lower,
        target_cpu_upper
    );

    // target memory within [300MiB*1.15, 300MiB*1.15*1.10] (bucket width ~8 % at 300 MiB)
    let target_memory_lower = (300.0 * MIB as f64) * 1.15;
    let target_memory_upper = (300.0 * MIB as f64) * 1.15 * 1.10;
    assert!(
        rec.target.memory_bytes as f64 >= target_memory_lower
            && rec.target.memory_bytes as f64 <= target_memory_upper,
        "target memory={} should be in [{}, {}]",
        rec.target.memory_bytes,
        target_memory_lower,
        target_memory_upper
    );

    // lower <= target <= upper
    assert!(rec.lower_bound.cpu_millis <= rec.target.cpu_millis);
    assert!(rec.target.cpu_millis <= rec.upper_bound.cpu_millis);
    assert!(rec.lower_bound.memory_bytes <= rec.target.memory_bytes);
    assert!(rec.target.memory_bytes <= rec.upper_bound.memory_bytes);

    // upper ≈ target*1.5 within 6%, so assert upper >= target*1.4
    let upper_to_target_ratio = rec.upper_bound.cpu_millis as f64 / rec.target.cpu_millis as f64;
    assert!(
        upper_to_target_ratio >= 1.4,
        "upper/target={upper_to_target_ratio} should be >= 1.4"
    );
}

#[test]
fn minimums_applied() {
    let cfg = VpaConfig::default();
    let mut vpa = Vpa::new(cfg, vec![true]);

    // Tiny usage: 1m CPU / 1MiB memory per pod
    let cpu_per_pod_millis = 1.0;
    let memory_per_pod_bytes = 1.0 * MIB as f64;
    let ready_pods = 2u32;

    let mut now = SimTime::from_secs(0);
    for _ in 0..48 {
        let obs = Observation {
            now,
            services: vec![ServiceObservation {
                desired: ready_pods,
                ready: ready_pods,
                pending: 0,
                unschedulable: 0,
                outdated: 0,
                requests: Resources::new(10, 10 * MIB),
                limits: Resources::new(100, 100 * MIB),
                signals: ServiceSignals {
                    cpu_usage_millis: cpu_per_pod_millis * f64::from(ready_pods),
                    memory_usage_bytes: memory_per_pod_bytes * f64::from(ready_pods),
                    ..Default::default()
                },
            }],
            pools: vec![],
            nodes: vec![],
            node_pods: vec![],
        };
        vpa.recommend_resources(&obs);
        now = now + SimDuration::from_secs(60);
    }

    let rec = vpa.recommendation(ServiceId(0));
    assert!(rec.is_some());
    let rec = rec.unwrap();

    // Should be raised to minimums
    assert!(rec.target.cpu_millis >= 25);
    assert!(rec.target.memory_bytes >= 250 * MIB);
}

#[test]
fn updater_auto_mode_below_lower_bound() {
    let cfg = VpaConfig::default();
    let mut vpa = Vpa::new(cfg, vec![true]);

    // Service requests 100m/256MiB, well below lower bound
    let mut now = SimTime::from_secs(0);

    // Add samples first to build up recommendations
    for _ in 0..48 {
        let obs = Observation {
            now,
            services: vec![ServiceObservation {
                desired: 4,
                ready: 4,
                pending: 0,
                unschedulable: 0,
                outdated: 0,
                requests: Resources::new(100, 256 * MIB),
                limits: Resources::new(500, 512 * MIB),
                signals: ServiceSignals {
                    cpu_usage_millis: 200.0 * 4.0,
                    memory_usage_bytes: (300.0 * MIB as f64) * 4.0,
                    ..Default::default()
                },
            }],
            pools: vec![],
            nodes: vec![],
            node_pods: vec![],
        };
        vpa.recommend_resources(&obs);
        now = now + SimDuration::from_secs(60);
    }

    // First updater run: outdated is 0
    let plan = vpa.recommend_resources(&Observation {
        now,
        services: vec![ServiceObservation {
            desired: 4,
            ready: 4,
            pending: 0,
            unschedulable: 0,
            outdated: 0,
            requests: Resources::new(100, 256 * MIB),
            limits: Resources::new(500, 512 * MIB),
            signals: ServiceSignals {
                cpu_usage_millis: 200.0 * 4.0,
                memory_usage_bytes: (300.0 * MIB as f64) * 4.0,
                ..Default::default()
            },
        }],
        pools: vec![],
        nodes: vec![],
        node_pods: vec![],
    });

    // Should return new requests (Some(target))
    assert!(!plan.requests.is_empty());
    assert!(plan.requests[0].is_some());
    // Should not evict outdated pods since outdated is 0
    assert_eq!(plan.evict[0], 0);

    // Next run (60s later) with outdated 4, desired 4
    now = now + SimDuration::from_secs(60);
    let plan = vpa.recommend_resources(&Observation {
        now,
        services: vec![ServiceObservation {
            desired: 4,
            ready: 4,
            pending: 0,
            unschedulable: 0,
            outdated: 4,
            requests: Resources::new(100, 256 * MIB),
            limits: Resources::new(500, 512 * MIB),
            signals: ServiceSignals {
                cpu_usage_millis: 200.0 * 4.0,
                memory_usage_bytes: (300.0 * MIB as f64) * 4.0,
                ..Default::default()
            },
        }],
        pools: vec![],
        nodes: vec![],
        node_pods: vec![],
    });

    // Should evict 2 (max(1, floor(0.5*4)))
    assert_eq!(plan.evict[0], 2);
}

#[test]
fn updater_auto_mode_min_replicas_check() {
    let cfg = VpaConfig::default();
    let mut vpa = Vpa::new(cfg, vec![true]);

    let mut now = SimTime::from_secs(0);

    // Add samples
    for _ in 0..48 {
        let obs = Observation {
            now,
            services: vec![ServiceObservation {
                desired: 1,
                ready: 1,
                pending: 0,
                unschedulable: 0,
                outdated: 0,
                requests: Resources::new(100, 256 * MIB),
                limits: Resources::new(500, 512 * MIB),
                signals: ServiceSignals {
                    cpu_usage_millis: 200.0,
                    memory_usage_bytes: 300.0 * MIB as f64,
                    ..Default::default()
                },
            }],
            pools: vec![],
            nodes: vec![],
            node_pods: vec![],
        };
        vpa.recommend_resources(&obs);
        now = now + SimDuration::from_secs(60);
    }

    // With desired 1 (below min_replicas 2), should not evict
    let plan = vpa.recommend_resources(&Observation {
        now,
        services: vec![ServiceObservation {
            desired: 1,
            ready: 1,
            pending: 0,
            unschedulable: 0,
            outdated: 1,
            requests: Resources::new(100, 256 * MIB),
            limits: Resources::new(500, 512 * MIB),
            signals: ServiceSignals {
                cpu_usage_millis: 200.0,
                memory_usage_bytes: 300.0 * MIB as f64,
                ..Default::default()
            },
        }],
        pools: vec![],
        nodes: vec![],
        node_pods: vec![],
    });

    assert_eq!(plan.evict[0], 0);
}

#[test]
fn updater_auto_mode_requests_in_bounds() {
    let cfg = VpaConfig::default();
    let mut vpa = Vpa::new(cfg, vec![true]);

    let mut now = SimTime::from_secs(0);

    // Add samples first
    for _ in 0..48 {
        let obs = Observation {
            now,
            services: vec![ServiceObservation {
                desired: 4,
                ready: 4,
                pending: 0,
                unschedulable: 0,
                outdated: 0,
                requests: Resources::new(200, 300 * MIB),
                limits: Resources::new(500, 512 * MIB),
                signals: ServiceSignals {
                    cpu_usage_millis: 200.0 * 4.0,
                    memory_usage_bytes: (300.0 * MIB as f64) * 4.0,
                    ..Default::default()
                },
            }],
            pools: vec![],
            nodes: vec![],
            node_pods: vec![],
        };
        vpa.recommend_resources(&obs);
        now = now + SimDuration::from_secs(60);
    }

    // Then run with requests already in bounds
    let plan = vpa.recommend_resources(&Observation {
        now,
        services: vec![ServiceObservation {
            desired: 4,
            ready: 4,
            pending: 0,
            unschedulable: 0,
            outdated: 0,
            requests: Resources::new(260, 400 * MIB),
            limits: Resources::new(500, 512 * MIB),
            signals: ServiceSignals {
                cpu_usage_millis: 200.0 * 4.0,
                memory_usage_bytes: (300.0 * MIB as f64) * 4.0,
                ..Default::default()
            },
        }],
        pools: vec![],
        nodes: vec![],
        node_pods: vec![],
    });

    // Should return None (no change needed)
    assert_eq!(plan.requests[0], None);
}

#[test]
fn initial_mode_never_evicts() {
    let mut cfg = VpaConfig::default();
    cfg = VpaConfig {
        mode: UpdateMode::Initial,
        ..cfg
    };
    let mut vpa = Vpa::new(cfg, vec![true]);

    let mut now = SimTime::from_secs(0);

    // Add samples
    for _ in 0..48 {
        let obs = Observation {
            now,
            services: vec![ServiceObservation {
                desired: 4,
                ready: 4,
                pending: 0,
                unschedulable: 0,
                outdated: 0,
                requests: Resources::new(100, 256 * MIB),
                limits: Resources::new(500, 512 * MIB),
                signals: ServiceSignals {
                    cpu_usage_millis: 200.0 * 4.0,
                    memory_usage_bytes: (300.0 * MIB as f64) * 4.0,
                    ..Default::default()
                },
            }],
            pools: vec![],
            nodes: vec![],
            node_pods: vec![],
        };
        vpa.recommend_resources(&obs);
        now = now + SimDuration::from_secs(60);
    }

    // Even with outdated pods, should not evict in Initial mode
    let plan = vpa.recommend_resources(&Observation {
        now,
        services: vec![ServiceObservation {
            desired: 4,
            ready: 4,
            pending: 0,
            unschedulable: 0,
            outdated: 4,
            requests: Resources::new(100, 256 * MIB),
            limits: Resources::new(500, 512 * MIB),
            signals: ServiceSignals {
                cpu_usage_millis: 200.0 * 4.0,
                memory_usage_bytes: (300.0 * MIB as f64) * 4.0,
                ..Default::default()
            },
        }],
        pools: vec![],
        nodes: vec![],
        node_pods: vec![],
    });

    assert_eq!(plan.evict[0], 0);
}

#[test]
fn off_mode_returns_empty() {
    let mut cfg = VpaConfig::default();
    cfg = VpaConfig {
        mode: UpdateMode::Off,
        ..cfg
    };
    let mut vpa = Vpa::new(cfg, vec![true]);

    let mut now = SimTime::from_secs(0);

    // Add samples
    for _ in 0..48 {
        let obs = Observation {
            now,
            services: vec![ServiceObservation {
                desired: 4,
                ready: 4,
                pending: 0,
                unschedulable: 0,
                outdated: 0,
                requests: Resources::new(100, 256 * MIB),
                limits: Resources::new(500, 512 * MIB),
                signals: ServiceSignals {
                    cpu_usage_millis: 200.0 * 4.0,
                    memory_usage_bytes: (300.0 * MIB as f64) * 4.0,
                    ..Default::default()
                },
            }],
            pools: vec![],
            nodes: vec![],
            node_pods: vec![],
        };
        vpa.recommend_resources(&obs);
        now = now + SimDuration::from_secs(60);
    }

    // Off mode should return None for everything
    let plan = vpa.recommend_resources(&Observation {
        now,
        services: vec![ServiceObservation {
            desired: 4,
            ready: 4,
            pending: 0,
            unschedulable: 0,
            outdated: 4,
            requests: Resources::new(100, 256 * MIB),
            limits: Resources::new(500, 512 * MIB),
            signals: ServiceSignals {
                cpu_usage_millis: 200.0 * 4.0,
                memory_usage_bytes: (300.0 * MIB as f64) * 4.0,
                ..Default::default()
            },
        }],
        pools: vec![],
        nodes: vec![],
        node_pods: vec![],
    });

    assert_eq!(plan.requests[0], None);
    assert_eq!(plan.evict[0], 0);
}

#[test]
fn recommend_resources_returns_one_entry_per_service() {
    let cfg = VpaConfig::default();
    let mut vpa = Vpa::new(cfg, vec![true, true, false]);

    let obs = Observation {
        now: SimTime::from_secs(0),
        services: vec![
            ServiceObservation {
                desired: 4,
                ready: 4,
                pending: 0,
                unschedulable: 0,
                outdated: 0,
                requests: Resources::new(100, 256 * MIB),
                limits: Resources::new(500, 512 * MIB),
                signals: ServiceSignals {
                    cpu_usage_millis: 200.0 * 4.0,
                    memory_usage_bytes: (300.0 * MIB as f64) * 4.0,
                    ..Default::default()
                },
            },
            ServiceObservation {
                desired: 2,
                ready: 2,
                pending: 0,
                unschedulable: 0,
                outdated: 0,
                requests: Resources::new(100, 256 * MIB),
                limits: Resources::new(500, 512 * MIB),
                signals: ServiceSignals {
                    cpu_usage_millis: 200.0 * 2.0,
                    memory_usage_bytes: (300.0 * MIB as f64) * 2.0,
                    ..Default::default()
                },
            },
            ServiceObservation {
                desired: 1,
                ready: 1,
                pending: 0,
                unschedulable: 0,
                outdated: 0,
                requests: Resources::new(100, 256 * MIB),
                limits: Resources::new(500, 512 * MIB),
                signals: ServiceSignals {
                    cpu_usage_millis: 200.0,
                    memory_usage_bytes: 300.0 * MIB as f64,
                    ..Default::default()
                },
            },
        ],
        pools: vec![],
        nodes: vec![],
        node_pods: vec![],
    };

    let plan = vpa.recommend_resources(&obs);
    assert_eq!(plan.requests.len(), 3);
    assert_eq!(plan.evict.len(), 3);
}
