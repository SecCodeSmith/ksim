use ksim_core::observation::Observation;

use crate::layout::Layout;

/// Writes the encoding documented on `layout::SERVICE_FEATURES`, `POOL_FEATURES` and
/// `TIME_FEATURES` into `out`, whose length is `layout.observation_len()`. Utilisations with
/// no ready pods are 0.
pub fn encode(observation: &Observation, layout: &Layout, out: &mut [f32]) {
    let _ = (observation, layout, out);
    todo!()
}
