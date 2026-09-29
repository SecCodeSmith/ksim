use crate::resources::Resources;

/// Per-tick autoscaler decisions, indexed by service or pool id; shorter vectors mean no change.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Actions {
    pub replicas: Vec<Option<u32>>,
    pub requests: Vec<Option<Resources>>,
    pub pool_delta: Vec<i32>,
}

impl Actions {
    #[must_use]
    pub fn none() -> Self {
        Self::default()
    }
}
