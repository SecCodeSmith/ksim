use ksim_core::{
    cluster::NodePhase,
    ids::{NodeId, PoolId},
    observation::{NodeObservation, Observation, PoolObservation, ServiceObservation},
    performance::ServiceSignals,
    resources::Resources,
    time::{SimDuration, SimTime},
};
use ksim_metrics::trace::{EpisodeTrace, TickSample};

#[allow(dead_code)]
/// Build a simple Observation for testing: 2 services, 1 pool with 2 nodes.
pub fn simple_observation(now: SimTime) -> Observation {
    Observation {
        now,
        services: vec![
            ServiceObservation {
                desired: 2,
                ready: 2,
                pending: 0,
                unschedulable: 0,
                outdated: 0,
                requests: Resources {
                    cpu_millis: 500,
                    memory_bytes: 512 * 1024 * 1024,
                },
                limits: Resources {
                    cpu_millis: 1000,
                    memory_bytes: 1024 * 1024 * 1024,
                },
                signals: ServiceSignals::default(),
            },
            ServiceObservation {
                desired: 1,
                ready: 1,
                pending: 0,
                unschedulable: 0,
                outdated: 0,
                requests: Resources {
                    cpu_millis: 250,
                    memory_bytes: 256 * 1024 * 1024,
                },
                limits: Resources {
                    cpu_millis: 500,
                    memory_bytes: 512 * 1024 * 1024,
                },
                signals: ServiceSignals::default(),
            },
        ],
        pools: vec![PoolObservation {
            ready: 2,
            provisioning: 0,
            draining: 0,
            allocatable: Resources {
                cpu_millis: 4000,
                memory_bytes: 4 * 1024 * 1024 * 1024,
            },
            min_nodes: 1,
            max_nodes: 10,
        }],
        nodes: vec![
            NodeObservation {
                id: NodeId(0),
                pool: PoolId(0),
                phase: NodePhase::Ready,
                allocatable: Resources {
                    cpu_millis: 2000,
                    memory_bytes: 2 * 1024 * 1024 * 1024,
                },
                requested: Resources {
                    cpu_millis: 750,
                    memory_bytes: 768 * 1024 * 1024,
                },
                pods: 0..2,
            },
            NodeObservation {
                id: NodeId(1),
                pool: PoolId(0),
                phase: NodePhase::Ready,
                allocatable: Resources {
                    cpu_millis: 2000,
                    memory_bytes: 2 * 1024 * 1024 * 1024,
                },
                requested: Resources {
                    cpu_millis: 750,
                    memory_bytes: 768 * 1024 * 1024,
                },
                pods: 2..3,
            },
        ],
        node_pods: vec![],
    }
}

#[allow(dead_code)]
/// Build an episode trace from tick samples.
pub fn episode_trace(tick_duration: SimDuration, ticks: Vec<TickSample>) -> EpisodeTrace {
    EpisodeTrace {
        tick: tick_duration,
        ticks,
    }
}
