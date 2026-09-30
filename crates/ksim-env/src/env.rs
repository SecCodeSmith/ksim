use std::collections::BTreeMap;

use ksim_autoscalers::traits::NodeProvisioner;
use ksim_bench::{policy::cluster_autoscaler, scenario::Scenario};
use ksim_core::{
    actions::Actions,
    engine::Simulation,
    ids::ServiceId,
    time::SimTime,
    workload::{ServiceDemand, WorkloadSource},
};
use ksim_metrics::{
    metric::standard_metrics,
    report::evaluate,
    trace::{TickSample, TraceRecorder},
};

use crate::{
    actions,
    error::EnvError,
    layout::{Control, EnvConfig, Layout},
    observation,
    reward::{RewardContext, RewardFn, WeightedReward},
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
/// observation. `step(action)` decodes the action (`actions::decode`) and adds the Cluster
/// Autoscaler's node plan under `Control::Replicas`. It then records the episode tick exactly
/// like `ksim_bench::runner::run_episode`: the pre-step observation, this step's actions and
/// the true demand at the pre-step time. Next it steps the simulation once, and scores a tick
/// sample built the same way from the post-step observation (this step's actions, demand at
/// the post-step time) with the reward. Finally it writes the new observation. So
/// `episode_metrics` after a full episode of zero-delta actions under `Control::Replicas`
/// equals `run_episode` with `Policy::Ca` for the same seed. Episodes never terminate. They
/// are truncated after `scenario.steps()` steps, after which `step` fails with `NeedsReset`
/// until the next `reset`.
#[derive(Debug)]
pub struct Env {
    scenario: Scenario,
    layout: Layout,
    reward: Box<dyn RewardFn>,
    context: RewardContext,
    episode: Option<Episode>,
}

impl Env {
    /// Uses `WeightedReward` with `config.reward` weights.
    ///
    /// # Errors
    /// When the layout is invalid (`Layout::new`).
    pub fn new(scenario: &Scenario, config: &EnvConfig) -> Result<Self, EnvError> {
        Ok(Self {
            layout: Layout::new(scenario, config)?,
            reward: Box::new(WeightedReward {
                weights: config.reward,
            }),
            context: reward_context(scenario),
            scenario: scenario.clone(),
            episode: None,
        })
    }

    #[must_use]
    pub fn with_reward(mut self, reward: Box<dyn RewardFn>) -> Self {
        self.reward = reward;
        self
    }

    #[must_use]
    pub fn layout(&self) -> &Layout {
        &self.layout
    }

    /// Steps per episode (`scenario.steps()`).
    #[must_use]
    pub fn episode_steps(&self) -> u64 {
        self.scenario.steps()
    }

    /// Whether `reset` has been called at least once.
    #[must_use]
    pub fn has_started(&self) -> bool {
        self.episode.is_some()
    }

    /// Whether the current episode was truncated, so only `reset` can continue it.
    #[must_use]
    pub fn is_over(&self) -> bool {
        self.episode.as_ref().is_some_and(|episode| episode.over)
    }

    /// # Errors
    /// `BufferLength` (buffer `"observation"`), or a scenario that cannot be simulated.
    pub fn reset(&mut self, seed: u64, observation: &mut [f32]) -> Result<(), EnvError> {
        check_observation_len(observation, &self.layout)?;
        let episode = Episode::start(&self.scenario, self.layout.control, seed)?;
        observation::encode(episode.simulation.observation(), &self.layout, observation);
        self.episode = Some(episode);
        Ok(())
    }

    /// # Errors
    /// `NeedsReset`, `BufferLength` or `ActionOutOfRange`; nothing is stepped on error.
    pub fn step(
        &mut self,
        action: &[u32],
        observation: &mut [f32],
    ) -> Result<StepOutcome, EnvError> {
        let episode = self
            .episode
            .as_mut()
            .filter(|episode| !episode.over)
            .ok_or(EnvError::NeedsReset)?;
        check_observation_len(observation, &self.layout)?;
        actions::validate(action, &self.layout)?;

        let chosen = episode.plan(action, &self.layout);
        episode.record_pre_step(&chosen);
        episode.simulation.step(&chosen);
        let sample = episode.sample_post_step(&chosen);
        let reward = self.reward.reward(&sample, &self.context);
        observation::encode(episode.simulation.observation(), &self.layout, observation);

        episode.steps_taken += 1;
        let truncated = episode.steps_taken >= self.scenario.steps();
        episode.over = truncated;
        Ok(StepOutcome {
            reward,
            terminated: false,
            truncated,
        })
    }

    /// `ksim_metrics::report::evaluate` with `standard_metrics(scenario.slo)` over the ticks
    /// recorded since the last reset (empty map before the first reset).
    #[must_use]
    pub fn episode_metrics(&self) -> BTreeMap<String, f64> {
        self.episode.as_ref().map_or_else(BTreeMap::new, |episode| {
            evaluate(
                episode.recorder.trace(),
                &standard_metrics(self.scenario.slo),
            )
        })
    }
}

fn check_observation_len(observation: &[f32], layout: &Layout) -> Result<(), EnvError> {
    let expected = layout.observation_len();
    if observation.len() == expected {
        Ok(())
    } else {
        Err(EnvError::BufferLength {
            buffer: "observation",
            expected,
            actual: observation.len(),
        })
    }
}

fn reward_context(scenario: &Scenario) -> RewardContext {
    RewardContext {
        slo: scenario.slo,
        max_hourly_cost: scenario
            .pools
            .iter()
            .map(|pool| f64::from(pool.max_nodes) * pool.price_per_hour)
            .sum(),
        services: scenario.services.len(),
    }
}

/// Everything that lives from one `reset` to the next.
#[derive(Debug)]
struct Episode {
    simulation: Simulation,
    workload: Box<dyn WorkloadSource>,
    node_provisioner: Option<Box<dyn NodeProvisioner>>,
    recorder: TraceRecorder,
    demands: Vec<ServiceDemand>,
    steps_taken: u64,
    over: bool,
}

impl Episode {
    fn start(scenario: &Scenario, control: Control, seed: u64) -> Result<Self, EnvError> {
        let prices = scenario
            .pools
            .iter()
            .map(|pool| pool.price_per_hour)
            .collect();
        Ok(Self {
            simulation: scenario.build_simulation(seed)?,
            workload: scenario.workload(seed),
            node_provisioner: (control == Control::Replicas).then(|| cluster_autoscaler(seed)),
            recorder: TraceRecorder::new(scenario.tick(), prices),
            demands: Vec::with_capacity(scenario.services.len()),
            steps_taken: 0,
            over: false,
        })
    }

    fn plan(&mut self, action: &[u32], layout: &Layout) -> Actions {
        let observation = self.simulation.observation();
        let mut chosen = actions::decode(action, observation, layout);
        if let Some(provisioner) = &mut self.node_provisioner {
            let plan = provisioner.provision(observation);
            chosen.pool_delta = plan.pool_delta;
            chosen.drain = plan.drain;
        }
        chosen
    }

    fn record_pre_step(&mut self, chosen: &Actions) {
        let observation = self.simulation.observation();
        fill_demands(&mut self.demands, self.workload.as_ref(), observation.now);
        self.recorder.record(observation, chosen, &self.demands);
    }

    fn sample_post_step(&mut self, chosen: &Actions) -> TickSample {
        let observation = self.simulation.observation();
        fill_demands(&mut self.demands, self.workload.as_ref(), observation.now);
        self.recorder.sample(observation, chosen, &self.demands)
    }
}

fn fill_demands(demands: &mut Vec<ServiceDemand>, workload: &dyn WorkloadSource, now: SimTime) {
    demands.clear();
    demands.extend(
        (0..)
            .map(ServiceId)
            .take(workload.services())
            .map(|service| workload.demand(service, now)),
    );
}
