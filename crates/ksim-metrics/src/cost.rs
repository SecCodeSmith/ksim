use crate::{metric::Metric, trace::EpisodeTrace};

/// `Σ hourly_cost * tick_hours`. Name `cost`.
#[derive(Debug, Clone, Copy, Default)]
pub struct Cost;

/// `Σ nodes * tick_hours`. Name `node_hours`.
#[derive(Debug, Clone, Copy, Default)]
pub struct NodeHours;

/// `Σ cpu_usage_millis / Σ supply_cpu_millis` over all samples; 0 without supply. Name
/// `cpu_request_utilization`.
#[derive(Debug, Clone, Copy, Default)]
pub struct CpuRequestUtilization;

impl Metric for Cost {
    fn name(&self) -> &'static str {
        "cost"
    }

    fn compute(&self, trace: &EpisodeTrace) -> f64 {
        let _ = trace;
        todo!()
    }
}

impl Metric for NodeHours {
    fn name(&self) -> &'static str {
        "node_hours"
    }

    fn compute(&self, trace: &EpisodeTrace) -> f64 {
        let _ = trace;
        todo!()
    }
}

impl Metric for CpuRequestUtilization {
    fn name(&self) -> &'static str {
        "cpu_request_utilization"
    }

    fn compute(&self, trace: &EpisodeTrace) -> f64 {
        let _ = trace;
        todo!()
    }
}
