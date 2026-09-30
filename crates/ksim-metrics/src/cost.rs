use crate::{metric::Metric, trace::EpisodeTrace};

fn tick_hours(trace: &EpisodeTrace) -> f64 {
    trace.tick.as_secs_f64() / 3600.0
}

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
        trace.ticks.iter().map(|tick| tick.hourly_cost).sum::<f64>() * tick_hours(trace)
    }
}

impl Metric for NodeHours {
    fn name(&self) -> &'static str {
        "node_hours"
    }

    fn compute(&self, trace: &EpisodeTrace) -> f64 {
        trace
            .ticks
            .iter()
            .map(|tick| f64::from(tick.nodes))
            .sum::<f64>()
            * tick_hours(trace)
    }
}

impl Metric for CpuRequestUtilization {
    fn name(&self) -> &'static str {
        "cpu_request_utilization"
    }

    fn compute(&self, trace: &EpisodeTrace) -> f64 {
        let samples = || trace.ticks.iter().flat_map(|tick| &tick.services);
        let supply: f64 = samples().map(|sample| sample.supply_cpu_millis).sum();
        if supply > 0.0 {
            samples().map(|sample| sample.cpu_usage_millis).sum::<f64>() / supply
        } else {
            0.0
        }
    }
}
