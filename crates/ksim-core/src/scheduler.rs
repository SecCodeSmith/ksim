use std::fmt::Debug;

use crate::{cluster::Cluster, ids::NodeId, ids::PodId, resources::Resources};

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
        let _ = (requested, allocatable);
        todo!()
    }
}

impl ScoringPolicy for MostAllocated {
    fn score(&self, requested: Resources, allocatable: Resources) -> u32 {
        let _ = (requested, allocatable);
        todo!()
    }
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
        let _ = (cluster, requests, &self.policy);
        todo!()
    }

    /// Binds every pending pod that fits, in ascending pod id order, and flags the rest
    /// `unschedulable` (cleared once bound). Returns the bindings made.
    pub fn schedule_pending(&self, cluster: &mut Cluster) -> Vec<(PodId, NodeId)> {
        let _ = cluster;
        todo!()
    }
}
