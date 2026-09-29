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
        todo!()
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
    let _ = (
        current,
        ready,
        not_ready,
        cpu_usage_millis,
        cpu_request_millis,
        target,
        tolerance,
    );
    todo!()
}

/// Horizontal Pod Autoscaler with upstream behaviour: per sync the raw recommendation is
/// recorded; scale-up takes the minimum and scale-down the maximum recommendation within the
/// respective stabilisation window; the change is then limited by the scaling policies
/// relative to the replica count at the start of each policy period (`Pods`: ±value,
/// `Percent`: up `ceil(start * (1 + v/100))`, down `floor(start * (1 - v/100))`; `Max`
/// selects the most permissive limit, `Min` the least, `Disabled` forbids the direction);
/// finally clamped to `[min_replicas, max_replicas]`.
#[derive(Debug)]
pub struct Hpa {
    specs: Vec<Option<HpaSpec>>,
}

impl Hpa {
    #[must_use]
    pub fn new(specs: Vec<Option<HpaSpec>>) -> Self {
        Self { specs }
    }

    /// Desired replicas for one service at `now`, updating the stabilisation and scale-event
    /// history.
    pub fn recommend(
        &mut self,
        service: ServiceId,
        observation: &ServiceObservation,
        now: SimTime,
    ) -> Option<u32> {
        let _ = (service, observation, now, &self.specs);
        todo!()
    }
}

impl ReplicaRecommender for Hpa {
    fn recommend_replicas(&mut self, observation: &Observation) -> Vec<Option<u32>> {
        let _ = observation;
        todo!()
    }
}
