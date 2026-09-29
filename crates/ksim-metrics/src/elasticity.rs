use crate::{metric::Metric, trace::EpisodeTrace};

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
        let _ = trace;
        todo!()
    }
}
