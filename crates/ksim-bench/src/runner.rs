use ksim_metrics::report::EpisodeReport;

use crate::{
    policy::Policy,
    scenario::{Scenario, ScenarioError},
};

/// Runs one episode: for each of `scenario.steps()` ticks, observe, let the policy act,
/// record the trace with the true demand, then step; evaluates `standard_metrics(slo)`.
///
/// # Errors
/// When the simulation cannot be built from the scenario.
pub fn run_episode(
    scenario: &Scenario,
    policy: Policy,
    seed: u64,
) -> Result<EpisodeReport, ScenarioError> {
    let _ = (scenario, policy, seed);
    todo!()
}

/// Episodes for `seeds` consecutive seeds starting at `first_seed`, in seed order.
///
/// # Errors
/// When any episode fails.
pub fn run_seeds(
    scenario: &Scenario,
    policy: Policy,
    first_seed: u64,
    seeds: u32,
) -> Result<Vec<EpisodeReport>, ScenarioError> {
    let _ = (scenario, policy, first_seed, seeds);
    todo!()
}
