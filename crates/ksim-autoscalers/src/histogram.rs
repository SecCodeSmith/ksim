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

/// Weight below which a histogram counts as empty (upstream `epsilon`).
const EMPTY_WEIGHT: f64 = 1e-15;
/// Exponent beyond which the reference timestamp is shifted, keeping factors below ~1e9.
const MAX_DECAY_EXPONENT: f64 = 30.0;

impl HistogramOptions {
    /// Upstream CPU histogram in millicores: first bucket 10m (0.01 core), ratio 1.05, max 1000 cores.
    #[must_use]
    pub fn cpu_millis() -> Self {
        Self {
            first_bucket_size: 10.0,
            ratio: 1.05,
            max_value: 1_000_000.0,
        }
    }

    /// Upstream memory histogram in bytes: first bucket 1e7, ratio 1.05, max 1e12.
    #[must_use]
    pub fn memory_bytes() -> Self {
        Self {
            first_bucket_size: 1e7,
            ratio: 1.05,
            max_value: 1e12,
        }
    }

    #[must_use]
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    pub fn bucket_count(&self) -> usize {
        self.bucket_index_of(self.max_value).ceil() as usize + 1
    }

    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn bucket_start(&self, bucket: usize) -> f64 {
        if bucket == 0 {
            return 0.0;
        }
        let growth = self.ratio.powf(bucket as f64);
        self.first_bucket_size * (growth - 1.0) / (self.ratio - 1.0)
    }

    /// Bucket containing `value`; values below zero map to bucket 0, above `max_value` to the last.
    #[must_use]
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    pub fn find_bucket(&self, value: f64) -> usize {
        let last = self.bucket_count() - 1;
        if value.is_nan() || value < self.first_bucket_size {
            return 0;
        }
        if value >= self.max_value {
            return last;
        }
        let estimate = (self.bucket_index_of(value).floor() as usize).min(last);
        self.correct_rounding(estimate, value, last)
    }

    fn bucket_index_of(&self, value: f64) -> f64 {
        let scaled = value * (self.ratio - 1.0) / self.first_bucket_size + 1.0;
        scaled.ln() / self.ratio.ln()
    }

    fn correct_rounding(&self, mut bucket: usize, value: f64, last: usize) -> usize {
        while bucket > 0 && self.bucket_start(bucket) > value {
            bucket -= 1;
        }
        while bucket < last && self.bucket_start(bucket + 1) <= value {
            bucket += 1;
        }
        bucket
    }
}

/// Upstream `DecayingHistogram`: a sample added at time `t` carries weight
/// `weight * 2^((t - reference) / half_life)`, so older samples lose half their relative
/// weight every `half_life`.
#[derive(Debug, Clone)]
pub struct DecayingHistogram {
    options: HistogramOptions,
    half_life: SimDuration,
    reference: SimTime,
    weights: Vec<f64>,
}

impl DecayingHistogram {
    #[must_use]
    pub fn new(options: HistogramOptions, half_life: SimDuration) -> Self {
        Self {
            options,
            half_life,
            reference: SimTime::ZERO,
            weights: Vec::new(),
        }
    }

    pub fn add_sample(&mut self, value: f64, weight: f64, time: SimTime) {
        self.shift_reference_if_needed(time);
        self.add_weight(value, weight * self.decay_factor(time));
    }

    /// Removes a sample previously added with the same arguments (used to replace a peak).
    pub fn subtract_sample(&mut self, value: f64, weight: f64, time: SimTime) {
        self.shift_reference_if_needed(time);
        self.add_weight(value, -weight * self.decay_factor(time));
    }

    /// End of the first bucket whose cumulative weight reaches `p * total` (upstream returns
    /// the start of the next bucket); the last bucket returns its start. Empty: 0.
    #[must_use]
    pub fn percentile(&self, p: f64) -> f64 {
        if self.is_empty() {
            return 0.0;
        }
        let threshold = p * self.total_weight();
        let last = self.options.bucket_count() - 1;
        let mut cumulative = 0.0;
        let mut chosen = last;
        for (bucket, &weight) in self.weights.iter().enumerate() {
            if weight <= 0.0 {
                continue;
            }
            cumulative += weight;
            chosen = bucket;
            if cumulative >= threshold {
                break;
            }
        }
        if chosen < last {
            self.options.bucket_start(chosen + 1)
        } else {
            self.options.bucket_start(last)
        }
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.total_weight() < EMPTY_WEIGHT
    }

    fn total_weight(&self) -> f64 {
        self.weights.iter().sum()
    }

    fn add_weight(&mut self, value: f64, delta: f64) {
        let bucket = self.options.find_bucket(value);
        if self.weights.len() <= bucket {
            self.weights.resize(bucket + 1, 0.0);
        }
        self.weights[bucket] = (self.weights[bucket] + delta).max(0.0);
    }

    fn decay_exponent(&self, time: SimTime) -> f64 {
        let half_life = self.half_life.as_secs_f64();
        if half_life <= 0.0 {
            return 0.0;
        }
        (time.as_secs_f64() - self.reference.as_secs_f64()) / half_life
    }

    fn decay_factor(&self, time: SimTime) -> f64 {
        self.decay_exponent(time).exp2()
    }

    fn shift_reference_if_needed(&mut self, time: SimTime) {
        let exponent = self.decay_exponent(time);
        if exponent <= MAX_DECAY_EXPONENT {
            return;
        }
        let rescale = (-exponent).exp2();
        for weight in &mut self.weights {
            *weight *= rescale;
        }
        self.reference = time;
    }
}
