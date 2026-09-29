use crate::numeric::count_f64;
use crate::{metric::Metric, trace::EpisodeTrace};

/// Operational counters over the episode.
///
/// - `ScalingActions`: `Σ replica_changes + Σ node_changes`, name `scaling_actions`;
/// - `PendingPodSeconds`: `Σ pending * tick_seconds`, name `pending_pod_seconds`;
/// - `OomEvents`: samples with `oom`, name `oom_events`;
/// - `Evictions`: `Σ evictions`, name `evictions`;
/// - `MeanReplicas`: mean of `ready` over all samples, name `mean_replicas`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operational {
    ScalingActions,
    PendingPodSeconds,
    OomEvents,
    Evictions,
    MeanReplicas,
}

impl Metric for Operational {
    fn name(&self) -> &'static str {
        match self {
            Self::ScalingActions => "scaling_actions",
            Self::PendingPodSeconds => "pending_pod_seconds",
            Self::OomEvents => "oom_events",
            Self::Evictions => "evictions",
            Self::MeanReplicas => "mean_replicas",
        }
    }

    fn compute(&self, trace: &EpisodeTrace) -> f64 {
        let ticks = &trace.ticks;
        let samples = || ticks.iter().flat_map(|tick| &tick.services);
        match self {
            Self::ScalingActions => sum(ticks.iter().map(|t| t.replica_changes + t.node_changes)),
            Self::PendingPodSeconds => {
                sum(samples().map(|sample| sample.pending)) * trace.tick.as_secs_f64()
            }
            Self::OomEvents => count_f64(samples().filter(|sample| sample.oom).count()),
            Self::Evictions => sum(ticks.iter().map(|tick| tick.evictions)),
            Self::MeanReplicas => {
                let total = sum(samples().map(|sample| sample.ready));
                let count = samples().count();
                if count == 0 {
                    0.0
                } else {
                    total / count_f64(count)
                }
            }
        }
    }
}

pub const OPERATIONAL_METRICS: [Operational; 5] = [
    Operational::ScalingActions,
    Operational::PendingPodSeconds,
    Operational::OomEvents,
    Operational::Evictions,
    Operational::MeanReplicas,
];

fn sum(values: impl Iterator<Item = u32>) -> f64 {
    values.map(f64::from).sum()
}
