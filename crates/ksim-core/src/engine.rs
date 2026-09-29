use rand_chacha::ChaCha8Rng;

use crate::{
    actions::Actions,
    cluster::{Cluster, NodePhase, NodePoolSpec, PodPhase, ServiceSpec},
    event::EventQueue,
    ids::{NodeId, PodId, PoolId, ServiceId},
    lifecycle::{DefaultLatencies, LatencyModel, crash_loop_backoff},
    observation::{
        NodeObservation, Observation, PodPlacement, PoolObservation, ScrapeConfig,
        ServiceObservation,
    },
    performance::{PerformanceModel, ServiceCapacity, ServiceSignals},
    resources::Resources,
    rng::{Concern, RngStreams},
    scheduler::{LeastAllocated, Scheduler, ScoringPolicy},
    scrape::MetricsPipeline,
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
        Self {
            seed,
            tick: SimDuration::from_secs(15),
            pools: Vec::new(),
            services: Vec::new(),
            workload: None,
            performance: None,
            scoring: None,
            latencies: None,
            scrape: ScrapeConfig::default(),
        }
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
        let workload = self.workload.ok_or(BuildError::MissingWorkload)?;
        let performance = self
            .performance
            .ok_or(BuildError::MissingPerformanceModel)?;
        if workload.services() != self.services.len() {
            return Err(BuildError::ServiceCountMismatch {
                workload: workload.services(),
                services: self.services.len(),
            });
        }
        if self.tick == SimDuration::ZERO {
            return Err(BuildError::ZeroTick);
        }
        let mut cluster = Cluster::new();
        for spec in self.pools {
            cluster.add_pool(spec);
        }
        for spec in self.services {
            cluster.add_service(spec);
        }
        let desired = (0..cluster.service_count())
            .map(|index| {
                cluster
                    .service_spec(ServiceId(index_to_id(index)))
                    .initial_replicas
            })
            .collect();
        let mut simulation = Simulation {
            metrics: MetricsPipeline::new(self.scrape, cluster.service_count()),
            cluster,
            observation: Observation::default(),
            now: SimTime::ZERO,
            tick: self.tick,
            desired,
            events: EventQueue::new(),
            scheduler: Scheduler::new(self.scoring.unwrap_or_else(|| Box::new(LeastAllocated))),
            latencies: self
                .latencies
                .unwrap_or_else(|| Box::new(DefaultLatencies::default())),
            lifecycle_rng: RngStreams::new(self.seed).stream(Concern::Lifecycle),
            workload,
            performance,
            signals: Vec::new(),
        };
        simulation.warm_start();
        Ok(simulation)
    }
}

/// Discrete-time cluster simulation. Each [`Simulation::step`] runs, in order:
///
/// 1. apply actions: requests first (so pods created by the same step get the new requests),
///    then replica changes, evictions of outdated pods, node drains, then node pool deltas;
///    afterwards every service is reconciled: missing non-terminating pods are created,
///    surplus ones deleted in `Cluster::pods_to_delete` order;
/// 2. advance the clock by one tick and apply due events (node ready, pod running, pod removed),
///    then remove draining nodes with no bound pods;
/// 3. schedule pending pods;
/// 4. evaluate the performance model per service, restarting running pods of a service reporting out-of-memory
///    with kubelet crash-loop backoff;
/// 5. scrape metrics on the resolution grid (consumers see the lagged snapshot);
/// 6. publish the observation.
///
/// Cost per step is proportional to live pods and nodes, never to pods or nodes removed
/// earlier in the episode.
#[derive(Debug)]
pub struct Simulation {
    cluster: Cluster,
    observation: Observation,
    now: SimTime,
    tick: SimDuration,
    desired: Vec<u32>,
    events: EventQueue<SimEvent>,
    scheduler: Scheduler,
    latencies: Box<dyn LatencyModel>,
    lifecycle_rng: ChaCha8Rng,
    workload: Box<dyn WorkloadSource>,
    performance: Box<dyn PerformanceModel>,
    metrics: MetricsPipeline,
    signals: Vec<ServiceSignals>,
}

#[derive(Debug, Clone, Copy)]
enum SimEvent {
    NodeReady(NodeId),
    PodRunning(PodId),
    PodRemoved(PodId),
}

fn index_to_id(index: usize) -> u32 {
    u32::try_from(index).expect("id space exhausted")
}

impl Simulation {
    pub fn step(&mut self, actions: &Actions) -> &Observation {
        self.apply_actions(actions);
        self.now = self.now + self.tick;
        self.apply_due_events();
        self.remove_drained_nodes();
        self.schedule_pending_pods();
        self.evaluate_services();
        self.metrics.scrape(self.now, &self.signals);
        self.publish_observation();
        &self.observation
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

    fn warm_start(&mut self) {
        for index in 0..self.cluster.pool_count() {
            let pool = PoolId(index_to_id(index));
            for _ in 0..self.cluster.pool_spec(pool).initial_nodes {
                self.cluster.add_node(pool, NodePhase::Ready);
            }
        }
        for (index, &replicas) in self.desired.clone().iter().enumerate() {
            for _ in 0..replicas {
                self.cluster
                    .create_pod(ServiceId(index_to_id(index)), self.now);
            }
        }
        for (pod, _) in self.scheduler.schedule_pending(&mut self.cluster) {
            self.cluster.set_pod_phase(pod, PodPhase::Running);
        }
        self.publish_observation();
    }

    fn apply_actions(&mut self, actions: &Actions) {
        self.apply_request_changes(&actions.requests);
        self.apply_replica_changes(&actions.replicas);
        self.apply_evictions(&actions.evict);
        self.apply_drains(&actions.drain);
        self.apply_pool_deltas(&actions.pool_delta);
        self.reconcile_all_services();
    }

    fn apply_request_changes(&mut self, requests: &[Option<Resources>]) {
        let services = self.cluster.service_count();
        for (index, requests) in requests.iter().enumerate().take(services) {
            if let Some(requests) = requests {
                self.cluster
                    .set_service_requests(ServiceId(index_to_id(index)), *requests);
            }
        }
    }

    fn apply_replica_changes(&mut self, replicas: &[Option<u32>]) {
        for (index, replicas) in replicas.iter().enumerate().take(self.desired.len()) {
            if let Some(replicas) = replicas {
                self.desired[index] = *replicas;
            }
        }
    }

    fn apply_evictions(&mut self, evictions: &[u32]) {
        let services = self.cluster.service_count();
        for (index, &count) in evictions.iter().enumerate().take(services) {
            let service = ServiceId(index_to_id(index));
            for pod in self.cluster.pods_to_evict(service, count as usize) {
                self.delete_pod(pod);
            }
        }
    }

    fn apply_drains(&mut self, nodes: &[NodeId]) {
        for &node in nodes {
            if self.may_drain(node) {
                self.drain_node(node);
            }
        }
    }

    fn may_drain(&self, node: NodeId) -> bool {
        let Some(phase) = self.cluster.node_phase(node) else {
            return false;
        };
        let pool = self.cluster.node_pool(node);
        let schedulable = self.cluster.count_nodes(pool, NodePhase::Ready)
            + self.cluster.count_nodes(pool, NodePhase::Provisioning);
        phase != NodePhase::Draining && schedulable > self.cluster.pool_spec(pool).min_nodes
    }

    fn drain_node(&mut self, node: NodeId) {
        self.cluster.set_node_phase(node, NodePhase::Draining);
        for pod in self.cluster.node_pod_ids(node).to_vec() {
            self.delete_pod(pod);
        }
    }

    fn reconcile_all_services(&mut self) {
        for index in 0..self.cluster.service_count() {
            self.reconcile_replicas(ServiceId(index_to_id(index)));
        }
    }

    fn reconcile_replicas(&mut self, service: ServiceId) {
        let desired = self.desired[service.index()] as usize;
        let active = self.active_pod_count(service);
        for _ in active..desired {
            self.cluster.create_pod(service, self.now);
        }
        if active > desired {
            for pod in self.cluster.pods_to_delete(service, active - desired) {
                self.delete_pod(pod);
            }
        }
    }

    fn active_pod_count(&self, service: ServiceId) -> usize {
        self.cluster.service_pod_ids(service).len()
            - self.cluster.count_pods(service, PodPhase::Terminating) as usize
    }

    fn delete_pod(&mut self, pod: PodId) {
        let Some(view) = self.cluster.pod(pod) else {
            return;
        };
        if view.phase == PodPhase::Terminating {
            return;
        }
        if view.node.is_none() {
            self.cluster.remove_pod(pod);
            return;
        }
        self.cluster.set_pod_phase(pod, PodPhase::Terminating);
        let termination = self.latencies.termination(&mut self.lifecycle_rng);
        self.events
            .push(self.now + termination, SimEvent::PodRemoved(pod));
    }

    fn apply_pool_deltas(&mut self, deltas: &[i32]) {
        let pools = self.cluster.pool_count();
        for (index, &delta) in deltas.iter().enumerate().take(pools) {
            let pool = PoolId(index_to_id(index));
            if delta > 0 {
                self.provision_nodes(pool, delta.unsigned_abs());
            } else {
                self.remove_empty_nodes(pool, delta.unsigned_abs());
            }
        }
    }

    fn pool_nodes(&self, pool: PoolId) -> Vec<NodeId> {
        self.cluster
            .nodes()
            .into_iter()
            .filter(|&node| self.cluster.node_pool(node) == pool)
            .collect()
    }

    fn provision_nodes(&mut self, pool: PoolId, requested: u32) {
        let existing = index_to_id(self.pool_nodes(pool).len());
        let headroom = self
            .cluster
            .pool_spec(pool)
            .max_nodes
            .saturating_sub(existing);
        for _ in 0..requested.min(headroom) {
            let node = self.cluster.add_node(pool, NodePhase::Provisioning);
            let delay = self.latencies.node_provisioning(&mut self.lifecycle_rng);
            self.events
                .push(self.now + delay, SimEvent::NodeReady(node));
        }
    }

    fn remove_empty_nodes(&mut self, pool: PoolId, requested: u32) {
        let min_nodes = self.cluster.pool_spec(pool).min_nodes as usize;
        let mut remaining = self.pool_nodes(pool).len();
        let removable = self.pool_nodes(pool).into_iter().rev().filter(|&node| {
            self.cluster.node_phase(node) == Some(NodePhase::Ready)
                && self.cluster.node_requested(node) == Resources::ZERO
        });
        let victims: Vec<NodeId> = removable
            .take(requested as usize)
            .take_while(|_| {
                let allowed = remaining > min_nodes;
                remaining = remaining.saturating_sub(1);
                allowed
            })
            .collect();
        for node in victims {
            self.cluster.remove_node(node);
        }
    }

    fn apply_due_events(&mut self) {
        while let Some((_, event)) = self.events.pop_due(self.now) {
            match event {
                SimEvent::NodeReady(node) => self.mark_node_ready(node),
                SimEvent::PodRunning(pod) => self.mark_pod_running(pod),
                SimEvent::PodRemoved(pod) => self.cluster.remove_pod(pod),
            }
        }
    }

    fn remove_drained_nodes(&mut self) {
        let drained: Vec<NodeId> = self
            .cluster
            .node_ids()
            .iter()
            .copied()
            .filter(|&node| {
                self.cluster.node_phase(node) == Some(NodePhase::Draining)
                    && self.cluster.node_pod_ids(node).is_empty()
            })
            .collect();
        for node in drained {
            self.cluster.remove_node(node);
        }
    }

    fn mark_node_ready(&mut self, node: NodeId) {
        if self.cluster.node_phase(node) == Some(NodePhase::Provisioning) {
            self.cluster.set_node_phase(node, NodePhase::Ready);
        }
    }

    fn mark_pod_running(&mut self, pod: PodId) {
        if self.cluster.pod(pod).map(|view| view.phase) == Some(PodPhase::Starting) {
            self.cluster.set_pod_phase(pod, PodPhase::Running);
        }
    }

    fn schedule_pending_pods(&mut self) {
        for (pod, _) in self.scheduler.schedule_pending(&mut self.cluster) {
            let startup = self.latencies.pod_startup(&mut self.lifecycle_rng);
            self.events
                .push(self.now + startup, SimEvent::PodRunning(pod));
        }
    }

    fn evaluate_services(&mut self) {
        let mut signals = std::mem::take(&mut self.signals);
        signals.clear();
        for index in 0..self.cluster.service_count() {
            let service = ServiceId(index_to_id(index));
            let evaluated = self.evaluate_service(service);
            if evaluated.oom {
                self.restart_running_pods(service);
            }
            signals.push(evaluated);
        }
        self.signals = signals;
    }

    fn evaluate_service(&self, service: ServiceId) -> ServiceSignals {
        let spec = self.cluster.service_spec(service);
        let capacity = ServiceCapacity {
            ready_pods: self.cluster.count_pods(service, PodPhase::Running),
            requests: spec.requests,
            limits: spec.limits,
        };
        let demand = self.workload.demand(service, self.now);
        self.performance.evaluate(&capacity, &demand)
    }

    fn restart_running_pods(&mut self, service: ServiceId) {
        for pod in self.cluster.pods_of(service) {
            let Some(view) = self.cluster.pod(pod) else {
                continue;
            };
            if view.phase != PodPhase::Running {
                continue;
            }
            self.cluster.record_restart(pod);
            self.cluster.set_pod_phase(pod, PodPhase::Starting);
            self.events.push(
                self.now + crash_loop_backoff(view.restarts),
                SimEvent::PodRunning(pod),
            );
        }
    }

    fn publish_observation(&mut self) {
        let mut observation = std::mem::take(&mut self.observation);
        observation.now = self.now;
        self.observe_services(&mut observation.services);
        self.observe_pools(&mut observation.pools);
        self.observe_nodes(&mut observation);
        self.observation = observation;
    }

    fn observe_services(&self, out: &mut Vec<ServiceObservation>) {
        let signals = self.metrics.published();
        out.clear();
        out.extend((0..self.cluster.service_count()).map(|index| {
            let service = ServiceId(index_to_id(index));
            let spec = self.cluster.service_spec(service);
            ServiceObservation {
                desired: self.desired[index],
                ready: self.cluster.count_pods(service, PodPhase::Running),
                pending: self.cluster.count_pods(service, PodPhase::Pending)
                    + self.cluster.count_pods(service, PodPhase::Starting),
                unschedulable: self.cluster.count_unschedulable(service),
                outdated: self.cluster.count_outdated(service),
                requests: spec.requests,
                limits: spec.limits,
                signals: signals[index],
            }
        }));
    }

    fn observe_pools(&self, out: &mut Vec<PoolObservation>) {
        out.clear();
        out.extend((0..self.cluster.pool_count()).map(|index| {
            let pool = PoolId(index_to_id(index));
            let spec = self.cluster.pool_spec(pool);
            PoolObservation {
                ready: self.cluster.count_nodes(pool, NodePhase::Ready),
                provisioning: self.cluster.count_nodes(pool, NodePhase::Provisioning),
                draining: self.cluster.count_nodes(pool, NodePhase::Draining),
                allocatable: spec.allocatable,
                min_nodes: spec.min_nodes,
                max_nodes: spec.max_nodes,
            }
        }));
    }

    fn observe_nodes(&self, observation: &mut Observation) {
        observation.nodes.clear();
        observation.node_pods.clear();
        for &node in self.cluster.node_ids() {
            let first_pod = observation.node_pods.len();
            let mut requested = Resources::ZERO;
            for view in self
                .cluster
                .node_pod_ids(node)
                .iter()
                .filter_map(|&pod| self.cluster.pod(pod))
                .filter(|view| view.phase != PodPhase::Terminating)
            {
                requested = requested.saturating_add(view.requests);
                observation.node_pods.push(PodPlacement {
                    service: view.service,
                    requests: view.requests,
                });
            }
            observation.nodes.push(NodeObservation {
                id: node,
                pool: self.cluster.node_pool(node),
                phase: self.cluster.node_phase(node).unwrap_or(NodePhase::Ready),
                allocatable: self.cluster.node_allocatable(node),
                requested,
                pods: first_pod..observation.node_pods.len(),
            });
        }
    }
}
