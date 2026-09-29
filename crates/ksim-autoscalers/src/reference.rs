use crate::traits::ReplicaRecommender;
use ksim_core::{
    actions::Actions,
    autoscaler::Autoscaler,
    ids::ServiceId,
    observation::Observation,
    time::{SimDuration, SimTime},
    workload::WorkloadSource,
};

/// Replicas needed to serve `cpu_demand_millis` at `target_utilization` of each pod's CPU
/// request: `max(1, ceil(demand / (request * target)))`.
#[must_use]
#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "replica counts are small non-negative values"
)]
pub fn replicas_for_demand(
    cpu_demand_millis: f64,
    cpu_request_millis: u64,
    target_utilization: f64,
) -> u32 {
    let capacity_per_replica = cpu_request_millis as f64 * target_utilization;
    if !capacity_per_replica.is_finite() || capacity_per_replica <= 0.0 {
        return 1;
    }
    let needed = (cpu_demand_millis / capacity_per_replica).ceil();
    (needed as u32).max(1)
}

fn sized_per_service(
    samples: &[Vec<f64>],
    cpu_requests_millis: &[u64],
    target_utilization: f64,
    summarize: fn(&[f64]) -> f64,
) -> Vec<u32> {
    samples
        .iter()
        .enumerate()
        .map(|(service, series)| {
            let request = cpu_requests_millis.get(service).copied().unwrap_or(0);
            replicas_for_demand(summarize(series), request, target_utilization)
        })
        .collect()
}

fn peak_of(series: &[f64]) -> f64 {
    series.iter().copied().fold(0.0, f64::max)
}

#[allow(
    clippy::cast_precision_loss,
    reason = "sample counts are far below 2^52"
)]
fn mean_of(series: &[f64]) -> f64 {
    if series.is_empty() {
        return 0.0;
    }
    series.iter().sum::<f64>() / series.len() as f64
}

/// Fixed replica counts, applied on every call; the static reference points of the benchmark.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StaticReplicas {
    replicas: Vec<u32>,
}

impl StaticReplicas {
    #[must_use]
    pub fn new(replicas: Vec<u32>) -> Self {
        Self { replicas }
    }

    /// Sized for each service's peak CPU demand over `samples` (one vector per service).
    #[must_use]
    pub fn peak(
        samples: &[Vec<f64>],
        cpu_requests_millis: &[u64],
        target_utilization: f64,
    ) -> Self {
        Self::new(sized_per_service(
            samples,
            cpu_requests_millis,
            target_utilization,
            peak_of,
        ))
    }

    /// Sized for each service's mean CPU demand over `samples`.
    #[must_use]
    pub fn average(
        samples: &[Vec<f64>],
        cpu_requests_millis: &[u64],
        target_utilization: f64,
    ) -> Self {
        Self::new(sized_per_service(
            samples,
            cpu_requests_millis,
            target_utilization,
            mean_of,
        ))
    }

    #[must_use]
    pub fn replicas(&self) -> &[u32] {
        &self.replicas
    }
}

impl Autoscaler for StaticReplicas {
    fn act(&mut self, observation: &Observation) -> Actions {
        Actions {
            replicas: self.recommend_replicas(observation),
            ..Actions::none()
        }
    }
}

/// Clairvoyant upper bound: knows the true future demand and sizes every service for the
/// highest CPU demand sampled every `step` over `[now, now + lookahead]`.
#[derive(Debug)]
pub struct Oracle {
    workload: Box<dyn WorkloadSource>,
    cpu_requests_millis: Vec<u64>,
    target_utilization: f64,
    lookahead: SimDuration,
    step: SimDuration,
}

impl Oracle {
    #[must_use]
    pub fn new(
        workload: Box<dyn WorkloadSource>,
        cpu_requests_millis: Vec<u64>,
        target_utilization: f64,
        lookahead: SimDuration,
        step: SimDuration,
    ) -> Self {
        Self {
            workload,
            cpu_requests_millis,
            target_utilization,
            lookahead,
            step,
        }
    }

    #[must_use]
    pub fn replicas_for(&self, service: ServiceId, observation: &Observation) -> u32 {
        let request = self.cpu_request_millis(service, observation);
        replicas_for_demand(
            self.peak_demand(service, observation.now),
            request,
            self.target_utilization,
        )
    }

    /// Falls back to the observed request when the service has no configured one, and to 0
    /// (a single replica) when the observation does not know the service either.
    fn cpu_request_millis(&self, service: ServiceId, observation: &Observation) -> u64 {
        self.cpu_requests_millis
            .get(service.index())
            .copied()
            .or_else(|| {
                observation
                    .services
                    .get(service.index())
                    .map(|observed| observed.requests.cpu_millis)
            })
            .unwrap_or(0)
    }

    fn peak_demand(&self, service: ServiceId, now: SimTime) -> f64 {
        self.sample_times(now)
            .map(|at| self.workload.demand(service, at).cpu_millis)
            .fold(0.0, f64::max)
    }

    fn sample_times(&self, now: SimTime) -> impl Iterator<Item = SimTime> + '_ {
        let step_millis = self.step.as_millis().max(1);
        let sample_count = if self.step.as_millis() == 0 {
            1
        } else {
            self.lookahead.as_millis() / step_millis + 1
        };
        (0..sample_count).map(move |index| now + self.step.saturating_mul(index))
    }
}

impl Autoscaler for Oracle {
    fn act(&mut self, observation: &Observation) -> Actions {
        Actions {
            replicas: self.recommend_replicas(observation),
            ..Actions::none()
        }
    }
}

impl ReplicaRecommender for StaticReplicas {
    fn recommend_replicas(&mut self, _observation: &Observation) -> Vec<Option<u32>> {
        self.replicas.iter().copied().map(Some).collect()
    }
}

impl ReplicaRecommender for Oracle {
    #[allow(
        clippy::cast_possible_truncation,
        reason = "service counts are far below u32::MAX"
    )]
    fn recommend_replicas(&mut self, observation: &Observation) -> Vec<Option<u32>> {
        (0..observation.services.len())
            .map(|index| Some(self.replicas_for(ServiceId(index as u32), observation)))
            .collect()
    }
}
