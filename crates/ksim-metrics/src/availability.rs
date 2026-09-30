use crate::numeric::count_f64;
use crate::{
    metric::{Metric, Slo, SloPercentile},
    trace::{EpisodeTrace, ServiceSample},
};

const MAX_NINES: f64 = 9.0;

fn samples(trace: &EpisodeTrace) -> impl Iterator<Item = &ServiceSample> {
    trace.ticks.iter().flat_map(|tick| &tick.services)
}

fn request_availability(trace: &EpisodeTrace) -> f64 {
    let demand: f64 = samples(trace).map(|sample| sample.demand_rps).sum();
    if demand <= 0.0 {
        return 1.0;
    }
    let good: f64 = samples(trace)
        .map(|sample| sample.demand_rps * (1.0 - sample.error_rate))
        .sum();
    good / demand
}

fn violates(slo: &Slo, sample: &ServiceSample) -> bool {
    if sample.demand_rps <= 0.0 {
        return false;
    }
    let latency = match slo.percentile {
        SloPercentile::P95 => sample.latency_p95_ms,
        SloPercentile::P99 => sample.latency_p99_ms,
    };
    latency > slo.latency_ms || sample.error_rate > slo.max_error_rate
}

/// Request availability (Beyer et al., *Site Reliability Engineering*, 2016): good requests
/// over valid requests, `Σ demand_rps * (1 - error_rate) / Σ demand_rps` over all ticks and
/// services; 1 when there is no demand. Name `availability`.
#[derive(Debug, Clone, Copy, Default)]
pub struct RequestAvailability;

/// Fraction of (tick, service) samples meeting the SLO: latency at the SLO percentile
/// `<= latency_ms` and `error_rate <= max_error_rate`; samples without demand comply.
/// Name `slo_compliance`.
#[derive(Debug, Clone, Copy)]
pub struct SloCompliance {
    pub slo: Slo,
}

/// `-log10(1 - availability)`, capped at 9. Name `availability_nines`.
#[derive(Debug, Clone, Copy, Default)]
pub struct AvailabilityNines;

/// Consumed error budget: `(1 - availability) / (1 - availability_target)`. Name
/// `error_budget_burn`.
#[derive(Debug, Clone, Copy)]
pub struct ErrorBudgetBurn {
    pub slo: Slo,
}

/// Longest contiguous run of ticks in which any service violates the SLO, in seconds.
/// Name `longest_slo_violation_s`.
#[derive(Debug, Clone, Copy)]
pub struct LongestSloViolation {
    pub slo: Slo,
}

impl Metric for RequestAvailability {
    fn name(&self) -> &'static str {
        "availability"
    }

    fn compute(&self, trace: &EpisodeTrace) -> f64 {
        request_availability(trace)
    }
}

impl Metric for SloCompliance {
    fn name(&self) -> &'static str {
        "slo_compliance"
    }

    fn compute(&self, trace: &EpisodeTrace) -> f64 {
        let total = samples(trace).count();
        if total == 0 {
            return 1.0;
        }
        let compliant = samples(trace)
            .filter(|sample| !violates(&self.slo, sample))
            .count();
        count_f64(compliant) / count_f64(total)
    }
}

impl Metric for AvailabilityNines {
    fn name(&self) -> &'static str {
        "availability_nines"
    }

    fn compute(&self, trace: &EpisodeTrace) -> f64 {
        let unavailability = 1.0 - request_availability(trace);
        if unavailability <= 0.0 {
            MAX_NINES
        } else {
            (-unavailability.log10()).min(MAX_NINES)
        }
    }
}

impl Metric for ErrorBudgetBurn {
    fn name(&self) -> &'static str {
        "error_budget_burn"
    }

    fn compute(&self, trace: &EpisodeTrace) -> f64 {
        let budget = 1.0 - self.slo.availability_target;
        let consumed = 1.0 - request_availability(trace);
        if budget > 0.0 {
            consumed / budget
        } else if consumed > 0.0 {
            f64::INFINITY
        } else {
            0.0
        }
    }
}

impl Metric for LongestSloViolation {
    fn name(&self) -> &'static str {
        "longest_slo_violation_s"
    }

    fn compute(&self, trace: &EpisodeTrace) -> f64 {
        let mut longest = 0_u32;
        let mut current = 0_u32;
        for tick in &trace.ticks {
            if tick
                .services
                .iter()
                .any(|sample| violates(&self.slo, sample))
            {
                current += 1;
                longest = longest.max(current);
            } else {
                current = 0;
            }
        }
        f64::from(longest) * trace.tick.as_secs_f64()
    }
}
