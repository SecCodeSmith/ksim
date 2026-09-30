use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Concern {
    Lifecycle,
    Workload,
    Scheduler,
}

impl Concern {
    const fn stream_index(self) -> u64 {
        match self {
            Concern::Lifecycle => 0,
            Concern::Workload => 1,
            Concern::Scheduler => 2,
        }
    }
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
        let mut rng = ChaCha8Rng::seed_from_u64(self.seed);
        rng.set_stream(concern.stream_index());
        rng
    }
}
