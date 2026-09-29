use ksim_core::{
    actions::Actions, autoscaler::Autoscaler, ids::ServiceId, observation::Observation,
    time::SimDuration, workload::WorkloadSource,
};

/// Replicas needed to serve `cpu_demand_millis` at `target_utilization` of each pod's CPU
/// request: `max(1, ceil(demand / (request * target)))`.
#[must_use]
pub fn replicas_for_demand(
    cpu_demand_millis: f64,
    cpu_request_millis: u64,
    target_utilization: f64,
) -> u32 {
    let _ = (cpu_demand_millis, cpu_request_millis, target_utilization);
    todo!()
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
        let _ = (samples, cpu_requests_millis, target_utilization);
        todo!()
    }

    /// Sized for each service's mean CPU demand over `samples`.
    #[must_use]
    pub fn average(
        samples: &[Vec<f64>],
        cpu_requests_millis: &[u64],
        target_utilization: f64,
    ) -> Self {
        let _ = (samples, cpu_requests_millis, target_utilization);
        todo!()
    }

    #[must_use]
    pub fn replicas(&self) -> &[u32] {
        &self.replicas
    }
}

impl Autoscaler for StaticReplicas {
    fn act(&mut self, observation: &Observation) -> Actions {
        let _ = observation;
        todo!()
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
        let _ = (
            service,
            observation,
            &self.workload,
            &self.cpu_requests_millis,
            self.target_utilization,
            self.lookahead,
            self.step,
        );
        todo!()
    }
}

impl Autoscaler for Oracle {
    fn act(&mut self, observation: &Observation) -> Actions {
        let _ = observation;
        todo!()
    }
}
