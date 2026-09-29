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
        let _ = trace;
        todo!()
    }
}
