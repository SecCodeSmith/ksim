//! Performance models translating demand and allocation into latency, errors and throttling.

pub mod queueing;
pub mod utilization;

/// Resource quantities stay far below 2^52, so the conversion is exact in practice.
#[allow(clippy::cast_precision_loss)]
pub(crate) fn quantity_as_f64(quantity: u64) -> f64 {
    quantity as f64
}
