use ksim_autoscalers::ca::{ClusterAutoscaler, ClusterAutoscalerConfig, Expander, nodes_needed};
use ksim_autoscalers::traits::NodeProvisioner;
use ksim_core::{
    cluster::NodePhase,
    ids::{NodeId, PoolId, ServiceId},
    observation::{
        NodeObservation, Observation, PodPlacement, PoolObservation, ServiceObservation,
    },
    resources::Resources,
    time::{SimDuration, SimTime},
};

#[test]
fn config_default() {
    let config = ClusterAutoscalerConfig::default();
    assert_eq!(config.scan_interval, SimDuration::from_secs(10));
    assert!((config.scale_down_utilization_threshold - 0.5).abs() < f64::EPSILON);
    assert_eq!(config.scale_down_unneeded_time, SimDuration::from_secs(600));
    assert_eq!(
        config.scale_down_delay_after_add,
        SimDuration::from_secs(600)
    );
    assert_eq!(config.max_empty_bulk_delete, 10);
    assert_eq!(config.expander, Expander::Random);
}

#[test]
fn nodes_needed_exact_fit() {
    // 1 pod of 300m/1GiB fits into node of 1000m/4GiB → 1 node needed
    let pods = vec![Resources::new(300, 1024 * 1024 * 1024)];
    let node_alloc = Resources::new(1000, 4 * 1024 * 1024 * 1024);
    assert_eq!(nodes_needed(&pods, node_alloc), 1);
}

#[test]
fn nodes_needed_multiple_pods() {
    // 5 pods of 300m/1GiB each, node 1000m/4GiB → ceil(5 * 300 / 1000) = 2 nodes
    let pods = vec![Resources::new(300, 1024 * 1024 * 1024); 5];
    let node_alloc = Resources::new(1000, 4 * 1024 * 1024 * 1024);
    assert_eq!(nodes_needed(&pods, node_alloc), 2);
}

#[test]
fn nodes_needed_pods_bigger_than_node() {
    // Pod of 1500m/1GiB bigger than node of 1000m/4GiB → ignored → 0 needed
    let pods = vec![Resources::new(1500, 1024 * 1024 * 1024)];
    let node_alloc = Resources::new(1000, 4 * 1024 * 1024 * 1024);
    assert_eq!(nodes_needed(&pods, node_alloc), 0);
}

#[test]
fn nodes_needed_all_pods_oversized() {
    // Only oversized pods → 0 needed
    let pods = vec![
        Resources::new(2000, 1024 * 1024 * 1024),
        Resources::new(3000, 1024 * 1024 * 1024),
    ];
    let node_alloc = Resources::new(1000, 4 * 1024 * 1024 * 1024);
    assert_eq!(nodes_needed(&pods, node_alloc), 0);
}

#[test]
fn nodes_needed_empty_pods() {
    let pods: Vec<Resources> = vec![];
    let node_alloc = Resources::new(1000, 4 * 1024 * 1024 * 1024);
    assert_eq!(nodes_needed(&pods, node_alloc), 0);
}

#[test]
fn scale_up_unschedulable_pods() {
    // 3 unschedulable pods of 600m/1GiB, pool A (1000m/4GiB, 1 ready, max 5)
    // → nodes_needed([600, 600, 600], 1000m/4GiB) = ceil(1800/1000) = 2
    // → room = max(5) - ready(1) - provisioning(0) = 4, capped at 2
    // → pool_delta [2]
    let mut ca = ClusterAutoscaler::new(ClusterAutoscalerConfig::default());

    let observation = Observation {
        now: SimTime::ZERO,
        services: vec![ServiceObservation {
            unschedulable: 3,
            requests: Resources::new(600, 1024 * 1024 * 1024),
            ..ServiceObservation::default()
        }],
        pools: vec![PoolObservation {
            ready: 1,
            provisioning: 0,
            draining: 0,
            allocatable: Resources::new(1000, 4 * 1024 * 1024 * 1024),
            min_nodes: 1,
            max_nodes: 5,
        }],
        nodes: vec![],
        node_pods: vec![],
    };

    let plan = ca.provision(&observation);
    assert_eq!(plan.pool_delta, vec![2]);
    assert!(plan.drain.is_empty());
}

#[test]
fn scale_up_capped_by_room() {
    // Provision room = max_nodes - ready - provisioning
    // Pool max 2, already 1 ready, 0 provisioning → room = 1
    // 3 unschedulable pods of 600m need 2 nodes → capped at room 1
    // → pool_delta [1]
    let mut ca = ClusterAutoscaler::new(ClusterAutoscalerConfig::default());

    let observation = Observation {
        now: SimTime::ZERO,
        services: vec![ServiceObservation {
            unschedulable: 3,
            requests: Resources::new(600, 1024 * 1024 * 1024),
            ..ServiceObservation::default()
        }],
        pools: vec![PoolObservation {
            ready: 1,
            provisioning: 0,
            draining: 0,
            allocatable: Resources::new(1000, 4 * 1024 * 1024 * 1024),
            min_nodes: 1,
            max_nodes: 2,
        }],
        nodes: vec![],
        node_pods: vec![],
    };

    let plan = ca.provision(&observation);
    assert_eq!(plan.pool_delta, vec![1]);
}

#[test]
fn scale_up_absorbed_by_provisioning() {
    // 3 unschedulable pods of 600m/1GiB (1800m total)
    // 1 provisioning node with 4000m allocatable → 1800m < 4000m fits
    // → no scale-up needed, pool_delta empty/all 0s
    let mut ca = ClusterAutoscaler::new(ClusterAutoscalerConfig::default());

    let observation = Observation {
        now: SimTime::ZERO,
        services: vec![ServiceObservation {
            unschedulable: 3,
            requests: Resources::new(600, 1024 * 1024 * 1024),
            ..ServiceObservation::default()
        }],
        pools: vec![PoolObservation {
            ready: 0,
            provisioning: 1,
            draining: 0,
            allocatable: Resources::new(4000, 16 * 1024 * 1024 * 1024),
            min_nodes: 1,
            max_nodes: 5,
        }],
        nodes: vec![NodeObservation {
            id: NodeId(0),
            pool: PoolId(0),
            phase: NodePhase::Provisioning,
            allocatable: Resources::new(4000, 16 * 1024 * 1024 * 1024),
            requested: Resources::ZERO,
            pods: 0..0,
        }],
        node_pods: vec![],
    };

    let plan = ca.provision(&observation);
    assert!(plan.pool_delta.is_empty() || plan.pool_delta == vec![0]);
    assert!(plan.drain.is_empty());
}

#[test]
fn expander_least_waste_single_pool() {
    // Verify least-waste picks the pool with lowest waste
    // Pools A (1000m/4GiB) and B (4000m/16GiB), 4 pods of 900m/1GiB each
    // A: 4 nodes, waste per node = (1000-900)/1000 + (4GiB-1GiB)/4GiB = 0.1 + 0.75 = 0.85, mean 0.85
    // B: 1 node holds 4*900=3600m, waste = (4000-3600)/4000 + (16GiB-4GiB)/16GiB = 0.1 + 0.75 = 0.85
    // Tie → pick lowest pool id A
    // Change to pods of 1500m/1GiB: A can't fit → only B option
    let mut ca = ClusterAutoscaler::new(ClusterAutoscalerConfig {
        expander: Expander::LeastWaste,
        ..ClusterAutoscalerConfig::default()
    });

    let observation = Observation {
        now: SimTime::ZERO,
        services: vec![ServiceObservation {
            unschedulable: 2,
            requests: Resources::new(1500, 1024 * 1024 * 1024),
            ..ServiceObservation::default()
        }],
        pools: vec![
            PoolObservation {
                allocatable: Resources::new(1000, 4 * 1024 * 1024 * 1024),
                min_nodes: 0,
                max_nodes: 10,
                ..PoolObservation::default()
            },
            PoolObservation {
                allocatable: Resources::new(4000, 16 * 1024 * 1024 * 1024),
                min_nodes: 0,
                max_nodes: 10,
                ..PoolObservation::default()
            },
        ],
        nodes: vec![],
        node_pods: vec![],
    };

    let plan = ca.provision(&observation);
    // Pool A can't fit 1500m pods → only pool B should get the delta
    assert_eq!(plan.pool_delta.len(), 2);
    assert_eq!(plan.pool_delta[0], 0); // A can't fit
    assert!(plan.pool_delta[1] > 0); // B should get the scale-up
}

#[test]
fn expander_random_deterministic() {
    // Random expander with fixed seed is deterministic across two instances
    let seed = 42;
    let config = ClusterAutoscalerConfig {
        expander: Expander::Random,
        seed,
        ..ClusterAutoscalerConfig::default()
    };

    let observation = Observation {
        now: SimTime::ZERO,
        services: vec![ServiceObservation {
            unschedulable: 1,
            requests: Resources::new(500, 1024 * 1024 * 1024),
            ..ServiceObservation::default()
        }],
        pools: vec![
            PoolObservation {
                allocatable: Resources::new(1000, 4 * 1024 * 1024 * 1024),
                max_nodes: 10,
                ..PoolObservation::default()
            },
            PoolObservation {
                allocatable: Resources::new(1000, 4 * 1024 * 1024 * 1024),
                max_nodes: 10,
                ..PoolObservation::default()
            },
        ],
        nodes: vec![],
        node_pods: vec![],
    };

    let mut ca1 = ClusterAutoscaler::new(config);
    let plan1 = ca1.provision(&observation);

    let mut ca2 = ClusterAutoscaler::new(config);
    let plan2 = ca2.provision(&observation);

    assert_eq!(plan1.pool_delta, plan2.pool_delta);
}

#[test]
fn scale_down_underutilized_node() {
    // Ready node with utilisation 0.2 (max of cpu/mem ratio)
    // cpu_used/cpu_alloc = 200/1000 = 0.2, mem_used/mem_alloc = 512/4GiB ≈ 0.125
    // max(0.2, 0.125) = 0.2 < 0.5 threshold → underutilized
    // But not drained before scale_down_unneeded_time elapsed (600s)
    let mut ca = ClusterAutoscaler::new(ClusterAutoscalerConfig::default());

    let observation = Observation {
        now: SimTime::from_secs(300), // Only 300s since node became underutilized
        services: vec![ServiceObservation {
            requests: Resources::new(200, 512 * 1024 * 1024),
            ..ServiceObservation::default()
        }],
        pools: vec![PoolObservation {
            ready: 1,
            allocatable: Resources::new(1000, 4 * 1024 * 1024 * 1024),
            min_nodes: 1,
            max_nodes: 5,
            ..PoolObservation::default()
        }],
        nodes: vec![NodeObservation {
            id: NodeId(0),
            pool: PoolId(0),
            phase: NodePhase::Ready,
            allocatable: Resources::new(1000, 4 * 1024 * 1024 * 1024),
            requested: Resources::new(200, 512 * 1024 * 1024),
            pods: 0..1,
        }],
        node_pods: vec![PodPlacement {
            service: ServiceId(0),
            requests: Resources::new(200, 512 * 1024 * 1024),
        }],
    };

    let plan = ca.provision(&observation);
    // Not yet drained
    assert!(!plan.drain.contains(&NodeId(0)));
}

#[test]
fn scale_down_node_after_unneeded_time() {
    // Node is unneeded since t0=0, drained at call with now >= t0 + 600s (scale_down_unneeded_time)
    // At t=600s, node should be drained
    let mut ca = ClusterAutoscaler::new(ClusterAutoscalerConfig::default());

    let observation = Observation {
        now: SimTime::from_secs(600),
        services: vec![ServiceObservation {
            requests: Resources::new(200, 512 * 1024 * 1024),
            ..ServiceObservation::default()
        }],
        pools: vec![PoolObservation {
            ready: 2,
            allocatable: Resources::new(1000, 4 * 1024 * 1024 * 1024),
            min_nodes: 1,
            max_nodes: 5,
            ..PoolObservation::default()
        }],
        nodes: vec![
            NodeObservation {
                id: NodeId(0),
                pool: PoolId(0),
                phase: NodePhase::Ready,
                allocatable: Resources::new(1000, 4 * 1024 * 1024 * 1024),
                requested: Resources::new(200, 512 * 1024 * 1024),
                pods: 0..1,
            },
            NodeObservation {
                id: NodeId(1),
                pool: PoolId(0),
                phase: NodePhase::Ready,
                allocatable: Resources::new(1000, 4 * 1024 * 1024 * 1024),
                requested: Resources::ZERO,
                pods: 1..1,
            },
        ],
        node_pods: vec![PodPlacement {
            service: ServiceId(0),
            requests: Resources::new(200, 512 * 1024 * 1024),
        }],
    };

    let plan = ca.provision(&observation);
    // Node 0 pods fit in Node 1 (200m/512MiB < 1000m/4GiB free) → should be drained
    assert!(plan.drain.contains(&NodeId(0)));
}

#[test]
fn scale_down_respects_min_nodes() {
    // Node would be drained but pool is at min_nodes → not drained
    let mut ca = ClusterAutoscaler::new(ClusterAutoscalerConfig::default());

    let observation = Observation {
        now: SimTime::from_secs(600),
        services: vec![ServiceObservation {
            requests: Resources::new(200, 512 * 1024 * 1024),
            ..ServiceObservation::default()
        }],
        pools: vec![PoolObservation {
            ready: 1,
            allocatable: Resources::new(1000, 4 * 1024 * 1024 * 1024),
            min_nodes: 1, // Can't go below 1
            max_nodes: 5,
            ..PoolObservation::default()
        }],
        nodes: vec![NodeObservation {
            id: NodeId(0),
            pool: PoolId(0),
            phase: NodePhase::Ready,
            allocatable: Resources::new(1000, 4 * 1024 * 1024 * 1024),
            requested: Resources::new(200, 512 * 1024 * 1024),
            pods: 0..1,
        }],
        node_pods: vec![PodPlacement {
            service: ServiceId(0),
            requests: Resources::new(200, 512 * 1024 * 1024),
        }],
    };

    let plan = ca.provision(&observation);
    // Can't drain because pool would go below min_nodes
    assert!(!plan.drain.contains(&NodeId(0)));
}

#[test]
fn scale_down_pods_dont_fit_elsewhere() {
    // Node's pods don't fit in other ready nodes → node not drained
    let mut ca = ClusterAutoscaler::new(ClusterAutoscalerConfig::default());

    let observation = Observation {
        now: SimTime::from_secs(600),
        services: vec![ServiceObservation {
            requests: Resources::new(900, 2 * 1024 * 1024 * 1024),
            ..ServiceObservation::default()
        }],
        pools: vec![PoolObservation {
            ready: 2,
            allocatable: Resources::new(1000, 4 * 1024 * 1024 * 1024),
            min_nodes: 1,
            max_nodes: 5,
            ..PoolObservation::default()
        }],
        nodes: vec![
            NodeObservation {
                id: NodeId(0),
                pool: PoolId(0),
                phase: NodePhase::Ready,
                allocatable: Resources::new(1000, 4 * 1024 * 1024 * 1024),
                requested: Resources::new(900, 2 * 1024 * 1024 * 1024),
                pods: 0..1,
            },
            NodeObservation {
                id: NodeId(1),
                pool: PoolId(0),
                phase: NodePhase::Ready,
                allocatable: Resources::new(1000, 4 * 1024 * 1024 * 1024),
                requested: Resources::new(900, 2 * 1024 * 1024 * 1024),
                pods: 1..2,
            },
        ],
        node_pods: vec![
            PodPlacement {
                service: ServiceId(0),
                requests: Resources::new(900, 2 * 1024 * 1024 * 1024),
            },
            PodPlacement {
                service: ServiceId(0),
                requests: Resources::new(900, 2 * 1024 * 1024 * 1024),
            },
        ],
    };

    let plan = ca.provision(&observation);
    // Node 0's pods (900m/2GiB) don't fit in Node 1 (also 900m/2GiB used) → not drained
    assert!(!plan.drain.contains(&NodeId(0)));
}

#[test]
fn scale_down_empty_nodes_bulk() {
    // 3 empty unneeded nodes → all 3 drained in one call (ascending id)
    let mut ca = ClusterAutoscaler::new(ClusterAutoscalerConfig::default());

    let observation = Observation {
        now: SimTime::from_secs(600),
        services: vec![],
        pools: vec![PoolObservation {
            ready: 3,
            allocatable: Resources::new(1000, 4 * 1024 * 1024 * 1024),
            min_nodes: 1,
            max_nodes: 10,
            ..PoolObservation::default()
        }],
        nodes: vec![
            NodeObservation {
                id: NodeId(0),
                pool: PoolId(0),
                phase: NodePhase::Ready,
                allocatable: Resources::new(1000, 4 * 1024 * 1024 * 1024),
                requested: Resources::ZERO,
                pods: 0..0,
            },
            NodeObservation {
                id: NodeId(1),
                pool: PoolId(0),
                phase: NodePhase::Ready,
                allocatable: Resources::new(1000, 4 * 1024 * 1024 * 1024),
                requested: Resources::ZERO,
                pods: 0..0,
            },
            NodeObservation {
                id: NodeId(2),
                pool: PoolId(0),
                phase: NodePhase::Ready,
                allocatable: Resources::new(1000, 4 * 1024 * 1024 * 1024),
                requested: Resources::ZERO,
                pods: 0..0,
            },
        ],
        node_pods: vec![],
    };

    let plan = ca.provision(&observation);
    // All 3 empty nodes should be drained in one call
    assert_eq!(plan.drain.len(), 3, "Expected 3 empty nodes to be drained");
    assert!(plan.drain.contains(&NodeId(0)));
    assert!(plan.drain.contains(&NodeId(1)));
    assert!(plan.drain.contains(&NodeId(2)));
}

#[test]
fn scale_down_node_busy_resets_timer() {
    // Node becomes busy (utilisation >= 0.5) → unneeded timer reset
    // At t=600s with utilisation 0.6, but node just became busy → not drained yet
    let mut ca = ClusterAutoscaler::new(ClusterAutoscalerConfig::default());

    let observation = Observation {
        now: SimTime::from_secs(600),
        services: vec![ServiceObservation {
            requests: Resources::new(600, 2 * 1024 * 1024 * 1024),
            ..ServiceObservation::default()
        }],
        pools: vec![PoolObservation {
            ready: 2,
            allocatable: Resources::new(1000, 4 * 1024 * 1024 * 1024),
            min_nodes: 1,
            max_nodes: 5,
            ..PoolObservation::default()
        }],
        nodes: vec![
            NodeObservation {
                id: NodeId(0),
                pool: PoolId(0),
                phase: NodePhase::Ready,
                allocatable: Resources::new(1000, 4 * 1024 * 1024 * 1024),
                requested: Resources::new(600, 2 * 1024 * 1024 * 1024),
                pods: 0..1,
            },
            NodeObservation {
                id: NodeId(1),
                pool: PoolId(0),
                phase: NodePhase::Ready,
                allocatable: Resources::new(1000, 4 * 1024 * 1024 * 1024),
                requested: Resources::ZERO,
                pods: 1..1,
            },
        ],
        node_pods: vec![PodPlacement {
            service: ServiceId(0),
            requests: Resources::new(600, 2 * 1024 * 1024 * 1024),
        }],
    };

    let plan = ca.provision(&observation);
    // Node 0 has utilisation 600/1000 = 0.6 > 0.5 threshold → not unneeded → not drained
    assert!(!plan.drain.contains(&NodeId(0)));
}

#[test]
fn scan_interval_throttles_second_call() {
    // First call at t=0 runs, second call within 10s returns empty plan
    let mut ca = ClusterAutoscaler::new(ClusterAutoscalerConfig::default());

    let observation = Observation {
        now: SimTime::ZERO,
        services: vec![ServiceObservation {
            unschedulable: 1,
            requests: Resources::new(500, 1024 * 1024 * 1024),
            ..ServiceObservation::default()
        }],
        pools: vec![PoolObservation {
            allocatable: Resources::new(1000, 4 * 1024 * 1024 * 1024),
            max_nodes: 10,
            ..PoolObservation::default()
        }],
        nodes: vec![],
        node_pods: vec![],
    };

    let plan1 = ca.provision(&observation);
    assert!(!plan1.pool_delta.is_empty(), "First call should act");

    // Second call at t=5s (within scan_interval of 10s)
    let observation2 = Observation {
        now: SimTime::from_secs(5),
        ..observation
    };
    let plan2 = ca.provision(&observation2);
    assert!(
        plan2.pool_delta.is_empty(),
        "Second call within scan_interval should return empty plan"
    );
}

#[test]
fn scan_interval_allows_call_after_interval() {
    // Call at t=0, then at t=10s (scan_interval) allows action
    let mut ca = ClusterAutoscaler::new(ClusterAutoscalerConfig::default());

    let observation = Observation {
        now: SimTime::ZERO,
        services: vec![ServiceObservation {
            unschedulable: 1,
            requests: Resources::new(500, 1024 * 1024 * 1024),
            ..ServiceObservation::default()
        }],
        pools: vec![PoolObservation {
            allocatable: Resources::new(1000, 4 * 1024 * 1024 * 1024),
            max_nodes: 10,
            ..PoolObservation::default()
        }],
        nodes: vec![],
        node_pods: vec![],
    };

    let _plan1 = ca.provision(&observation);

    // Call at t=10s (at scan_interval)
    let observation2 = Observation {
        now: SimTime::from_secs(10),
        ..observation
    };
    let plan2 = ca.provision(&observation2);
    assert!(
        !plan2.pool_delta.is_empty(),
        "Call after scan_interval should allow action"
    );
}
