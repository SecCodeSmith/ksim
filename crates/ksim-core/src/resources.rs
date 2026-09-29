pub const MIB: u64 = 1 << 20;
pub const GIB: u64 = 1 << 30;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Resources {
    pub cpu_millis: u64,
    pub memory_bytes: u64,
}

impl Resources {
    pub const ZERO: Self = Self::new(0, 0);

    #[must_use]
    pub const fn new(cpu_millis: u64, memory_bytes: u64) -> Self {
        Self {
            cpu_millis,
            memory_bytes,
        }
    }

    #[must_use]
    pub fn fits_in(self, available: Resources) -> bool {
        let _ = available;
        todo!()
    }

    #[must_use]
    pub fn checked_sub(self, other: Resources) -> Option<Resources> {
        let _ = other;
        todo!()
    }

    #[must_use]
    pub fn saturating_add(self, other: Resources) -> Resources {
        let _ = other;
        todo!()
    }
}
