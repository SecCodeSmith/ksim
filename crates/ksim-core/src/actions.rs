use crate::{ids::NodeId, resources::Resources};

/// Per-tick autoscaler decisions, indexed by service or pool id; shorter vectors mean no change.
///
/// `drain` cordons nodes and evicts their pods; the node is removed once no pod is bound,
/// and drains that would take a pool below `min_nodes` are ignored. `evict` asks for up to
/// `n` outdated pods of a service (requests differing from the service's current requests)
/// to be evicted, oldest first; replacements are created with the current requests.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Actions {
    pub replicas: Vec<Option<u32>>,
    pub requests: Vec<Option<Resources>>,
    pub pool_delta: Vec<i32>,
    pub drain: Vec<NodeId>,
    pub evict: Vec<u32>,
}

impl Actions {
    #[must_use]
    pub fn none() -> Self {
        Self::default()
    }
}
