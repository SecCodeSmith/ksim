use ksim_bench::scenario::Scenario;

use crate::error::EnvError;

/// Per service slot: CPU utilisation of ready requests, memory utilisation of ready requests,
/// `ln(1 + served_rps)`, SLO-percentile latency / SLO latency, error rate, then ready,
/// desired, pending and outdated pods divided by the service's `max_replicas`, then a mask
/// (1 for a real service, 0 for padding; padding slots are all zero).
pub const SERVICE_FEATURES: usize = 10;
/// Per pool: ready, provisioning and draining nodes divided by `max_nodes`, requested CPU /
/// allocatable CPU over the pool's existing nodes (0 without nodes), then a mask of 1.
pub const POOL_FEATURES: usize = 5;
/// `sin` and `cos` of the time of day (period 86 400 s).
pub const TIME_FEATURES: usize = 2;
/// Every feature is clipped to `[OBSERVATION_LOW, OBSERVATION_HIGH]`; only time features can
/// be negative.
pub const OBSERVATION_LOW: f32 = -1.0;
pub const OBSERVATION_HIGH: f32 = 10.0;

/// Which decisions the agent makes. With `Replicas`, nodes are managed by the Cluster
/// Autoscaler exactly as in the `k8s` benchmark policy (seeded with the episode seed).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Control {
    #[default]
    Replicas,
    ReplicasAndNodes,
}

/// `max_replica_step` is `k` in the per-service action `0..=2k`, meaning a replica delta of
/// `action - k`. `service_slots` pads the observation and action to a fixed service count
/// (`None` means the scenario's count). Reward weights feed the default `WeightedReward`.
#[derive(Debug, Clone, PartialEq)]
pub struct EnvConfig {
    pub max_replica_step: u32,
    pub control: Control,
    pub service_slots: Option<usize>,
    pub reward: WeightedRewardWeights,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WeightedRewardWeights {
    pub slo: f64,
    pub cost: f64,
    pub actions: f64,
}

impl Default for WeightedRewardWeights {
    fn default() -> Self {
        Self {
            slo: 1.0,
            cost: 0.5,
            actions: 0.01,
        }
    }
}

impl Default for EnvConfig {
    fn default() -> Self {
        Self {
            max_replica_step: 3,
            control: Control::Replicas,
            service_slots: None,
            reward: WeightedRewardWeights::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReplicaBounds {
    pub min_replicas: u32,
    pub max_replicas: u32,
}

/// Shapes and bounds shared by encoding and decoding, derived from a scenario and a config.
/// Observation: `service_slots * SERVICE_FEATURES`, then `pools * POOL_FEATURES`, then
/// `TIME_FEATURES`. Action (`MultiDiscrete` sizes in `action_nvec`): one `2k + 1` entry per
/// service slot, then with `ReplicasAndNodes` one `3` entry per pool (node delta
/// `action - 1`).
#[derive(Debug, Clone, PartialEq)]
pub struct Layout {
    pub service_slots: usize,
    pub services: Vec<ReplicaBounds>,
    pub pools: usize,
    pub max_replica_step: u32,
    pub control: Control,
    pub slo_latency_ms: f64,
    pub slo_uses_p99: bool,
}

impl Layout {
    /// # Errors
    /// `ServiceSlots` when `config.service_slots` is smaller than the scenario's services.
    pub fn new(scenario: &Scenario, config: &EnvConfig) -> Result<Self, EnvError> {
        let _ = (scenario, config);
        todo!()
    }

    #[must_use]
    pub fn observation_len(&self) -> usize {
        todo!()
    }

    #[must_use]
    pub fn action_nvec(&self) -> Vec<u32> {
        todo!()
    }
}
