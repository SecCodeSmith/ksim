use ksim_core::{
    ids::ServiceId,
    performance::{PerformanceModel, ServiceCapacity, ServiceSignals},
    time::SimTime,
    workload::{ServiceDemand, WorkloadSource},
};

/// `WorkloadSource` that returns fixed demands per service.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct FixedDemand {
    pub demands: Vec<ServiceDemand>,
}

impl FixedDemand {
    #[allow(dead_code)]
    pub fn new(demands: Vec<ServiceDemand>) -> Self {
        Self { demands }
    }
}

impl WorkloadSource for FixedDemand {
    fn services(&self) -> usize {
        self.demands.len()
    }

    fn demand(&self, service: ServiceId, _now: SimTime) -> ServiceDemand {
        self.demands[service.index()]
    }
}

/// `PerformanceModel` that encodes `ready_pods` count in `served_rps` signal.
/// Configurable oom flag for testing restart behavior.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ReadyPodsModel {
    pub oom: bool,
}

impl ReadyPodsModel {
    #[allow(dead_code)]
    pub fn new(oom: bool) -> Self {
        Self { oom }
    }
}

impl PerformanceModel for ReadyPodsModel {
    fn evaluate(&self, capacity: &ServiceCapacity, _demand: &ServiceDemand) -> ServiceSignals {
        ServiceSignals {
            cpu_usage_millis: 0.0,
            memory_usage_bytes: 0.0,
            served_rps: f64::from(capacity.ready_pods),
            latency_p50_ms: 0.0,
            latency_p95_ms: 0.0,
            latency_p99_ms: 0.0,
            error_rate: 0.0,
            throttled_fraction: 0.0,
            oom: self.oom,
        }
    }
}
