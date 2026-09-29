use std::fmt::Debug;

use rand::RngExt;
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
        Self {
            image_pull: SimDuration::ZERO,
            pod_start_min: SimDuration::from_secs(1),
            pod_start_max: SimDuration::from_secs(5),
            node_provisioning_min: SimDuration::from_secs(60),
            node_provisioning_max: SimDuration::from_secs(180),
            termination: SimDuration::from_secs(30),
        }
    }
}

impl LatencyModel for DefaultLatencies {
    fn pod_startup(&self, rng: &mut ChaCha8Rng) -> SimDuration {
        let start = uniform(rng, self.pod_start_min, self.pod_start_max);
        SimDuration::from_millis(
            self.image_pull
                .as_millis()
                .saturating_add(start.as_millis()),
        )
    }

    fn node_provisioning(&self, rng: &mut ChaCha8Rng) -> SimDuration {
        uniform(rng, self.node_provisioning_min, self.node_provisioning_max)
    }

    fn termination(&self, _rng: &mut ChaCha8Rng) -> SimDuration {
        self.termination
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FixedLatencies {
    pub pod_startup: SimDuration,
    pub node_provisioning: SimDuration,
    pub termination: SimDuration,
}

impl LatencyModel for FixedLatencies {
    fn pod_startup(&self, _rng: &mut ChaCha8Rng) -> SimDuration {
        self.pod_startup
    }

    fn node_provisioning(&self, _rng: &mut ChaCha8Rng) -> SimDuration {
        self.node_provisioning
    }

    fn termination(&self, _rng: &mut ChaCha8Rng) -> SimDuration {
        self.termination
    }
}

/// Kubelet parity: restart delay starts at 10 s and doubles per restart, capped at 300 s.
#[must_use]
pub fn crash_loop_backoff(restarts: u32) -> SimDuration {
    const INITIAL_SECS: u64 = 10;
    const CAP_SECS: u64 = 300;
    let doublings = restarts.min(6);
    SimDuration::from_secs((INITIAL_SECS << doublings).min(CAP_SECS))
}

fn uniform(rng: &mut ChaCha8Rng, min: SimDuration, max: SimDuration) -> SimDuration {
    let low = min.as_millis();
    let high = max.as_millis().max(low);
    SimDuration::from_millis(rng.random_range(low..=high))
}
