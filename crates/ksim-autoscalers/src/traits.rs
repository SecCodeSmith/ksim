use std::fmt::Debug;

use ksim_core::{ids::NodeId, observation::Observation, resources::Resources};

/// Horizontal scaling: desired replicas per service (`None` = leave unchanged).
pub trait ReplicaRecommender: Debug + Send {
    fn recommend_replicas(&mut self, observation: &Observation) -> Vec<Option<u32>>;
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ResourcePlan {
    pub requests: Vec<Option<Resources>>,
    pub evict: Vec<u32>,
}

/// Vertical scaling: new per-pod requests and how many outdated pods to evict.
pub trait ResourceRecommender: Debug + Send {
    fn recommend_resources(&mut self, observation: &Observation) -> ResourcePlan;
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct NodePlan {
    pub pool_delta: Vec<i32>,
    pub drain: Vec<NodeId>,
}

/// Cluster scaling: nodes to add per pool and nodes to drain.
pub trait NodeProvisioner: Debug + Send {
    fn provision(&mut self, observation: &Observation) -> NodePlan;
}
