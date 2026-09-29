use ksim_core::{
    ids::ServiceId,
    observation::Observation,
    resources::{MIB, Resources},
    time::SimDuration,
};

use crate::traits::{ResourcePlan, ResourceRecommender};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateMode {
    Off,
    Initial,
    Auto,
}

/// Upstream VPA defaults: recommender and updater every 60 s, histogram half-life 24 h,
/// target P90, lower bound P50, upper bound P95, safety margin 15 %, minimum 25m CPU and
/// 250 MiB memory, memory peaks aggregated per 24 h, eviction only when a service has at
/// least 2 replicas, eviction tolerance 50 %, mode `Auto`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VpaConfig {
    pub recommender_interval: SimDuration,
    pub updater_interval: SimDuration,
    pub half_life: SimDuration,
    pub target_percentile: f64,
    pub lower_percentile: f64,
    pub upper_percentile: f64,
    pub safety_margin: f64,
    pub min_cpu_millis: u64,
    pub min_memory_bytes: u64,
    pub memory_aggregation_interval: SimDuration,
    pub min_replicas: u32,
    pub eviction_tolerance: f64,
    pub mode: UpdateMode,
}

impl Default for VpaConfig {
    fn default() -> Self {
        let _ = MIB;
        todo!()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Recommendation {
    pub target: Resources,
    pub lower_bound: Resources,
    pub upper_bound: Resources,
}

/// Vertical Pod Autoscaler (recommender + updater).
///
/// Recommender, every `recommender_interval`, for services with ready pods: adds the per-pod
/// CPU usage (`cpu_usage_millis / ready`, weight `max(request cores, 0.1)`) to a decaying CPU
/// histogram, and tracks the per-pod memory peak (`memory_usage_bytes / ready`) of the current
/// aggregation interval in a decaying memory histogram (the current peak is replaced when
/// exceeded). Each estimate is `percentile * (1 + safety_margin)`; with `c` = days of history
/// (at least one recommender interval), the upper bound is multiplied by `1 + 1/c` and the
/// lower bound by `(1 + 0.001/c)^-2`; all three are raised to the minimums and rounded up to
/// whole millicores and bytes.
///
/// Updater, every `updater_interval`, in `Initial` or `Auto` mode: when the current requests
/// fall outside `[lower_bound, upper_bound]` in CPU or memory, sets requests to `target`
/// (new pods only). In `Auto` mode it also evicts outdated pods of services with at least
/// `min_replicas` desired replicas, at most `max(1, floor(eviction_tolerance * desired))`
/// per run.
#[derive(Debug)]
pub struct Vpa {
    config: VpaConfig,
    enabled: Vec<bool>,
}

impl Vpa {
    #[must_use]
    pub fn new(config: VpaConfig, enabled: Vec<bool>) -> Self {
        Self { config, enabled }
    }

    #[must_use]
    pub fn recommendation(&self, service: ServiceId) -> Option<Recommendation> {
        let _ = (service, self.config, &self.enabled);
        todo!()
    }
}

impl ResourceRecommender for Vpa {
    fn recommend_resources(&mut self, observation: &Observation) -> ResourcePlan {
        let _ = observation;
        todo!()
    }
}
