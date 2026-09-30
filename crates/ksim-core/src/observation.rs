use std::ops::Range;

use crate::{
    cluster::NodePhase,
    ids::{NodeId, PoolId, ServiceId},
    performance::ServiceSignals,
    resources::Resources,
    time::{SimDuration, SimTime},
};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ServiceObservation {
    pub desired: u32,
    pub ready: u32,
    /// Not-ready pods of the service (`Pending` plus `Starting`).
    pub pending: u32,
    pub unschedulable: u32,
    /// Non-terminating bound pods whose requests differ from `requests`.
    pub outdated: u32,
    pub requests: Resources,
    pub limits: Resources,
    pub signals: ServiceSignals,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PoolObservation {
    pub ready: u32,
    pub provisioning: u32,
    pub draining: u32,
    pub allocatable: Resources,
    pub min_nodes: u32,
    pub max_nodes: u32,
}

/// A node that exists (not removed); its bound non-terminating pods are
/// `Observation::node_pods[pods]`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeObservation {
    pub id: NodeId,
    pub pool: PoolId,
    pub phase: NodePhase,
    pub allocatable: Resources,
    pub requested: Resources,
    pub pods: Range<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PodPlacement {
    pub service: ServiceId,
    pub requests: Resources,
}

/// Counts are current (API server view); `signals` come from the metrics pipeline and lag
/// by `ScrapeConfig::lag_scrapes` scrapes.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Observation {
    pub now: SimTime,
    pub services: Vec<ServiceObservation>,
    pub pools: Vec<PoolObservation>,
    /// Existing nodes in ascending id order.
    pub nodes: Vec<NodeObservation>,
    pub node_pods: Vec<PodPlacement>,
}

/// Metrics Server parity: 15 s resolution, consumers see the previous scrape.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScrapeConfig {
    pub resolution: SimDuration,
    pub lag_scrapes: u32,
}

impl Default for ScrapeConfig {
    fn default() -> Self {
        Self {
            resolution: SimDuration::from_secs(15),
            lag_scrapes: 1,
        }
    }
}
