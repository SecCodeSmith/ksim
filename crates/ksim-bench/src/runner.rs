use ksim_core::{autoscaler::Autoscaler, engine::Simulation, workload::ServiceDemand};
use ksim_metrics::{
    metric::standard_metrics,
    report::{EpisodeReport, evaluate},
    trace::TraceRecorder,
};

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
    let mut simulation = scenario.build_simulation(seed)?;
    let mut autoscaler = policy.build(scenario, seed);
    let trace = record_trace(scenario, seed, &mut simulation, autoscaler.as_mut());
    Ok(EpisodeReport {
        scenario: scenario.name.clone(),
        policy: policy.name().to_owned(),
        seed,
        metrics: evaluate(&trace, &standard_metrics(scenario.slo)),
    })
}

fn record_trace(
    scenario: &Scenario,
    seed: u64,
    simulation: &mut Simulation,
    autoscaler: &mut dyn Autoscaler,
) -> ksim_metrics::trace::EpisodeTrace {
    let workload = scenario.workload(seed);
    let prices = scenario
        .pools
        .iter()
        .map(|pool| pool.price_per_hour)
        .collect();
    let mut recorder = TraceRecorder::new(scenario.tick(), prices);
    for _ in 0..scenario.steps() {
        let observation = simulation.observation().clone();
        let actions = autoscaler.act(&observation);
        let demands: Vec<ServiceDemand> = scenario
            .service_ids()
            .map(|service| workload.demand(service, observation.now))
            .collect();
        recorder.record(&observation, &actions, &demands);
        simulation.step(&actions);
    }
    recorder.finish()
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
    (first_seed..first_seed.saturating_add(u64::from(seeds)))
        .map(|seed| run_episode(scenario, policy, seed))
        .collect()
}
