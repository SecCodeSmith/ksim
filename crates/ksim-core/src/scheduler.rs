use std::fmt::Debug;

use crate::{
    cluster::{Cluster, NodePhase},
    ids::NodeId,
    ids::PodId,
    resources::Resources,
};

/// Scores a node from 0 to 100 given its requested resources including the incoming pod.
pub trait ScoringPolicy: Debug + Send {
    fn score(&self, requested: Resources, allocatable: Resources) -> u32;
}

/// kube-scheduler `NodeResourcesFit` `LeastAllocated`: mean of `(allocatable - requested) * 100 / allocatable`
/// over CPU and memory with weight 1, integer arithmetic.
#[derive(Debug, Clone, Copy, Default)]
pub struct LeastAllocated;

/// kube-scheduler `NodeResourcesFit` `MostAllocated`: mean of `requested * 100 / allocatable`.
#[derive(Debug, Clone, Copy, Default)]
pub struct MostAllocated;

impl ScoringPolicy for LeastAllocated {
    fn score(&self, requested: Resources, allocatable: Resources) -> u32 {
        mean_per_resource(requested, allocatable, |used, total| {
            total.saturating_sub(used) * 100 / total
        })
    }
}

impl ScoringPolicy for MostAllocated {
    fn score(&self, requested: Resources, allocatable: Resources) -> u32 {
        mean_per_resource(requested, allocatable, |used, total| {
            used.min(total) * 100 / total
        })
    }
}

fn mean_per_resource(
    requested: Resources,
    allocatable: Resources,
    score_one: impl Fn(u64, u64) -> u64,
) -> u32 {
    let score = |used: u64, total: u64| {
        if total == 0 {
            0
        } else {
            score_one(used, total)
        }
    };
    let sum = score(requested.cpu_millis, allocatable.cpu_millis)
        + score(requested.memory_bytes, allocatable.memory_bytes);
    u32::try_from(sum / 2).unwrap_or(u32::MAX)
}

#[derive(Debug)]
pub struct Scheduler {
    policy: Box<dyn ScoringPolicy>,
}

impl Scheduler {
    #[must_use]
    pub fn new(policy: Box<dyn ScoringPolicy>) -> Self {
        Self { policy }
    }

    /// Highest-scoring `Ready` node that fits `requests`; ties go to the lowest node id.
    #[must_use]
    pub fn select_node(&self, cluster: &Cluster, requests: Resources) -> Option<NodeId> {
        cluster
            .nodes()
            .into_iter()
            .filter(|&node| cluster.node_phase(node) == Some(NodePhase::Ready))
            .filter_map(|node| {
                self.score_if_fits(cluster, node, requests)
                    .map(|s| (s, node))
            })
            .fold(None, |best: Option<(u32, NodeId)>, candidate| match best {
                Some(current) if current.0 >= candidate.0 => Some(current),
                _ => Some(candidate),
            })
            .map(|(_, node)| node)
    }

    /// Binds every pending pod that fits, in ascending pod id order, and flags the rest
    /// `unschedulable` (cleared once bound). Returns the bindings made.
    pub fn schedule_pending(&self, cluster: &mut Cluster) -> Vec<(PodId, NodeId)> {
        let mut bindings = Vec::new();
        for pod in cluster.pending_pods() {
            let requests = cluster
                .pod(pod)
                .map(|view| view.requests)
                .unwrap_or_default();
            if let Some(node) = self.select_node(cluster, requests) {
                cluster.bind_pod(pod, node);
                cluster.set_unschedulable(pod, false);
                bindings.push((pod, node));
            } else {
                cluster.set_unschedulable(pod, true);
            }
        }
        bindings
    }

    fn score_if_fits(&self, cluster: &Cluster, node: NodeId, requests: Resources) -> Option<u32> {
        let allocatable = cluster.node_allocatable(node);
        let requested = cluster.node_requested(node).saturating_add(requests);
        requested
            .fits_in(allocatable)
            .then(|| self.policy.score(requested, allocatable))
    }
}
