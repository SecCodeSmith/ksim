# Baseline autoscalers

Status: **Implemented** (Milestone 3) in `crates/ksim-autoscalers`. Algorithms follow upstream Kubernetes (`pkg/controller/podautoscaler`) and kubernetes/autoscaler (`vertical-pod-autoscaler`, `cluster-autoscaler`) with upstream default parameters, on the simulator's aggregate per-service metrics. Unit tests port key cases from the upstream test files.

All three baselines implement one small trait each (`ReplicaRecommender`, `ResourceRecommender`, `NodeProvisioner`, in `traits.rs`); `KubernetesStack` composes them into a single `ksim_core::autoscaler::Autoscaler`.

## Horizontal Pod Autoscaler (`hpa.rs`)

Target: average CPU utilisation as a fraction of requests (`HpaSpec::target_cpu_utilization`, 0.8 = 80 %). One `Option<HpaSpec>` per service; `None` leaves the service alone.

Replica calculator (`desired_replicas`), from aggregate metrics:

```
utilization = cpu_usage / (ready * cpu_request)
ratio       = utilization / target
```

- `|1 - ratio| <= tolerance`: keep current replicas.
- `ratio < 1` (scale down) or no not-ready pods: `ceil(ratio * ready)`; not-ready pods are ignored.
- Scale up with not-ready pods: they count at 0 % usage; the ratio is recomputed over `ready + not_ready`. If it is within tolerance or below 1, keep current; else `ceil(ratio' * (ready + not_ready))`.
- No ready pods, zero request, or a non-finite/negative usage: keep current.

Mapping from `ServiceObservation`: `current = desired`, `ready = ready`, `not_ready = pending` (Pending plus Starting), usage from `signals.cpu_usage_millis`, requests from `requests.cpu_millis`.

Behaviour (`stabilizeRecommendationWithBehaviors`, then policy limits, then clamping):

1. Each sync records the raw recommendation. The scale-up candidate is the minimum of the raw value and all recommendations inside the scale-up window; the scale-down candidate is the maximum inside the scale-down window. A recommendation at time `t` is inside a window `W` when `t + W > now` (strict boundary: exactly `W` old is outside). Result: `candidate_up` if `current < candidate_up`, `candidate_down` if `current > candidate_down`, else `current`. History is retained for the larger window.
2. Policy limits, relative to the replicas at the start of each policy period (`start = current - added_in_period` for up, `current + removed_in_period` for down; scale events older than the period are ignored):
   - `Pods(v)`: up `start + v`, down `start - v` (saturating).
   - `Percent(v)`: up `ceil(start * (100 + v) / 100)`, down `floor(start * (100 - v) / 100)`.
   - `Max` picks the most permissive limit, `Min` the least, `Disabled` forbids that direction (limit = current). Scale-up takes `min(stabilised, limit)`, scale-down `max(stabilised, limit)`.
3. Clamp to `[min_replicas, max_replicas]`.

## Vertical Pod Autoscaler (`vpa.rs`, `histogram.rs`)

Recommender plus updater, one `enabled` flag per service.

Histograms (`DecayingHistogram`): exponential buckets, bucket `i` starts at `first * (1.05^i - 1) / 0.05`. CPU (millicores): first bucket 10m, ratio 1.05, max 1e6. Memory (bytes): first bucket 1e7, ratio 1.05, max 1e12. Half-life decay: a sample at `t` has weight `w * 2^((t - reference) / half_life)`; when the exponent exceeds 30 the reference moves to `t` and all weights are rescaled (keeps factors bounded, same relative weights as upstream). A percentile is the start of the bucket after the first whose cumulative weight reaches `p * total`.

Recommender (every `recommender_interval`, services with ready pods):
- CPU sample: `cpu_usage / ready` with weight `max(request_cores, 0.1)`.
- Memory: per-pod usage `memory_usage / ready`; only the peak per 24 h aggregation window is kept (the previous peak is subtracted and the higher one added, both at the window start time).
- Estimate = `percentile * (1 + 0.15)` times a confidence multiplier, then raised to the minimums and rounded up. With `c` = days of history (at least one recommender interval): upper bound multiplier `(1 + 1/c)`, lower bound `(1 + 0.001/c)^-2`, target 1.
- Target P90, lower bound P50, upper bound P95.

Updater (every `updater_interval`):
- `Off`: nothing. `Initial`: sets new requests only. `Auto`: also evicts.
- When current requests are outside `[lower, upper]` in CPU or memory, `Actions.requests` is set to `target` (applies to new pods).
- `Auto` evicts outdated pods (`ServiceObservation::outdated`) only if `desired >= min_replicas`, at most `max(1, floor(eviction_tolerance * desired))` per run, via `Actions.evict`.

## Cluster Autoscaler (`ca.rs`)

Runs at most once per `scan_interval` (the first call always runs); other calls return an empty plan.

Scale-up:
1. Unschedulable pods (per service, `unschedulable` copies of its requests) are sorted largest first and first packed into the free capacity of `Provisioning` nodes (absorption), so already-requested nodes are not requested twice.
2. Per pool with room (`max_nodes - ready - provisioning > 0`), the remaining pods that fit one of its nodes are packed by the FFD binpacking estimator (`nodes_needed`: sort by CPU then memory descending, first fit, pods larger than a node ignored). Option = `min(nodes_needed, room)`.
3. Expander picks one option: `Random` (seeded ChaCha8, `seed`) or `LeastWaste` (lowest mean of CPU and memory idle fraction over the new nodes, ties to the lowest pool id). Result is `Actions.pool_delta`.

Scale-down (only in loops without scale-up, and at least `scale_down_delay_after_add` after the last scale-up):
1. Unneeded tracking: a `Ready` node with `max(cpu, memory)` requested/allocatable below the threshold is stamped with the loop time at which it was first seen unneeded; the stamp is dropped as soon as the node is no longer unneeded.
2. A node is removable when unneeded for `scale_down_unneeded_time`, its pool stays at or above `min_nodes` counting nodes already chosen for removal in the same loop, and its pods fit (FFD) into the free capacity of the other `Ready` nodes not chosen in this loop.
3. Empty removable nodes are drained in bulk (up to `max_empty_bulk_delete`, ascending id); otherwise one non-empty removable node with the lowest utilisation (ties to the lowest id). Output is `Actions.drain`; the engine cordons, evicts and removes the node.

## KubernetesStack (`stack.rs`)

Builder-composed `Autoscaler`; each component is optional.

| Component | Fills in `Actions` |
|---|---|
| `with_replicas` (`ReplicaRecommender`) | `replicas` |
| `with_resources` (`ResourceRecommender`) | `requests`, `evict` (from `ResourcePlan`) |
| `with_nodes` (`NodeProvisioner`) | `pool_delta`, `drain` (from `NodePlan`) |

A missing component leaves its fields empty (no change). As upstream warns, an HPA and a VPA that both act on CPU for the same service interfere; the stack does not prevent it, so enable them on disjoint services or metrics.

## Defaults

| Component | Parameter (Rust field) | Upstream flag / field | Default |
|---|---|---|---|
| HPA | `behavior.tolerance` | `--horizontal-pod-autoscaler-tolerance` | 0.1 |
| HPA | `scale_down.stabilization_window` | `behavior.scaleDown.stabilizationWindowSeconds` | 300 s |
| HPA | `scale_up.stabilization_window` | `behavior.scaleUp.stabilizationWindowSeconds` | 0 s |
| HPA | `scale_up.policies`, `select` | `behavior.scaleUp.policies`, `selectPolicy` | Percent 100 and Pods 4 per 15 s, `Max` |
| HPA | `scale_down.policies`, `select` | `behavior.scaleDown.policies`, `selectPolicy` | Percent 100 per 15 s, `Max` |
| VPA | `recommender_interval` | `--recommender-interval` | 60 s |
| VPA | `updater_interval` | `--updater-interval` | 60 s |
| VPA | `half_life` | `--cpu-histogram-decay-half-life`, `--memory-histogram-decay-half-life` | 24 h |
| VPA | `target_percentile` | `--target-cpu-percentile` | 0.9 |
| VPA | `lower_percentile` | `--recommendation-lower-bound-cpu-percentile` | 0.5 |
| VPA | `upper_percentile` | `--recommendation-upper-bound-cpu-percentile` | 0.95 |
| VPA | `safety_margin` | `--recommendation-margin-fraction` | 0.15 |
| VPA | `min_cpu_millis` | `--pod-recommendation-min-cpu-millicores` | 25 |
| VPA | `min_memory_bytes` | `--pod-recommendation-min-memory-mb` | 250 MiB |
| VPA | `memory_aggregation_interval` | `--memory-aggregation-interval` | 24 h |
| VPA | `min_replicas` | `--min-replicas` (updater) | 2 |
| VPA | `eviction_tolerance` | `--eviction-tolerance` (updater) | 0.5 |
| VPA | `mode` | `updatePolicy.updateMode` | `Auto` |
| CA | `scan_interval` | `--scan-interval` | 10 s |
| CA | `scale_down_utilization_threshold` | `--scale-down-utilization-threshold` | 0.5 |
| CA | `scale_down_unneeded_time` | `--scale-down-unneeded-time` | 10 min |
| CA | `scale_down_delay_after_add` | `--scale-down-delay-after-add` | 10 min |
| CA | `max_empty_bulk_delete` | `--max-empty-bulk-delete` | 10 |
| CA | `expander` | `--expander` | `Random` (`LeastWaste` = `least-waste`) |
| CA | `seed` | none (simulator determinism) | 0 |

## Simplifications versus upstream

- Metrics are per-service aggregates (`cpu_usage / ready`), not per-pod; there is no per-pod missing-metrics handling, CPU initialisation period or initial readiness delay.
- HPA has no sync period of its own: it runs on every `act` call, so the tick is the sync period. Only CPU utilisation targets exist (no memory, custom or external metrics).
- VPA models one container per pod, and there is no in-place resize: new requests apply to new pods, and existing outdated pods are evicted. No PDB checks; eviction is bounded by `eviction_tolerance`. Only the CPU percentile flags are exposed and reused for memory.
- Cluster Autoscaler: no per-node-group template nodes beyond the pool's allocatable, no node-group price expander (only `random` and `least-waste`), no `max-graceful-termination-sec` or `max-node-provision-time` (provisioning delays come from the engine).

## Using the baselines

```rust
use ksim_autoscalers::{
    ca::ClusterAutoscaler,
    hpa::{Hpa, HpaBehavior, HpaSpec},
    stack::KubernetesStack,
};
use ksim_core::autoscaler::Autoscaler;

let hpa = Hpa::new(vec![Some(HpaSpec {
    min_replicas: 1,
    max_replicas: 20,
    target_cpu_utilization: 0.8,
    behavior: HpaBehavior::default(),
})]);
let mut stack = KubernetesStack::new()
    .with_replicas(Box::new(hpa))
    .with_nodes(Box::new(ClusterAutoscaler::new(Default::default())));

// `simulation` comes from SimulationBuilder (see tests/integration.rs).
for _ in 0..240 {
    let observation = simulation.observation().clone();
    let actions = stack.act(&observation);
    simulation.step(&actions);
}
```

## Reference policies

Static peak over-provisioning, static average, and a clairvoyant oracle that knows future demand (upper bound). **Planned** (Milestone 4).
