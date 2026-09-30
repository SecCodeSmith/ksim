use std::time::{Duration, Instant};

use ksim_bench::scenario::Scenario;

use rayon::prelude::*;

use crate::{
    actions,
    env::{Env, StepOutcome},
    error::EnvError,
    layout::EnvConfig,
};

/// `num_envs` independent `Env`s stepped into caller-provided buffers laid out env-major:
/// observations `num_envs * observation_len`, actions `num_envs * action_nvec.len()`,
/// rewards/terminated/truncated `num_envs`.
///
/// `with_parallel_threshold` overrides `PARALLEL_THRESHOLD` (`Duration::ZERO` always runs in
/// parallel, `Duration::MAX` never does). A step runs the envs on the calling thread when the previous step's measured serial cost
/// (its per-env step time times `num_envs`) is below `PARALLEL_THRESHOLD`, and in parallel
/// with rayon otherwise: small scenarios step in about a microsecond, far less than waking
/// the thread pool. The schedule changes only speed; results are identical either way.
///
/// Autoreset follows Gymnasium's `NEXT_STEP` mode: the step after an env reports
/// `truncated` (or `terminated`) resets that env instead of stepping it, ignores its action,
/// writes the reset observation, reward 0 and both flags false. Episode `e` (0-based) of env
/// `i` uses seed `seed + i + e * num_envs`, where `seed` is the last `reset` seed. All buffer
/// lengths and actions are validated before any env is stepped.
#[derive(Debug)]
pub struct VectorEnv {
    envs: Vec<Env>,
    episodes: Vec<u64>,
    seed: u64,
    parallel_threshold: Duration,
    serial_cost: Option<Duration>,
}

/// Estimated serial cost of one vector step below which `VectorEnv` skips the thread pool.
pub const PARALLEL_THRESHOLD: Duration = Duration::from_micros(200);

impl VectorEnv {
    /// # Errors
    /// `NoEnvironments` for zero envs, or `Env::new` errors.
    pub fn new(scenario: &Scenario, config: &EnvConfig, num_envs: usize) -> Result<Self, EnvError> {
        if num_envs == 0 {
            return Err(EnvError::NoEnvironments);
        }
        let envs = (0..num_envs)
            .map(|_| Env::new(scenario, config))
            .collect::<Result<_, _>>()?;
        Ok(Self {
            envs,
            episodes: vec![0; num_envs],
            seed: 0,
            parallel_threshold: PARALLEL_THRESHOLD,
            serial_cost: None,
        })
    }

    #[must_use]
    pub fn with_parallel_threshold(mut self, threshold: Duration) -> Self {
        self.parallel_threshold = threshold;
        self
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
        let observation_len = self.first().layout().observation_len();
        check_len(
            "observation",
            observations.len(),
            self.num_envs() * observation_len,
        )?;
        self.seed = seed;
        self.episodes.fill(0);
        self.envs
            .par_iter_mut()
            .zip(observations.par_chunks_mut(observation_len))
            .enumerate()
            .try_for_each(|(index, (env, observation))| {
                env.reset(seed + index as u64, observation)
            })?;
        Ok(())
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
        self.validate_step(actions, observations.len(), rewards.len())?;
        check_len("terminated", terminated.len(), self.num_envs())?;
        check_len("truncated", truncated.len(), self.num_envs())?;
        let layout = self.first().layout();
        let (observation_len, action_len) = (layout.observation_len(), layout.action_len());
        let lane_layout = LaneLayout {
            seed: self.seed,
            num_envs: self.envs.len() as u64,
        };
        let serial_cost = if self.should_run_in_parallel() {
            let probe_cost = (
                &mut self.envs,
                &mut self.episodes,
                actions.par_chunks(action_len),
                observations.par_chunks_mut(observation_len),
                rewards,
                terminated,
                truncated,
            )
                .into_par_iter()
                .enumerate()
                .map(|(index, slot)| Slot::from(slot).advance(&lane_layout.lane(index), index == 0))
                .try_reduce(|| Duration::ZERO, |a, b| Ok(a.max(b)))?;
            probe_cost * u32::try_from(self.envs.len()).unwrap_or(u32::MAX)
        } else {
            let started = Instant::now();
            self.envs
                .iter_mut()
                .zip(self.episodes.iter_mut())
                .zip(actions.chunks(action_len))
                .zip(observations.chunks_mut(observation_len))
                .zip(rewards)
                .zip(terminated)
                .zip(truncated)
                .enumerate()
                .try_for_each(|(index, slot)| {
                    Slot::from(slot)
                        .advance(&lane_layout.lane(index), false)
                        .map(drop)
                })?;
            started.elapsed()
        };
        self.serial_cost = Some(serial_cost);
        Ok(())
    }

    fn should_run_in_parallel(&self) -> bool {
        self.serial_cost.unwrap_or(Duration::ZERO) >= self.parallel_threshold
    }

    fn validate_step(
        &self,
        actions: &[u32],
        observations_len: usize,
        rewards_len: usize,
    ) -> Result<(), EnvError> {
        let layout = self.first().layout();
        let num_envs = self.num_envs();
        check_len(
            "observation",
            observations_len,
            num_envs * layout.observation_len(),
        )?;
        check_len("action", actions.len(), num_envs * layout.action_len())?;
        check_len("reward", rewards_len, num_envs)?;
        if self.envs.iter().any(|env| !env.has_started()) {
            return Err(EnvError::NeedsReset);
        }
        actions
            .chunks(layout.action_len())
            .enumerate()
            .try_for_each(|(env, action)| {
                actions::validate(action, layout)
                    .map_err(|error| offset_index(error, env * layout.action_len()))
            })
    }
}

#[allow(clippy::cast_possible_truncation)]
fn reward_as_f32(reward: f64) -> f32 {
    reward as f32
}

fn check_len(buffer: &'static str, actual: usize, expected: usize) -> Result<(), EnvError> {
    if actual == expected {
        Ok(())
    } else {
        Err(EnvError::BufferLength {
            buffer,
            expected,
            actual,
        })
    }
}

fn offset_index(error: EnvError, offset: usize) -> EnvError {
    match error {
        EnvError::ActionOutOfRange {
            index,
            value,
            bound,
        } => EnvError::ActionOutOfRange {
            index: index + offset,
            value,
            bound,
        },
        other => other,
    }
}

#[derive(Clone, Copy)]
struct LaneLayout {
    seed: u64,
    num_envs: u64,
}

impl LaneLayout {
    fn lane(self, index: usize) -> Lane {
        Lane {
            index,
            seed: self.seed,
            num_envs: self.num_envs,
        }
    }
}

/// One env with its per-env slices of the caller's buffers.
struct Slot<'a> {
    env: &'a mut Env,
    episode: &'a mut u64,
    action: &'a [u32],
    observation: &'a mut [f32],
    reward: &'a mut f32,
    terminated: &'a mut bool,
    truncated: &'a mut bool,
}

type FlatSlot<'a> = (
    &'a mut Env,
    &'a mut u64,
    &'a [u32],
    &'a mut [f32],
    &'a mut f32,
    &'a mut bool,
    &'a mut bool,
);

type NestedSlot<'a> = (
    (
        (
            (((&'a mut Env, &'a mut u64), &'a [u32]), &'a mut [f32]),
            &'a mut f32,
        ),
        &'a mut bool,
    ),
    &'a mut bool,
);

impl<'a> From<FlatSlot<'a>> for Slot<'a> {
    fn from(
        (env, episode, action, observation, reward, terminated, truncated): FlatSlot<'a>,
    ) -> Self {
        Self {
            env,
            episode,
            action,
            observation,
            reward,
            terminated,
            truncated,
        }
    }
}

impl<'a> From<NestedSlot<'a>> for Slot<'a> {
    fn from(
        ((((((env, episode), action), observation), reward), terminated), truncated): NestedSlot<
            'a,
        >,
    ) -> Self {
        Self::from((
            env,
            episode,
            action,
            observation,
            reward,
            terminated,
            truncated,
        ))
    }
}

impl Slot<'_> {
    /// Advances the env and writes its results; returns the step time when `timed`, else zero.
    fn advance(self, lane: &Lane, timed: bool) -> Result<Duration, EnvError> {
        let started = timed.then(Instant::now);
        let outcome = lane.advance(self.env, self.episode, self.action, self.observation)?;
        *self.reward = reward_as_f32(outcome.reward);
        *self.terminated = outcome.terminated;
        *self.truncated = outcome.truncated;
        Ok(started.map_or(Duration::ZERO, |started| started.elapsed()))
    }
}

/// Position of one env in the batch, from which its autoreset seeds derive.
struct Lane {
    index: usize,
    seed: u64,
    num_envs: u64,
}

impl Lane {
    /// Steps `env`, or under `NEXT_STEP` autoreset starts its next episode when the last one
    /// ended.
    fn advance(
        &self,
        env: &mut Env,
        episode: &mut u64,
        action: &[u32],
        observation: &mut [f32],
    ) -> Result<StepOutcome, EnvError> {
        if !env.is_over() {
            return env.step(action, observation);
        }
        *episode += 1;
        env.reset(
            self.seed + self.index as u64 + *episode * self.num_envs,
            observation,
        )?;
        Ok(StepOutcome {
            reward: 0.0,
            terminated: false,
            truncated: false,
        })
    }
}
