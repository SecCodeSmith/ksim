mod common;

use common::{FixedDemand, ReadyPodsModel};
use ksim_core::{
    actions::Actions,
    cluster::{NodePoolSpec, PodPhase, ServiceSpec},
    engine::{BuildError, SimulationBuilder},
    ids::ServiceId,
    lifecycle::FixedLatencies,
    observation::ScrapeConfig,
    resources::Resources,
    time::{SimDuration, SimTime},
    workload::ServiceDemand,
};

#[test]
fn build_error_missing_workload() {
    let result = SimulationBuilder::new(42)
        .pool(NodePoolSpec {
            name: "pool".to_string(),
            allocatable: Resources::new(1000, 1024 * 1024 * 1024),
            min_nodes: 1,
            max_nodes: 10,
            initial_nodes: 1,
            price_per_hour: 0.1,
        })
        .service(ServiceSpec {
            name: "svc".to_string(),
            requests: Resources::new(100, 128 * 1024 * 1024),
            limits: Resources::new(500, 512 * 1024 * 1024),
            initial_replicas: 1,
        })
        .performance(Box::new(ReadyPodsModel::new(false)))
        .build();

    assert!(matches!(result, Err(BuildError::MissingWorkload)));
}

#[test]
fn build_error_missing_performance_model() {
    let result = SimulationBuilder::new(42)
        .pool(NodePoolSpec {
            name: "pool".to_string(),
            allocatable: Resources::new(1000, 1024 * 1024 * 1024),
            min_nodes: 1,
            max_nodes: 10,
            initial_nodes: 1,
            price_per_hour: 0.1,
        })
        .service(ServiceSpec {
            name: "svc".to_string(),
            requests: Resources::new(100, 128 * 1024 * 1024),
            limits: Resources::new(500, 512 * 1024 * 1024),
            initial_replicas: 1,
        })
        .workload(Box::new(FixedDemand::new(vec![ServiceDemand {
            request_rate: 10.0,
            cpu_millis: 100.0,
            memory_bytes: 128.0,
        }])))
        .build();

    assert!(matches!(result, Err(BuildError::MissingPerformanceModel)));
}

#[test]
fn build_error_service_count_mismatch() {
    let result = SimulationBuilder::new(42)
        .pool(NodePoolSpec {
            name: "pool".to_string(),
            allocatable: Resources::new(1000, 1024 * 1024 * 1024),
            min_nodes: 1,
            max_nodes: 10,
            initial_nodes: 1,
            price_per_hour: 0.1,
        })
        .service(ServiceSpec {
            name: "svc".to_string(),
            requests: Resources::new(100, 128 * 1024 * 1024),
            limits: Resources::new(500, 512 * 1024 * 1024),
            initial_replicas: 1,
        })
        .workload(Box::new(FixedDemand::new(vec![
            ServiceDemand {
                request_rate: 10.0,
                cpu_millis: 100.0,
                memory_bytes: 128.0,
            },
            ServiceDemand {
                request_rate: 20.0,
                cpu_millis: 200.0,
                memory_bytes: 256.0,
            },
        ])))
        .performance(Box::new(ReadyPodsModel::new(false)))
        .build();

    assert!(matches!(
        result,
        Err(BuildError::ServiceCountMismatch {
            workload: 2,
            services: 1
        })
    ));
}

#[test]
fn build_error_zero_tick() {
    let result = SimulationBuilder::new(42)
        .tick(SimDuration::ZERO)
        .pool(NodePoolSpec {
            name: "pool".to_string(),
            allocatable: Resources::new(1000, 1024 * 1024 * 1024),
            min_nodes: 1,
            max_nodes: 10,
            initial_nodes: 1,
            price_per_hour: 0.1,
        })
        .service(ServiceSpec {
            name: "svc".to_string(),
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
        .build();

    assert!(matches!(result, Err(BuildError::ZeroTick)));
}

#[test]
fn warm_start_initial_pods_running_at_t0() {
    let sim = SimulationBuilder::new(42)
        .tick(SimDuration::from_secs(15))
        .pool(NodePoolSpec {
            name: "pool".to_string(),
            allocatable: Resources::new(1000, 1024 * 1024 * 1024),
            min_nodes: 1,
            max_nodes: 10,
            initial_nodes: 1,
            price_per_hour: 0.1,
        })
        .service(ServiceSpec {
            name: "svc".to_string(),
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

    assert_eq!(sim.now(), SimTime::ZERO);
    let obs = sim.observation();
    assert_eq!(obs.now, SimTime::ZERO);
    assert_eq!(obs.services[0].ready, 3);
}

#[test]
fn step_advances_time_by_tick() {
    let mut sim = SimulationBuilder::new(42)
        .tick(SimDuration::from_secs(15))
        .pool(NodePoolSpec {
            name: "pool".to_string(),
            allocatable: Resources::new(1000, 1024 * 1024 * 1024),
            min_nodes: 1,
            max_nodes: 10,
            initial_nodes: 1,
            price_per_hour: 0.1,
        })
        .service(ServiceSpec {
            name: "svc".to_string(),
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

    assert_eq!(sim.now(), SimTime::ZERO);
    sim.step(&Actions::none());
    assert_eq!(sim.now(), SimTime::from_secs(15));

    sim.step(&Actions::none());
    assert_eq!(sim.now(), SimTime::from_secs(30));
}

#[test]
fn scale_up_creates_pending_pods() {
    let mut sim = SimulationBuilder::new(42)
        .tick(SimDuration::from_secs(15))
        .pool(NodePoolSpec {
            name: "pool".to_string(),
            allocatable: Resources::new(1000, 1024 * 1024 * 1024),
            min_nodes: 1,
            max_nodes: 10,
            initial_nodes: 1,
            price_per_hour: 0.1,
        })
        .service(ServiceSpec {
            name: "svc".to_string(),
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

    let obs = sim.observation();
    assert_eq!(obs.services[0].ready, 1);

    let mut actions = Actions::none();
    actions.replicas.push(Some(3));
    sim.step(&actions);

    let obs = sim.observation();
    assert_eq!(obs.services[0].desired, 3);
    assert_eq!(obs.services[0].pending, 2);
}

#[test]
fn scale_up_pods_become_running_after_startup_latency() {
    let mut sim = SimulationBuilder::new(42)
        .tick(SimDuration::from_secs(15))
        .pool(NodePoolSpec {
            name: "pool".to_string(),
            allocatable: Resources::new(10000, 10 * 1024 * 1024 * 1024),
            min_nodes: 1,
            max_nodes: 10,
            initial_nodes: 1,
            price_per_hour: 0.1,
        })
        .service(ServiceSpec {
            name: "svc".to_string(),
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

    let mut actions = Actions::none();
    actions.replicas.push(Some(2));
    sim.step(&actions);

    let obs = sim.observation();
    assert_eq!(obs.services[0].ready, 1);
    assert_eq!(obs.services[0].pending, 1);

    sim.step(&Actions::none());
    let obs = sim.observation();
    assert_eq!(obs.services[0].ready, 2);
    assert_eq!(obs.services[0].pending, 0);
}

#[test]
fn scale_down_deletes_pods_per_pods_to_delete_order() {
    let mut sim = SimulationBuilder::new(42)
        .tick(SimDuration::from_secs(15))
        .pool(NodePoolSpec {
            name: "pool".to_string(),
            allocatable: Resources::new(10000, 10 * 1024 * 1024 * 1024),
            min_nodes: 1,
            max_nodes: 10,
            initial_nodes: 1,
            price_per_hour: 0.1,
        })
        .service(ServiceSpec {
            name: "svc".to_string(),
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

    let mut actions = Actions::none();
    actions.replicas.push(Some(1));
    sim.step(&actions);

    let obs = sim.observation();
    assert_eq!(obs.services[0].desired, 1);
}

#[test]
fn pool_delta_adds_provisioning_node() {
    let mut sim = SimulationBuilder::new(42)
        .tick(SimDuration::from_secs(15))
        .pool(NodePoolSpec {
            name: "pool".to_string(),
            allocatable: Resources::new(1000, 1024 * 1024 * 1024),
            min_nodes: 0,
            max_nodes: 10,
            initial_nodes: 0,
            price_per_hour: 0.1,
        })
        .service(ServiceSpec {
            name: "svc".to_string(),
            requests: Resources::new(100, 128 * 1024 * 1024),
            limits: Resources::new(500, 512 * 1024 * 1024),
            initial_replicas: 0,
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

    let obs = sim.observation();
    assert_eq!(obs.pools[0].ready, 0);
    assert_eq!(obs.pools[0].provisioning, 0);

    let mut actions = Actions::none();
    actions.pool_delta.push(1);
    sim.step(&actions);

    let obs = sim.observation();
    assert_eq!(obs.pools[0].provisioning, 1);
}

#[test]
fn pool_delta_node_becomes_ready_after_provisioning_latency() {
    let mut sim = SimulationBuilder::new(42)
        .tick(SimDuration::from_secs(15))
        .pool(NodePoolSpec {
            name: "pool".to_string(),
            allocatable: Resources::new(1000, 1024 * 1024 * 1024),
            min_nodes: 0,
            max_nodes: 10,
            initial_nodes: 0,
            price_per_hour: 0.1,
        })
        .service(ServiceSpec {
            name: "svc".to_string(),
            requests: Resources::new(100, 128 * 1024 * 1024),
            limits: Resources::new(500, 512 * 1024 * 1024),
            initial_replicas: 0,
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

    let mut actions = Actions::none();
    actions.pool_delta.push(1);
    sim.step(&actions);

    for _ in 0..4 {
        sim.step(&Actions::none());
    }

    let obs = sim.observation();
    assert_eq!(obs.pools[0].ready, 1);
    assert_eq!(obs.pools[0].provisioning, 0);
}

#[test]
fn requests_action_applies_to_new_pods_only() {
    let mut sim = SimulationBuilder::new(42)
        .tick(SimDuration::from_secs(15))
        .pool(NodePoolSpec {
            name: "pool".to_string(),
            allocatable: Resources::new(10000, 10 * 1024 * 1024 * 1024),
            min_nodes: 1,
            max_nodes: 10,
            initial_nodes: 1,
            price_per_hour: 0.1,
        })
        .service(ServiceSpec {
            name: "svc".to_string(),
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

    let cluster = sim.cluster();
    let pods_before = cluster.pods_of(ServiceId(0));
    let old_pod_requests = cluster.pod(pods_before[0]).unwrap().requests;

    let mut actions = Actions::none();
    actions.replicas.push(Some(2));
    actions
        .requests
        .push(Some(Resources::new(200, 256 * 1024 * 1024)));
    sim.step(&actions);

    let cluster = sim.cluster();
    let pods_after = cluster.pods_of(ServiceId(0));
    let old_pod_after = cluster.pod(pods_before[0]).unwrap();
    let new_pod = cluster.pod(pods_after[1]).unwrap();

    assert_eq!(old_pod_after.requests, old_pod_requests);
    assert_eq!(new_pod.requests, Resources::new(200, 256 * 1024 * 1024));
}

#[test]
fn oom_signal_causes_restarts() {
    let mut sim = SimulationBuilder::new(42)
        .tick(SimDuration::from_secs(15))
        .pool(NodePoolSpec {
            name: "pool".to_string(),
            allocatable: Resources::new(10000, 10 * 1024 * 1024 * 1024),
            min_nodes: 1,
            max_nodes: 10,
            initial_nodes: 1,
            price_per_hour: 0.1,
        })
        .service(ServiceSpec {
            name: "svc".to_string(),
            requests: Resources::new(100, 128 * 1024 * 1024),
            limits: Resources::new(500, 512 * 1024 * 1024),
            initial_replicas: 1,
        })
        .workload(Box::new(FixedDemand::new(vec![ServiceDemand {
            request_rate: 10.0,
            cpu_millis: 100.0,
            memory_bytes: 128.0,
        }])))
        .performance(Box::new(ReadyPodsModel::new(true)))
        .latencies(Box::new(FixedLatencies {
            pod_startup: SimDuration::from_secs(5),
            node_provisioning: SimDuration::from_secs(60),
            termination: SimDuration::from_secs(30),
        }))
        .build()
        .expect("build should succeed");

    let cluster = sim.cluster();
    let pods = cluster.pods_of(ServiceId(0));
    let pod_view = cluster.pod(pods[0]).unwrap();
    assert_eq!(pod_view.phase, PodPhase::Running);
    assert_eq!(pod_view.restarts, 0);

    sim.step(&Actions::none());

    let cluster = sim.cluster();
    let pod_view = cluster.pod(pods[0]).unwrap();
    assert_eq!(pod_view.phase, PodPhase::Starting);
    assert_eq!(pod_view.restarts, 1);
}

#[test]
fn observation_signals_lag_by_one_scrape() {
    let mut sim = SimulationBuilder::new(42)
        .tick(SimDuration::from_secs(15))
        .scrape(ScrapeConfig {
            resolution: SimDuration::from_secs(15),
            lag_scrapes: 1,
        })
        .pool(NodePoolSpec {
            name: "pool".to_string(),
            allocatable: Resources::new(10000, 10 * 1024 * 1024 * 1024),
            min_nodes: 1,
            max_nodes: 10,
            initial_nodes: 1,
            price_per_hour: 0.1,
        })
        .service(ServiceSpec {
            name: "svc".to_string(),
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

    let mut actions = Actions::none();
    actions.replicas.push(Some(2));
    sim.step(&actions);

    sim.step(&Actions::none());

    let obs = sim.observation();
    assert!((obs.services[0].signals.served_rps - 1.0).abs() < 1e-10);
}
