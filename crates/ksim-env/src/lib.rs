//! Reinforcement-learning environment over the simulator: fixed-size `f32` observations,
//! `MultiDiscrete` actions, pluggable rewards, and a parallel vector environment writing into
//! caller-provided buffers.

pub mod actions;
pub mod env;
pub mod error;
pub mod layout;
pub mod observation;
pub mod reward;
pub mod vector;

pub use env::{Env, StepOutcome};
pub use error::EnvError;
pub use layout::{Control, EnvConfig, Layout, WeightedRewardWeights};
pub use reward::{RewardContext, RewardFn, WeightedReward};
pub use vector::VectorEnv;
