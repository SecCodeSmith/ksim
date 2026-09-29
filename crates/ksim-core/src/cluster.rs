use crate::{
    ids::{NodeId, PodId, PoolId, ServiceId},
    resources::Resources,
    time::SimTime,
};

#[derive(Debug, Clone, PartialEq)]
pub struct NodePoolSpec {
    pub name: String,
    pub allocatable: Resources,
    pub min_nodes: u32,
    pub max_nodes: u32,
    pub initial_nodes: u32,
    pub price_per_hour: f64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceSpec {
    pub name: String,
    pub requests: Resources,
    pub limits: Resources,
    pub initial_replicas: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NodePhase {
    Provisioning,
    Ready,
    Draining,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PodPhase {
    Pending,
    Starting,
    Running,
    Terminating,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PodView {
    pub service: ServiceId,
    pub node: Option<NodeId>,
    pub phase: PodPhase,
    pub requests: Resources,
    pub created_at: SimTime,
    pub unschedulable: bool,
    pub restarts: u32,
}

/// Struct-of-arrays cluster state; removed pods and nodes leave tombstones so ids stay stable.
#[derive(Debug, Default)]
pub struct Cluster {
    pools: Vec<NodePoolSpec>,
    services: Vec<ServiceSpec>,
}

impl Cluster {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_pool(&mut self, spec: NodePoolSpec) -> PoolId {
        let _ = (spec, &self.pools);
        todo!()
    }

    pub fn add_service(&mut self, spec: ServiceSpec) -> ServiceId {
        let _ = (spec, &self.services);
        todo!()
    }

    #[must_use]
    pub fn pool_spec(&self, pool: PoolId) -> &NodePoolSpec {
        let _ = pool;
        todo!()
    }

    #[must_use]
    pub fn service_spec(&self, service: ServiceId) -> &ServiceSpec {
        let _ = service;
        todo!()
    }

    #[must_use]
    pub fn pool_count(&self) -> usize {
        todo!()
    }

    #[must_use]
    pub fn service_count(&self) -> usize {
        todo!()
    }

    /// Requests used for pods created from now on; existing pods keep theirs.
    pub fn set_service_requests(&mut self, service: ServiceId, requests: Resources) {
        let _ = (service, requests);
        todo!()
    }

    pub fn add_node(&mut self, pool: PoolId, phase: NodePhase) -> NodeId {
        let _ = (pool, phase);
        todo!()
    }

    pub fn set_node_phase(&mut self, node: NodeId, phase: NodePhase) {
        let _ = (node, phase);
        todo!()
    }

    pub fn remove_node(&mut self, node: NodeId) {
        let _ = node;
        todo!()
    }

    /// `None` once the node has been removed.
    #[must_use]
    pub fn node_phase(&self, node: NodeId) -> Option<NodePhase> {
        let _ = node;
        todo!()
    }

    #[must_use]
    pub fn node_pool(&self, node: NodeId) -> PoolId {
        let _ = node;
        todo!()
    }

    #[must_use]
    pub fn node_allocatable(&self, node: NodeId) -> Resources {
        let _ = node;
        todo!()
    }

    /// Sum of requests of pods bound to the node that are not yet removed.
    #[must_use]
    pub fn node_requested(&self, node: NodeId) -> Resources {
        let _ = node;
        todo!()
    }

    /// Existing (non-removed) nodes in ascending id order.
    #[must_use]
    pub fn nodes(&self) -> Vec<NodeId> {
        todo!()
    }

    #[must_use]
    pub fn count_nodes(&self, pool: PoolId, phase: NodePhase) -> u32 {
        let _ = (pool, phase);
        todo!()
    }

    /// Creates an unbound `Pending` pod with the service's current requests.
    pub fn create_pod(&mut self, service: ServiceId, now: SimTime) -> PodId {
        let _ = (service, now);
        todo!()
    }

    /// Binds a pending pod to a node and moves it to `Starting`.
    pub fn bind_pod(&mut self, pod: PodId, node: NodeId) {
        let _ = (pod, node);
        todo!()
    }

    pub fn set_pod_phase(&mut self, pod: PodId, phase: PodPhase) {
        let _ = (pod, phase);
        todo!()
    }

    pub fn set_unschedulable(&mut self, pod: PodId, unschedulable: bool) {
        let _ = (pod, unschedulable);
        todo!()
    }

    pub fn record_restart(&mut self, pod: PodId) {
        let _ = pod;
        todo!()
    }

    pub fn remove_pod(&mut self, pod: PodId) {
        let _ = pod;
        todo!()
    }

    /// `None` once the pod has been removed.
    #[must_use]
    pub fn pod(&self, pod: PodId) -> Option<PodView> {
        let _ = pod;
        todo!()
    }

    /// Existing pods of the service in ascending id order.
    #[must_use]
    pub fn pods_of(&self, service: ServiceId) -> Vec<PodId> {
        let _ = service;
        todo!()
    }

    /// Existing unbound `Pending` pods in ascending id order.
    #[must_use]
    pub fn pending_pods(&self) -> Vec<PodId> {
        todo!()
    }

    #[must_use]
    pub fn count_pods(&self, service: ServiceId, phase: PodPhase) -> u32 {
        let _ = (service, phase);
        todo!()
    }

    /// Victims for a scale-down of `count` pods, excluding `Terminating` pods.
    ///
    /// Upstream parity (`ReplicaSet` `ActivePodsWithRanks`): unbound pods first, then not-ready
    /// (`Pending`/`Starting`) pods, then `Running` pods newest first; ties go to the higher id.
    #[must_use]
    pub fn pods_to_delete(&self, service: ServiceId, count: usize) -> Vec<PodId> {
        let _ = (service, count);
        todo!()
    }
}
