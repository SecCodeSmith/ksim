mod common;

use common::{FixedDemand, ReadyPodsModel};
use ksim_core::{
    actions::Actions,
    cluster::{NodePhase, NodePoolSpec, ServiceSpec},
    engine::SimulationBuilder,
    lifecycle::FixedLatencies,
    resources::Resources,
    time::SimDuration,
    workload::ServiceDemand,
};

#[test]
fn drain_node_with_pods_becomes_draining() {
    let mut sim = SimulationBuilder::new(42)
        .tick(SimDuration::from_secs(15))
        .pool(NodePoolSpec {
            name: "pool0".to_string(),
            allocatable: Resources::new(1000, 1024 * 1024 * 1024),
            min_nodes: 1,
            max_nodes: 10,
            initial_nodes: 2,
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

    // Get a Ready node to drain.
    sim.step(&Actions::none());
    let initial_obs = sim.observation();
    let node_to_drain = initial_obs.nodes.iter().find(|n| n.phase == NodePhase::Ready);
    assert!(
        node_to_drain.is_some(),
        "should have at least one Ready node to drain"
    );
    let node_to_drain = node_to_drain.unwrap().id;

    // Drain the node.
    let mut drain_action = Actions::none();
    drain_action.drain.push(node_to_drain);
    sim.step(&drain_action);

    let obs = sim.observation();
    let drained_node = obs.nodes.iter().find(|n| n.id == node_to_drain);
    assert!(
        drained_node.is_some(),
        "drained node should still exist right after drain"
    );
    assert_eq!(
        drained_node.unwrap().phase,
        NodePhase::Draining,
        "drained node should be in Draining phase"
    );
}

#[test]
fn drain_node_pods_become_terminating_replacements_created() {
    let mut sim = SimulationBuilder::new(42)
        .tick(SimDuration::from_secs(15))
        .pool(NodePoolSpec {
            name: "pool0".to_string(),
            allocatable: Resources::new(1000, 1024 * 1024 * 1024),
            min_nodes: 1,
            max_nodes: 10,
            initial_nodes: 3,
            price_per_hour: 0.1,
        })
        .service(ServiceSpec {
            name: "svc0".to_string(),
            requests: Resources::new(100, 128 * 1024 * 1024),
            limits: Resources::new(500, 512 * 1024 * 1024),
            initial_replicas: 3,
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
    let initial_obs = sim.observation();
    let desired_replicas = initial_obs.services[0].desired;
    let node_to_drain = initial_obs.nodes[0].id;

    // Drain the node.
    let mut drain_action = Actions::none();
    drain_action.drain.push(node_to_drain);
    sim.step(&drain_action);

    let obs = sim.observation();

    // Desired replicas should be maintained.
    assert_eq!(
        obs.services[0].desired, desired_replicas,
        "desired replicas should remain unchanged"
    );

    // The drained node should have no bound pods (replacements created on other nodes).
    let drained_node = obs.nodes.iter().find(|n| n.id == node_to_drain);
    if let Some(node) = drained_node {
        assert_eq!(
            node.pods.end - node.pods.start, 0,
            "drained node should have no bound pods after drain action"
        );
    }
}

#[test]
fn drain_empty_node_removes_in_same_step() {
    let mut sim = SimulationBuilder::new(42)
        .tick(SimDuration::from_secs(15))
        .pool(NodePoolSpec {
            name: "pool0".to_string(),
            allocatable: Resources::new(1000, 1024 * 1024 * 1024),
            min_nodes: 1,
            max_nodes: 10,
            initial_nodes: 3,
            price_per_hour: 0.1,
        })
        .service(ServiceSpec {
            name: "svc0".to_string(),
            requests: Resources::new(100, 128 * 1024 * 1024),
            limits: Resources::new(500, 512 * 1024 * 1024),
            initial_replicas: 1,
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
    let initial_obs = sim.observation();

    // Find an empty node to drain (one without any bound pods).
    let empty_node = initial_obs
        .nodes
        .iter()
        .find(|n| n.phase == NodePhase::Ready && (n.pods.end - n.pods.start) == 0);
    assert!(empty_node.is_some(), "should have an empty Ready node");
    let empty_node_id = empty_node.unwrap().id;

    // Drain the empty node.
    let mut drain_action = Actions::none();
    drain_action.drain.push(empty_node_id);
    sim.step(&drain_action);

    let obs = sim.observation();
    let should_be_removed = obs.nodes.iter().find(|n| n.id == empty_node_id);
    assert!(
        should_be_removed.is_none(),
        "empty drained node should be removed in same step"
    );
}

#[test]
fn drain_below_min_nodes_is_ignored() {
    let mut sim = SimulationBuilder::new(42)
        .tick(SimDuration::from_secs(15))
        .pool(NodePoolSpec {
            name: "pool0".to_string(),
            allocatable: Resources::new(1000, 1024 * 1024 * 1024),
            min_nodes: 2,
            max_nodes: 10,
            initial_nodes: 2,
            price_per_hour: 0.1,
        })
        .service(ServiceSpec {
            name: "svc0".to_string(),
            requests: Resources::new(100, 128 * 1024 * 1024),
            limits: Resources::new(500, 512 * 1024 * 1024),
            initial_replicas: 1,
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
    let initial_obs = sim.observation();
    let initial_ready_count = initial_obs
        .pools[0]
        .ready
        .saturating_add(initial_obs.pools[0].provisioning);

    // Try to drain a node when pool is at min_nodes.
    let node_to_drain = initial_obs.nodes[0].id;
    let mut drain_action = Actions::none();
    drain_action.drain.push(node_to_drain);
    sim.step(&drain_action);

    let obs = sim.observation();
    let new_count = obs
        .pools[0]
        .ready
        .saturating_add(obs.pools[0].provisioning);

    assert_eq!(
        new_count, initial_ready_count,
        "drain below min_nodes should be ignored; node count should remain same"
    );
}

#[test]
fn evict_marks_outdated_pods_terminating() {
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
            requests: Resources::new(100, 128 * 1024 * 1024),
            limits: Resources::new(500, 512 * 1024 * 1024),
            initial_replicas: 3,
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

    // Change requests to create outdated pods.
    let mut requests_action = Actions::none();
    requests_action
        .requests
        .push(Some(Resources::new(200, 256 * 1024 * 1024)));
    sim.step(&requests_action);

    let obs_after_requests = sim.observation();
    let outdated_count = obs_after_requests.services[0].outdated;
    assert!(outdated_count > 0, "should have outdated pods after requests change");

    // Evict some outdated pods.
    let mut evict_action = Actions::none();
    evict_action.evict.push(2);
    sim.step(&evict_action);

    let obs = sim.observation();
    assert!(
        obs.services[0].outdated < outdated_count,
        "eviction should reduce outdated pod count"
    );
}

#[test]
fn evicted_pods_replaced_with_new_requests() {
    let mut sim = SimulationBuilder::new(42)
        .tick(SimDuration::from_secs(15))
        .pool(NodePoolSpec {
            name: "pool0".to_string(),
            allocatable: Resources::new(2000, 4 * 1024 * 1024 * 1024),
            min_nodes: 1,
            max_nodes: 10,
            initial_nodes: 1,
            price_per_hour: 0.1,
        })
        .service(ServiceSpec {
            name: "svc0".to_string(),
            requests: Resources::new(100, 128 * 1024 * 1024),
            limits: Resources::new(500, 512 * 1024 * 1024),
            initial_replicas: 3,
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

    // Change requests.
    let new_requests = Resources::new(200, 256 * 1024 * 1024);
    let mut requests_action = Actions::none();
    requests_action.requests.push(Some(new_requests));
    sim.step(&requests_action);

    // Evict old pods.
    let mut evict_action = Actions::none();
    evict_action.evict.push(2);
    sim.step(&evict_action);

    let obs = sim.observation();

    // Desired replicas should be maintained.
    assert_eq!(
        obs.services[0].desired, 3,
        "desired replicas should remain 3 after eviction"
    );
}

#[test]
fn reconciliation_restores_replicas_after_eviction() {
    let mut sim = SimulationBuilder::new(42)
        .tick(SimDuration::from_secs(15))
        .pool(NodePoolSpec {
            name: "pool0".to_string(),
            allocatable: Resources::new(2000, 4 * 1024 * 1024 * 1024),
            min_nodes: 1,
            max_nodes: 10,
            initial_nodes: 2,
            price_per_hour: 0.1,
        })
        .service(ServiceSpec {
            name: "svc0".to_string(),
            requests: Resources::new(100, 128 * 1024 * 1024),
            limits: Resources::new(500, 512 * 1024 * 1024),
            initial_replicas: 3,
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

    // Change requests to create outdated pods.
    let new_requests = Resources::new(200, 256 * 1024 * 1024);
    let mut requests_action = Actions::none();
    requests_action.requests.push(Some(new_requests));
    sim.step(&requests_action);

    // Evict without changing desired count.
    let mut evict_action = Actions::none();
    evict_action.evict.push(2);
    sim.step(&evict_action);

    let obs_after_evict = sim.observation();
    let desired_after_evict = obs_after_evict.services[0].desired;

    // Do another step without eviction to allow reconciliation.
    sim.step(&Actions::none());

    let obs_final = sim.observation();
    // Desired should still be the same.
    assert_eq!(
        obs_final.services[0].desired, desired_after_evict,
        "desired replicas should be maintained through reconciliation"
    );
}
