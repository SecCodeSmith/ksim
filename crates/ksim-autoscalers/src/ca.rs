use std::collections::BTreeMap;

use ksim_core::{
    cluster::NodePhase,
    ids::NodeId,
    observation::{NodeObservation, Observation, PoolObservation},
    resources::Resources,
    time::{SimDuration, SimTime},
};
use rand::{RngExt, SeedableRng};
use rand_chacha::ChaCha8Rng;

use crate::traits::{NodePlan, NodeProvisioner};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Expander {
    Random,
    LeastWaste,
}

/// Upstream Cluster Autoscaler defaults: scan interval 10 s, scale-down utilisation threshold
/// 0.5, scale-down-unneeded-time 10 min, scale-down-delay-after-add 10 min, at most 10 empty
/// nodes deleted per loop, expander `random`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClusterAutoscalerConfig {
    pub scan_interval: SimDuration,
    pub scale_down_utilization_threshold: f64,
    pub scale_down_unneeded_time: SimDuration,
    pub scale_down_delay_after_add: SimDuration,
    pub max_empty_bulk_delete: u32,
    pub expander: Expander,
    pub seed: u64,
}

impl Default for ClusterAutoscalerConfig {
    fn default() -> Self {
        Self {
            scan_interval: SimDuration::from_secs(10),
            scale_down_utilization_threshold: 0.5,
            scale_down_unneeded_time: SimDuration::from_secs(600),
            scale_down_delay_after_add: SimDuration::from_secs(600),
            max_empty_bulk_delete: 10,
            expander: Expander::Random,
            seed: 0,
        }
    }
}

/// Nodes of size `allocatable` needed to hold `pods`, packed first-fit decreasing by CPU then
/// memory (upstream binpacking estimator). Pods larger than a node are ignored.
#[must_use]
pub fn nodes_needed(pods: &[Resources], allocatable: Resources) -> u32 {
    let mut fitting: Vec<Resources> = pods
        .iter()
        .copied()
        .filter(|pod| pod.fits_in(allocatable))
        .collect();
    sort_largest_first(&mut fitting);
    let mut free_per_node: Vec<Resources> = Vec::new();
    for pod in fitting {
        if !place_first_fit(pod, &mut free_per_node) {
            free_per_node.push(allocatable.checked_sub(pod).unwrap_or_default());
        }
    }
    u32::try_from(free_per_node.len()).unwrap_or(u32::MAX)
}

fn sort_largest_first(pods: &mut [Resources]) {
    pods.sort_by(|a, b| {
        b.cpu_millis
            .cmp(&a.cpu_millis)
            .then(b.memory_bytes.cmp(&a.memory_bytes))
    });
}

fn place_first_fit(pod: Resources, free: &mut [Resources]) -> bool {
    for slot in free.iter_mut() {
        if let Some(rest) = slot.checked_sub(pod) {
            *slot = rest;
            return true;
        }
    }
    false
}

#[allow(clippy::cast_precision_loss)]
fn ratio(part: u64, whole: u64) -> f64 {
    if whole == 0 {
        0.0
    } else {
        part as f64 / whole as f64
    }
}

fn utilization(node: &NodeObservation) -> f64 {
    ratio(node.requested.cpu_millis, node.allocatable.cpu_millis).max(ratio(
        node.requested.memory_bytes,
        node.allocatable.memory_bytes,
    ))
}

fn free_capacity(node: &NodeObservation) -> Resources {
    node.allocatable
        .checked_sub(node.requested)
        .unwrap_or_default()
}

#[derive(Debug, Clone, Copy)]
struct PoolOption {
    pool: usize,
    nodes: u32,
    waste: f64,
}

/// Cluster Autoscaler; runs at most once per `scan_interval` (the first call always runs).
///
/// Scale-up: unschedulable pods (per service: `unschedulable` pods with the service's
/// requests) are first packed into the free capacity of `Provisioning` nodes; for each pool
/// with room below `max_nodes` (counting ready and provisioning nodes) the remaining pods that
/// fit one of its nodes form an option of `min(nodes_needed, room)` nodes. The expander picks
/// one option (`LeastWaste`: lowest mean CPU/memory idle fraction of the new nodes, ties to the
/// lowest pool id; `Random`: seeded uniform choice) and returns it as `pool_delta`.
///
/// Scale-down, only in loops without scale-up and at least `scale_down_delay_after_add` after
/// the last scale-up: `Ready` nodes with `max(cpu, memory)` requested/allocatable below the
/// threshold are unneeded since first seen so (and forgotten once they are not). A node
/// unneeded for `scale_down_unneeded_time` is removable when its pool stays at or above
/// `min_nodes` and its pods fit, first-fit decreasing, into the free capacity of the other
/// `Ready` nodes not removed in this loop. Empty removable nodes are drained in bulk (up to
/// `max_empty_bulk_delete`, ascending id); otherwise one non-empty removable node (lowest
/// utilisation, ties to the lowest id).
///
/// # Panics
/// If a `NodeObservation::pods` range lies outside `Observation::node_pods`, which the engine
/// never produces.
#[derive(Debug)]
pub struct ClusterAutoscaler {
    config: ClusterAutoscalerConfig,
    rng: ChaCha8Rng,
    last_run: Option<SimTime>,
    last_scale_up: Option<SimTime>,
    unneeded_since: BTreeMap<NodeId, SimTime>,
}

impl ClusterAutoscaler {
    #[must_use]
    pub fn new(config: ClusterAutoscalerConfig) -> Self {
        Self {
            config,
            rng: ChaCha8Rng::seed_from_u64(config.seed),
            last_run: None,
            last_scale_up: None,
            unneeded_since: BTreeMap::new(),
        }
    }

    fn is_throttled(&self, now: SimTime) -> bool {
        self.last_run
            .is_some_and(|last| now - last < self.config.scan_interval)
    }

    fn plan_scale_up(&mut self, observation: &Observation) -> Option<Vec<i32>> {
        let mut pods = unschedulable_pods(observation);
        absorb_into_provisioning(&mut pods, observation);
        let options = scale_up_options(&pods, &observation.pools);
        let chosen = self.expand(&options)?;
        let mut pool_delta = vec![0; observation.pools.len()];
        pool_delta[chosen.pool] = i32::try_from(chosen.nodes).unwrap_or(i32::MAX);
        Some(pool_delta)
    }

    fn expand(&mut self, options: &[PoolOption]) -> Option<PoolOption> {
        if options.is_empty() {
            return None;
        }
        match self.config.expander {
            Expander::Random => Some(options[self.rng.random_range(0..options.len())]),
            Expander::LeastWaste => least_waste(options),
        }
    }

    fn track_unneeded(&mut self, observation: &Observation) {
        let threshold = self.config.scale_down_utilization_threshold;
        let unneeded: Vec<NodeId> = observation
            .nodes
            .iter()
            .filter(|node| node.phase == NodePhase::Ready && utilization(node) < threshold)
            .map(|node| node.id)
            .collect();
        self.unneeded_since.retain(|id, _| unneeded.contains(id));
        for id in unneeded {
            self.unneeded_since.entry(id).or_insert(observation.now);
        }
    }

    fn scale_down_allowed(&self, now: SimTime) -> bool {
        self.last_scale_up
            .is_none_or(|added| now - added >= self.config.scale_down_delay_after_add)
    }

    fn plan_scale_down(&self, observation: &Observation) -> Vec<NodeId> {
        let removable = self.removable_nodes(observation);
        let empty: Vec<NodeId> = removable
            .iter()
            .filter(|node| node.pods.is_empty())
            .map(|node| node.id)
            .take(self.config.max_empty_bulk_delete as usize)
            .collect();
        if !empty.is_empty() {
            return empty;
        }
        removable
            .iter()
            .min_by(|a, b| utilization(a).total_cmp(&utilization(b)))
            .map(|node| vec![node.id])
            .unwrap_or_default()
    }

    fn removable_nodes<'a>(&self, observation: &'a Observation) -> Vec<&'a NodeObservation> {
        let mut removed: Vec<&NodeObservation> = Vec::new();
        for node in &observation.nodes {
            if self.has_been_unneeded_long_enough(node, observation.now)
                && pool_keeps_minimum(node, &removed, &observation.pools)
                && fits_elsewhere(node, &removed, observation)
            {
                removed.push(node);
            }
        }
        removed
    }

    fn has_been_unneeded_long_enough(&self, node: &NodeObservation, now: SimTime) -> bool {
        self.unneeded_since
            .get(&node.id)
            .is_some_and(|since| now - *since >= self.config.scale_down_unneeded_time)
    }
}

fn unschedulable_pods(observation: &Observation) -> Vec<Resources> {
    let mut pods = Vec::new();
    for service in &observation.services {
        pods.extend((0..service.unschedulable).map(|_| service.requests));
    }
    sort_largest_first(&mut pods);
    pods
}

fn absorb_into_provisioning(pods: &mut Vec<Resources>, observation: &Observation) {
    let mut free: Vec<Resources> = observation
        .nodes
        .iter()
        .filter(|node| node.phase == NodePhase::Provisioning)
        .map(free_capacity)
        .collect();
    pods.retain(|pod| !place_first_fit(*pod, &mut free));
}

fn scale_up_options(pods: &[Resources], pools: &[PoolObservation]) -> Vec<PoolOption> {
    pools
        .iter()
        .enumerate()
        .filter_map(|(pool, observed)| pool_option(pool, observed, pods))
        .collect()
}

fn pool_option(pool: usize, observed: &PoolObservation, pods: &[Resources]) -> Option<PoolOption> {
    let in_use = observed.ready.saturating_add(observed.provisioning);
    let room = observed.max_nodes.saturating_sub(in_use);
    let fitting: Vec<Resources> = pods
        .iter()
        .copied()
        .filter(|pod| pod.fits_in(observed.allocatable))
        .collect();
    let nodes = nodes_needed(&fitting, observed.allocatable).min(room);
    (nodes > 0).then(|| PoolOption {
        pool,
        nodes,
        waste: idle_fraction(&fitting, observed.allocatable, nodes),
    })
}

/// Mean of the CPU and memory idle fractions across `nodes` new nodes holding `pods`.
fn idle_fraction(pods: &[Resources], allocatable: Resources, nodes: u32) -> f64 {
    let requested = pods
        .iter()
        .fold(Resources::ZERO, |sum, pod| sum.saturating_add(*pod));
    let capacity_cpu = allocatable.cpu_millis.saturating_mul(u64::from(nodes));
    let capacity_memory = allocatable.memory_bytes.saturating_mul(u64::from(nodes));
    let idle_cpu = 1.0 - ratio(requested.cpu_millis, capacity_cpu).min(1.0);
    let idle_memory = 1.0 - ratio(requested.memory_bytes, capacity_memory).min(1.0);
    f64::midpoint(idle_cpu, idle_memory)
}

fn least_waste(options: &[PoolOption]) -> Option<PoolOption> {
    options
        .iter()
        .copied()
        .reduce(|best, next| if next.waste < best.waste { next } else { best })
}

fn pool_keeps_minimum(
    node: &NodeObservation,
    removed: &[&NodeObservation],
    pools: &[PoolObservation],
) -> bool {
    let removed_from_pool = removed.iter().filter(|r| r.pool == node.pool).count();
    let Some(pool) = pools.get(node.pool.index()) else {
        return false;
    };
    let remaining = (pool.ready as usize).saturating_sub(removed_from_pool + 1);
    remaining >= pool.min_nodes as usize
}

fn fits_elsewhere(
    node: &NodeObservation,
    removed: &[&NodeObservation],
    observation: &Observation,
) -> bool {
    let mut free: Vec<Resources> = observation
        .nodes
        .iter()
        .filter(|other| other.phase == NodePhase::Ready && other.id != node.id)
        .filter(|other| removed.iter().all(|r| r.id != other.id))
        .map(free_capacity)
        .collect();
    let mut pods: Vec<Resources> = observation.node_pods[node.pods.clone()]
        .iter()
        .map(|placement| placement.requests)
        .collect();
    sort_largest_first(&mut pods);
    pods.into_iter().all(|pod| place_first_fit(pod, &mut free))
}

impl NodeProvisioner for ClusterAutoscaler {
    fn provision(&mut self, observation: &Observation) -> NodePlan {
        if self.is_throttled(observation.now) {
            return NodePlan::default();
        }
        self.track_unneeded(observation);
        let scale_up = self.plan_scale_up(observation);
        self.last_run = Some(observation.now);
        if let Some(pool_delta) = scale_up {
            self.last_scale_up = Some(observation.now);
            return NodePlan {
                pool_delta,
                drain: Vec::new(),
            };
        }
        let drain = if self.scale_down_allowed(observation.now) {
            self.plan_scale_down(observation)
        } else {
            Vec::new()
        };
        NodePlan {
            pool_delta: Vec::new(),
            drain,
        }
    }
}
