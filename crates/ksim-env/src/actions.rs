use ksim_core::{actions::Actions, observation::Observation};

use crate::{error::EnvError, layout::Layout};

/// Checks `action` against `layout.action_nvec()` (length, then each value below its bound).
///
/// # Errors
/// `BufferLength` (buffer `"action"`) or `ActionOutOfRange`.
pub fn validate(action: &[u32], layout: &Layout) -> Result<(), EnvError> {
    let _ = (action, layout);
    todo!()
}

/// Decodes a validated action: every real service gets `Some(desired + action - k)` clamped
/// to its `[min_replicas, max_replicas]`; padding slots are ignored. With `ReplicasAndNodes`
/// `pool_delta` holds `action - 1` per pool; otherwise it is empty. Nothing else is set.
#[must_use]
pub fn decode(action: &[u32], observation: &Observation, layout: &Layout) -> Actions {
    let _ = (action, observation, layout);
    todo!()
}
