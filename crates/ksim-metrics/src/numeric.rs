#[expect(
    clippy::cast_precision_loss,
    reason = "counts stay far below 2^52, where usize converts to f64 exactly"
)]
pub(crate) fn count_f64(count: usize) -> f64 {
    count as f64
}

#[expect(
    clippy::cast_precision_loss,
    reason = "millicore quantities stay far below 2^52, where u64 converts to f64 exactly"
)]
pub(crate) fn quantity_f64(quantity: u64) -> f64 {
    quantity as f64
}
