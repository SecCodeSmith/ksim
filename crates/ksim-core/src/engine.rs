use crate::{
    actions::Actions,
    cluster::{Cluster, NodePoolSpec, ServiceSpec},
    lifecycle::LatencyModel,
    observation::{Observation, ScrapeConfig},
    performance::PerformanceModel,
    scheduler::ScoringPolicy,
    time::{SimDuration, SimTime},
    workload::WorkloadSource,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BuildError {
    MissingWorkload,
    MissingPerformanceModel,
    ServiceCountMismatch { workload: usize, services: usize },
    ZeroTick,
}

/// Defaults: tick 15 s (HPA sync period), `LeastAllocated`, `DefaultLatencies`,
/// `ScrapeConfig::default()`.
#[derive(Debug)]
pub struct SimulationBuilder {
    seed: u64,
    tick: SimDuration,
    pools: Vec<NodePoolSpec>,
    services: Vec<ServiceSpec>,
    workload: Option<Box<dyn WorkloadSource>>,
    performance: Option<Box<dyn PerformanceModel>>,
    scoring: Option<Box<dyn ScoringPolicy>>,
    latencies: Option<Box<dyn LatencyModel>>,
    scrape: ScrapeConfig,
}

impl SimulationBuilder {
    #[must_use]
    pub fn new(seed: u64) -> Self {
        let _ = seed;
        todo!()
    }

    #[must_use]
    pub fn tick(mut self, tick: SimDuration) -> Self {
        self.tick = tick;
        self
    }

    #[must_use]
    pub fn pool(mut self, spec: NodePoolSpec) -> Self {
        self.pools.push(spec);
        self
    }

    #[must_use]
    pub fn service(mut self, spec: ServiceSpec) -> Self {
        self.services.push(spec);
        self
    }

    #[must_use]
    pub fn workload(mut self, workload: Box<dyn WorkloadSource>) -> Self {
        self.workload = Some(workload);
        self
    }

    #[must_use]
    pub fn performance(mut self, model: Box<dyn PerformanceModel>) -> Self {
        self.performance = Some(model);
        self
    }

    #[must_use]
    pub fn scoring(mut self, policy: Box<dyn ScoringPolicy>) -> Self {
        self.scoring = Some(policy);
        self
    }

    #[must_use]
    pub fn latencies(mut self, model: Box<dyn LatencyModel>) -> Self {
        self.latencies = Some(model);
        self
    }

    #[must_use]
    pub fn scrape(mut self, config: ScrapeConfig) -> Self {
        self.scrape = config;
        self
    }

    /// Starts warm: `initial_nodes` are `Ready` and `initial_replicas` pods are `Running`,
    /// placed by the scheduler at time zero.
    ///
    /// # Errors
    /// Returns a [`BuildError`] when a required component is missing or inconsistent.
    pub fn build(self) -> Result<Simulation, BuildError> {
        let _ = (
            self.seed,
            self.tick,
            self.pools,
            self.services,
            self.workload,
            self.performance,
            self.scoring,
            self.latencies,
            self.scrape,
        );
        todo!()
    }
}

/// One `step` advances time by one tick:
/// 1. apply actions at the current time: desired replicas (create `Pending` pods, or delete
///    victims from `Cluster::pods_to_delete`: unbound pods vanish, bound pods become
///    `Terminating` and are removed after the termination latency), new requests (new pods
///    only), pool deltas (add `Provisioning` nodes up to `max_nodes`, or remove empty
///    `Ready` nodes, highest id first, down to `min_nodes`);
/// 2. advance `now` by the tick and apply every due event (node ready, pod running, pod
///    removed, pod restarted);
/// 3. schedule pending pods; bound pods become `Running` after the pod startup latency;
/// 4. evaluate the performance model per service from `Running` pods; an `oom` signal
///    restarts every running pod of the service after `crash_loop_backoff(restarts)`;
/// 5. scrape metrics when `now` is a multiple of the scrape resolution and publish the
///    observation.
#[derive(Debug)]
pub struct Simulation {
    cluster: Cluster,
    observation: Observation,
    now: SimTime,
}

impl Simulation {
    pub fn step(&mut self, actions: &Actions) -> &Observation {
        let _ = actions;
        todo!()
    }

    #[must_use]
    pub fn now(&self) -> SimTime {
        self.now
    }

    #[must_use]
    pub fn observation(&self) -> &Observation {
        &self.observation
    }

    #[must_use]
    pub fn cluster(&self) -> &Cluster {
        &self.cluster
    }
}
