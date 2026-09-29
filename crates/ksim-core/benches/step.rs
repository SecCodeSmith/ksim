use std::hint::black_box;

use criterion::{BatchSize, Criterion, Throughput, criterion_group, criterion_main};
use ksim_core::{
    actions::Actions,
    cluster::{NodePoolSpec, ServiceSpec},
    engine::{Simulation, SimulationBuilder},
    ids::ServiceId,
    performance::{PerformanceModel, ServiceCapacity, ServiceSignals},
    resources::{GIB, Resources},
    time::SimTime,
    workload::{ServiceDemand, WorkloadSource},
};

const SERVICES: usize = 50;
const NODES: u32 = 20;
const STEPS_PER_BATCH: u64 = 20;

#[derive(Debug)]
struct SteadyWorkload;

impl WorkloadSource for SteadyWorkload {
    fn services(&self) -> usize {
        SERVICES
    }

    fn demand(&self, _service: ServiceId, _now: SimTime) -> ServiceDemand {
        ServiceDemand {
            request_rate: 100.0,
            cpu_millis: 500.0,
            memory_bytes: 256.0 * 1024.0 * 1024.0,
        }
    }
}

#[derive(Debug)]
struct EchoPerformance;

impl PerformanceModel for EchoPerformance {
    fn evaluate(&self, capacity: &ServiceCapacity, demand: &ServiceDemand) -> ServiceSignals {
        ServiceSignals {
            cpu_usage_millis: demand.cpu_millis,
            served_rps: demand.request_rate * f64::from(capacity.ready_pods.min(1)),
            ..ServiceSignals::default()
        }
    }
}

fn simulation() -> Simulation {
    let mut builder = SimulationBuilder::new(42)
        .pool(NodePoolSpec {
            name: "default".into(),
            allocatable: Resources::new(32_000, 128 * GIB),
            min_nodes: 1,
            max_nodes: NODES,
            initial_nodes: NODES,
            price_per_hour: 1.0,
        })
        .workload(Box::new(SteadyWorkload))
        .performance(Box::new(EchoPerformance));
    for index in 0..SERVICES {
        builder = builder.service(ServiceSpec {
            name: format!("service-{index}"),
            requests: Resources::new(250, GIB / 4),
            limits: Resources::new(500, GIB / 2),
            initial_replicas: 4,
        });
    }
    builder.build().expect("valid benchmark simulation")
}

fn alternating_replicas(tick: u64) -> Actions {
    let replicas = if tick.is_multiple_of(2) { 6 } else { 3 };
    Actions {
        replicas: vec![Some(replicas); SERVICES],
        ..Actions::none()
    }
}

/// Pod ids are never reused, so cost grows with history; a fresh simulation per batch keeps the
/// measurement on a bounded window of steps.
fn step(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("step_50_services_20_nodes");
    group.throughput(Throughput::Elements(STEPS_PER_BATCH));
    group.bench_function("changing_replicas", |bencher| {
        bencher.iter_batched(
            simulation,
            |mut simulation| {
                for tick in 1..=STEPS_PER_BATCH {
                    black_box(simulation.step(&alternating_replicas(tick)));
                }
            },
            BatchSize::SmallInput,
        );
    });
    group.finish();
}

criterion_group!(benches, step);
criterion_main!(benches);
