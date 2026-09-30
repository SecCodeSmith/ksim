use ksim_bench::scenario::Scenario;

use crate::{env::Env, error::EnvError, layout::EnvConfig};

/// `num_envs` independent `Env`s stepped in parallel (rayon) into caller-provided buffers laid
/// out env-major: observations `num_envs * observation_len`, actions
/// `num_envs * action_nvec.len()`, rewards/terminated/truncated `num_envs`.
///
/// Autoreset follows Gymnasium's `NEXT_STEP` mode: the step after an env reports
/// `truncated` (or `terminated`) resets that env instead of stepping it, ignores its action,
/// writes the reset observation, reward 0 and both flags false. Episode `e` (0-based) of env
/// `i` uses seed `seed + i + e * num_envs`, where `seed` is the last `reset` seed. All buffer
/// lengths and actions are validated before any env is stepped.
#[derive(Debug)]
pub struct VectorEnv {
    envs: Vec<Env>,
}

impl VectorEnv {
    /// # Errors
    /// `NoEnvironments` for zero envs, or `Env::new` errors.
    pub fn new(scenario: &Scenario, config: &EnvConfig, num_envs: usize) -> Result<Self, EnvError> {
        let _ = (scenario, config, num_envs);
        todo!()
    }

    #[must_use]
    pub fn num_envs(&self) -> usize {
        self.envs.len()
    }

    #[must_use]
    pub fn first(&self) -> &Env {
        &self.envs[0]
    }

    /// # Errors
    /// `BufferLength`, or any env's reset error.
    pub fn reset(&mut self, seed: u64, observations: &mut [f32]) -> Result<(), EnvError> {
        let _ = (seed, observations);
        todo!()
    }

    /// # Errors
    /// `BufferLength` or `ActionOutOfRange` (index into the flat action buffer).
    pub fn step(
        &mut self,
        actions: &[u32],
        observations: &mut [f32],
        rewards: &mut [f32],
        terminated: &mut [bool],
        truncated: &mut [bool],
    ) -> Result<(), EnvError> {
        let _ = (actions, observations, rewards, terminated, truncated);
        todo!()
    }
}
