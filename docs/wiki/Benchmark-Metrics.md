# Benchmark metrics

Status: **Implemented** (Milestone 4, crate `ksim-metrics`). Every metric implements the `Metric` trait (`name()`, `compute(&EpisodeTrace) -> f64`); `standard_metrics(slo)` returns them all and the report is a map from metric name to value. The `EpisodeTrace` holds one `TickSample` per tick and one `ServiceSample` per (tick, service); the recorder is called once per step, before `Simulation::step`, with the true demand from the workload.

Notation: `d_t` demanded CPU (millicores), `s_t` supplied CPU, `T` number of ticks. **Supply = ready pods x CPU request** of the service. Demand and SLO inputs come from the workload, not from the observed load.

## Availability and SLO (Beyer et al., SRE book, 2016)

The SLO is per scenario: `latency_ms`, `percentile` (P95 or P99), `max_error_rate`, `availability_target`.

| Report name | Formula as implemented |
|---|---|
| `availability` | `sum(demand_rps * (1 - error_rate)) / sum(demand_rps)` over all ticks and services, i.e. demand-weighted good requests over valid requests; 1 when there is no demand |
| `slo_compliance` | share of (tick, service) samples that comply; a sample violates when `demand_rps > 0` and (latency at the SLO percentile `> latency_ms` or `error_rate > max_error_rate`); samples without demand comply; 1 with no samples |
| `availability_nines` | `-log10(1 - availability)`, capped at 9 (also 9 when unavailability is 0) |
| `error_budget_burn` | `(1 - availability) / (1 - availability_target)`; above 1 the budget is exhausted; infinite if the budget is 0 and any error occurred, else 0 |
| `longest_slo_violation_s` | longest run of consecutive ticks in which at least one service violates the SLO, times the tick length in seconds |

## SPEC RG elasticity (Herbst et al., ICAC 2013; Herbst et al., ACM TOMPECS 2018)

Computed per service on `d_t` versus `s_t` (ticks have equal length, so the time weight `Δt` cancels), then **averaged over services**. Lower is better except where noted.

| Report name | Formula |
|---|---|
| `under_provisioning_accuracy` (theta_U, %) | `100/T * sum_t max(d_t - s_t, 0) / d_t` |
| `over_provisioning_accuracy` (theta_O, %) | `100/T * sum_t max(s_t - d_t, 0) / d_t` |
| `under_provisioning_timeshare` (tau_U, %) | `100/T * #{t : s_t < d_t}` |
| `over_provisioning_timeshare` (tau_O, %) | `100/T * #{t : s_t > d_t}` |
| `jitter_per_hour` | `(E_S - E_D) / hours`; `E_x` counts ticks `t >= 1` with `x_t != x_{t-1}`; `hours = T * tick / 3600` |
| `instability` (%) | `100/(T-1) * #{t >= 1 : sgn(Δs_t) != sgn(Δd_t)}` |

Zero-demand handling: ticks with `d_t = 0` add 0 to theta_U and theta_O (they still count in `T`); they count as over-provisioned in tau_O when `s_t > 0`. Empty traces, `hours = 0`, or a single tick yield 0. Because theta is normalised by `d_t`, over-provisioning accuracy can exceed 100 %. Positive jitter means supply changes more often than demand (oscillation), negative means less often.

## Cost and efficiency

| Report name | Formula |
|---|---|
| `cost` | `sum_t hourly_cost_t * tick_hours`; `hourly_cost_t` sums the pool `price_per_hour` of every existing node (ready, provisioning or draining) |
| `node_hours` | `sum_t nodes_t * tick_hours` |
| `cpu_request_utilization` | `sum(cpu_usage_millis) / sum(supply_cpu_millis)` over all samples; 0 without supply |

## Operational counters

| Report name | Meaning |
|---|---|
| `scaling_actions` | `sum(replica_changes + node_changes)`; a replica change is a service whose replicas action differs from its desired count; node changes are `abs(pool_delta)` plus drained nodes |
| `pending_pod_seconds` | `sum(pending) * tick_seconds` |
| `oom_events` | (tick, service) samples with an OOM kill |
| `evictions` | sum of the evict action |
| `mean_replicas` | mean of ready pods over all samples |

## Reward (training, Planned)

Configurable through the `RewardFn` trait: `reward = -w_slo * SLO_violation - w_cost * resource_cost - w_act * scaling_actions`, weights documented per scenario suite (Milestone 6).

## References

- Beyer, Jones, Petoff, Murphy (eds.), *Site Reliability Engineering*, O'Reilly, 2016 (SLIs, SLOs, error budgets).
- Herbst, Kounev, Reussner, "Elasticity in Cloud Computing: What It Is, and What It Is Not", ICAC 2013.
- Herbst et al., "Quantifying Cloud Performance and Dependability: Taxonomy, Metric Design, and Emerging Challenges", ACM TOMPECS 3(4), 2018.
