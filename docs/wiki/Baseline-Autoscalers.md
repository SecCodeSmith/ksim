# Baseline autoscalers

Status: **Planned** (Milestone 3). These are specifications for reimplementing upstream algorithms with upstream default parameters. Defaults are re-verified against the newest stable Kubernetes and kubernetes-autoscaler release tag at implementation time, and the tag is recorded here. Unit tests port key cases from upstream test files.

## Horizontal Pod Autoscaler

Source: Kubernetes `pkg/controller/podautoscaler` (`horizontal.go`, `replica_calculator.go`) and the HPA docs.

```
desiredReplicas = ceil( currentReplicas * currentMetricValue / desiredMetricValue )
```

No scaling if `|currentMetricValue / desiredMetricValue - 1| <= tolerance`.

| Parameter | Default |
|---|---|
| tolerance | 0.1 |
| sync period | 15 s |
| scale-down stabilization window | 300 s (uses the max recommendation in the window) |
| scale-up stabilization window | 0 s |
| scale-up policies | max(100 % of current, 4 pods) per 15 s, `selectPolicy: Max` |
| scale-down policies | 100 % per 15 s |
| CPU initialization period | 5 min |
| initial readiness delay | 30 s |

Handling: pods without metrics are assumed 0 % usage on scale-up and 100 % on scale-down; unready and recently started pods are excluded or treated conservatively; the result is clamped to `minReplicas`/`maxReplicas` and to the behavior policies.

## Vertical Pod Autoscaler (recommender and updater, `Auto` mode)

Source: kubernetes/autoscaler `vertical-pod-autoscaler`.

| Parameter | Default |
|---|---|
| CPU histogram decay half-life | 24 h |
| memory histogram decay half-life | 24 h |
| target percentile (CPU) | P90 |
| lower / upper bound percentile | P50 / P95 |
| memory recommendation | peak per 24 h aggregation interval |
| safety margin | +15 % |
| minimum recommendation | 25m CPU, 250 Mi memory |

The updater evicts a pod when its requests fall outside the recommended lower/upper bounds, subject to a PDB-like disruption limit. Using VPA on CPU together with an HPA on CPU/memory is flagged as a conflict, as in the upstream docs.

## Cluster Autoscaler

Source: kubernetes/autoscaler `cluster-autoscaler` (FAQ lists the flags).

| Parameter | Default |
|---|---|
| scan interval | 10 s |
| expander | `random` (also `least-waste` supported) |
| scale-down-utilization-threshold | 0.5 |
| scale-down-unneeded-time | 10 min |
| scale-down-delay-after-add | 10 min |
| max-graceful-termination-sec | 600 |
| max-node-provision-time | 15 min |

Scale-up simulates scheduling of unschedulable pods on a template node of each node pool and picks a pool via the expander. Scale-down removes nodes whose requested-resource utilization is below the threshold for the unneeded time and whose pods can be rescheduled elsewhere.

## Reference policies

Static peak over-provisioning, static average, and a clairvoyant oracle that knows future demand (upper bound).
