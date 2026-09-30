use ksim_core::{actions::Actions, observation::Observation};

use crate::{
    error::EnvError,
    layout::{Control, Layout},
};

/// Checks `action` against `layout.action_nvec()` (length, then each value below its bound).
///
/// # Errors
/// `BufferLength` (buffer `"action"`) or `ActionOutOfRange`.
pub fn validate(action: &[u32], layout: &Layout) -> Result<(), EnvError> {
    let expected = layout.action_len();
    if action.len() != expected {
        return Err(EnvError::BufferLength {
            buffer: "action",
            expected,
            actual: action.len(),
        });
    }
    match action
        .iter()
        .enumerate()
        .find(|&(index, &value)| value >= layout.action_bound(index))
    {
        Some((index, &value)) => Err(EnvError::ActionOutOfRange {
            index,
            value,
            bound: layout.action_bound(index),
        }),
        None => Ok(()),
    }
}

/// Decodes a validated action: every real service gets `Some(desired + action - k)` clamped
/// to its `[min_replicas, max_replicas]`; padding slots are ignored. With `ReplicasAndNodes`
/// `pool_delta` holds `action - 1` per pool; otherwise it is empty. Nothing else is set.
#[must_use]
pub fn decode(action: &[u32], observation: &Observation, layout: &Layout) -> Actions {
    Actions {
        replicas: decode_replicas(action, observation, layout),
        pool_delta: decode_pool_delta(action, observation, layout),
        ..Actions::none()
    }
}

fn decode_replicas(action: &[u32], observation: &Observation, layout: &Layout) -> Vec<Option<u32>> {
    observation
        .services
        .iter()
        .zip(&layout.services)
        .zip(action)
        .map(|((service, bounds), &choice)| {
            let target = service
                .desired
                .saturating_add(choice)
                .saturating_sub(layout.max_replica_step);
            Some(target.clamp(bounds.min_replicas, bounds.max_replicas))
        })
        .collect()
}

fn decode_pool_delta(action: &[u32], observation: &Observation, layout: &Layout) -> Vec<i32> {
    if layout.control == Control::Replicas {
        return Vec::new();
    }
    action
        .iter()
        .skip(layout.service_slots)
        .take(observation.pools.len())
        .map(|&choice| i32::try_from(choice).map_or(i32::MAX, |choice| choice - 1))
        .collect()
}
