use crate::{metric::Metric, metric::Slo, trace::EpisodeTrace};

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
        let _ = trace;
        todo!()
    }
}

impl Metric for SloCompliance {
    fn name(&self) -> &'static str {
        "slo_compliance"
    }

    fn compute(&self, trace: &EpisodeTrace) -> f64 {
        let _ = (trace, self.slo);
        todo!()
    }
}

impl Metric for AvailabilityNines {
    fn name(&self) -> &'static str {
        "availability_nines"
    }

    fn compute(&self, trace: &EpisodeTrace) -> f64 {
        let _ = trace;
        todo!()
    }
}

impl Metric for ErrorBudgetBurn {
    fn name(&self) -> &'static str {
        "error_budget_burn"
    }

    fn compute(&self, trace: &EpisodeTrace) -> f64 {
        let _ = (trace, self.slo);
        todo!()
    }
}

impl Metric for LongestSloViolation {
    fn name(&self) -> &'static str {
        "longest_slo_violation_s"
    }

    fn compute(&self, trace: &EpisodeTrace) -> f64 {
        let _ = (trace, self.slo);
        todo!()
    }
}
