# Benchmark metrics

Status: **Planned** (Milestone 4). Each metric is implemented test-first from a hand-computed example taken from the cited source. Notation: d(t) demanded resource units, s(t) supplied units, T total duration, Δt step length.

## Availability and SLO (Beyer et al., SRE book)

```
availability = good events / valid events
```

For request-driven services an event is good if it succeeded (non-error) AND latency <= SLO threshold. Invalid events (e.g. client errors) are excluded. Availability is also reported in "nines": `nines = -log10(1 - availability)` (0.999 is three nines). Related outputs:

- time-based availability: fraction of windows meeting the SLO;
- SLO violation rate for p95 / p99 latency targets;
- error budget = 1 - SLO target; burn rate = observed error ratio / error budget.

## SPEC RG elasticity metrics (Herbst et al. 2013, 2018)

Computed from supplied vs. demanded resources. Definitions follow Herbst et al.; the exact normalization is checked against the paper text when the tests are written.

```
theta_U = 100/T * sum_t  ( max(d_t - s_t, 0) / d_t ) * Δt        # under-provisioning accuracy [%]
theta_O = 100/T * sum_t  ( max(s_t - d_t, 0) / d_t ) * Δt        # over-provisioning accuracy  [%]
tau_U   = 100/T * sum_t  max(sgn(d_t - s_t), 0) * Δt             # under-provisioning timeshare [%]
tau_O   = 100/T * sum_t  max(sgn(s_t - d_t), 0) * Δt             # over-provisioning timeshare  [%]
jitter  = (E_S - E_D) / T                                        # E_S supply changes, E_D demand changes
```

Lower is better for theta and tau. Positive jitter means the system adapts more often than demand changes (oscillation); negative means it adapts too rarely. Also reported: instability (share of time supply and demand move in opposite directions) and elastic speedup versus a reference system (Herbst et al. 2018).

## Cost and efficiency

Node-hours x price per hour; CPU and memory request utilization (used / requested); allocated-vs-used waste.

## Operational

Number of scaling actions, oscillations, time to recover after a surge, pending-pod time, OOMKills, evictions, cold starts.

## Reward (training)

Configurable: `reward = -w_slo * SLO_violation - w_cost * resource_cost - w_act * scaling_actions`, weights documented per scenario suite.
