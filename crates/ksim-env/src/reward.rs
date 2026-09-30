use std::fmt::Debug;

use ksim_metrics::{
    metric::{Slo, SloPercentile},
    trace::{ServiceSample, TickSample},
};

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
    let (violating, total) = sample
        .services
        .iter()
        .filter(|service| service.demand_rps > 0.0)
        .fold((0.0, 0.0), |(violating, total), service| {
            let weight = service.demand_rps;
            let violates = violates_slo(service, slo);
            (
                violating + if violates { weight } else { 0.0 },
                total + weight,
            )
        });
    if total > 0.0 { violating / total } else { 0.0 }
}

fn violates_slo(service: &ServiceSample, slo: &Slo) -> bool {
    let latency_ms = match slo.percentile {
        SloPercentile::P95 => service.latency_p95_ms,
        SloPercentile::P99 => service.latency_p99_ms,
    };
    latency_ms > slo.latency_ms || service.error_rate > slo.max_error_rate
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
        let weights = &self.weights;
        let cost_share = share(sample.hourly_cost, context.max_hourly_cost);
        let changes = f64::from(sample.replica_changes) + f64::from(sample.node_changes);
        #[allow(clippy::cast_precision_loss)]
        let changes_per_service = share(changes, context.services as f64);
        -(weights.slo * slo_violation(sample, &context.slo)
            + weights.cost * cost_share
            + weights.actions * changes_per_service)
    }
}

fn share(part: f64, whole: f64) -> f64 {
    if whole > 0.0 { part / whole } else { 0.0 }
}
