use std::fmt::Debug;

use crate::{actions::Actions, observation::Observation};

/// Anything that turns an observation into actions: Kubernetes baselines and learned policies.
pub trait Autoscaler: Debug + Send {
    fn act(&mut self, observation: &Observation) -> Actions;
}
