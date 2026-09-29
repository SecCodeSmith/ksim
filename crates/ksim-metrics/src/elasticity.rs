use crate::numeric::count_f64;
use std::cmp::Ordering;

use crate::{metric::Metric, trace::EpisodeTrace};

pub const ELASTICITY_METRICS: [Elasticity; 6] = [
    Elasticity::UnderProvisioningAccuracy,
    Elasticity::OverProvisioningAccuracy,
    Elasticity::UnderProvisioningTimeshare,
    Elasticity::OverProvisioningTimeshare,
    Elasticity::Jitter,
    Elasticity::Instability,
];

/// Demand and supply of one service, one entry per tick.
struct ServiceSeries {
    demand: Vec<f64>,
    supply: Vec<f64>,
}

/// SPEC RG elasticity metrics (Herbst et al., "Elasticity in Cloud Computing: What It Is,
/// and What It Is Not", ICAC 2013; Herbst et al., "Quantifying Cloud Performance and
/// Dependability", ACM TOMPECS 2018) on CPU demand `d_t` versus supply `s_t` per service,
/// averaged over services. With `T` ticks of equal length:
///
/// - `theta_u = 100/T * Σ max(d_t - s_t, 0) / d_t` (ticks with `d_t = 0` add 0), name
///   `under_provisioning_accuracy`;
/// - `theta_o = 100/T * Σ max(s_t - d_t, 0) / d_t` (ticks with `d_t = 0` add 0), name
///   `over_provisioning_accuracy`;
/// - `tau_u = 100/T * Σ [s_t < d_t]`, name `under_provisioning_timeshare`;
/// - `tau_o = 100/T * Σ [s_t > d_t]`, name `over_provisioning_timeshare`;
/// - jitter `= (E_s - E_d) / hours`, where `E_x` counts ticks `t >= 1` with `x_t != x_{t-1}`,
///   name `jitter_per_hour`;
/// - instability `= 100/(T-1) * Σ_{t>=1} [sgn(Δs_t) != sgn(Δd_t)]`, name `instability`.
///
/// Empty traces (or a single tick for jitter/instability) yield 0.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Elasticity {
    UnderProvisioningAccuracy,
    OverProvisioningAccuracy,
    UnderProvisioningTimeshare,
    OverProvisioningTimeshare,
    Jitter,
    Instability,
}

impl Metric for Elasticity {
    fn name(&self) -> &'static str {
        match self {
            Self::UnderProvisioningAccuracy => "under_provisioning_accuracy",
            Self::OverProvisioningAccuracy => "over_provisioning_accuracy",
            Self::UnderProvisioningTimeshare => "under_provisioning_timeshare",
            Self::OverProvisioningTimeshare => "over_provisioning_timeshare",
            Self::Jitter => "jitter_per_hour",
            Self::Instability => "instability",
        }
    }

    fn compute(&self, trace: &EpisodeTrace) -> f64 {
        let series = service_series(trace);
        if series.is_empty() {
            return 0.0;
        }
        let hours = count_f64(trace.ticks.len()) * trace.tick.as_secs_f64() / 3600.0;
        let total: f64 = series.iter().map(|s| self.for_service(s, hours)).sum();
        total / count_f64(series.len())
    }
}

impl Elasticity {
    fn for_service(self, series: &ServiceSeries, hours: f64) -> f64 {
        let (demand, supply) = (&series.demand, &series.supply);
        match self {
            Self::UnderProvisioningAccuracy => accuracy(demand, supply, |d, s| (d - s).max(0.0)),
            Self::OverProvisioningAccuracy => accuracy(demand, supply, |d, s| (s - d).max(0.0)),
            Self::UnderProvisioningTimeshare => timeshare(demand, supply, |d, s| s < d),
            Self::OverProvisioningTimeshare => timeshare(demand, supply, |d, s| s > d),
            Self::Jitter if hours > 0.0 => {
                (count_f64(change_count(supply)) - count_f64(change_count(demand))) / hours
            }
            Self::Jitter => 0.0,
            Self::Instability => instability(demand, supply),
        }
    }
}

fn service_series(trace: &EpisodeTrace) -> Vec<ServiceSeries> {
    let services = trace
        .ticks
        .iter()
        .map(|t| t.services.len())
        .max()
        .unwrap_or(0);
    (0..services)
        .map(|index| {
            let samples: Vec<_> = trace
                .ticks
                .iter()
                .filter_map(|tick| tick.services.get(index))
                .collect();
            ServiceSeries {
                demand: samples.iter().map(|s| s.demand_cpu_millis).collect(),
                supply: samples.iter().map(|s| s.supply_cpu_millis).collect(),
            }
        })
        .collect()
}

fn percent_of_ticks(count: f64, ticks: usize) -> f64 {
    if ticks == 0 {
        0.0
    } else {
        100.0 * count / count_f64(ticks)
    }
}

fn accuracy(demand: &[f64], supply: &[f64], gap: impl Fn(f64, f64) -> f64) -> f64 {
    let relative_gaps: f64 = demand
        .iter()
        .zip(supply)
        .filter(|(d, _)| **d > 0.0)
        .map(|(d, s)| gap(*d, *s) / d)
        .sum();
    percent_of_ticks(relative_gaps, demand.len())
}

fn timeshare(demand: &[f64], supply: &[f64], matches: impl Fn(f64, f64) -> bool) -> f64 {
    let hits = demand
        .iter()
        .zip(supply)
        .filter(|(d, s)| matches(**d, **s))
        .count();
    percent_of_ticks(count_f64(hits), demand.len())
}

fn change_count(series: &[f64]) -> usize {
    series
        .windows(2)
        .filter(|pair| direction(pair) != Ordering::Equal)
        .count()
}

fn direction(pair: &[f64]) -> Ordering {
    pair[1].partial_cmp(&pair[0]).unwrap_or(Ordering::Equal)
}

fn instability(demand: &[f64], supply: &[f64]) -> f64 {
    let mismatches = demand
        .windows(2)
        .zip(supply.windows(2))
        .filter(|(d, s)| direction(d) != direction(s))
        .count();
    percent_of_ticks(count_f64(mismatches), demand.len().saturating_sub(1))
}
