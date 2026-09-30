use std::fmt::Debug;

use crate::{ids::ServiceId, time::SimTime};

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ServiceDemand {
    pub request_rate: f64,
    pub cpu_millis: f64,
    pub memory_bytes: f64,
}

/// Demand must be a pure function of `(service, now)` so replays are deterministic.
pub trait WorkloadSource: Debug + Send {
    fn services(&self) -> usize;
    fn demand(&self, service: ServiceId, now: SimTime) -> ServiceDemand;
}
