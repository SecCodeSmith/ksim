use ksim_core::{
    performance::{PerformanceModel, ServiceCapacity, ServiceSignals},
    workload::ServiceDemand,
};

/// Utilisation-driven services (Alibaba 2018, Borg): CPU usage is capped at
/// `ready_pods * limits.cpu_millis`; the excess fraction is CFS-throttled and inflates every
/// latency quantile of `base_latency_ms` by `demand / capacity`. Per-pod memory above
/// `limits.memory_bytes` sets `oom`. No ready pods with non-zero demand: `error_rate` 1.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UtilizationModel {
    pub base_latency_ms: f64,
}

impl PerformanceModel for UtilizationModel {
    fn evaluate(&self, capacity: &ServiceCapacity, demand: &ServiceDemand) -> ServiceSignals {
        let _ = (capacity, demand);
        todo!()
    }
}
