use rand_chacha::ChaCha8Rng;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Concern {
    Lifecycle,
    Workload,
    Scheduler,
}

/// Independent reproducible random streams, one per concern, so adding draws in one
/// concern never shifts the sequence observed by another.
#[derive(Debug, Clone)]
pub struct RngStreams {
    seed: u64,
}

impl RngStreams {
    #[must_use]
    pub fn new(seed: u64) -> Self {
        Self { seed }
    }

    #[must_use]
    pub fn stream(&self, concern: Concern) -> ChaCha8Rng {
        let _ = (self.seed, concern);
        todo!()
    }
}
