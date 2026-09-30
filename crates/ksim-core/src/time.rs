use std::ops::{Add, Sub};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct SimTime(u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct SimDuration(u64);

impl SimTime {
    pub const ZERO: Self = Self(0);

    #[must_use]
    pub const fn from_millis(millis: u64) -> Self {
        Self(millis)
    }

    #[must_use]
    pub const fn from_secs(secs: u64) -> Self {
        Self(secs.saturating_mul(1000))
    }

    #[must_use]
    pub const fn as_millis(self) -> u64 {
        self.0
    }

    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn as_secs_f64(self) -> f64 {
        self.0 as f64 / 1000.0
    }
}

impl SimDuration {
    pub const ZERO: Self = Self(0);

    #[must_use]
    pub const fn from_millis(millis: u64) -> Self {
        Self(millis)
    }

    #[must_use]
    pub const fn from_secs(secs: u64) -> Self {
        Self(secs.saturating_mul(1000))
    }

    #[must_use]
    pub const fn as_millis(self) -> u64 {
        self.0
    }

    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn as_secs_f64(self) -> f64 {
        self.0 as f64 / 1000.0
    }

    #[must_use]
    pub fn saturating_mul(self, factor: u64) -> Self {
        Self(self.0.saturating_mul(factor))
    }
}

impl Add<SimDuration> for SimTime {
    type Output = SimTime;

    fn add(self, rhs: SimDuration) -> SimTime {
        SimTime(self.0.saturating_add(rhs.0))
    }
}

/// Saturates at zero when `rhs` is later than `self`.
impl Sub for SimTime {
    type Output = SimDuration;

    fn sub(self, rhs: SimTime) -> SimDuration {
        SimDuration(self.0.saturating_sub(rhs.0))
    }
}
