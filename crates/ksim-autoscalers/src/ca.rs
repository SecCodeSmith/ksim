use ksim_core::{observation::Observation, resources::Resources, time::SimDuration};

use crate::traits::{NodePlan, NodeProvisioner};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Expander {
    Random,
    LeastWaste,
}

/// Upstream Cluster Autoscaler defaults: scan interval 10 s, scale-down utilisation threshold
/// 0.5, scale-down-unneeded-time 10 min, scale-down-delay-after-add 10 min, at most 10 empty
/// nodes deleted per loop, expander `random`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClusterAutoscalerConfig {
    pub scan_interval: SimDuration,
    pub scale_down_utilization_threshold: f64,
    pub scale_down_unneeded_time: SimDuration,
    pub scale_down_delay_after_add: SimDuration,
    pub max_empty_bulk_delete: u32,
    pub expander: Expander,
    pub seed: u64,
}

impl Default for ClusterAutoscalerConfig {
    fn default() -> Self {
        todo!()
    }
}

/// Nodes of size `allocatable` needed to hold `pods`, packed first-fit decreasing by CPU then
/// memory (upstream binpacking estimator). Pods larger than a node are ignored.
#[must_use]
pub fn nodes_needed(pods: &[Resources], allocatable: Resources) -> u32 {
    let _ = (pods, allocatable);
    todo!()
}

/// Cluster Autoscaler; runs at most once per `scan_interval` (the first call always runs).
///
/// Scale-up: unschedulable pods (per service: `unschedulable` pods with the service's
/// requests) are first packed into the free capacity of `Provisioning` nodes; for each pool
/// with room below `max_nodes` (counting ready and provisioning nodes) the remaining pods that
/// fit one of its nodes form an option of `min(nodes_needed, room)` nodes. The expander picks
/// one option (`LeastWaste`: lowest mean CPU/memory idle fraction of the new nodes, ties to the
/// lowest pool id; `Random`: seeded uniform choice) and returns it as `pool_delta`.
///
/// Scale-down, only in loops without scale-up and at least `scale_down_delay_after_add` after
/// the last scale-up: `Ready` nodes with `max(cpu, memory)` requested/allocatable below the
/// threshold are unneeded since first seen so (and forgotten once they are not). A node
/// unneeded for `scale_down_unneeded_time` is removable when its pool stays at or above
/// `min_nodes` and its pods fit, first-fit decreasing, into the free capacity of the other
/// `Ready` nodes not removed in this loop. Empty removable nodes are drained in bulk (up to
/// `max_empty_bulk_delete`, ascending id); otherwise one non-empty removable node (lowest
/// utilisation, ties to the lowest id).
#[derive(Debug)]
pub struct ClusterAutoscaler {
    config: ClusterAutoscalerConfig,
}

impl ClusterAutoscaler {
    #[must_use]
    pub fn new(config: ClusterAutoscalerConfig) -> Self {
        Self { config }
    }
}

impl NodeProvisioner for ClusterAutoscaler {
    fn provision(&mut self, observation: &Observation) -> NodePlan {
        let _ = (observation, self.config);
        todo!()
    }
}
