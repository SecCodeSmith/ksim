use ksim_core::time::{SimDuration, SimTime};

/// Exponential bucket layout from the VPA recommender: bucket `i` starts at
/// `first_bucket_size * (ratio^i - 1) / (ratio - 1)`; the last bucket starts below
/// `max_value` and is unbounded.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HistogramOptions {
    pub first_bucket_size: f64,
    pub ratio: f64,
    pub max_value: f64,
}

impl HistogramOptions {
    /// Upstream CPU histogram in millicores: first bucket 10m (0.01 core), ratio 1.05, max 1000 cores.
    #[must_use]
    pub fn cpu_millis() -> Self {
        todo!()
    }

    /// Upstream memory histogram in bytes: first bucket 1e7, ratio 1.05, max 1e12.
    #[must_use]
    pub fn memory_bytes() -> Self {
        todo!()
    }

    #[must_use]
    pub fn bucket_count(&self) -> usize {
        todo!()
    }

    #[must_use]
    pub fn bucket_start(&self, bucket: usize) -> f64 {
        let _ = bucket;
        todo!()
    }

    /// Bucket containing `value`; values below zero map to bucket 0, above `max_value` to the last.
    #[must_use]
    pub fn find_bucket(&self, value: f64) -> usize {
        let _ = value;
        todo!()
    }
}

/// Upstream `DecayingHistogram`: a sample added at time `t` carries weight
/// `weight * 2^((t - reference) / half_life)`, so older samples lose half their relative
/// weight every `half_life`.
#[derive(Debug, Clone)]
pub struct DecayingHistogram {
    options: HistogramOptions,
    half_life: SimDuration,
    weights: Vec<f64>,
}

impl DecayingHistogram {
    #[must_use]
    pub fn new(options: HistogramOptions, half_life: SimDuration) -> Self {
        Self {
            options,
            half_life,
            weights: Vec::new(),
        }
    }

    pub fn add_sample(&mut self, value: f64, weight: f64, time: SimTime) {
        let _ = (
            value,
            weight,
            time,
            self.options,
            self.half_life,
            &self.weights,
        );
        todo!()
    }

    /// Removes a sample previously added with the same arguments (used to replace a peak).
    pub fn subtract_sample(&mut self, value: f64, weight: f64, time: SimTime) {
        let _ = (value, weight, time);
        todo!()
    }

    /// End of the first bucket whose cumulative weight reaches `p * total` (upstream returns
    /// the start of the next bucket); the last bucket returns its start. Empty: 0.
    #[must_use]
    pub fn percentile(&self, p: f64) -> f64 {
        let _ = p;
        todo!()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        todo!()
    }
}
