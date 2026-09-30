use crate::numeric::quantity_f64;
use ksim_core::{
    actions::Actions, observation::Observation, time::SimDuration, time::SimTime,
    workload::ServiceDemand,
};

/// One service at one tick; `supply_cpu_millis` is `ready * requests.cpu_millis`.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ServiceSample {
    pub demand_rps: f64,
    pub demand_cpu_millis: f64,
    pub supply_cpu_millis: f64,
    pub cpu_usage_millis: f64,
    pub served_rps: f64,
    pub error_rate: f64,
    pub latency_p50_ms: f64,
    pub latency_p95_ms: f64,
    pub latency_p99_ms: f64,
    pub desired: u32,
    pub ready: u32,
    pub pending: u32,
    pub oom: bool,
}

/// `hourly_cost` sums `price_per_hour` over every existing node (ready, provisioning or
/// draining). `replica_changes` counts services whose replicas action differs from their
/// desired count; `evictions` sums the evict action; `node_changes` sums `|pool_delta|` plus
/// drained nodes.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct TickSample {
    pub time: SimTime,
    pub services: Vec<ServiceSample>,
    pub nodes: u32,
    pub hourly_cost: f64,
    pub replica_changes: u32,
    pub evictions: u32,
    pub node_changes: u32,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct EpisodeTrace {
    pub tick: SimDuration,
    pub ticks: Vec<TickSample>,
}

/// Builds an [`EpisodeTrace`] from the observation, the actions chosen for it and the true
/// demand at that time; call once per step, before `Simulation::step`.
#[derive(Debug)]
pub struct TraceRecorder {
    pool_prices: Vec<f64>,
    trace: EpisodeTrace,
}

impl TraceRecorder {
    #[must_use]
    pub fn new(tick: SimDuration, pool_prices: Vec<f64>) -> Self {
        Self {
            pool_prices,
            trace: EpisodeTrace {
                tick,
                ticks: Vec::new(),
            },
        }
    }

    pub fn record(
        &mut self,
        observation: &Observation,
        actions: &Actions,
        demands: &[ServiceDemand],
    ) {
        let tick = self.sample(observation, actions, demands);
        self.trace.ticks.push(tick);
    }

    /// The tick `record` would append for these inputs, without recording it.
    #[must_use]
    pub fn sample(
        &self,
        observation: &Observation,
        actions: &Actions,
        demands: &[ServiceDemand],
    ) -> TickSample {
        TickSample {
            time: observation.now,
            services: service_samples(observation, demands),
            nodes: count(observation.nodes.len()),
            hourly_cost: self.hourly_cost(observation),
            replica_changes: replica_changes(observation, actions),
            evictions: actions.evict.iter().sum(),
            node_changes: node_changes(actions),
        }
    }

    /// The ticks recorded so far.
    #[must_use]
    pub fn trace(&self) -> &EpisodeTrace {
        &self.trace
    }

    fn hourly_cost(&self, observation: &Observation) -> f64 {
        observation
            .nodes
            .iter()
            .map(|node| {
                self.pool_prices
                    .get(node.pool.0 as usize)
                    .copied()
                    .unwrap_or(0.0)
            })
            .sum()
    }

    #[must_use]
    pub fn finish(self) -> EpisodeTrace {
        self.trace
    }
}

fn count(len: usize) -> u32 {
    u32::try_from(len).unwrap_or(u32::MAX)
}

fn service_samples(observation: &Observation, demands: &[ServiceDemand]) -> Vec<ServiceSample> {
    observation
        .services
        .iter()
        .enumerate()
        .map(|(index, service)| {
            let demand = demands.get(index).copied().unwrap_or_default();
            let signals = &service.signals;
            ServiceSample {
                demand_rps: demand.request_rate,
                demand_cpu_millis: demand.cpu_millis,
                supply_cpu_millis: f64::from(service.ready)
                    * quantity_f64(service.requests.cpu_millis),
                cpu_usage_millis: signals.cpu_usage_millis,
                served_rps: signals.served_rps,
                error_rate: signals.error_rate,
                latency_p50_ms: signals.latency_p50_ms,
                latency_p95_ms: signals.latency_p95_ms,
                latency_p99_ms: signals.latency_p99_ms,
                desired: service.desired,
                ready: service.ready,
                pending: service.pending,
                oom: signals.oom,
            }
        })
        .collect()
}

fn replica_changes(observation: &Observation, actions: &Actions) -> u32 {
    let changed = actions
        .replicas
        .iter()
        .zip(&observation.services)
        .filter(|(target, service)| target.is_some_and(|replicas| replicas != service.desired))
        .count();
    count(changed)
}

fn node_changes(actions: &Actions) -> u32 {
    let scaled: u32 = actions
        .pool_delta
        .iter()
        .map(|delta| delta.unsigned_abs())
        .sum();
    scaled + count(actions.drain.len())
}

#[cfg(test)]
mod tests {
    use ksim_core::observation::{NodeObservation, PoolObservation, ServiceObservation};

    use super::*;

    fn observation_at(seconds: u64) -> Observation {
        Observation {
            now: SimTime::from_secs(seconds),
            services: vec![ServiceObservation {
                desired: 3,
                ready: 2,
                ..ServiceObservation::default()
            }],
            pools: vec![PoolObservation::default()],
            nodes: Vec::<NodeObservation>::new(),
            node_pods: Vec::new(),
        }
    }

    fn demand(request_rate: f64) -> ServiceDemand {
        ServiceDemand {
            request_rate,
            ..ServiceDemand::default()
        }
    }

    #[test]
    fn sample_does_not_record_and_equals_the_recorded_tick() {
        let mut recorder = TraceRecorder::new(SimDuration::from_secs(15), vec![0.5]);
        let observation = observation_at(30);
        let actions = Actions {
            replicas: vec![Some(5)],
            ..Actions::none()
        };
        let demands = [demand(10.0)];

        let sample = recorder.sample(&observation, &actions, &demands);
        assert!(recorder.trace().ticks.is_empty());

        recorder.record(&observation, &actions, &demands);
        assert_eq!(recorder.trace().ticks, vec![sample]);
    }

    #[test]
    fn sample_counts_replica_changes_against_the_given_observation() {
        let recorder = TraceRecorder::new(SimDuration::from_secs(15), vec![0.5]);
        let actions = Actions {
            replicas: vec![Some(5)],
            ..Actions::none()
        };

        let sample = recorder.sample(&observation_at(0), &actions, &[demand(1.0)]);

        assert_eq!(sample.replica_changes, 1);
        assert_eq!(sample.time, SimTime::from_secs(0));
    }
}
