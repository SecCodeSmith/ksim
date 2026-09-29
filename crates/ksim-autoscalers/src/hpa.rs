use ksim_core::{
    ids::ServiceId,
    observation::{Observation, ServiceObservation},
    time::{SimDuration, SimTime},
};

use crate::traits::ReplicaRecommender;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScalingPolicy {
    Pods { value: u32, period: SimDuration },
    Percent { value: u32, period: SimDuration },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectPolicy {
    Max,
    Min,
    Disabled,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScalingRules {
    pub stabilization_window: SimDuration,
    pub policies: Vec<ScalingPolicy>,
    pub select: SelectPolicy,
}

/// Upstream `autoscaling/v2` defaults: scale up with a 0 s window, `Max` of 100 % or 4 pods
/// per 15 s; scale down with a 300 s window, 100 % per 15 s; tolerance 0.1.
#[derive(Debug, Clone, PartialEq)]
pub struct HpaBehavior {
    pub scale_up: ScalingRules,
    pub scale_down: ScalingRules,
    pub tolerance: f64,
}

impl Default for HpaBehavior {
    fn default() -> Self {
        let period = SimDuration::from_secs(15);
        Self {
            scale_up: ScalingRules {
                stabilization_window: SimDuration::ZERO,
                policies: vec![
                    ScalingPolicy::Percent { value: 100, period },
                    ScalingPolicy::Pods { value: 4, period },
                ],
                select: SelectPolicy::Max,
            },
            scale_down: ScalingRules {
                stabilization_window: SimDuration::from_secs(300),
                policies: vec![ScalingPolicy::Percent { value: 100, period }],
                select: SelectPolicy::Max,
            },
            tolerance: 0.1,
        }
    }
}

/// Resource metric target: average CPU utilisation as a fraction of requests (0.8 = 80 %).
#[derive(Debug, Clone, PartialEq)]
pub struct HpaSpec {
    pub min_replicas: u32,
    pub max_replicas: u32,
    pub target_cpu_utilization: f64,
    pub behavior: HpaBehavior,
}

/// Upstream `replica_calculator` for a CPU utilisation target, from aggregate metrics.
///
/// `utilization = usage / (ready * request)`, `ratio = utilization / target`. Within
/// `tolerance` of 1 the current replicas are kept. Scaling down ignores not-ready pods;
/// scaling up counts them at 0 % usage, recomputes the ratio over `ready + not_ready`, and
/// keeps the current replicas if that ratio falls within tolerance or reverses direction.
/// Otherwise `ceil(ratio * pods_counted)`. No ready pods or no usage metric: current replicas.
#[must_use]
pub fn desired_replicas(
    current: u32,
    ready: u32,
    not_ready: u32,
    cpu_usage_millis: f64,
    cpu_request_millis: u64,
    target: f64,
    tolerance: f64,
) -> u32 {
    let has_metrics = ready > 0
        && cpu_request_millis > 0
        && cpu_usage_millis.is_finite()
        && cpu_usage_millis >= 0.0
        && target > 0.0;
    if !has_metrics {
        return current;
    }
    let ratio = usage_ratio(cpu_usage_millis, ready, cpu_request_millis, target);
    if within_tolerance(ratio, tolerance) {
        return current;
    }
    if ratio < 1.0 || not_ready == 0 {
        return ceil_to_replicas(ratio * f64::from(ready));
    }
    let pods_counted = ready.saturating_add(not_ready);
    let diluted_ratio = usage_ratio(cpu_usage_millis, pods_counted, cpu_request_millis, target);
    if within_tolerance(diluted_ratio, tolerance) || diluted_ratio < 1.0 {
        return current;
    }
    ceil_to_replicas(diluted_ratio * f64::from(pods_counted))
}

#[allow(clippy::cast_precision_loss)]
fn usage_ratio(usage_millis: f64, pods: u32, request_millis: u64, target: f64) -> f64 {
    let utilization = usage_millis / (f64::from(pods) * request_millis as f64);
    utilization / target
}

fn within_tolerance(ratio: f64, tolerance: f64) -> bool {
    (1.0 - ratio).abs() <= tolerance
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn ceil_to_replicas(value: f64) -> u32 {
    value.ceil().clamp(0.0, f64::from(u32::MAX)) as u32
}

#[derive(Debug, Default)]
struct ServiceHistory {
    recommendations: Vec<(SimTime, u32)>,
    scale_events: Vec<(SimTime, i64)>,
}

impl ServiceHistory {
    /// Upstream `stabilizeRecommendationWithBehaviors`: the window minimum bounds scale-up,
    /// the window maximum bounds scale-down.
    fn stabilize(&mut self, raw: u32, current: u32, now: SimTime, behavior: &HpaBehavior) -> u32 {
        let up_window = behavior.scale_up.stabilization_window;
        let down_window = behavior.scale_down.stabilization_window;
        let mut up_candidate = raw;
        let mut down_candidate = raw;
        for &(time, recommendation) in &self.recommendations {
            if is_within(time, up_window, now) {
                up_candidate = up_candidate.min(recommendation);
            }
            if is_within(time, down_window, now) {
                down_candidate = down_candidate.max(recommendation);
            }
        }
        self.recommendations.push((now, raw));
        let retained = up_window.max(down_window);
        self.recommendations
            .retain(|&(time, _)| is_within(time, retained, now));
        if current < up_candidate {
            up_candidate
        } else if current > down_candidate {
            down_candidate
        } else {
            current
        }
    }

    fn record_scale_event(&mut self, now: SimTime, current: u32, result: u32) {
        if result != current {
            self.scale_events
                .push((now, i64::from(result) - i64::from(current)));
        }
    }

    fn prune_scale_events(&mut self, now: SimTime, behavior: &HpaBehavior) {
        let longest = policy_periods(&behavior.scale_up)
            .chain(policy_periods(&behavior.scale_down))
            .max()
            .unwrap_or(SimDuration::ZERO);
        self.scale_events.retain(|&(time, _)| time + longest > now);
    }

    fn added_within(&self, period: SimDuration, now: SimTime) -> u32 {
        self.sum_within(period, now, |change| change.max(0))
    }

    fn removed_within(&self, period: SimDuration, now: SimTime) -> u32 {
        self.sum_within(period, now, |change| (-change).max(0))
    }

    fn sum_within(&self, period: SimDuration, now: SimTime, magnitude: fn(i64) -> i64) -> u32 {
        let total: i64 = self
            .scale_events
            .iter()
            .filter(|&&(time, _)| time + period > now)
            .map(|&(_, change)| magnitude(change))
            .sum();
        u32::try_from(total).unwrap_or(u32::MAX)
    }
}

fn is_within(time: SimTime, window: SimDuration, now: SimTime) -> bool {
    time + window > now
}

fn policy_periods(rules: &ScalingRules) -> impl Iterator<Item = SimDuration> + '_ {
    rules.policies.iter().map(|policy| match *policy {
        ScalingPolicy::Pods { period, .. } | ScalingPolicy::Percent { period, .. } => period,
    })
}

fn scale_up_limit(
    rules: &ScalingRules,
    current: u32,
    history: &ServiceHistory,
    now: SimTime,
) -> u32 {
    let limits = rules.policies.iter().map(|policy| match *policy {
        ScalingPolicy::Pods { value, period } => {
            period_start_for_up(current, history, period, now).saturating_add(value)
        }
        ScalingPolicy::Percent { value, period } => {
            let start = period_start_for_up(current, history, period, now);
            percent_of(start, 100u32.saturating_add(value), u64::div_ceil)
        }
    });
    match rules.select {
        SelectPolicy::Max => limits.max().unwrap_or(u32::MAX),
        SelectPolicy::Min => limits.min().unwrap_or(u32::MAX),
        SelectPolicy::Disabled => current,
    }
}

fn scale_down_limit(
    rules: &ScalingRules,
    current: u32,
    history: &ServiceHistory,
    now: SimTime,
) -> u32 {
    let limits = rules.policies.iter().map(|policy| match *policy {
        ScalingPolicy::Pods { value, period } => {
            period_start_for_down(current, history, period, now).saturating_sub(value)
        }
        ScalingPolicy::Percent { value, period } => {
            let start = period_start_for_down(current, history, period, now);
            percent_of(start, 100u32.saturating_sub(value), |a, b| a / b)
        }
    });
    match rules.select {
        SelectPolicy::Max => limits.min().unwrap_or(0),
        SelectPolicy::Min => limits.max().unwrap_or(0),
        SelectPolicy::Disabled => current,
    }
}

fn period_start_for_up(
    current: u32,
    history: &ServiceHistory,
    period: SimDuration,
    now: SimTime,
) -> u32 {
    current.saturating_sub(history.added_within(period, now))
}

fn period_start_for_down(
    current: u32,
    history: &ServiceHistory,
    period: SimDuration,
    now: SimTime,
) -> u32 {
    current.saturating_add(history.removed_within(period, now))
}

fn percent_of(start: u32, percent: u32, divide: fn(u64, u64) -> u64) -> u32 {
    let scaled = divide(u64::from(start) * u64::from(percent), 100);
    u32::try_from(scaled).unwrap_or(u32::MAX)
}

fn clamp_replicas(replicas: u32, spec: &HpaSpec) -> u32 {
    if replicas < spec.min_replicas {
        spec.min_replicas
    } else if replicas > spec.max_replicas {
        spec.max_replicas
    } else {
        replicas
    }
}

/// Horizontal Pod Autoscaler with upstream behaviour: per sync the raw recommendation is
/// recorded; scale-up takes the minimum and scale-down the maximum recommendation within the
/// respective stabilisation window; the change is then limited by the scaling policies
/// relative to the replica count at the start of each policy period (`Pods`: +-value,
/// `Percent`: up `ceil(start * (1 + v/100))`, down `floor(start * (1 - v/100))`; `Max`
/// selects the most permissive limit, `Min` the least, `Disabled` forbids the direction);
/// finally clamped to `[min_replicas, max_replicas]`.
#[derive(Debug)]
pub struct Hpa {
    specs: Vec<Option<HpaSpec>>,
    histories: Vec<ServiceHistory>,
}

impl Hpa {
    #[must_use]
    pub fn new(specs: Vec<Option<HpaSpec>>) -> Self {
        let histories = specs.iter().map(|_| ServiceHistory::default()).collect();
        Self { specs, histories }
    }

    /// Desired replicas for one service at `now`, updating the stabilisation and scale-event
    /// history.
    pub fn recommend(
        &mut self,
        service: ServiceId,
        observation: &ServiceObservation,
        now: SimTime,
    ) -> Option<u32> {
        let spec = self.specs.get(service.index())?.as_ref()?;
        let history = self.histories.get_mut(service.index())?;
        let current = observation.desired;
        let raw = desired_replicas(
            current,
            observation.ready,
            observation.pending,
            observation.signals.cpu_usage_millis,
            observation.requests.cpu_millis,
            spec.target_cpu_utilization,
            spec.behavior.tolerance,
        );
        let stabilized = history.stabilize(raw, current, now, &spec.behavior);
        let limited = if stabilized > current {
            stabilized.min(scale_up_limit(
                &spec.behavior.scale_up,
                current,
                history,
                now,
            ))
        } else {
            stabilized.max(scale_down_limit(
                &spec.behavior.scale_down,
                current,
                history,
                now,
            ))
        };
        let result = clamp_replicas(limited, spec);
        history.record_scale_event(now, current, result);
        history.prune_scale_events(now, &spec.behavior);
        Some(result)
    }
}

impl ReplicaRecommender for Hpa {
    fn recommend_replicas(&mut self, observation: &Observation) -> Vec<Option<u32>> {
        observation
            .services
            .iter()
            .enumerate()
            .map(|(index, service)| {
                let id = ServiceId(u32::try_from(index).ok()?);
                self.recommend(id, service, observation.now)
            })
            .collect()
    }
}
