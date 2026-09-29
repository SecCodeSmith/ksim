use std::fmt::Debug;

use crate::{resources::Resources, workload::ServiceDemand};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServiceCapacity {
    pub ready_pods: u32,
    pub requests: Resources,
    pub limits: Resources,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ServiceSignals {
    pub cpu_usage_millis: f64,
    pub memory_usage_bytes: f64,
    pub served_rps: f64,
    pub latency_p50_ms: f64,
    pub latency_p95_ms: f64,
    pub latency_p99_ms: f64,
    pub error_rate: f64,
    pub throttled_fraction: f64,
    pub oom: bool,
}

pub trait PerformanceModel: Debug + Send {
    fn evaluate(&self, capacity: &ServiceCapacity, demand: &ServiceDemand) -> ServiceSignals;
}
