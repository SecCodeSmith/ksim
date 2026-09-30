use ksim_core::cluster::{Cluster, NodePhase, NodePoolSpec, ServiceSpec};
use ksim_core::resources::{GIB, Resources};
use ksim_core::scheduler::{LeastAllocated, MostAllocated, Scheduler, ScoringPolicy};
use ksim_core::time::SimTime;

#[test]
fn least_allocated_score_1000m_2gib_on_4000m_8gib() {
    let policy = LeastAllocated;
    let requested = Resources::new(1000, 2 * GIB);
    let allocatable = Resources::new(4000, 8 * GIB);

    let score = policy.score(requested, allocatable);
    // CPU: (4000-1000)*100/4000 = 75, Memory: (8-2)*100/8 = 75, mean = 75
    assert_eq!(score, 75);
}

#[test]
fn least_allocated_score_2000m_2gib_on_4000m_8gib() {
    let policy = LeastAllocated;
    let requested = Resources::new(2000, 2 * GIB);
    let allocatable = Resources::new(4000, 8 * GIB);

    let score = policy.score(requested, allocatable);
    // CPU: (4000-2000)*100/4000 = 50, Memory: (8-2)*100/8 = 75, mean = 62
    assert_eq!(score, 62);
}

#[test]
fn most_allocated_score_1000m_2gib_on_4000m_8gib() {
    let policy = MostAllocated;
    let requested = Resources::new(1000, 2 * GIB);
    let allocatable = Resources::new(4000, 8 * GIB);

    let score = policy.score(requested, allocatable);
    // CPU: 1000*100/4000 = 25, Memory: 2*100/8 = 25, mean = 25
    assert_eq!(score, 25);
}

#[test]
fn most_allocated_score_2000m_2gib_on_4000m_8gib() {
    let policy = MostAllocated;
    let requested = Resources::new(2000, 2 * GIB);
    let allocatable = Resources::new(4000, 8 * GIB);

    let score = policy.score(requested, allocatable);
    // CPU: 2000*100/4000 = 50, Memory: 2*100/8 = 25, mean = 37
    assert_eq!(score, 37);
}

#[test]
fn select_node_picks_higher_score() {
    let mut cluster = Cluster::new();

    let pool_spec = NodePoolSpec {
        name: "default".to_string(),
        allocatable: Resources::new(4000, 8 * GIB),
        min_nodes: 1,
        max_nodes: 10,
        initial_nodes: 0,
        price_per_hour: 0.1,
    };

    let pool = cluster.add_pool(pool_spec);

    let node1 = cluster.add_node(pool, NodePhase::Ready);
    let node2 = cluster.add_node(pool, NodePhase::Ready);

    // Create a service and schedule pods
    let service_spec = ServiceSpec {
        name: "test".to_string(),
        requests: Resources::new(1000, 2 * GIB),
        limits: Resources::new(2000, 4 * GIB),
        initial_replicas: 0,
    };

    let service = cluster.add_service(service_spec);

    // Create pods on node1 to make it more loaded
    let pod1 = cluster.create_pod(service, SimTime::ZERO);
    cluster.bind_pod(pod1, node1);
    cluster.set_pod_phase(pod1, ksim_core::cluster::PodPhase::Running);

    let scheduler = Scheduler::new(Box::new(LeastAllocated));
    let requests = Resources::new(1000, 2 * GIB);

    // LeastAllocated should pick node2 (less loaded)
    let selected = scheduler.select_node(&cluster, requests);
    assert_eq!(selected, Some(node2));
}

#[test]
fn select_node_picks_emptier_node_under_least_allocated() {
    let mut cluster = Cluster::new();

    let pool_spec = NodePoolSpec {
        name: "default".to_string(),
        allocatable: Resources::new(4000, 8 * GIB),
        min_nodes: 1,
        max_nodes: 10,
        initial_nodes: 0,
        price_per_hour: 0.1,
    };

    let pool = cluster.add_pool(pool_spec);

    let node1 = cluster.add_node(pool, NodePhase::Ready);
    let node2 = cluster.add_node(pool, NodePhase::Ready);

    let service_spec = ServiceSpec {
        name: "test".to_string(),
        requests: Resources::new(500, GIB),
        limits: Resources::new(1000, 2 * GIB),
        initial_replicas: 0,
    };

    let service = cluster.add_service(service_spec);

    // Put a pod on node1
    let pod1 = cluster.create_pod(service, SimTime::ZERO);
    cluster.bind_pod(pod1, node1);
    cluster.set_pod_phase(pod1, ksim_core::cluster::PodPhase::Running);

    let scheduler = Scheduler::new(Box::new(LeastAllocated));
    let requests = Resources::new(500, GIB);

    // Should pick node2 as it's emptier
    let selected = scheduler.select_node(&cluster, requests);
    assert_eq!(selected, Some(node2));
}

#[test]
fn select_node_picks_fuller_node_under_most_allocated() {
    let mut cluster = Cluster::new();

    let pool_spec = NodePoolSpec {
        name: "default".to_string(),
        allocatable: Resources::new(4000, 8 * GIB),
        min_nodes: 1,
        max_nodes: 10,
        initial_nodes: 0,
        price_per_hour: 0.1,
    };

    let pool = cluster.add_pool(pool_spec);

    let node1 = cluster.add_node(pool, NodePhase::Ready);
    let _node2 = cluster.add_node(pool, NodePhase::Ready);

    let service_spec = ServiceSpec {
        name: "test".to_string(),
        requests: Resources::new(500, GIB),
        limits: Resources::new(1000, 2 * GIB),
        initial_replicas: 0,
    };

    let service = cluster.add_service(service_spec);

    // Put a pod on node1
    let pod1 = cluster.create_pod(service, SimTime::ZERO);
    cluster.bind_pod(pod1, node1);
    cluster.set_pod_phase(pod1, ksim_core::cluster::PodPhase::Running);

    let scheduler = Scheduler::new(Box::new(MostAllocated));
    let requests = Resources::new(500, GIB);

    // Should pick node1 as it's fuller
    let selected = scheduler.select_node(&cluster, requests);
    assert_eq!(selected, Some(node1));
}

#[test]
fn select_node_ties_go_to_lowest_node_id() {
    let mut cluster = Cluster::new();

    let pool_spec = NodePoolSpec {
        name: "default".to_string(),
        allocatable: Resources::new(4000, 8 * GIB),
        min_nodes: 1,
        max_nodes: 10,
        initial_nodes: 0,
        price_per_hour: 0.1,
    };

    let pool = cluster.add_pool(pool_spec);

    let node1 = cluster.add_node(pool, NodePhase::Ready);
    let _node2 = cluster.add_node(pool, NodePhase::Ready);
    let _node3 = cluster.add_node(pool, NodePhase::Ready);

    let service_spec = ServiceSpec {
        name: "test".to_string(),
        requests: Resources::new(500, GIB),
        limits: Resources::new(1000, 2 * GIB),
        initial_replicas: 0,
    };

    let _service = cluster.add_service(service_spec);

    let scheduler = Scheduler::new(Box::new(LeastAllocated));
    let requests = Resources::new(500, GIB);

    // All nodes empty, should pick node1 (lowest ID)
    let selected = scheduler.select_node(&cluster, requests);
    assert_eq!(selected, Some(node1));
}

#[test]
fn select_node_ignores_provisioning_nodes() {
    let mut cluster = Cluster::new();

    let pool_spec = NodePoolSpec {
        name: "default".to_string(),
        allocatable: Resources::new(4000, 8 * GIB),
        min_nodes: 1,
        max_nodes: 10,
        initial_nodes: 0,
        price_per_hour: 0.1,
    };

    let pool = cluster.add_pool(pool_spec);

    let _prov_node = cluster.add_node(pool, NodePhase::Provisioning);
    let ready_node = cluster.add_node(pool, NodePhase::Ready);

    let service_spec = ServiceSpec {
        name: "test".to_string(),
        requests: Resources::new(500, GIB),
        limits: Resources::new(1000, 2 * GIB),
        initial_replicas: 0,
    };

    let _service = cluster.add_service(service_spec);

    let scheduler = Scheduler::new(Box::new(LeastAllocated));
    let requests = Resources::new(500, GIB);

    // Should only consider the ready node
    let selected = scheduler.select_node(&cluster, requests);
    assert_eq!(selected, Some(ready_node));
}

#[test]
fn select_node_returns_none_when_nothing_fits() {
    let mut cluster = Cluster::new();

    let pool_spec = NodePoolSpec {
        name: "default".to_string(),
        allocatable: Resources::new(1000, 2 * GIB),
        min_nodes: 1,
        max_nodes: 10,
        initial_nodes: 0,
        price_per_hour: 0.1,
    };

    let pool = cluster.add_pool(pool_spec);

    let _node = cluster.add_node(pool, NodePhase::Ready);

    let service_spec = ServiceSpec {
        name: "test".to_string(),
        requests: Resources::new(2000, 4 * GIB),
        limits: Resources::new(4000, 8 * GIB),
        initial_replicas: 0,
    };

    let _service = cluster.add_service(service_spec);

    let scheduler = Scheduler::new(Box::new(LeastAllocated));
    let requests = Resources::new(2000, 4 * GIB);

    // Nothing fits
    let selected = scheduler.select_node(&cluster, requests);
    assert_eq!(selected, None);
}

#[test]
fn schedule_pending_binds_fitting_pods() {
    let mut cluster = Cluster::new();

    let pool_spec = NodePoolSpec {
        name: "default".to_string(),
        allocatable: Resources::new(4000, 8 * GIB),
        min_nodes: 1,
        max_nodes: 10,
        initial_nodes: 0,
        price_per_hour: 0.1,
    };

    let pool = cluster.add_pool(pool_spec);

    let node = cluster.add_node(pool, NodePhase::Ready);

    let service_spec = ServiceSpec {
        name: "test".to_string(),
        requests: Resources::new(500, GIB),
        limits: Resources::new(1000, 2 * GIB),
        initial_replicas: 0,
    };

    let service = cluster.add_service(service_spec);

    // Create pending pods
    let _pod1 = cluster.create_pod(service, SimTime::ZERO);
    let _pod2 = cluster.create_pod(service, SimTime::ZERO);

    let scheduler = Scheduler::new(Box::new(LeastAllocated));

    let bindings = scheduler.schedule_pending(&mut cluster);

    // Both pods should be bound
    assert_eq!(bindings.len(), 2);
    assert_eq!(bindings[0].1, node);
    assert_eq!(bindings[1].1, node);
}

#[test]
fn schedule_pending_marks_nonfitting_pods_unschedulable() {
    let mut cluster = Cluster::new();

    let pool_spec = NodePoolSpec {
        name: "default".to_string(),
        allocatable: Resources::new(1000, 2 * GIB),
        min_nodes: 1,
        max_nodes: 10,
        initial_nodes: 0,
        price_per_hour: 0.1,
    };

    let pool = cluster.add_pool(pool_spec);

    let _node = cluster.add_node(pool, NodePhase::Ready);

    let service_spec = ServiceSpec {
        name: "test".to_string(),
        requests: Resources::new(2000, 4 * GIB),
        limits: Resources::new(4000, 8 * GIB),
        initial_replicas: 0,
    };

    let service = cluster.add_service(service_spec);

    // Create pending pod that doesn't fit
    let pod = cluster.create_pod(service, SimTime::ZERO);

    let scheduler = Scheduler::new(Box::new(LeastAllocated));

    let bindings = scheduler.schedule_pending(&mut cluster);

    // No bindings
    assert_eq!(bindings.len(), 0);

    // Pod should be marked unschedulable
    assert!(cluster.pod(pod).unwrap().unschedulable);
}

#[test]
fn schedule_pending_returns_bindings_in_ascending_pod_id_order() {
    let mut cluster = Cluster::new();

    let pool_spec = NodePoolSpec {
        name: "default".to_string(),
        allocatable: Resources::new(4000, 8 * GIB),
        min_nodes: 1,
        max_nodes: 10,
        initial_nodes: 0,
        price_per_hour: 0.1,
    };

    let pool = cluster.add_pool(pool_spec);

    let _node = cluster.add_node(pool, NodePhase::Ready);

    let service_spec = ServiceSpec {
        name: "test".to_string(),
        requests: Resources::new(500, GIB),
        limits: Resources::new(1000, 2 * GIB),
        initial_replicas: 0,
    };

    let service = cluster.add_service(service_spec);

    let pod1 = cluster.create_pod(service, SimTime::ZERO);
    let pod2 = cluster.create_pod(service, SimTime::ZERO);
    let pod3 = cluster.create_pod(service, SimTime::ZERO);

    let scheduler = Scheduler::new(Box::new(LeastAllocated));

    let bindings = scheduler.schedule_pending(&mut cluster);

    assert_eq!(bindings[0].0, pod1);
    assert_eq!(bindings[1].0, pod2);
    assert_eq!(bindings[2].0, pod3);
}

#[test]
fn schedule_pending_sets_phase_to_starting() {
    let mut cluster = Cluster::new();

    let pool_spec = NodePoolSpec {
        name: "default".to_string(),
        allocatable: Resources::new(4000, 8 * GIB),
        min_nodes: 1,
        max_nodes: 10,
        initial_nodes: 0,
        price_per_hour: 0.1,
    };

    let pool = cluster.add_pool(pool_spec);

    let _node = cluster.add_node(pool, NodePhase::Ready);

    let service_spec = ServiceSpec {
        name: "test".to_string(),
        requests: Resources::new(500, GIB),
        limits: Resources::new(1000, 2 * GIB),
        initial_replicas: 0,
    };

    let service = cluster.add_service(service_spec);

    let pod = cluster.create_pod(service, SimTime::ZERO);

    let scheduler = Scheduler::new(Box::new(LeastAllocated));

    let _ = scheduler.schedule_pending(&mut cluster);

    // Pod should be in Starting phase
    assert_eq!(
        cluster.pod(pod).unwrap().phase,
        ksim_core::cluster::PodPhase::Starting
    );
}

#[test]
fn schedule_pending_empty_queue() {
    let mut cluster = Cluster::new();

    let pool_spec = NodePoolSpec {
        name: "default".to_string(),
        allocatable: Resources::new(4000, 8 * GIB),
        min_nodes: 1,
        max_nodes: 10,
        initial_nodes: 0,
        price_per_hour: 0.1,
    };

    let pool = cluster.add_pool(pool_spec);

    let _node = cluster.add_node(pool, NodePhase::Ready);

    let service_spec = ServiceSpec {
        name: "test".to_string(),
        requests: Resources::new(500, GIB),
        limits: Resources::new(1000, 2 * GIB),
        initial_replicas: 0,
    };

    let _service = cluster.add_service(service_spec);

    let scheduler = Scheduler::new(Box::new(LeastAllocated));

    let bindings = scheduler.schedule_pending(&mut cluster);

    // No bindings
    assert_eq!(bindings.len(), 0);
}
