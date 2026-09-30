use std::collections::BTreeMap;

use ksim_bench::scenario::Scenario;

use crate::{
    error::EnvError,
    layout::{EnvConfig, Layout},
    reward::RewardFn,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StepOutcome {
    pub reward: f64,
    pub terminated: bool,
    pub truncated: bool,
}

/// One simulated cluster driven by an agent.
///
/// `reset(seed)` builds the scenario's simulation for `seed` and writes the initial
/// observation. `step(action)` decodes the action (`actions::decode`), adds the Cluster
/// Autoscaler's node plan under `Control::Replicas`, steps the simulation once, records the
/// tick (post-step observation, this step's actions, the true demand at the post-step time)
/// with `ksim_metrics::trace::TraceRecorder`, scores that tick with the reward and writes the
/// new observation. Episodes never terminate; they are truncated after `scenario.steps()`
/// steps, after which `step` fails with `NeedsReset` until the next `reset`.
#[derive(Debug)]
pub struct Env {
    _private: (),
}

impl Env {
    /// Uses `WeightedReward` with `config.reward` weights.
    ///
    /// # Errors
    /// When the layout is invalid (`Layout::new`).
    pub fn new(scenario: &Scenario, config: &EnvConfig) -> Result<Self, EnvError> {
        let _ = (scenario, config);
        todo!()
    }

    #[must_use]
    pub fn with_reward(self, reward: Box<dyn RewardFn>) -> Self {
        drop(reward);
        todo!()
    }

    #[must_use]
    pub fn layout(&self) -> &Layout {
        todo!()
    }

    /// Steps per episode (`scenario.steps()`).
    #[must_use]
    pub fn episode_steps(&self) -> u64 {
        todo!()
    }

    /// # Errors
    /// `BufferLength` (buffer `"observation"`), or a scenario that cannot be simulated.
    pub fn reset(&mut self, seed: u64, observation: &mut [f32]) -> Result<(), EnvError> {
        let _ = (seed, observation);
        todo!()
    }

    /// # Errors
    /// `NeedsReset`, `BufferLength` or `ActionOutOfRange`; nothing is stepped on error.
    pub fn step(
        &mut self,
        action: &[u32],
        observation: &mut [f32],
    ) -> Result<StepOutcome, EnvError> {
        let _ = (action, observation);
        todo!()
    }

    /// `ksim_metrics::report::evaluate` with `standard_metrics(scenario.slo)` over the ticks
    /// recorded since the last reset (empty map before the first reset).
    #[must_use]
    pub fn episode_metrics(&self) -> BTreeMap<String, f64> {
        todo!()
    }
}
