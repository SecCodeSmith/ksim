use std::sync::Arc;

use ksim_core::{
    ids::ServiceId,
    time::{SimDuration, SimTime},
    workload::{ServiceDemand, WorkloadSource},
};

use super::format::{TraceError, TraceFile};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Interpolation {
    Step,
    Linear,
}

/// How simulated time maps onto the trace: trace time `= start + now * compress`, sampled at
/// tick index `trace time / tick`. Past the last tick the trace wraps around when `wrap`,
/// otherwise holds the last value. `scale` multiplies request rate and CPU (memory is a
/// footprint and is not scaled). `services` selects and orders services by name; `None`
/// keeps every service in file order.
#[derive(Debug, Clone, PartialEq)]
pub struct TraceOptions {
    pub services: Option<Vec<String>>,
    pub start: SimDuration,
    pub compress: f64,
    pub scale: f64,
    pub interpolation: Interpolation,
    pub wrap: bool,
}

impl Default for TraceOptions {
    fn default() -> Self {
        todo!()
    }
}

/// Replays a [`TraceFile`] as a pure function of time.
#[derive(Debug)]
pub struct TraceWorkload {
    file: Arc<TraceFile>,
    indices: Vec<usize>,
    options: TraceOptions,
}

impl TraceWorkload {
    /// # Errors
    /// [`TraceError::UnknownService`] for a selected name missing from the file, and
    /// [`TraceError::Malformed`] for a non-positive or non-finite `compress` or `scale`.
    pub fn new(file: Arc<TraceFile>, options: TraceOptions) -> Result<Self, TraceError> {
        let _ = (file, options);
        todo!()
    }
}

impl WorkloadSource for TraceWorkload {
    fn services(&self) -> usize {
        todo!()
    }

    fn demand(&self, service: ServiceId, now: SimTime) -> ServiceDemand {
        let _ = (service, now, &self.file, &self.indices, &self.options);
        todo!()
    }
}
