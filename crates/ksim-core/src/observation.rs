use crate::{performance::ServiceSignals, resources::Resources, time::SimDuration, time::SimTime};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ServiceObservation {
    pub desired: u32,
    pub ready: u32,
    /// Not-ready pods of the service (`Pending` plus `Starting`).
    pub pending: u32,
    pub unschedulable: u32,
    pub requests: Resources,
    pub signals: ServiceSignals,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PoolObservation {
    pub ready: u32,
    pub provisioning: u32,
}

/// Counts are current (API server view); `signals` come from the metrics pipeline and lag
/// by `ScrapeConfig::lag_scrapes` scrapes.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Observation {
    pub now: SimTime,
    pub services: Vec<ServiceObservation>,
    pub pools: Vec<PoolObservation>,
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
