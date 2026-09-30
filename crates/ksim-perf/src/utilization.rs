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
        if capacity.ready_pods == 0 {
            return unserved(demand);
        }
        let cpu_capacity =
            f64::from(capacity.ready_pods) * crate::quantity_as_f64(capacity.limits.cpu_millis);
        let cpu_usage_millis = demand.cpu_millis.min(cpu_capacity);
        let latency_ms = self.base_latency_ms * (demand.cpu_millis / cpu_capacity).max(1.0);
        let memory_per_pod = demand.memory_bytes / f64::from(capacity.ready_pods);
        ServiceSignals {
            cpu_usage_millis,
            memory_usage_bytes: demand.memory_bytes,
            served_rps: demand.request_rate,
            latency_p50_ms: latency_ms,
            latency_p95_ms: latency_ms,
            latency_p99_ms: latency_ms,
            error_rate: 0.0,
            throttled_fraction: (demand.cpu_millis - cpu_usage_millis)
                / demand.cpu_millis.max(f64::MIN_POSITIVE),
            oom: memory_per_pod > crate::quantity_as_f64(capacity.limits.memory_bytes),
        }
    }
}

fn unserved(demand: &ServiceDemand) -> ServiceSignals {
    let has_demand = demand.request_rate > 0.0 || demand.cpu_millis > 0.0;
    ServiceSignals {
        memory_usage_bytes: demand.memory_bytes,
        error_rate: if has_demand { 1.0 } else { 0.0 },
        ..ServiceSignals::default()
    }
}
