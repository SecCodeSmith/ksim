use crate::time::SimTime;

/// Min-queue of timed events; events scheduled for the same time pop in insertion order.
#[derive(Debug)]
pub struct EventQueue<E> {
    pending: Vec<(SimTime, u64, E)>,
}

impl<E> Default for EventQueue<E> {
    fn default() -> Self {
        Self::new()
    }
}

impl<E> EventQueue<E> {
    #[must_use]
    pub fn new() -> Self {
        Self {
            pending: Vec::new(),
        }
    }

    pub fn push(&mut self, at: SimTime, event: E) {
        let _ = (at, event, &self.pending);
        todo!()
    }

    /// Pops the earliest event whose time is `<= now`.
    pub fn pop_due(&mut self, now: SimTime) -> Option<(SimTime, E)> {
        let _ = now;
        todo!()
    }

    #[must_use]
    pub fn next_time(&self) -> Option<SimTime> {
        todo!()
    }

    #[must_use]
    pub fn len(&self) -> usize {
        todo!()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        todo!()
    }
}
