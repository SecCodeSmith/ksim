use std::cmp::Reverse;

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

#[derive(Debug, Clone, Copy)]
struct NodeRecord {
    pool: PoolId,
    phase: NodePhase,
}

/// Cluster state with stable ids; removed pods and nodes leave tombstones.
#[derive(Debug, Default)]
pub struct Cluster {
    pools: Vec<NodePoolSpec>,
    services: Vec<ServiceSpec>,
    nodes: Vec<Option<NodeRecord>>,
    pods: Vec<Option<PodView>>,
}

fn id_from_index(index: usize) -> u32 {
    u32::try_from(index).expect("id space exhausted")
}

impl Cluster {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_pool(&mut self, spec: NodePoolSpec) -> PoolId {
        self.pools.push(spec);
        PoolId(id_from_index(self.pools.len() - 1))
    }

    pub fn add_service(&mut self, spec: ServiceSpec) -> ServiceId {
        self.services.push(spec);
        ServiceId(id_from_index(self.services.len() - 1))
    }

    #[must_use]
    pub fn pool_spec(&self, pool: PoolId) -> &NodePoolSpec {
        &self.pools[pool.index()]
    }

    #[must_use]
    pub fn service_spec(&self, service: ServiceId) -> &ServiceSpec {
        &self.services[service.index()]
    }

    #[must_use]
    pub fn pool_count(&self) -> usize {
        self.pools.len()
    }

    #[must_use]
    pub fn service_count(&self) -> usize {
        self.services.len()
    }

    /// Requests used for pods created from now on; existing pods keep theirs.
    pub fn set_service_requests(&mut self, service: ServiceId, requests: Resources) {
        self.services[service.index()].requests = requests;
    }

    pub fn add_node(&mut self, pool: PoolId, phase: NodePhase) -> NodeId {
        self.nodes.push(Some(NodeRecord { pool, phase }));
        NodeId(id_from_index(self.nodes.len() - 1))
    }

    pub fn set_node_phase(&mut self, node: NodeId, phase: NodePhase) {
        if let Some(record) = self.nodes[node.index()].as_mut() {
            record.phase = phase;
        }
    }

    pub fn remove_node(&mut self, node: NodeId) {
        self.nodes[node.index()] = None;
    }

    /// `None` once the node has been removed.
    #[must_use]
    pub fn node_phase(&self, node: NodeId) -> Option<NodePhase> {
        self.nodes[node.index()].map(|record| record.phase)
    }

    #[must_use]
    pub fn node_pool(&self, node: NodeId) -> PoolId {
        self.node_record(node).pool
    }

    #[must_use]
    pub fn node_allocatable(&self, node: NodeId) -> Resources {
        self.pool_spec(self.node_pool(node)).allocatable
    }

    /// Sum of requests of pods bound to the node that are not yet removed.
    #[must_use]
    pub fn node_requested(&self, node: NodeId) -> Resources {
        self.live_pods()
            .filter(|(_, view)| view.node == Some(node))
            .fold(Resources::ZERO, |sum, (_, view)| {
                sum.saturating_add(view.requests)
            })
    }

    /// Existing (non-removed) nodes in ascending id order.
    #[must_use]
    pub fn nodes(&self) -> Vec<NodeId> {
        self.nodes
            .iter()
            .enumerate()
            .filter(|(_, record)| record.is_some())
            .map(|(index, _)| NodeId(id_from_index(index)))
            .collect()
    }

    #[must_use]
    pub fn count_nodes(&self, pool: PoolId, phase: NodePhase) -> u32 {
        let count = self
            .nodes
            .iter()
            .flatten()
            .filter(|record| record.pool == pool && record.phase == phase)
            .count();
        id_from_index(count)
    }

    /// Creates an unbound `Pending` pod with the service's current requests.
    pub fn create_pod(&mut self, service: ServiceId, now: SimTime) -> PodId {
        self.pods.push(Some(PodView {
            service,
            node: None,
            phase: PodPhase::Pending,
            requests: self.service_spec(service).requests,
            created_at: now,
            unschedulable: false,
            restarts: 0,
        }));
        PodId(id_from_index(self.pods.len() - 1))
    }

    /// Binds a pending pod to a node and moves it to `Starting`.
    pub fn bind_pod(&mut self, pod: PodId, node: NodeId) {
        if let Some(view) = self.pods[pod.index()].as_mut() {
            view.node = Some(node);
            view.phase = PodPhase::Starting;
        }
    }

    pub fn set_pod_phase(&mut self, pod: PodId, phase: PodPhase) {
        if let Some(view) = self.pods[pod.index()].as_mut() {
            view.phase = phase;
        }
    }

    pub fn set_unschedulable(&mut self, pod: PodId, unschedulable: bool) {
        if let Some(view) = self.pods[pod.index()].as_mut() {
            view.unschedulable = unschedulable;
        }
    }

    pub fn record_restart(&mut self, pod: PodId) {
        if let Some(view) = self.pods[pod.index()].as_mut() {
            view.restarts += 1;
        }
    }

    pub fn remove_pod(&mut self, pod: PodId) {
        self.pods[pod.index()] = None;
    }

    /// `None` once the pod has been removed.
    #[must_use]
    pub fn pod(&self, pod: PodId) -> Option<PodView> {
        self.pods[pod.index()]
    }

    /// Existing pods of the service in ascending id order.
    #[must_use]
    pub fn pods_of(&self, service: ServiceId) -> Vec<PodId> {
        self.live_pods()
            .filter(|(_, view)| view.service == service)
            .map(|(id, _)| id)
            .collect()
    }

    /// Existing unbound `Pending` pods in ascending id order.
    #[must_use]
    pub fn pending_pods(&self) -> Vec<PodId> {
        self.live_pods()
            .filter(|(_, view)| view.node.is_none() && view.phase == PodPhase::Pending)
            .map(|(id, _)| id)
            .collect()
    }

    #[must_use]
    pub fn count_pods(&self, service: ServiceId, phase: PodPhase) -> u32 {
        let count = self
            .live_pods()
            .filter(|(_, view)| view.service == service && view.phase == phase)
            .count();
        id_from_index(count)
    }

    /// Victims for a scale-down of `count` pods, excluding `Terminating` pods.
    ///
    /// Upstream parity (`ReplicaSet` `ActivePodsWithRanks`): unbound pods first, then not-ready
    /// (`Pending`/`Starting`) pods, then `Running` pods newest first; ties go to the higher id.
    #[must_use]
    pub fn pods_to_delete(&self, service: ServiceId, count: usize) -> Vec<PodId> {
        let mut candidates: Vec<(PodId, PodView)> = self
            .live_pods()
            .filter(|(_, view)| view.service == service && view.phase != PodPhase::Terminating)
            .collect();
        candidates.sort_by_key(|&(id, view)| {
            (deletion_rank(&view), Reverse(view.created_at), Reverse(id))
        });
        candidates
            .into_iter()
            .take(count)
            .map(|(id, _)| id)
            .collect()
    }

    fn node_record(&self, node: NodeId) -> NodeRecord {
        self.nodes[node.index()].expect("node has been removed")
    }

    fn live_pods(&self) -> impl Iterator<Item = (PodId, PodView)> + '_ {
        self.pods
            .iter()
            .enumerate()
            .filter_map(|(index, view)| view.map(|view| (PodId(id_from_index(index)), view)))
    }
}

fn deletion_rank(view: &PodView) -> u8 {
    match (view.node, view.phase) {
        (None, _) => 0,
        (Some(_), PodPhase::Running) => 2,
        (Some(_), _) => 1,
    }
}
