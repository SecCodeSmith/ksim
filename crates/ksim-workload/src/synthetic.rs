use std::f64::consts::TAU;
use std::fmt::Debug;

use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use rand_distr::{Distribution, Normal};

use ksim_core::{
    ids::ServiceId,
    time::{SimDuration, SimTime},
    workload::{ServiceDemand, WorkloadSource},
};

/// 64-bit golden-ratio constant (SplitMix64) decorrelating adjacent timestamps.
const GOLDEN_GAMMA: u64 = 0x9E37_79B9_7F4A_7C15;

/// Request rate (requests per second) as a pure function of time.
pub trait DemandCurve: Debug + Send {
    fn load(&self, now: SimTime) -> f64;
}

/// Maps a request rate to resource demand:
/// `cpu = rate * cpu_millis_per_request`, `memory = memory_base_bytes + rate * memory_bytes_per_rps`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DemandProfile {
    pub cpu_millis_per_request: f64,
    pub memory_base_bytes: f64,
    pub memory_bytes_per_rps: f64,
}

impl DemandProfile {
    #[must_use]
    pub fn demand(&self, request_rate: f64) -> ServiceDemand {
        ServiceDemand {
            request_rate,
            cpu_millis: request_rate * self.cpu_millis_per_request,
            memory_bytes: self.memory_base_bytes + request_rate * self.memory_bytes_per_rps,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Constant {
    pub rate: f64,
}

/// `before` until `at`, then `after`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Step {
    pub before: f64,
    pub after: f64,
    pub at: SimTime,
}

/// Linear from `from` at `start` to `to` at `end`, flat outside.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ramp {
    pub from: f64,
    pub to: f64,
    pub start: SimTime,
    pub end: SimTime,
}

/// `mean + amplitude * sin(2π t / period)` plus Gaussian noise with `noise_sd`, clamped at 0.
/// Noise is derived from `(seed, now)` so the curve stays a pure function of time.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sinusoid {
    pub mean: f64,
    pub amplitude: f64,
    pub period: SimDuration,
    pub noise_sd: f64,
    pub seed: u64,
}

/// `base` before `start`; afterwards `base + (peak - base) * exp(-(t - start) / decay)`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FlashCrowd {
    pub base: f64,
    pub peak: f64,
    pub start: SimTime,
    pub decay: SimDuration,
}

impl Sinusoid {
    fn noise_at(&self, now: SimTime) -> f64 {
        if self.noise_sd == 0.0 {
            return 0.0;
        }
        let mut rng =
            ChaCha8Rng::seed_from_u64(self.seed ^ now.as_millis().wrapping_mul(GOLDEN_GAMMA));
        Normal::new(0.0, self.noise_sd).map_or(0.0, |normal| normal.sample(&mut rng))
    }
}

impl DemandCurve for Constant {
    fn load(&self, now: SimTime) -> f64 {
        let _ = now;
        self.rate
    }
}

impl DemandCurve for Step {
    fn load(&self, now: SimTime) -> f64 {
        if now < self.at {
            self.before
        } else {
            self.after
        }
    }
}

impl DemandCurve for Ramp {
    fn load(&self, now: SimTime) -> f64 {
        if now <= self.start {
            return self.from;
        }
        if now >= self.end {
            return self.to;
        }
        let elapsed = now.as_secs_f64() - self.start.as_secs_f64();
        let span = self.end.as_secs_f64() - self.start.as_secs_f64();
        self.from + (self.to - self.from) * elapsed / span
    }
}

impl DemandCurve for Sinusoid {
    fn load(&self, now: SimTime) -> f64 {
        let phase = TAU * now.as_secs_f64() / self.period.as_secs_f64();
        let noiseless = self.mean + self.amplitude * phase.sin();
        (noiseless + self.noise_at(now)).max(0.0)
    }
}

impl DemandCurve for FlashCrowd {
    fn load(&self, now: SimTime) -> f64 {
        if now < self.start {
            return self.base;
        }
        let elapsed = now.as_secs_f64() - self.start.as_secs_f64();
        self.base + (self.peak - self.base) * (-elapsed / self.decay.as_secs_f64()).exp()
    }
}

/// One curve and profile per service, service `i` using entry `i`.
#[derive(Debug)]
pub struct CurveWorkload {
    services: Vec<(Box<dyn DemandCurve>, DemandProfile)>,
}

impl CurveWorkload {
    #[must_use]
    pub fn new(services: Vec<(Box<dyn DemandCurve>, DemandProfile)>) -> Self {
        Self { services }
    }
}

impl WorkloadSource for CurveWorkload {
    fn services(&self) -> usize {
        self.services.len()
    }

    fn demand(&self, service: ServiceId, now: SimTime) -> ServiceDemand {
        let (curve, profile) = &self.services[service.index()];
        profile.demand(curve.load(now))
    }
}
