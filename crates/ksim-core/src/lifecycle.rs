use std::fmt::Debug;

use rand_chacha::ChaCha8Rng;

use crate::time::SimDuration;

pub trait LatencyModel: Debug + Send {
    fn pod_startup(&self, rng: &mut ChaCha8Rng) -> SimDuration;
    fn node_provisioning(&self, rng: &mut ChaCha8Rng) -> SimDuration;
    fn termination(&self, rng: &mut ChaCha8Rng) -> SimDuration;
}

/// Pod startup = `image_pull` + uniform(`pod_start_min`, `pod_start_max`); defaults follow the
/// Kubernetes SIG-scalability pod startup SLO (p99 <= 5 s excluding image pull).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DefaultLatencies {
    pub image_pull: SimDuration,
    pub pod_start_min: SimDuration,
    pub pod_start_max: SimDuration,
    pub node_provisioning_min: SimDuration,
    pub node_provisioning_max: SimDuration,
    pub termination: SimDuration,
}

impl Default for DefaultLatencies {
    fn default() -> Self {
        todo!()
    }
}

impl LatencyModel for DefaultLatencies {
    fn pod_startup(&self, rng: &mut ChaCha8Rng) -> SimDuration {
        let _ = rng;
        todo!()
    }

    fn node_provisioning(&self, rng: &mut ChaCha8Rng) -> SimDuration {
        let _ = rng;
        todo!()
    }

    fn termination(&self, rng: &mut ChaCha8Rng) -> SimDuration {
        let _ = rng;
        todo!()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FixedLatencies {
    pub pod_startup: SimDuration,
    pub node_provisioning: SimDuration,
    pub termination: SimDuration,
}

impl LatencyModel for FixedLatencies {
    fn pod_startup(&self, rng: &mut ChaCha8Rng) -> SimDuration {
        let _ = rng;
        todo!()
    }

    fn node_provisioning(&self, rng: &mut ChaCha8Rng) -> SimDuration {
        let _ = rng;
        todo!()
    }

    fn termination(&self, rng: &mut ChaCha8Rng) -> SimDuration {
        let _ = rng;
        todo!()
    }
}

/// Kubelet parity: restart delay starts at 10 s and doubles per restart, capped at 300 s.
#[must_use]
pub fn crash_loop_backoff(restarts: u32) -> SimDuration {
    let _ = restarts;
    todo!()
}
