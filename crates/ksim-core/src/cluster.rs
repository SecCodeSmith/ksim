use std::{cmp::Reverse, collections::BTreeSet};

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

const NO_SLOT: u32 = u32::MAX;

#[derive(Debug, Clone)]
struct NodeRecord {
    pool: PoolId,
    phase: NodePhase,
    requested: Resources,
    bound: Vec<PodId>,
}

#[derive(Debug, Clone, Copy)]
struct PodRecord {
    id: PodId,
    view: PodView,
}

#[derive(Debug, Clone, Copy, Default)]
struct ServiceCounters {
    by_phase: [u32; 4],
    unschedulable: u32,
}

impl NodePhase {
    const fn slot(self) -> usize {
        match self {
            Self::Provisioning => 0,
            Self::Ready => 1,
            Self::Draining => 2,
        }
    }
}

impl PodPhase {
    const fn slot(self) -> usize {
        match self {
            Self::Pending => 0,
            Self::Starting => 1,
            Self::Running => 2,
            Self::Terminating => 3,
        }
    }
}

fn is_pending(view: &PodView) -> bool {
    view.node.is_none() && view.phase == PodPhase::Pending
}

/// Cluster state with stable ids; removed pods and nodes leave no trace except their unused ids.
///
/// Pods live in dense storage behind an id-to-slot table, and per-service pod lists, per-node
/// bound lists and requested totals, phase counters and the pending set are kept incrementally,
/// so every query costs in proportion to live objects, never to objects removed earlier.
#[derive(Debug, Default)]
pub struct Cluster {
    pools: Vec<NodePoolSpec>,
    services: Vec<ServiceSpec>,
    nodes: Vec<Option<NodeRecord>>,
    live_nodes: Vec<NodeId>,
    pool_node_counts: Vec<[u32; 3]>,
    pods: Vec<PodRecord>,
    pod_slots: Vec<u32>,
    service_pods: Vec<Vec<PodId>>,
    service_counters: Vec<ServiceCounters>,
    pending: BTreeSet<PodId>,
}

fn id_from_index(index: usize) -> u32 {
    u32::try_from(index).expect("id space exhausted")
}

fn remove_sorted(ids: &mut Vec<PodId>, pod: PodId) {
    if let Ok(position) = ids.binary_search(&pod) {
        ids.remove(position);
    }
}

fn insert_sorted(ids: &mut Vec<PodId>, pod: PodId) {
    if let Err(position) = ids.binary_search(&pod) {
        ids.insert(position, pod);
    }
}

impl Cluster {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_pool(&mut self, spec: NodePoolSpec) -> PoolId {
        self.pools.push(spec);
        self.pool_node_counts.push([0; 3]);
        PoolId(id_from_index(self.pools.len() - 1))
    }

    pub fn add_service(&mut self, spec: ServiceSpec) -> ServiceId {
        self.services.push(spec);
        self.service_pods.push(Vec::new());
        self.service_counters.push(ServiceCounters::default());
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
        let node = NodeId(id_from_index(self.nodes.len()));
        self.nodes.push(Some(NodeRecord {
            pool,
            phase,
            requested: Resources::ZERO,
            bound: Vec::new(),
        }));
        self.live_nodes.push(node);
        self.pool_node_counts[pool.index()][phase.slot()] += 1;
        node
    }

    pub fn set_node_phase(&mut self, node: NodeId, phase: NodePhase) {
        let Some(record) = self.nodes[node.index()].as_mut() else {
            return;
        };
        let counts = &mut self.pool_node_counts[record.pool.index()];
        counts[record.phase.slot()] -= 1;
        counts[phase.slot()] += 1;
        record.phase = phase;
    }

    pub fn remove_node(&mut self, node: NodeId) {
        let Some(record) = self.nodes[node.index()].take() else {
            return;
        };
        self.pool_node_counts[record.pool.index()][record.phase.slot()] -= 1;
        if let Ok(position) = self.live_nodes.binary_search(&node) {
            self.live_nodes.remove(position);
        }
    }

    /// `None` once the node has been removed.
    #[must_use]
    pub fn node_phase(&self, node: NodeId) -> Option<NodePhase> {
        self.nodes[node.index()].as_ref().map(|record| record.phase)
    }

    /// # Panics
    /// If the node has been removed; check [`Cluster::node_phase`] first for tombstoned ids.
    #[must_use]
    pub fn node_pool(&self, node: NodeId) -> PoolId {
        self.node_record(node).pool
    }

    /// # Panics
    /// If the node has been removed; check [`Cluster::node_phase`] first for tombstoned ids.
    #[must_use]
    pub fn node_allocatable(&self, node: NodeId) -> Resources {
        self.pool_spec(self.node_pool(node)).allocatable
    }

    /// Sum of requests of pods bound to the node that are not yet removed.
    #[must_use]
    pub fn node_requested(&self, node: NodeId) -> Resources {
        self.nodes[node.index()]
            .as_ref()
            .map_or(Resources::ZERO, |record| record.requested)
    }

    /// Pods bound to the node that are not yet removed, in ascending id order.
    #[must_use]
    pub fn node_pod_ids(&self, node: NodeId) -> &[PodId] {
        self.nodes[node.index()]
            .as_ref()
            .map_or(&[], |record| record.bound.as_slice())
    }

    /// Existing (non-removed) nodes in ascending id order.
    #[must_use]
    pub fn nodes(&self) -> Vec<NodeId> {
        self.live_nodes.clone()
    }

    /// Same as [`Cluster::nodes`] without allocating.
    #[must_use]
    pub fn node_ids(&self) -> &[NodeId] {
        &self.live_nodes
    }

    #[must_use]
    pub fn count_nodes(&self, pool: PoolId, phase: NodePhase) -> u32 {
        self.pool_node_counts[pool.index()][phase.slot()]
    }

    /// Creates an unbound `Pending` pod with the service's current requests.
    pub fn create_pod(&mut self, service: ServiceId, now: SimTime) -> PodId {
        let id = PodId(id_from_index(self.pod_slots.len()));
        let view = PodView {
            service,
            node: None,
            phase: PodPhase::Pending,
            requests: self.service_spec(service).requests,
            created_at: now,
            unschedulable: false,
            restarts: 0,
        };
        self.pod_slots.push(id_from_index(self.pods.len()));
        self.pods.push(PodRecord { id, view });
        self.service_pods[service.index()].push(id);
        self.service_counters[service.index()].by_phase[PodPhase::Pending.slot()] += 1;
        self.pending.insert(id);
        id
    }

    /// Binds a pending pod to a node and moves it to `Starting`.
    pub fn bind_pod(&mut self, pod: PodId, node: NodeId) {
        let Some(before) = self.pod(pod) else {
            return;
        };
        self.detach_from_node(pod, &before);
        self.update_pod(pod, |view| {
            view.node = Some(node);
            view.phase = PodPhase::Starting;
        });
        if let Some(record) = self.nodes[node.index()].as_mut() {
            insert_sorted(&mut record.bound, pod);
            record.requested = record.requested.saturating_add(before.requests);
        }
    }

    pub fn set_pod_phase(&mut self, pod: PodId, phase: PodPhase) {
        self.update_pod(pod, |view| view.phase = phase);
    }

    pub fn set_unschedulable(&mut self, pod: PodId, unschedulable: bool) {
        self.update_pod(pod, |view| view.unschedulable = unschedulable);
    }

    pub fn record_restart(&mut self, pod: PodId) {
        self.update_pod(pod, |view| view.restarts += 1);
    }

    pub fn remove_pod(&mut self, pod: PodId) {
        let Some(slot) = self.slot_of(pod) else {
            return;
        };
        let view = self.pods[slot].view;
        self.detach_from_node(pod, &view);
        self.forget_counters(pod, &view);
        remove_sorted(&mut self.service_pods[view.service.index()], pod);
        self.pod_slots[pod.index()] = NO_SLOT;
        self.pods.swap_remove(slot);
        if let Some(moved) = self.pods.get(slot) {
            self.pod_slots[moved.id.index()] = id_from_index(slot);
        }
    }

    /// `None` once the pod has been removed.
    #[must_use]
    pub fn pod(&self, pod: PodId) -> Option<PodView> {
        self.slot_of(pod).map(|slot| self.pods[slot].view)
    }

    /// Existing pods of the service in ascending id order.
    #[must_use]
    pub fn pods_of(&self, service: ServiceId) -> Vec<PodId> {
        self.service_pod_ids(service).to_vec()
    }

    /// Same as [`Cluster::pods_of`] without allocating.
    #[must_use]
    pub fn service_pod_ids(&self, service: ServiceId) -> &[PodId] {
        &self.service_pods[service.index()]
    }

    /// Existing unbound `Pending` pods in ascending id order.
    #[must_use]
    pub fn pending_pods(&self) -> Vec<PodId> {
        self.pending.iter().copied().collect()
    }

    #[must_use]
    pub fn count_pods(&self, service: ServiceId, phase: PodPhase) -> u32 {
        self.service_counters[service.index()].by_phase[phase.slot()]
    }

    #[must_use]
    pub fn count_unschedulable(&self, service: ServiceId) -> u32 {
        self.service_counters[service.index()].unschedulable
    }

    /// Non-terminating bound pods of the service whose requests differ from the service's current ones.
    #[must_use]
    pub fn count_outdated(&self, service: ServiceId) -> u32 {
        let count = self
            .service_pod_ids(service)
            .iter()
            .filter_map(|&pod| self.pod(pod))
            .filter(|view| view.node.is_some() && self.is_outdated(view))
            .count();
        id_from_index(count)
    }

    /// Up to `count` non-terminating pods of the service whose requests differ from the
    /// service's current ones, oldest first; ties go to the lower id.
    #[must_use]
    pub fn pods_to_evict(&self, service: ServiceId, count: usize) -> Vec<PodId> {
        let mut outdated: Vec<(SimTime, PodId)> = self
            .service_pod_ids(service)
            .iter()
            .filter_map(|&pod| self.pod(pod).map(|view| (pod, view)))
            .filter(|(_, view)| self.is_outdated(view))
            .map(|(pod, view)| (view.created_at, pod))
            .collect();
        outdated.sort_unstable();
        outdated
            .into_iter()
            .take(count)
            .map(|(_, pod)| pod)
            .collect()
    }

    /// Victims for a scale-down of `count` pods, excluding `Terminating` pods.
    ///
    /// Upstream parity (`ReplicaSet` `ActivePodsWithRanks`): unbound pods first, then not-ready
    /// (`Pending`/`Starting`) pods, then `Running` pods newest first; ties go to the higher id.
    #[must_use]
    pub fn pods_to_delete(&self, service: ServiceId, count: usize) -> Vec<PodId> {
        let mut candidates: Vec<(PodId, PodView)> = self
            .service_pod_ids(service)
            .iter()
            .filter_map(|&pod| self.pod(pod).map(|view| (pod, view)))
            .filter(|(_, view)| view.phase != PodPhase::Terminating)
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

    fn node_record(&self, node: NodeId) -> &NodeRecord {
        self.nodes[node.index()]
            .as_ref()
            .expect("node has been removed")
    }

    fn slot_of(&self, pod: PodId) -> Option<usize> {
        match self.pod_slots.get(pod.index()) {
            Some(&slot) if slot != NO_SLOT => Some(slot as usize),
            _ => None,
        }
    }

    fn is_outdated(&self, view: &PodView) -> bool {
        view.phase != PodPhase::Terminating
            && view.requests != self.service_spec(view.service).requests
    }

    fn update_pod(&mut self, pod: PodId, change: impl FnOnce(&mut PodView)) {
        let Some(slot) = self.slot_of(pod) else {
            return;
        };
        let before = self.pods[slot].view;
        change(&mut self.pods[slot].view);
        let after = self.pods[slot].view;
        self.forget_counters(pod, &before);
        self.record_counters(pod, &after);
    }

    fn forget_counters(&mut self, pod: PodId, view: &PodView) {
        let counters = &mut self.service_counters[view.service.index()];
        counters.by_phase[view.phase.slot()] -= 1;
        counters.unschedulable -= u32::from(view.unschedulable);
        if is_pending(view) {
            self.pending.remove(&pod);
        }
    }

    fn record_counters(&mut self, pod: PodId, view: &PodView) {
        let counters = &mut self.service_counters[view.service.index()];
        counters.by_phase[view.phase.slot()] += 1;
        counters.unschedulable += u32::from(view.unschedulable);
        if is_pending(view) {
            self.pending.insert(pod);
        }
    }

    fn detach_from_node(&mut self, pod: PodId, view: &PodView) {
        let Some(record) = view.node.and_then(|node| self.nodes[node.index()].as_mut()) else {
            return;
        };
        remove_sorted(&mut record.bound, pod);
        record.requested = record
            .requested
            .checked_sub(view.requests)
            .unwrap_or(Resources::ZERO);
    }
}

fn deletion_rank(view: &PodView) -> u8 {
    match (view.node, view.phase) {
        (None, _) => 0,
        (Some(_), PodPhase::Running) => 2,
        (Some(_), _) => 1,
    }
}
