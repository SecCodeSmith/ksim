use ksim_core::observation::{Observation, PoolObservation, ServiceObservation};

use crate::layout::{
    Layout, OBSERVATION_HIGH, OBSERVATION_LOW, POOL_FEATURES, ReplicaBounds, SERVICE_FEATURES,
    TIME_FEATURES,
};

const SECONDS_PER_DAY: f64 = 86_400.0;

/// Writes the encoding documented on `layout::SERVICE_FEATURES`, `POOL_FEATURES` and
/// `TIME_FEATURES` into `out`, whose length is `layout.observation_len()`. Utilisations with
/// no ready pods are 0.
pub fn encode(observation: &Observation, layout: &Layout, out: &mut [f32]) {
    out.fill(0.0);
    let (services, rest) = out.split_at_mut(layout.service_slots * SERVICE_FEATURES);
    let (pools, time) = rest.split_at_mut(layout.pools * POOL_FEATURES);
    encode_services(observation, layout, services);
    encode_pools(observation, pools);
    encode_time(observation, time);
}

fn encode_services(observation: &Observation, layout: &Layout, out: &mut [f32]) {
    let (slots, _) = out.as_chunks_mut::<SERVICE_FEATURES>();
    observation
        .services
        .iter()
        .zip(&layout.services)
        .zip(slots)
        .for_each(|((service, &bounds), slot)| encode_service(service, bounds, layout, slot));
}

fn encode_service(
    service: &ServiceObservation,
    bounds: ReplicaBounds,
    layout: &Layout,
    slot: &mut [f32],
) {
    let signals = &service.signals;
    let ready = f64::from(service.ready);
    let latency_ms = if layout.slo_uses_p99 {
        signals.latency_p99_ms
    } else {
        signals.latency_p95_ms
    };
    let max_replicas = f64::from(bounds.max_replicas);
    #[allow(clippy::cast_precision_loss)]
    let requests = (
        service.requests.cpu_millis as f64,
        service.requests.memory_bytes as f64,
    );
    let features = [
        ratio(signals.cpu_usage_millis, ready * requests.0),
        ratio(signals.memory_usage_bytes, ready * requests.1),
        signals.served_rps.ln_1p(),
        ratio(latency_ms, layout.slo_latency_ms),
        signals.error_rate,
        ratio(ready, max_replicas),
        ratio(f64::from(service.desired), max_replicas),
        ratio(f64::from(service.pending), max_replicas),
        ratio(f64::from(service.outdated), max_replicas),
        1.0,
    ];
    write_features(&features, slot);
}

fn encode_pools(observation: &Observation, out: &mut [f32]) {
    let (slots, _) = out.as_chunks_mut::<POOL_FEATURES>();
    observation
        .pools
        .iter()
        .zip(slots)
        .enumerate()
        .for_each(|(index, (pool, slot))| encode_pool(observation, index, pool, slot));
}

fn encode_pool(observation: &Observation, index: usize, pool: &PoolObservation, slot: &mut [f32]) {
    let max_nodes = f64::from(pool.max_nodes);
    let features = [
        ratio(f64::from(pool.ready), max_nodes),
        ratio(f64::from(pool.provisioning), max_nodes),
        ratio(f64::from(pool.draining), max_nodes),
        requested_cpu_share(observation, index),
        1.0,
    ];
    write_features(&features, slot);
}

#[allow(clippy::cast_precision_loss)]
fn requested_cpu_share(observation: &Observation, pool_index: usize) -> f64 {
    let (requested, allocatable) = observation
        .nodes
        .iter()
        .filter(|node| node.pool.index() == pool_index)
        .fold((0.0, 0.0), |(requested, allocatable), node| {
            (
                requested + node.requested.cpu_millis as f64,
                allocatable + node.allocatable.cpu_millis as f64,
            )
        });
    ratio(requested, allocatable)
}

fn encode_time(observation: &Observation, out: &mut [f32]) {
    debug_assert_eq!(out.len(), TIME_FEATURES);
    let seconds_into_day = observation.now.as_secs_f64() % SECONDS_PER_DAY;
    let angle = std::f64::consts::TAU * seconds_into_day / SECONDS_PER_DAY;
    write_features(&[angle.sin(), angle.cos()], out);
}

fn ratio(numerator: f64, denominator: f64) -> f64 {
    if denominator > 0.0 {
        numerator / denominator
    } else {
        0.0
    }
}

fn write_features(features: &[f64], out: &mut [f32]) {
    for (slot, &feature) in out.iter_mut().zip(features) {
        *slot = clamp_feature(feature);
    }
}

#[allow(clippy::cast_possible_truncation)]
fn clamp_feature(value: f64) -> f32 {
    if value.is_nan() {
        return 0.0;
    }
    value.clamp(f64::from(OBSERVATION_LOW), f64::from(OBSERVATION_HIGH)) as f32
}
