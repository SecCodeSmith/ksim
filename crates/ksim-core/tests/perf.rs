mod common;

use common::{FixedDemand, ReadyPodsModel};
use ksim_core::{
    actions::Actions,
    cluster::{NodePoolSpec, ServiceSpec},
    engine::SimulationBuilder,
    lifecycle::FixedLatencies,
    resources::Resources,
    time::SimDuration,
    workload::ServiceDemand,
};
use std::time::Instant;

#[test]
#[ignore = "release-mode throughput gate; run with --release -- --ignored"]
fn step_throughput_stays_high_after_long_churn() {
    if cfg!(debug_assertions) {
        return;
    }

    // Create 50 services with consistent configuration.
    let mut services = Vec::new();
    for _ in 0..50 {
        services.push(ServiceSpec {
            name: "svc".to_string(),
            requests: Resources::new(250, 512 * 1024 * 1024),
            limits: Resources::new(500, 1024 * 1024 * 1024),
            initial_replicas: 4,
        });
    }

    // Create 50 service demands for the workload.
    let mut demands = Vec::new();
    for _ in 0..50 {
        demands.push(ServiceDemand {
            request_rate: 100.0,
            cpu_millis: 250.0,
            memory_bytes: 512.0,
        });
    }

    let mut builder = SimulationBuilder::new(42)
        .tick(SimDuration::from_secs(15))
        .pool(NodePoolSpec {
            name: "pool0".to_string(),
            allocatable: Resources::new(16000, 64 * 1024 * 1024 * 1024),
            min_nodes: 20,
            max_nodes: 20,
            initial_nodes: 20,
            price_per_hour: 1.0,
        })
        .workload(Box::new(FixedDemand::new(demands)))
        .performance(Box::new(ReadyPodsModel::new(false)))
        .latencies(Box::new(FixedLatencies {
            pod_startup: SimDuration::from_secs(5),
            node_provisioning: SimDuration::from_secs(60),
            termination: SimDuration::from_secs(30),
        }));

    for spec in services {
        builder = builder.service(spec);
    }

    let mut sim = builder.build().expect("build should succeed");

    // Warm up for 20_000 steps.
    for step in 0..20_000 {
        let mut actions = Actions::none();
        // Each service gets replicas: 3 + ((step + service_idx) % 6)
        let step_u32 = u32::try_from(step).expect("step fits in u32");
        let replicas = 3 + ((step_u32 + (step_u32 % 50)) % 6);
        actions.replicas.push(Some(replicas));
        sim.step(&actions);
    }

    // Time 20_000 steps.
    let start = Instant::now();
    for step in 20_000..40_000 {
        let step_u32 = u32::try_from(step).expect("step fits in u32");
        let service_idx = step_u32 % 50;
        let mut actions = Actions::none();
        let replicas = 3 + ((step_u32 + service_idx) % 6);
        actions.replicas.push(Some(replicas));
        sim.step(&actions);
    }
    let elapsed = start.elapsed();

    let steps = 20_000_u64;
    let steps_per_second = f64::from(u32::try_from(steps).expect("steps fits in u32")) / elapsed.as_secs_f64();

    eprintln!("Throughput: {steps_per_second:.1} steps/s");
    assert!(
        steps_per_second >= 20_000.0,
        "throughput should be >= 20_000.0 steps/s, got {steps_per_second:.1}"
    );
}
