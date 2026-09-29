mod common;

use ksim_core::{
    cluster::{Cluster, NodePhase, NodePoolSpec, PodPhase, ServiceSpec},
    ids::{NodeId, PodId, PoolId, ServiceId},
    resources::Resources,
    time::SimTime,
};

#[test]
fn pool_ids_sequential_from_zero() {
    let mut cluster = Cluster::new();
    let pool0 = cluster.add_pool(NodePoolSpec {
        name: "pool0".to_string(),
        allocatable: Resources::new(1000, 1024 * 1024 * 1024),
        min_nodes: 1,
        max_nodes: 10,
        initial_nodes: 1,
        price_per_hour: 0.1,
    });
    let pool1 = cluster.add_pool(NodePoolSpec {
        name: "pool1".to_string(),
        allocatable: Resources::new(2000, 2 * 1024 * 1024 * 1024),
        min_nodes: 0,
        max_nodes: 5,
        initial_nodes: 0,
        price_per_hour: 0.2,
    });

    assert_eq!(pool0, PoolId(0));
    assert_eq!(pool1, PoolId(1));
}

#[test]
fn service_ids_sequential_from_zero() {
    let mut cluster = Cluster::new();
    let svc0 = cluster.add_service(ServiceSpec {
        name: "svc0".to_string(),
        requests: Resources::new(100, 128 * 1024 * 1024),
        limits: Resources::new(500, 512 * 1024 * 1024),
        initial_replicas: 1,
    });
    let svc1 = cluster.add_service(ServiceSpec {
        name: "svc1".to_string(),
        requests: Resources::new(200, 256 * 1024 * 1024),
        limits: Resources::new(1000, 1024 * 1024 * 1024),
        initial_replicas: 2,
    });

    assert_eq!(svc0, ServiceId(0));
    assert_eq!(svc1, ServiceId(1));
}

#[test]
fn node_ids_sequential_from_zero() {
    let mut cluster = Cluster::new();
    let pool = cluster.add_pool(NodePoolSpec {
        name: "pool".to_string(),
        allocatable: Resources::new(1000, 1024 * 1024 * 1024),
        min_nodes: 0,
        max_nodes: 10,
        initial_nodes: 0,
        price_per_hour: 0.1,
    });

    let node0 = cluster.add_node(pool, NodePhase::Ready);
    let node1 = cluster.add_node(pool, NodePhase::Ready);
    let node2 = cluster.add_node(pool, NodePhase::Provisioning);

    assert_eq!(node0, NodeId(0));
    assert_eq!(node1, NodeId(1));
    assert_eq!(node2, NodeId(2));
}

#[test]
fn pod_ids_sequential_from_zero() {
    let mut cluster = Cluster::new();
    cluster.add_service(ServiceSpec {
        name: "svc".to_string(),
        requests: Resources::new(100, 128 * 1024 * 1024),
        limits: Resources::new(500, 512 * 1024 * 1024),
        initial_replicas: 0,
    });

    let pod0 = cluster.create_pod(ServiceId(0), SimTime::ZERO);
    let pod1 = cluster.create_pod(ServiceId(0), SimTime::ZERO);
    let pod2 = cluster.create_pod(ServiceId(0), SimTime::ZERO);

    assert_eq!(pod0, PodId(0));
    assert_eq!(pod1, PodId(1));
    assert_eq!(pod2, PodId(2));
}

#[test]
fn create_pod_gets_service_current_requests() {
    let mut cluster = Cluster::new();
    let svc = cluster.add_service(ServiceSpec {
        name: "svc".to_string(),
        requests: Resources::new(100, 128 * 1024 * 1024),
        limits: Resources::new(500, 512 * 1024 * 1024),
        initial_replicas: 0,
    });

    let pod0 = cluster.create_pod(svc, SimTime::ZERO);
    let view0 = cluster.pod(pod0).expect("pod should exist");
    assert_eq!(view0.requests, Resources::new(100, 128 * 1024 * 1024));

    cluster.set_service_requests(svc, Resources::new(200, 256 * 1024 * 1024));

    let pod1 = cluster.create_pod(svc, SimTime::ZERO);
    let view0_after = cluster.pod(pod0).expect("pod should exist");
    let view1 = cluster.pod(pod1).expect("pod should exist");

    assert_eq!(view0_after.requests, Resources::new(100, 128 * 1024 * 1024));
    assert_eq!(view1.requests, Resources::new(200, 256 * 1024 * 1024));
}

#[test]
fn create_pod_returns_pending_unbound() {
    let mut cluster = Cluster::new();
    let svc = cluster.add_service(ServiceSpec {
        name: "svc".to_string(),
        requests: Resources::new(100, 128 * 1024 * 1024),
        limits: Resources::new(500, 512 * 1024 * 1024),
        initial_replicas: 0,
    });

    let pod = cluster.create_pod(svc, SimTime::ZERO);
    let view = cluster.pod(pod).expect("pod should exist");

    assert_eq!(view.phase, PodPhase::Pending);
    assert_eq!(view.node, None);
    assert!(!view.unschedulable);
    assert_eq!(view.restarts, 0);
}

#[test]
fn bind_pod_sets_node_and_starting() {
    let mut cluster = Cluster::new();
    let pool = cluster.add_pool(NodePoolSpec {
        name: "pool".to_string(),
        allocatable: Resources::new(1000, 1024 * 1024 * 1024),
        min_nodes: 0,
        max_nodes: 10,
        initial_nodes: 0,
        price_per_hour: 0.1,
    });
    let svc = cluster.add_service(ServiceSpec {
        name: "svc".to_string(),
        requests: Resources::new(100, 128 * 1024 * 1024),
        limits: Resources::new(500, 512 * 1024 * 1024),
        initial_replicas: 0,
    });

    let pod = cluster.create_pod(svc, SimTime::ZERO);
    let node = cluster.add_node(pool, NodePhase::Ready);

    cluster.bind_pod(pod, node);

    let view = cluster.pod(pod).expect("pod should exist");
    assert_eq!(view.node, Some(node));
    assert_eq!(view.phase, PodPhase::Starting);
}

#[test]
fn node_requested_sums_bound_non_removed_pods() {
    let mut cluster = Cluster::new();
    let pool = cluster.add_pool(NodePoolSpec {
        name: "pool".to_string(),
        allocatable: Resources::new(10000, 10 * 1024 * 1024 * 1024),
        min_nodes: 0,
        max_nodes: 10,
        initial_nodes: 0,
        price_per_hour: 0.1,
    });
    let svc = cluster.add_service(ServiceSpec {
        name: "svc".to_string(),
        requests: Resources::new(100, 128 * 1024 * 1024),
        limits: Resources::new(500, 512 * 1024 * 1024),
        initial_replicas: 0,
    });

    let node = cluster.add_node(pool, NodePhase::Ready);

    let pod1 = cluster.create_pod(svc, SimTime::ZERO);
    cluster.bind_pod(pod1, node);

    let pod2 = cluster.create_pod(svc, SimTime::ZERO);
    cluster.bind_pod(pod2, node);

    let _pod3 = cluster.create_pod(svc, SimTime::ZERO);

    assert_eq!(
        cluster.node_requested(node),
        Resources::new(200, 256 * 1024 * 1024)
    );

    cluster.remove_pod(pod1);
    assert_eq!(
        cluster.node_requested(node),
        Resources::new(100, 128 * 1024 * 1024)
    );
}

#[test]
fn remove_pod_tombstones_and_ids_stay_stable() {
    let mut cluster = Cluster::new();
    let svc = cluster.add_service(ServiceSpec {
        name: "svc".to_string(),
        requests: Resources::new(100, 128 * 1024 * 1024),
        limits: Resources::new(500, 512 * 1024 * 1024),
        initial_replicas: 0,
    });

    let pod0 = cluster.create_pod(svc, SimTime::ZERO);
    let pod1 = cluster.create_pod(svc, SimTime::ZERO);
    let pod2 = cluster.create_pod(svc, SimTime::ZERO);

    assert_eq!(pod0, PodId(0));
    assert_eq!(pod1, PodId(1));
    assert_eq!(pod2, PodId(2));

    cluster.remove_pod(pod1);

    let next_pod = cluster.create_pod(svc, SimTime::ZERO);
    assert_eq!(next_pod, PodId(3));

    assert!(cluster.pod(pod0).is_some());
    assert!(cluster.pod(pod1).is_none());
    assert!(cluster.pod(pod2).is_some());
}

#[test]
fn remove_node_tombstones_and_ids_stay_stable() {
    let mut cluster = Cluster::new();
    let pool = cluster.add_pool(NodePoolSpec {
        name: "pool".to_string(),
        allocatable: Resources::new(1000, 1024 * 1024 * 1024),
        min_nodes: 0,
        max_nodes: 10,
        initial_nodes: 0,
        price_per_hour: 0.1,
    });

    let node0 = cluster.add_node(pool, NodePhase::Ready);
    let node1 = cluster.add_node(pool, NodePhase::Ready);
    let node2 = cluster.add_node(pool, NodePhase::Ready);

    assert_eq!(node0, NodeId(0));
    assert_eq!(node1, NodeId(1));
    assert_eq!(node2, NodeId(2));

    cluster.remove_node(node1);

    let next_node = cluster.add_node(pool, NodePhase::Ready);
    assert_eq!(next_node, NodeId(3));

    assert!(cluster.node_phase(node0).is_some());
    assert!(cluster.node_phase(node1).is_none());
    assert!(cluster.node_phase(node2).is_some());
}

#[test]
fn count_pods_by_phase() {
    let mut cluster = Cluster::new();
    let svc = cluster.add_service(ServiceSpec {
        name: "svc".to_string(),
        requests: Resources::new(100, 128 * 1024 * 1024),
        limits: Resources::new(500, 512 * 1024 * 1024),
        initial_replicas: 0,
    });
    let pool = cluster.add_pool(NodePoolSpec {
        name: "pool".to_string(),
        allocatable: Resources::new(1000, 1024 * 1024 * 1024),
        min_nodes: 0,
        max_nodes: 10,
        initial_nodes: 0,
        price_per_hour: 0.1,
    });
    let node = cluster.add_node(pool, NodePhase::Ready);

    let _pod1 = cluster.create_pod(svc, SimTime::ZERO);
    let pod2 = cluster.create_pod(svc, SimTime::ZERO);
    cluster.bind_pod(pod2, node);

    assert_eq!(cluster.count_pods(svc, PodPhase::Pending), 1);
    assert_eq!(cluster.count_pods(svc, PodPhase::Starting), 1);
    assert_eq!(cluster.count_pods(svc, PodPhase::Running), 0);
    assert_eq!(cluster.count_pods(svc, PodPhase::Terminating), 0);
}

#[test]
fn count_nodes_by_phase() {
    let mut cluster = Cluster::new();
    let pool = cluster.add_pool(NodePoolSpec {
        name: "pool".to_string(),
        allocatable: Resources::new(1000, 1024 * 1024 * 1024),
        min_nodes: 0,
        max_nodes: 10,
        initial_nodes: 0,
        price_per_hour: 0.1,
    });

    let node1 = cluster.add_node(pool, NodePhase::Ready);
    let _node2 = cluster.add_node(pool, NodePhase::Ready);
    let _node3 = cluster.add_node(pool, NodePhase::Provisioning);

    assert_eq!(cluster.count_nodes(pool, NodePhase::Ready), 2);
    assert_eq!(cluster.count_nodes(pool, NodePhase::Provisioning), 1);
    assert_eq!(cluster.count_nodes(pool, NodePhase::Draining), 0);

    cluster.set_node_phase(node1, NodePhase::Draining);
    assert_eq!(cluster.count_nodes(pool, NodePhase::Ready), 1);
    assert_eq!(cluster.count_nodes(pool, NodePhase::Draining), 1);
}

#[test]
fn pods_to_delete_order_unbound_pending_first() {
    let mut cluster = Cluster::new();
    let svc = cluster.add_service(ServiceSpec {
        name: "svc".to_string(),
        requests: Resources::new(100, 128 * 1024 * 1024),
        limits: Resources::new(500, 512 * 1024 * 1024),
        initial_replicas: 0,
    });
    let pool = cluster.add_pool(NodePoolSpec {
        name: "pool".to_string(),
        allocatable: Resources::new(1000, 1024 * 1024 * 1024),
        min_nodes: 0,
        max_nodes: 10,
        initial_nodes: 0,
        price_per_hour: 0.1,
    });
    let node = cluster.add_node(pool, NodePhase::Ready);

    let pod_pending_unbound = cluster.create_pod(svc, SimTime::ZERO);
    let pod_starting = cluster.create_pod(svc, SimTime::ZERO);
    cluster.bind_pod(pod_starting, node);
    let pod_running = cluster.create_pod(svc, SimTime::ZERO);
    cluster.bind_pod(pod_running, node);
    cluster.set_pod_phase(pod_running, PodPhase::Running);

    let victims = cluster.pods_to_delete(svc, 2);
    assert_eq!(victims.len(), 2);
    assert_eq!(victims[0], pod_pending_unbound);
    assert_eq!(victims[1], pod_starting);
}

#[test]
fn pods_to_delete_order_running_newest_first() {
    let mut cluster = Cluster::new();
    let svc = cluster.add_service(ServiceSpec {
        name: "svc".to_string(),
        requests: Resources::new(100, 128 * 1024 * 1024),
        limits: Resources::new(500, 512 * 1024 * 1024),
        initial_replicas: 0,
    });
    let pool = cluster.add_pool(NodePoolSpec {
        name: "pool".to_string(),
        allocatable: Resources::new(1000, 1024 * 1024 * 1024),
        min_nodes: 0,
        max_nodes: 10,
        initial_nodes: 0,
        price_per_hour: 0.1,
    });
    let node = cluster.add_node(pool, NodePhase::Ready);

    let pod1 = cluster.create_pod(svc, SimTime::ZERO);
    cluster.bind_pod(pod1, node);
    cluster.set_pod_phase(pod1, PodPhase::Running);

    let pod2 = cluster.create_pod(svc, SimTime::ZERO);
    cluster.bind_pod(pod2, node);
    cluster.set_pod_phase(pod2, PodPhase::Running);

    let pod3 = cluster.create_pod(svc, SimTime::ZERO);
    cluster.bind_pod(pod3, node);
    cluster.set_pod_phase(pod3, PodPhase::Running);

    let victims = cluster.pods_to_delete(svc, 2);
    assert_eq!(victims.len(), 2);
    assert_eq!(victims[0], pod3);
    assert_eq!(victims[1], pod2);
}

#[test]
fn pods_to_delete_order_ties_higher_id_first() {
    let mut cluster = Cluster::new();
    let svc = cluster.add_service(ServiceSpec {
        name: "svc".to_string(),
        requests: Resources::new(100, 128 * 1024 * 1024),
        limits: Resources::new(500, 512 * 1024 * 1024),
        initial_replicas: 0,
    });

    let _pod1 = cluster.create_pod(svc, SimTime::ZERO);
    let pod2 = cluster.create_pod(svc, SimTime::ZERO);
    let pod3 = cluster.create_pod(svc, SimTime::ZERO);

    let victims = cluster.pods_to_delete(svc, 2);
    assert_eq!(victims.len(), 2);
    assert_eq!(victims[0], pod3);
    assert_eq!(victims[1], pod2);
}

#[test]
fn pods_to_delete_excludes_terminating() {
    let mut cluster = Cluster::new();
    let svc = cluster.add_service(ServiceSpec {
        name: "svc".to_string(),
        requests: Resources::new(100, 128 * 1024 * 1024),
        limits: Resources::new(500, 512 * 1024 * 1024),
        initial_replicas: 0,
    });
    let pool = cluster.add_pool(NodePoolSpec {
        name: "pool".to_string(),
        allocatable: Resources::new(1000, 1024 * 1024 * 1024),
        min_nodes: 0,
        max_nodes: 10,
        initial_nodes: 0,
        price_per_hour: 0.1,
    });
    let node = cluster.add_node(pool, NodePhase::Ready);

    let pod1 = cluster.create_pod(svc, SimTime::ZERO);
    cluster.bind_pod(pod1, node);
    cluster.set_pod_phase(pod1, PodPhase::Running);

    let pod2 = cluster.create_pod(svc, SimTime::ZERO);
    cluster.bind_pod(pod2, node);
    cluster.set_pod_phase(pod2, PodPhase::Terminating);

    let pod3 = cluster.create_pod(svc, SimTime::ZERO);
    cluster.bind_pod(pod3, node);
    cluster.set_pod_phase(pod3, PodPhase::Running);

    let victims = cluster.pods_to_delete(svc, 2);
    assert_eq!(victims.len(), 2);
    assert!(!victims.contains(&pod2));
}

#[test]
fn pods_of_returns_ascending_id_order() {
    let mut cluster = Cluster::new();
    let svc = cluster.add_service(ServiceSpec {
        name: "svc".to_string(),
        requests: Resources::new(100, 128 * 1024 * 1024),
        limits: Resources::new(500, 512 * 1024 * 1024),
        initial_replicas: 0,
    });

    let pod0 = cluster.create_pod(svc, SimTime::ZERO);
    let pod1 = cluster.create_pod(svc, SimTime::ZERO);
    let pod2 = cluster.create_pod(svc, SimTime::ZERO);

    let result = cluster.pods_of(svc);
    assert_eq!(result, vec![pod0, pod1, pod2]);
}

#[test]
fn pending_pods_returns_unbound_pending() {
    let mut cluster = Cluster::new();
    let svc = cluster.add_service(ServiceSpec {
        name: "svc".to_string(),
        requests: Resources::new(100, 128 * 1024 * 1024),
        limits: Resources::new(500, 512 * 1024 * 1024),
        initial_replicas: 0,
    });
    let pool = cluster.add_pool(NodePoolSpec {
        name: "pool".to_string(),
        allocatable: Resources::new(1000, 1024 * 1024 * 1024),
        min_nodes: 0,
        max_nodes: 10,
        initial_nodes: 0,
        price_per_hour: 0.1,
    });
    let node = cluster.add_node(pool, NodePhase::Ready);

    let pod1 = cluster.create_pod(svc, SimTime::ZERO);
    let pod2 = cluster.create_pod(svc, SimTime::ZERO);
    cluster.bind_pod(pod2, node);
    let pod3 = cluster.create_pod(svc, SimTime::ZERO);

    let pending = cluster.pending_pods();
    assert_eq!(pending.len(), 2);
    assert!(pending.contains(&pod1));
    assert!(pending.contains(&pod3));
    assert!(!pending.contains(&pod2));
}

#[test]
fn set_unschedulable_flag() {
    let mut cluster = Cluster::new();
    let svc = cluster.add_service(ServiceSpec {
        name: "svc".to_string(),
        requests: Resources::new(100, 128 * 1024 * 1024),
        limits: Resources::new(500, 512 * 1024 * 1024),
        initial_replicas: 0,
    });

    let pod = cluster.create_pod(svc, SimTime::ZERO);
    let view = cluster.pod(pod).expect("pod should exist");
    assert!(!view.unschedulable);

    cluster.set_unschedulable(pod, true);
    let view = cluster.pod(pod).expect("pod should exist");
    assert!(view.unschedulable);
}

#[test]
fn record_restart_increments_counter() {
    let mut cluster = Cluster::new();
    let svc = cluster.add_service(ServiceSpec {
        name: "svc".to_string(),
        requests: Resources::new(100, 128 * 1024 * 1024),
        limits: Resources::new(500, 512 * 1024 * 1024),
        initial_replicas: 0,
    });

    let pod = cluster.create_pod(svc, SimTime::ZERO);
    let view = cluster.pod(pod).expect("pod should exist");
    assert_eq!(view.restarts, 0);

    cluster.record_restart(pod);
    let view = cluster.pod(pod).expect("pod should exist");
    assert_eq!(view.restarts, 1);

    cluster.record_restart(pod);
    let view = cluster.pod(pod).expect("pod should exist");
    assert_eq!(view.restarts, 2);
}

#[test]
fn set_pod_phase() {
    let mut cluster = Cluster::new();
    let svc = cluster.add_service(ServiceSpec {
        name: "svc".to_string(),
        requests: Resources::new(100, 128 * 1024 * 1024),
        limits: Resources::new(500, 512 * 1024 * 1024),
        initial_replicas: 0,
    });

    let pod = cluster.create_pod(svc, SimTime::ZERO);
    assert_eq!(cluster.pod(pod).unwrap().phase, PodPhase::Pending);

    cluster.set_pod_phase(pod, PodPhase::Running);
    assert_eq!(cluster.pod(pod).unwrap().phase, PodPhase::Running);

    cluster.set_pod_phase(pod, PodPhase::Terminating);
    assert_eq!(cluster.pod(pod).unwrap().phase, PodPhase::Terminating);
}

#[test]
fn set_node_phase() {
    let mut cluster = Cluster::new();
    let pool = cluster.add_pool(NodePoolSpec {
        name: "pool".to_string(),
        allocatable: Resources::new(1000, 1024 * 1024 * 1024),
        min_nodes: 0,
        max_nodes: 10,
        initial_nodes: 0,
        price_per_hour: 0.1,
    });

    let node = cluster.add_node(pool, NodePhase::Provisioning);
    assert_eq!(cluster.node_phase(node).unwrap(), NodePhase::Provisioning);

    cluster.set_node_phase(node, NodePhase::Ready);
    assert_eq!(cluster.node_phase(node).unwrap(), NodePhase::Ready);

    cluster.set_node_phase(node, NodePhase::Draining);
    assert_eq!(cluster.node_phase(node).unwrap(), NodePhase::Draining);
}

#[test]
fn pool_count() {
    let mut cluster = Cluster::new();
    assert_eq!(cluster.pool_count(), 0);

    cluster.add_pool(NodePoolSpec {
        name: "pool0".to_string(),
        allocatable: Resources::new(1000, 1024 * 1024 * 1024),
        min_nodes: 0,
        max_nodes: 10,
        initial_nodes: 0,
        price_per_hour: 0.1,
    });
    assert_eq!(cluster.pool_count(), 1);

    cluster.add_pool(NodePoolSpec {
        name: "pool1".to_string(),
        allocatable: Resources::new(2000, 2 * 1024 * 1024 * 1024),
        min_nodes: 0,
        max_nodes: 5,
        initial_nodes: 0,
        price_per_hour: 0.2,
    });
    assert_eq!(cluster.pool_count(), 2);
}

#[test]
fn service_count() {
    let mut cluster = Cluster::new();
    assert_eq!(cluster.service_count(), 0);

    cluster.add_service(ServiceSpec {
        name: "svc0".to_string(),
        requests: Resources::new(100, 128 * 1024 * 1024),
        limits: Resources::new(500, 512 * 1024 * 1024),
        initial_replicas: 0,
    });
    assert_eq!(cluster.service_count(), 1);

    cluster.add_service(ServiceSpec {
        name: "svc1".to_string(),
        requests: Resources::new(200, 256 * 1024 * 1024),
        limits: Resources::new(1000, 1024 * 1024 * 1024),
        initial_replicas: 0,
    });
    assert_eq!(cluster.service_count(), 2);
}

#[test]
fn pool_spec_and_service_spec_accessors() {
    let mut cluster = Cluster::new();
    let pool_spec = NodePoolSpec {
        name: "pool".to_string(),
        allocatable: Resources::new(1000, 1024 * 1024 * 1024),
        min_nodes: 1,
        max_nodes: 10,
        initial_nodes: 1,
        price_per_hour: 0.1,
    };
    let svc_spec = ServiceSpec {
        name: "svc".to_string(),
        requests: Resources::new(100, 128 * 1024 * 1024),
        limits: Resources::new(500, 512 * 1024 * 1024),
        initial_replicas: 0,
    };

    let pool = cluster.add_pool(pool_spec.clone());
    let svc = cluster.add_service(svc_spec.clone());

    assert_eq!(*cluster.pool_spec(pool), pool_spec);
    assert_eq!(*cluster.service_spec(svc), svc_spec);
}

#[test]
fn node_pool_and_allocatable() {
    let mut cluster = Cluster::new();
    let pool_spec = NodePoolSpec {
        name: "pool".to_string(),
        allocatable: Resources::new(1000, 1024 * 1024 * 1024),
        min_nodes: 0,
        max_nodes: 10,
        initial_nodes: 0,
        price_per_hour: 0.1,
    };
    let pool = cluster.add_pool(pool_spec);
    let node = cluster.add_node(pool, NodePhase::Ready);

    assert_eq!(cluster.node_pool(node), pool);
    assert_eq!(
        cluster.node_allocatable(node),
        Resources::new(1000, 1024 * 1024 * 1024)
    );
}

#[test]
fn nodes_returns_ascending_id_order() {
    let mut cluster = Cluster::new();
    let pool = cluster.add_pool(NodePoolSpec {
        name: "pool".to_string(),
        allocatable: Resources::new(1000, 1024 * 1024 * 1024),
        min_nodes: 0,
        max_nodes: 10,
        initial_nodes: 0,
        price_per_hour: 0.1,
    });

    let node0 = cluster.add_node(pool, NodePhase::Ready);
    let node1 = cluster.add_node(pool, NodePhase::Ready);
    let node2 = cluster.add_node(pool, NodePhase::Ready);

    let result_nodes = cluster.nodes();
    assert_eq!(result_nodes, vec![node0, node1, node2]);
}

#[test]
fn nodes_excludes_removed() {
    let mut cluster = Cluster::new();
    let pool = cluster.add_pool(NodePoolSpec {
        name: "pool".to_string(),
        allocatable: Resources::new(1000, 1024 * 1024 * 1024),
        min_nodes: 0,
        max_nodes: 10,
        initial_nodes: 0,
        price_per_hour: 0.1,
    });

    let node0 = cluster.add_node(pool, NodePhase::Ready);
    let node1 = cluster.add_node(pool, NodePhase::Ready);
    let node2 = cluster.add_node(pool, NodePhase::Ready);

    cluster.remove_node(node1);
    let remaining_nodes = cluster.nodes();
    assert_eq!(remaining_nodes, vec![node0, node2]);
}
