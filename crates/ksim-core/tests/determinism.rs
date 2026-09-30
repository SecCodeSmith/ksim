mod common;

use common::{FixedDemand, ReadyPodsModel};
use ksim_core::{
    actions::Actions,
    cluster::{NodePoolSpec, ServiceSpec},
    engine::SimulationBuilder,
    resources::Resources,
    time::SimDuration,
    workload::ServiceDemand,
};

#[test]
fn identical_seed_and_latencies_produce_identical_observations() {
    let mut sim1 = SimulationBuilder::new(42)
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
        .build()
        .expect("build should succeed");

    let mut sim2 = SimulationBuilder::new(42)
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
        .build()
        .expect("build should succeed");

    for _ in 0..200 {
        let mut actions = Actions::none();
        actions.replicas.push(Some(2));
        sim1.step(&actions);
        sim2.step(&actions);

        let obs1 = sim1.observation();
        let obs2 = sim2.observation();

        assert_eq!(obs1, obs2);
    }
}

#[test]
fn different_seeds_produce_different_observations() {
    let mut sim1 = SimulationBuilder::new(42)
        .tick(SimDuration::from_secs(15))
        .pool(NodePoolSpec {
            name: "pool".to_string(),
            allocatable: Resources::new(10000, 10 * 1024 * 1024 * 1024),
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
        .build()
        .expect("build should succeed");

    let mut sim2 = SimulationBuilder::new(43)
        .tick(SimDuration::from_secs(15))
        .pool(NodePoolSpec {
            name: "pool".to_string(),
            allocatable: Resources::new(10000, 10 * 1024 * 1024 * 1024),
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
        .build()
        .expect("build should succeed");

    let mut different_count = 0;
    for _ in 0..200 {
        let mut actions = Actions::none();
        actions.pool_delta.push(1);
        sim1.step(&actions);
        sim2.step(&actions);

        let obs1 = sim1.observation();
        let obs2 = sim2.observation();

        if obs1 != obs2 {
            different_count += 1;
        }
    }

    assert!(different_count > 0);
}
