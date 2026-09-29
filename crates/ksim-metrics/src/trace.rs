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
        let _ = (observation, actions, demands, &self.pool_prices);
        todo!()
    }

    #[must_use]
    pub fn finish(self) -> EpisodeTrace {
        self.trace
    }
}
