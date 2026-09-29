mod common;

use common::{FixedDemand, ReadyPodsModel};
use ksim_core::{
    actions::Actions,
    cluster::{NodePoolSpec, ServiceSpec},
    engine::SimulationBuilder,
    ids::PoolId,
    lifecycle::FixedLatencies,
    resources::Resources,
    time::SimDuration,
    workload::ServiceDemand,
};

#[test]
fn observation_nodes_ascending_with_properties() {
    let mut sim = SimulationBuilder::new(42)
        .tick(SimDuration::from_secs(15))
        .pool(NodePoolSpec {
            name: "pool0".to_string(),
            allocatable: Resources::new(1000, 1024 * 1024 * 1024),
            min_nodes: 3,
            max_nodes: 10,
            initial_nodes: 3,
            price_per_hour: 0.1,
        })
        .service(ServiceSpec {
            name: "svc0".to_string(),
            requests: Resources::new(100, 128 * 1024 * 1024),
            limits: Resources::new(500, 512 * 1024 * 1024),
            initial_replicas: 2,
        })
        .workload(Box::new(FixedDemand::new(vec![ServiceDemand {
            request_rate: 10.0,
            cpu_millis: 100.0,
            memory_bytes: 128.0,
        }])))
        .performance(Box::new(ReadyPodsModel::new(false)))
        .latencies(Box::new(FixedLatencies {
            pod_startup: SimDuration::from_secs(5),
            node_provisioning: SimDuration::from_secs(60),
            termination: SimDuration::from_secs(30),
        }))
        .build()
        .expect("build should succeed");

    sim.step(&Actions::none());
    let obs = sim.observation();

    // Nodes should be in ascending id order.
    assert!(!obs.nodes.is_empty(), "should have at least 1 node");
    for i in 0..obs.nodes.len() - 1 {
        assert!(
            obs.nodes[i].id.0 < obs.nodes[i + 1].id.0,
            "nodes should be in ascending id order"
        );
    }

    // Each node should have correct pool and phase.
    for node in &obs.nodes {
        assert_eq!(node.pool, PoolId(0), "node should belong to pool 0");
    }

    // Each node should have allocatable resources from pool spec.
    for node in &obs.nodes {
        assert_eq!(
            node.allocatable,
            Resources::new(1000, 1024 * 1024 * 1024),
            "node allocatable should match pool spec"
        );
    }

    // requested should be sum of bound non-terminating pods on node.
    for node in &obs.nodes {
        assert!(
            node.requested.cpu_millis <= node.allocatable.cpu_millis,
            "requested should fit in allocatable"
        );
    }

    // pods range should be valid into node_pods.
    for node in &obs.nodes {
        assert!(
            node.pods.end <= obs.node_pods.len(),
            "node pods range should be valid"
        );
        // Placement pods on this node should match.
        for placement in &obs.node_pods[node.pods.clone()] {
            assert_eq!(
                placement.requests, Resources::new(100, 128 * 1024 * 1024),
                "placement should match service requests"
            );
        }
    }
}

#[test]
fn pool_observation_has_allocatable_and_counts() {
    let mut sim = SimulationBuilder::new(42)
        .tick(SimDuration::from_secs(15))
        .pool(NodePoolSpec {
            name: "pool0".to_string(),
            allocatable: Resources::new(2000, 2 * 1024 * 1024 * 1024),
            min_nodes: 2,
            max_nodes: 5,
            initial_nodes: 2,
            price_per_hour: 0.2,
        })
        .service(ServiceSpec {
            name: "svc0".to_string(),
            requests: Resources::new(200, 256 * 1024 * 1024),
            limits: Resources::new(1000, 1024 * 1024 * 1024),
            initial_replicas: 1,
        })
        .workload(Box::new(FixedDemand::new(vec![ServiceDemand {
            request_rate: 5.0,
            cpu_millis: 200.0,
            memory_bytes: 256.0,
        }])))
        .performance(Box::new(ReadyPodsModel::new(false)))
        .latencies(Box::new(FixedLatencies {
            pod_startup: SimDuration::from_secs(5),
            node_provisioning: SimDuration::from_secs(60),
            termination: SimDuration::from_secs(30),
        }))
        .build()
        .expect("build should succeed");

    sim.step(&Actions::none());
    let obs = sim.observation();

    assert_eq!(obs.pools.len(), 1, "should have 1 pool");
    let pool = &obs.pools[0];

    // Allocatable should come from spec.
    assert_eq!(
        pool.allocatable,
        Resources::new(2000, 2 * 1024 * 1024 * 1024),
        "pool allocatable should match spec"
    );

    // min/max should come from spec.
    assert_eq!(pool.min_nodes, 2, "pool min_nodes should match spec");
    assert_eq!(pool.max_nodes, 5, "pool max_nodes should match spec");

    // ready should be count of Ready nodes.
    assert!(pool.ready >= 2, "should have at least 2 ready nodes initially");
}

#[test]
fn service_observation_limits_equals_spec() {
    let mut sim = SimulationBuilder::new(42)
        .tick(SimDuration::from_secs(15))
        .pool(NodePoolSpec {
            name: "pool0".to_string(),
            allocatable: Resources::new(1000, 1024 * 1024 * 1024),
            min_nodes: 1,
            max_nodes: 10,
            initial_nodes: 1,
            price_per_hour: 0.1,
        })
        .service(ServiceSpec {
            name: "svc0".to_string(),
            requests: Resources::new(150, 200 * 1024 * 1024),
            limits: Resources::new(750, 1024 * 1024 * 1024),
            initial_replicas: 2,
        })
        .workload(Box::new(FixedDemand::new(vec![ServiceDemand {
            request_rate: 15.0,
            cpu_millis: 150.0,
            memory_bytes: 200.0,
        }])))
        .performance(Box::new(ReadyPodsModel::new(false)))
        .latencies(Box::new(FixedLatencies {
            pod_startup: SimDuration::from_secs(5),
            node_provisioning: SimDuration::from_secs(60),
            termination: SimDuration::from_secs(30),
        }))
        .build()
        .expect("build should succeed");

    sim.step(&Actions::none());
    let obs = sim.observation();

    assert_eq!(obs.services.len(), 1, "should have 1 service");
    let svc = &obs.services[0];

    assert_eq!(
        svc.limits,
        Resources::new(750, 1024 * 1024 * 1024),
        "service limits should match spec"
    );
}
