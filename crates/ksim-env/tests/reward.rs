mod common;

use approx::assert_abs_diff_eq;
use ksim_env::{
    layout::WeightedRewardWeights,
    reward::{RewardContext, RewardFn, WeightedReward, slo_violation},
};
use ksim_metrics::{
    metric::{Slo, SloPercentile},
    trace::{ServiceSample, TickSample},
};

#[test]
fn slo_violation_zero_demand() {
    let sample = TickSample {
        services: vec![ServiceSample {
            demand_rps: 0.0,
            demand_cpu_millis: 0.0,
            supply_cpu_millis: 500.0,
            cpu_usage_millis: 200.0,
            served_rps: 0.0,
            error_rate: 0.0,
            latency_p50_ms: 10.0,
            latency_p95_ms: 50.0,
            latency_p99_ms: 100.0,
            desired: 5,
            ready: 5,
            pending: 0,
            oom: false,
        }],
        ..Default::default()
    };

    let slo = Slo {
        latency_ms: 200.0,
        percentile: SloPercentile::P95,
        max_error_rate: 0.01,
        availability_target: 0.999,
    };

    let result = slo_violation(&sample, &slo);
    assert_abs_diff_eq!(result, 0.0);
}

#[test]
fn slo_violation_latency_p95_breach() {
    let sample = TickSample {
        services: vec![ServiceSample {
            demand_rps: 100.0,
            demand_cpu_millis: 4000.0,
            supply_cpu_millis: 500.0,
            cpu_usage_millis: 500.0,
            served_rps: 100.0,
            error_rate: 0.0,
            latency_p50_ms: 10.0,
            latency_p95_ms: 300.0,
            latency_p99_ms: 400.0,
            desired: 5,
            ready: 5,
            pending: 0,
            oom: false,
        }],
        ..Default::default()
    };

    let slo = Slo {
        latency_ms: 200.0,
        percentile: SloPercentile::P95,
        max_error_rate: 0.01,
        availability_target: 0.999,
    };

    let result = slo_violation(&sample, &slo);
    assert!(result > 0.0);
}

#[test]
fn slo_violation_latency_p99_breach() {
    let sample = TickSample {
        services: vec![ServiceSample {
            demand_rps: 100.0,
            demand_cpu_millis: 4000.0,
            supply_cpu_millis: 500.0,
            cpu_usage_millis: 500.0,
            served_rps: 100.0,
            error_rate: 0.0,
            latency_p50_ms: 10.0,
            latency_p95_ms: 150.0,
            latency_p99_ms: 250.0,
            desired: 5,
            ready: 5,
            pending: 0,
            oom: false,
        }],
        ..Default::default()
    };

    let slo = Slo {
        latency_ms: 200.0,
        percentile: SloPercentile::P99,
        max_error_rate: 0.01,
        availability_target: 0.999,
    };

    let result = slo_violation(&sample, &slo);
    assert!(result > 0.0);
}

#[test]
fn slo_violation_error_rate_breach() {
    let sample = TickSample {
        services: vec![ServiceSample {
            demand_rps: 100.0,
            demand_cpu_millis: 4000.0,
            supply_cpu_millis: 500.0,
            cpu_usage_millis: 500.0,
            served_rps: 100.0,
            error_rate: 0.05,
            latency_p50_ms: 10.0,
            latency_p95_ms: 50.0,
            latency_p99_ms: 100.0,
            desired: 5,
            ready: 5,
            pending: 0,
            oom: false,
        }],
        ..Default::default()
    };

    let slo = Slo {
        latency_ms: 200.0,
        percentile: SloPercentile::P95,
        max_error_rate: 0.01,
        availability_target: 0.999,
    };

    let result = slo_violation(&sample, &slo);
    assert!(result > 0.0);
}

#[test]
fn slo_violation_demand_weighted_two_services() {
    let sample = TickSample {
        services: vec![
            ServiceSample {
                demand_rps: 100.0,
                demand_cpu_millis: 4000.0,
                supply_cpu_millis: 500.0,
                cpu_usage_millis: 500.0,
                served_rps: 100.0,
                error_rate: 0.02,
                latency_p50_ms: 10.0,
                latency_p95_ms: 50.0,
                latency_p99_ms: 100.0,
                desired: 5,
                ready: 5,
                pending: 0,
                oom: false,
            },
            ServiceSample {
                demand_rps: 200.0,
                demand_cpu_millis: 8000.0,
                supply_cpu_millis: 1000.0,
                cpu_usage_millis: 1000.0,
                served_rps: 200.0,
                error_rate: 0.02,
                latency_p50_ms: 10.0,
                latency_p95_ms: 50.0,
                latency_p99_ms: 100.0,
                desired: 5,
                ready: 5,
                pending: 0,
                oom: false,
            },
        ],
        ..Default::default()
    };

    let slo = Slo {
        latency_ms: 200.0,
        percentile: SloPercentile::P95,
        max_error_rate: 0.01,
        availability_target: 0.999,
    };

    let result = slo_violation(&sample, &slo);
    assert!(result > 0.0);
}

#[test]
fn weighted_reward_basic() {
    let sample = TickSample {
        time: ksim_core::time::SimTime::from_secs(0),
        services: vec![ServiceSample {
            demand_rps: 100.0,
            demand_cpu_millis: 4000.0,
            supply_cpu_millis: 500.0,
            cpu_usage_millis: 500.0,
            served_rps: 100.0,
            error_rate: 0.0,
            latency_p50_ms: 10.0,
            latency_p95_ms: 50.0,
            latency_p99_ms: 100.0,
            desired: 5,
            ready: 5,
            pending: 0,
            oom: false,
        }],
        nodes: 2,
        hourly_cost: 0.5,
        replica_changes: 0,
        evictions: 0,
        node_changes: 0,
    };

    let context = RewardContext {
        slo: Slo {
            latency_ms: 200.0,
            percentile: SloPercentile::P95,
            max_error_rate: 0.01,
            availability_target: 0.999,
        },
        max_hourly_cost: 1.0,
        services: 1,
    };

    let mut reward_fn = WeightedReward {
        weights: WeightedRewardWeights {
            slo: 1.0,
            cost: 0.5,
            actions: 0.01,
        },
    };

    let reward = reward_fn.reward(&sample, &context);
    assert!(reward <= 0.0);
}

#[test]
fn weighted_reward_zero_max_hourly_cost_drops_cost_term() {
    let sample = TickSample {
        time: ksim_core::time::SimTime::from_secs(0),
        services: vec![ServiceSample {
            demand_rps: 100.0,
            demand_cpu_millis: 4000.0,
            supply_cpu_millis: 500.0,
            cpu_usage_millis: 500.0,
            served_rps: 100.0,
            error_rate: 0.0,
            latency_p50_ms: 10.0,
            latency_p95_ms: 50.0,
            latency_p99_ms: 100.0,
            desired: 5,
            ready: 5,
            pending: 0,
            oom: false,
        }],
        nodes: 2,
        hourly_cost: 0.5,
        replica_changes: 0,
        evictions: 0,
        node_changes: 0,
    };

    let context = RewardContext {
        slo: Slo {
            latency_ms: 200.0,
            percentile: SloPercentile::P95,
            max_error_rate: 0.01,
            availability_target: 0.999,
        },
        max_hourly_cost: 0.0,
        services: 1,
    };

    let mut reward_fn = WeightedReward {
        weights: WeightedRewardWeights {
            slo: 1.0,
            cost: 0.5,
            actions: 0.01,
        },
    };

    let _reward = reward_fn.reward(&sample, &context);
}

#[test]
fn weighted_reward_zero_services_drops_action_term() {
    let sample = TickSample {
        time: ksim_core::time::SimTime::from_secs(0),
        services: vec![],
        nodes: 2,
        hourly_cost: 0.5,
        replica_changes: 1,
        evictions: 0,
        node_changes: 1,
    };

    let context = RewardContext {
        slo: Slo {
            latency_ms: 200.0,
            percentile: SloPercentile::P95,
            max_error_rate: 0.01,
            availability_target: 0.999,
        },
        max_hourly_cost: 1.0,
        services: 0,
    };

    let mut reward_fn = WeightedReward {
        weights: WeightedRewardWeights {
            slo: 1.0,
            cost: 0.5,
            actions: 0.01,
        },
    };

    let _reward = reward_fn.reward(&sample, &context);
}

#[test]
fn weighted_reward_negated() {
    let sample = TickSample {
        time: ksim_core::time::SimTime::from_secs(0),
        services: vec![ServiceSample {
            demand_rps: 100.0,
            demand_cpu_millis: 4000.0,
            supply_cpu_millis: 500.0,
            cpu_usage_millis: 500.0,
            served_rps: 100.0,
            error_rate: 0.0,
            latency_p50_ms: 10.0,
            latency_p95_ms: 50.0,
            latency_p99_ms: 100.0,
            desired: 5,
            ready: 5,
            pending: 0,
            oom: false,
        }],
        nodes: 2,
        hourly_cost: 0.5,
        replica_changes: 1,
        evictions: 0,
        node_changes: 0,
    };

    let context = RewardContext {
        slo: Slo {
            latency_ms: 200.0,
            percentile: SloPercentile::P95,
            max_error_rate: 0.01,
            availability_target: 0.999,
        },
        max_hourly_cost: 1.0,
        services: 1,
    };

    let mut reward_fn = WeightedReward {
        weights: WeightedRewardWeights {
            slo: 1.0,
            cost: 0.5,
            actions: 0.01,
        },
    };

    let reward = reward_fn.reward(&sample, &context);
    assert!(reward <= 0.0);
}
