mod common;

use common::{ConstantLatencyModel, FixedDemand};
use ksim_core::{
    actions::Actions,
    cluster::{NodePoolSpec, ServiceSpec},
    engine::{BuildError, SimulationBuilder},
    ids::ServiceId,
    lifecycle::FixedLatencies,
    observation::ScrapeConfig,
    resources::Resources,
    time::SimDuration,
    workload::ServiceDemand,
};

#[test]
fn performance_override_routing_after_scrape_lag() {
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
            name: "svc0".to_string(),
            requests: Resources::new(100, 128 * 1024 * 1024),
            limits: Resources::new(500, 512 * 1024 * 1024),
            initial_replicas: 1,
        })
        .service(ServiceSpec {
            name: "svc1".to_string(),
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
        .performance(Box::new(ConstantLatencyModel(10.0)))
        .performance_for(ServiceId(1), Box::new(ConstantLatencyModel(99.0)))
        .latencies(Box::new(FixedLatencies {
            pod_startup: SimDuration::from_secs(5),
            node_provisioning: SimDuration::from_secs(60),
            termination: SimDuration::from_secs(30),
        }))
        .scrape(ScrapeConfig {
            resolution: SimDuration::from_secs(15),
            lag_scrapes: 1,
        })
        .build()
        .expect("build should succeed");

    let actions = Actions::default();

    sim.step(&actions);
    sim.step(&actions);

    let obs = sim.observation();
    assert!((obs.services[0].signals.latency_p95_ms - 10.0).abs() < 1e-9);
    assert!((obs.services[1].signals.latency_p95_ms - 99.0).abs() < 1e-9);
}

#[test]
fn performance_override_all_services_no_default() {
    let _sim = SimulationBuilder::new(42)
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
            name: "svc0".to_string(),
            requests: Resources::new(100, 128 * 1024 * 1024),
            limits: Resources::new(500, 512 * 1024 * 1024),
            initial_replicas: 1,
        })
        .service(ServiceSpec {
            name: "svc1".to_string(),
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
        .performance_for(ServiceId(0), Box::new(ConstantLatencyModel(10.0)))
        .performance_for(ServiceId(1), Box::new(ConstantLatencyModel(20.0)))
        .latencies(Box::new(FixedLatencies {
            pod_startup: SimDuration::from_secs(5),
            node_provisioning: SimDuration::from_secs(60),
            termination: SimDuration::from_secs(30),
        }))
        .build()
        .expect("build should succeed with overrides for all services");
}

#[test]
fn performance_missing_override_and_no_default() {
    let result = SimulationBuilder::new(42)
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
        .latencies(Box::new(FixedLatencies {
            pod_startup: SimDuration::from_secs(5),
            node_provisioning: SimDuration::from_secs(60),
            termination: SimDuration::from_secs(30),
        }))
        .build();

    assert!(matches!(result, Err(BuildError::MissingPerformanceModel)));
}

#[test]
fn performance_override_nonexistent_service() {
    let result = SimulationBuilder::new(42)
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
            name: "svc0".to_string(),
            requests: Resources::new(100, 128 * 1024 * 1024),
            limits: Resources::new(500, 512 * 1024 * 1024),
            initial_replicas: 1,
        })
        .service(ServiceSpec {
            name: "svc1".to_string(),
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
        .performance_for(ServiceId(5), Box::new(ConstantLatencyModel(99.0)))
        .latencies(Box::new(FixedLatencies {
            pod_startup: SimDuration::from_secs(5),
            node_provisioning: SimDuration::from_secs(60),
            termination: SimDuration::from_secs(30),
        }))
        .build();

    assert!(matches!(
        result,
        Err(BuildError::ServiceCountMismatch {
            workload: 2,
            services: 2
        })
    ));
}
