use ksim_bench::scenario::ScenarioError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum EnvError {
    #[error(transparent)]
    Scenario(#[from] ScenarioError),
    #[error("the scenario has {services} services but only {slots} service slots")]
    ServiceSlots { services: usize, slots: usize },
    #[error("a vector environment needs at least one environment")]
    NoEnvironments,
    #[error("{buffer} buffer has length {actual}, expected {expected}")]
    BufferLength {
        buffer: &'static str,
        expected: usize,
        actual: usize,
    },
    #[error("action {index} is {value}, expected less than {bound}")]
    ActionOutOfRange {
        index: usize,
        value: u32,
        bound: u32,
    },
    #[error("the episode is over or was never started; call reset")]
    NeedsReset,
}
