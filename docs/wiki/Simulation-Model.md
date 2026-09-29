# Simulation model

Status: **Planned** (Milestone 2). This page is the specification the tests are written from.

## Time model

Hybrid: a fixed control tick (default 15 s, the HPA sync period) plus a binary-heap event queue for delayed events (pod start, node provisioning, eviction, image pull). Fully deterministic; randomness comes from seeded `rand_chacha` streams, one per concern.

## Objects

Struct-of-arrays, preallocated, no per-tick allocation: `Node`, `Pod`, `Service`/`Deployment`, `NodePool` (instance type, min/max, price per hour, provisioning delay distribution).

## Scheduler

Simplified kube-scheduler: filter `NodeResourcesFit`; score `LeastAllocated` (default) or `MostAllocated`; pending queue with backoff. Unschedulable pods feed the Cluster Autoscaler.

## Lifecycle latencies

Configurable distributions: image pull, container start, readiness delay, node provisioning (default 60-180 s), graceful termination (default 30 s). Pod startup defaults follow the Kubernetes SIG-scalability SLO (p99 <= 5 s excluding image pull).

## Performance model

Request-driven services (Alibaba 2021/22, Azure): each pod is a server with service rate mu derived from its CPU limit and CPU per request. With c pods, arrival rate lambda and offered load a = lambda/mu, utilization rho = a/c (must be < 1):

```
Erlang C:  C(c, a) = [ a^c / c! * 1/(1-rho) ] / [ sum_{k=0}^{c-1} a^k/k! + a^c / c! * 1/(1-rho) ]
M/M/c mean wait:  Wq = C(c, a) / (c*mu - lambda)
Allen-Cunneen G/G/c:  Wq(G/G/c) ~= Wq(M/M/c) * (ca^2 + cs^2) / 2
```

ca^2 and cs^2 are squared coefficients of variation of inter-arrival and service times, estimated from the trace. Latency percentiles derive from the waiting-time distribution; requests beyond the queue bound or timeout are dropped and counted as errors.

Utilization-driven services (Alibaba 2018, Borg): CPU demand above the limit causes CFS throttling and latency inflation; memory above the limit causes OOMKill plus restart delay; node memory pressure causes eviction.

Observable outputs match what Metrics Server or Prometheus would expose: per-pod CPU and memory, RPS, p50/p95/p99 latency, error rate, pending pods, with a configurable scrape delay and window (Metrics Server default resolution 15 s).

## Assumptions and limitations

- Homogeneous pods within a service; no network or noisy-neighbor model; steady-state queueing per tick rather than transient queue dynamics.
- Approximations are validated against a real `kind` cluster in Milestone 7 (Planned); limitations are documented there.
