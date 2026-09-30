use std::{cmp::Reverse, collections::BinaryHeap};

use crate::time::SimTime;

/// Min-queue of timed events; events scheduled for the same time pop in insertion order.
#[derive(Debug)]
pub struct EventQueue<E> {
    pending: BinaryHeap<Entry<E>>,
    next_sequence: u64,
}

#[derive(Debug)]
struct Entry<E> {
    order: Reverse<(SimTime, u64)>,
    event: E,
}

impl<E> PartialEq for Entry<E> {
    fn eq(&self, other: &Self) -> bool {
        self.order == other.order
    }
}

impl<E> Eq for Entry<E> {}

impl<E> PartialOrd for Entry<E> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<E> Ord for Entry<E> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.order.cmp(&other.order)
    }
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
            pending: BinaryHeap::new(),
            next_sequence: 0,
        }
    }

    pub fn push(&mut self, at: SimTime, event: E) {
        let order = Reverse((at, self.next_sequence));
        self.next_sequence += 1;
        self.pending.push(Entry { order, event });
    }

    /// Pops the earliest event whose time is `<= now`.
    pub fn pop_due(&mut self, now: SimTime) -> Option<(SimTime, E)> {
        if self.next_time()? > now {
            return None;
        }
        let Entry {
            order: Reverse((at, _)),
            event,
        } = self.pending.pop()?;
        Some((at, event))
    }

    #[must_use]
    pub fn next_time(&self) -> Option<SimTime> {
        self.pending.peek().map(|entry| entry.order.0.0)
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.pending.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.pending.is_empty()
    }
}
