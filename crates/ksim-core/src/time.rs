use std::ops::{Add, Sub};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct SimTime(u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct SimDuration(u64);

impl SimTime {
    pub const ZERO: Self = Self(0);

    #[must_use]
    pub const fn from_millis(millis: u64) -> Self {
        let _ = millis;
        todo!()
    }

    #[must_use]
    pub const fn from_secs(secs: u64) -> Self {
        let _ = secs;
        todo!()
    }

    #[must_use]
    pub const fn as_millis(self) -> u64 {
        todo!()
    }

    #[must_use]
    pub fn as_secs_f64(self) -> f64 {
        todo!()
    }
}

impl SimDuration {
    pub const ZERO: Self = Self(0);

    #[must_use]
    pub const fn from_millis(millis: u64) -> Self {
        let _ = millis;
        todo!()
    }

    #[must_use]
    pub const fn from_secs(secs: u64) -> Self {
        let _ = secs;
        todo!()
    }

    #[must_use]
    pub const fn as_millis(self) -> u64 {
        todo!()
    }

    #[must_use]
    pub fn as_secs_f64(self) -> f64 {
        todo!()
    }

    #[must_use]
    pub fn saturating_mul(self, factor: u64) -> Self {
        let _ = factor;
        todo!()
    }
}

impl Add<SimDuration> for SimTime {
    type Output = SimTime;

    fn add(self, rhs: SimDuration) -> SimTime {
        let _ = rhs;
        todo!()
    }
}

/// Saturates at zero when `rhs` is later than `self`.
impl Sub for SimTime {
    type Output = SimDuration;

    fn sub(self, rhs: SimTime) -> SimDuration {
        let _ = rhs;
        todo!()
    }
}
