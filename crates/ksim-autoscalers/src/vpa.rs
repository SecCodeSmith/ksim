use ksim_core::{
    ids::ServiceId,
    observation::{Observation, ServiceObservation},
    resources::{MIB, Resources},
    time::{SimDuration, SimTime},
};

use crate::{
    histogram::{DecayingHistogram, HistogramOptions},
    traits::{ResourcePlan, ResourceRecommender},
};

const SECONDS_PER_DAY: f64 = 24.0 * 3600.0;
const MIN_CPU_SAMPLE_WEIGHT_CORES: f64 = 0.1;
/// Upstream `lowerBoundConfidenceMultiplier` exponent and base.
const LOWER_CONFIDENCE_BASE: f64 = 0.001;
const LOWER_CONFIDENCE_EXPONENT: f64 = -2.0;

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
        Self {
            recommender_interval: SimDuration::from_secs(60),
            updater_interval: SimDuration::from_secs(60),
            half_life: SimDuration::from_secs(24 * 3600),
            target_percentile: 0.9,
            lower_percentile: 0.5,
            upper_percentile: 0.95,
            safety_margin: 0.15,
            min_cpu_millis: 25,
            min_memory_bytes: 250 * MIB,
            memory_aggregation_interval: SimDuration::from_secs(24 * 3600),
            min_replicas: 2,
            eviction_tolerance: 0.5,
            mode: UpdateMode::Auto,
        }
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
#[derive(Debug, Clone, Copy)]
struct MemoryWindow {
    start: SimTime,
    peak: Option<f64>,
}

#[derive(Debug)]
struct ServiceState {
    cpu: DecayingHistogram,
    memory: DecayingHistogram,
    window: MemoryWindow,
    first_sample: Option<SimTime>,
    last_sample: SimTime,
    last_recommender_run: Option<SimTime>,
    last_updater_run: Option<SimTime>,
}

impl ServiceState {
    fn new(half_life: SimDuration) -> Self {
        Self {
            cpu: DecayingHistogram::new(HistogramOptions::cpu_millis(), half_life),
            memory: DecayingHistogram::new(HistogramOptions::memory_bytes(), half_life),
            window: MemoryWindow {
                start: SimTime::ZERO,
                peak: None,
            },
            first_sample: None,
            last_sample: SimTime::ZERO,
            last_recommender_run: None,
            last_updater_run: None,
        }
    }

    fn history_days(&self, recommender_interval: SimDuration) -> f64 {
        let first = self.first_sample.unwrap_or(self.last_sample);
        let history = (self.last_sample - first).max(recommender_interval);
        history.as_secs_f64() / SECONDS_PER_DAY
    }
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
    states: Vec<ServiceState>,
}

impl Vpa {
    #[must_use]
    pub fn new(config: VpaConfig, enabled: Vec<bool>) -> Self {
        let states = enabled
            .iter()
            .map(|_| ServiceState::new(config.half_life))
            .collect();
        Self {
            config,
            enabled,
            states,
        }
    }

    #[must_use]
    pub fn recommendation(&self, service: ServiceId) -> Option<Recommendation> {
        self.recommendation_at(service.index())
    }

    fn recommendation_at(&self, index: usize) -> Option<Recommendation> {
        let state = self.states.get(index)?;
        if !self.is_enabled(index) || state.cpu.is_empty() {
            return None;
        }
        let days = state.history_days(self.config.recommender_interval);
        Some(Recommendation {
            target: self.estimate(state, self.config.target_percentile, 1.0),
            lower_bound: self.estimate(
                state,
                self.config.lower_percentile,
                (1.0 + LOWER_CONFIDENCE_BASE / days).powf(LOWER_CONFIDENCE_EXPONENT),
            ),
            upper_bound: self.estimate(state, self.config.upper_percentile, 1.0 + 1.0 / days),
        })
    }

    fn is_enabled(&self, index: usize) -> bool {
        self.enabled.get(index).copied().unwrap_or(false)
    }

    fn estimate(&self, state: &ServiceState, percentile: f64, confidence: f64) -> Resources {
        let scale = (1.0 + self.config.safety_margin) * confidence;
        let cpu = state.cpu.percentile(percentile) * scale;
        let memory = state.memory.percentile(percentile) * scale;
        Resources::new(
            round_up(cpu).max(self.config.min_cpu_millis),
            round_up(memory).max(self.config.min_memory_bytes),
        )
    }

    fn run_recommender(&mut self, index: usize, now: SimTime, service: &ServiceObservation) {
        let interval = self.config.recommender_interval;
        let window_length = self.config.memory_aggregation_interval;
        let Some(state) = self.states.get_mut(index) else {
            return;
        };
        if !is_due(state.last_recommender_run, now, interval) {
            return;
        }
        state.last_recommender_run = Some(now);
        if service.ready == 0 {
            return;
        }
        let ready = f64::from(service.ready);
        state.first_sample.get_or_insert(now);
        state.last_sample = now;
        add_cpu_sample(state, service, ready, now);
        record_memory_peak(
            state,
            service.signals.memory_usage_bytes / ready,
            now,
            window_length,
        );
    }

    fn run_updater(
        &mut self,
        index: usize,
        now: SimTime,
        service: &ServiceObservation,
    ) -> (Option<Resources>, u32) {
        if self.config.mode == UpdateMode::Off || !self.updater_due(index, now) {
            return (None, 0);
        }
        let Some(recommendation) = self.recommendation_at(index) else {
            return (None, 0);
        };
        let requests = requests_outside_bounds(service.requests, &recommendation)
            .then_some(recommendation.target);
        (requests, self.eviction_count(service))
    }

    fn updater_due(&mut self, index: usize, now: SimTime) -> bool {
        let interval = self.config.updater_interval;
        let Some(state) = self.states.get_mut(index) else {
            return false;
        };
        if !is_due(state.last_updater_run, now, interval) {
            return false;
        }
        state.last_updater_run = Some(now);
        true
    }

    fn eviction_count(&self, service: &ServiceObservation) -> u32 {
        if self.config.mode != UpdateMode::Auto || service.desired < self.config.min_replicas {
            return 0;
        }
        service.outdated.min(self.eviction_limit(service.desired))
    }

    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    fn eviction_limit(&self, desired: u32) -> u32 {
        let by_tolerance = (self.config.eviction_tolerance * f64::from(desired)).floor() as u32;
        by_tolerance.max(1)
    }
}

impl ResourceRecommender for Vpa {
    fn recommend_resources(&mut self, observation: &Observation) -> ResourcePlan {
        let count = observation.services.len();
        let mut plan = ResourcePlan {
            requests: vec![None; count],
            evict: vec![0; count],
        };
        for (index, service) in observation.services.iter().enumerate() {
            if !self.is_enabled(index) {
                continue;
            }
            self.run_recommender(index, observation.now, service);
            let (requests, evict) = self.run_updater(index, observation.now, service);
            plan.requests[index] = requests;
            plan.evict[index] = evict;
        }
        plan
    }
}

fn is_due(last_run: Option<SimTime>, now: SimTime, interval: SimDuration) -> bool {
    last_run.is_none_or(|last| now - last >= interval)
}

fn add_cpu_sample(
    state: &mut ServiceState,
    service: &ServiceObservation,
    ready: f64,
    now: SimTime,
) {
    #[allow(clippy::cast_precision_loss)]
    let request_cores = service.requests.cpu_millis as f64 / 1000.0;
    let weight = request_cores.max(MIN_CPU_SAMPLE_WEIGHT_CORES);
    state
        .cpu
        .add_sample(service.signals.cpu_usage_millis / ready, weight, now);
}

fn record_memory_peak(
    state: &mut ServiceState,
    usage: f64,
    now: SimTime,
    window_length: SimDuration,
) {
    if now - state.window.start >= window_length || state.first_sample == Some(now) {
        state.window = MemoryWindow {
            start: now,
            peak: None,
        };
    }
    let window = state.window;
    if window.peak.is_some_and(|peak| usage <= peak) {
        return;
    }
    if let Some(old_peak) = window.peak {
        state.memory.subtract_sample(old_peak, 1.0, window.start);
    }
    state.memory.add_sample(usage, 1.0, window.start);
    state.window.peak = Some(usage);
}

fn requests_outside_bounds(requests: Resources, recommendation: &Recommendation) -> bool {
    let (lower, upper) = (recommendation.lower_bound, recommendation.upper_bound);
    requests.cpu_millis < lower.cpu_millis
        || requests.cpu_millis > upper.cpu_millis
        || requests.memory_bytes < lower.memory_bytes
        || requests.memory_bytes > upper.memory_bytes
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn round_up(value: f64) -> u64 {
    value.ceil() as u64
}
