use std::fmt::Debug;

use ksim_metrics::{metric::Slo, trace::TickSample};

use crate::layout::WeightedRewardWeights;

/// What a reward may know beyond the tick: the SLO, the cost of every pool at `max_nodes`
/// (per hour) and the real service count.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RewardContext {
    pub slo: Slo,
    pub max_hourly_cost: f64,
    pub services: usize,
}

/// Scores one tick sample recorded after the agent's action took effect.
pub trait RewardFn: Debug + Send {
    fn reward(&mut self, sample: &TickSample, context: &RewardContext) -> f64;
}

/// Demand-weighted share of services violating the SLO at this tick: a service with positive
/// demand violates when its SLO-percentile latency exceeds `slo.latency_ms` or its error rate
/// exceeds `slo.max_error_rate`. 0 when there is no demand.
#[must_use]
pub fn slo_violation(sample: &TickSample, slo: &Slo) -> f64 {
    let _ = (sample, slo);
    todo!()
}

/// Negated weighted sum of `slo_violation`, `hourly_cost / max_hourly_cost` and
/// `(replica_changes + node_changes) / services`, weighted by `slo`, `cost` and `actions`;
/// a zero `max_hourly_cost` or `services` drops that term.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct WeightedReward {
    pub weights: WeightedRewardWeights,
}

impl RewardFn for WeightedReward {
    fn reward(&mut self, sample: &TickSample, context: &RewardContext) -> f64 {
        let _ = (sample, context);
        todo!()
    }
}
