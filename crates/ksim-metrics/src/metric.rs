use std::fmt::Debug;

use serde::{Deserialize, Serialize};

use crate::{
    availability::{
        AvailabilityNines, ErrorBudgetBurn, LongestSloViolation, RequestAvailability, SloCompliance,
    },
    cost::{Cost, CpuRequestUtilization, NodeHours},
    elasticity::ELASTICITY_METRICS,
    operational::OPERATIONAL_METRICS,
    trace::EpisodeTrace,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SloPercentile {
    P95,
    P99,
}

/// Service level objective shared by all services of a scenario.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Slo {
    pub latency_ms: f64,
    pub percentile: SloPercentile,
    pub max_error_rate: f64,
    pub availability_target: f64,
}

/// A scalar computed from a whole episode; new metrics are new implementations.
pub trait Metric: Debug + Send {
    fn name(&self) -> &'static str;
    fn compute(&self, trace: &EpisodeTrace) -> f64;
}

/// Every metric of the benchmark protocol, in report order.
#[must_use]
pub fn standard_metrics(slo: Slo) -> Vec<Box<dyn Metric>> {
    let mut metrics: Vec<Box<dyn Metric>> = vec![
        Box::new(RequestAvailability),
        Box::new(SloCompliance { slo }),
        Box::new(AvailabilityNines),
        Box::new(ErrorBudgetBurn { slo }),
        Box::new(LongestSloViolation { slo }),
    ];
    metrics.extend(ELASTICITY_METRICS.map(|variant| Box::new(variant) as Box<dyn Metric>));
    metrics.push(Box::new(Cost));
    metrics.push(Box::new(NodeHours));
    metrics.push(Box::new(CpuRequestUtilization));
    metrics.extend(OPERATIONAL_METRICS.map(|variant| Box::new(variant) as Box<dyn Metric>));
    metrics
}
