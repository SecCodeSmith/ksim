use ksim_bench::scenario::Scenario;
use ksim_metrics::metric::SloPercentile;

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

const NODE_ACTION_CHOICES: u32 = 3;

impl Layout {
    /// # Errors
    /// `ServiceSlots` when `config.service_slots` is smaller than the scenario's services.
    pub fn new(scenario: &Scenario, config: &EnvConfig) -> Result<Self, EnvError> {
        let services = scenario.services.len();
        let service_slots = config.service_slots.unwrap_or(services);
        if service_slots < services {
            return Err(EnvError::ServiceSlots {
                services,
                slots: service_slots,
            });
        }
        Ok(Self {
            service_slots,
            services: scenario
                .services
                .iter()
                .map(|service| ReplicaBounds {
                    min_replicas: service.hpa.min_replicas,
                    max_replicas: service.hpa.max_replicas,
                })
                .collect(),
            pools: scenario.pools.len(),
            max_replica_step: config.max_replica_step,
            control: config.control,
            slo_latency_ms: scenario.slo.latency_ms,
            slo_uses_p99: scenario.slo.percentile == SloPercentile::P99,
        })
    }

    #[must_use]
    pub fn observation_len(&self) -> usize {
        self.service_slots * SERVICE_FEATURES + self.pools * POOL_FEATURES + TIME_FEATURES
    }

    /// Number of entries in an action: a service slot each, plus a pool each when nodes are
    /// controlled.
    #[must_use]
    pub fn action_len(&self) -> usize {
        match self.control {
            Control::Replicas => self.service_slots,
            Control::ReplicasAndNodes => self.service_slots + self.pools,
        }
    }

    /// Exclusive upper bound of the action entry at `index`.
    #[must_use]
    pub fn action_bound(&self, index: usize) -> u32 {
        if index < self.service_slots {
            2 * self.max_replica_step + 1
        } else {
            NODE_ACTION_CHOICES
        }
    }

    #[must_use]
    pub fn action_nvec(&self) -> Vec<u32> {
        (0..self.action_len())
            .map(|index| self.action_bound(index))
            .collect()
    }
}
