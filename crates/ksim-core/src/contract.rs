//! Shared contract test suites every trait implementation must pass (Liskov substitutability).

/// Generate contract tests for `WorkloadSource` implementations.
/// Tests: `services()` > 0; `demand` finite and non-negative for every service at t = 0..24h every 15 min;
/// `demand(s, t)` called twice returns equal values (purity).
#[macro_export]
macro_rules! workload_source_contract_tests {
    ($mod_name:ident, $source_fn:expr) => {
        #[cfg(test)]
        mod $mod_name {
            use $crate::time::SimTime;
            use $crate::workload::WorkloadSource;

            #[test]
            fn services_positive() {
                let source = $source_fn();
                assert!(source.services() > 0, "services() must be positive");
            }

            #[test]
            fn demand_finite_and_nonnegative() {
                let source = $source_fn();
                let services_count = source.services();

                // Test every 15 minutes for 24 hours
                for minute_offset in (0..=1440).step_by(15) {
                    let t = SimTime::from_millis((minute_offset as u64) * 60 * 1000);
                    for svc_idx in 0..services_count {
                        let service_id = $crate::ids::ServiceId(svc_idx as u32);
                        let demand = source.demand(service_id, t);

                        assert!(
                            demand.request_rate.is_finite(),
                            "request_rate must be finite at service {} time {}",
                            svc_idx,
                            minute_offset
                        );
                        assert!(
                            demand.request_rate >= 0.0,
                            "request_rate must be non-negative at service {} time {}",
                            svc_idx,
                            minute_offset
                        );
                        assert!(
                            demand.cpu_millis.is_finite(),
                            "cpu_millis must be finite at service {} time {}",
                            svc_idx,
                            minute_offset
                        );
                        assert!(
                            demand.cpu_millis >= 0.0,
                            "cpu_millis must be non-negative at service {} time {}",
                            svc_idx,
                            minute_offset
                        );
                        assert!(
                            demand.memory_bytes.is_finite(),
                            "memory_bytes must be finite at service {} time {}",
                            svc_idx,
                            minute_offset
                        );
                        assert!(
                            demand.memory_bytes >= 0.0,
                            "memory_bytes must be non-negative at service {} time {}",
                            svc_idx,
                            minute_offset
                        );
                    }
                }
            }

            #[test]
            fn demand_is_pure() {
                let source = $source_fn();
                let services_count = source.services();

                for svc_idx in 0..services_count {
                    let service_id = $crate::ids::ServiceId(svc_idx as u32);
                    let t = SimTime::from_millis(3600000); // 1 hour

                    let demand1 = source.demand(service_id, t);
                    let demand2 = source.demand(service_id, t);

                    assert_eq!(
                        demand1, demand2,
                        "demand must be pure: same inputs must give same outputs for service {}",
                        svc_idx
                    );
                }
            }
        }
    };
}

/// Generate contract tests for `PerformanceModel` implementations.
/// Tests: zero demand with ready pods ⇒ zero cpu usage, `served_rps` 0, `error_rate` 0;
/// outputs finite and non-negative; `error_rate` and `throttled_fraction` within [0,1];
/// latency p50 <= p95 <= p99; deterministic.
#[macro_export]
macro_rules! performance_model_contract_tests {
    ($mod_name:ident, $model_fn:expr) => {
        #[cfg(test)]
        mod $mod_name {
            use $crate::performance::{PerformanceModel, ServiceCapacity, ServiceSignals};
            use $crate::resources::Resources;
            use $crate::workload::ServiceDemand;

            #[test]
            fn zero_demand_zero_usage() {
                let model = $model_fn();
                let capacity = ServiceCapacity {
                    ready_pods: 1,
                    requests: Resources::new(100, (128 * 1024 * 1024) as u64),
                    limits: Resources::new(1000, (512 * 1024 * 1024) as u64),
                };
                let demand = ServiceDemand {
                    request_rate: 0.0,
                    cpu_millis: 0.0,
                    memory_bytes: 0.0,
                };

                let signals = model.evaluate(&capacity, &demand);

                assert!(
                    signals.cpu_usage_millis == 0.0,
                    "zero demand must give zero cpu usage"
                );
                assert!(
                    signals.served_rps == 0.0,
                    "zero demand must give zero served_rps"
                );
                assert!(
                    signals.error_rate == 0.0,
                    "zero demand must give zero error_rate"
                );
            }

            #[test]
            fn outputs_finite_nonnegative() {
                let model = $model_fn();
                let capacity = ServiceCapacity {
                    ready_pods: 2,
                    requests: Resources::new(100, (128 * 1024 * 1024) as u64),
                    limits: Resources::new(1000, (512 * 1024 * 1024) as u64),
                };
                let demand = ServiceDemand {
                    request_rate: 10.0,
                    cpu_millis: 500.0,
                    memory_bytes: f64::from(256 * 1024 * 1024),
                };

                let signals = model.evaluate(&capacity, &demand);

                assert!(
                    signals.cpu_usage_millis.is_finite(),
                    "cpu_usage_millis must be finite"
                );
                assert!(
                    signals.cpu_usage_millis >= 0.0,
                    "cpu_usage_millis must be non-negative"
                );

                assert!(
                    signals.memory_usage_bytes.is_finite(),
                    "memory_usage_bytes must be finite"
                );
                assert!(
                    signals.memory_usage_bytes >= 0.0,
                    "memory_usage_bytes must be non-negative"
                );

                assert!(signals.served_rps.is_finite(), "served_rps must be finite");
                assert!(signals.served_rps >= 0.0, "served_rps must be non-negative");

                assert!(
                    signals.latency_p50_ms.is_finite(),
                    "latency_p50_ms must be finite"
                );
                assert!(
                    signals.latency_p50_ms >= 0.0,
                    "latency_p50_ms must be non-negative"
                );

                assert!(
                    signals.latency_p95_ms.is_finite(),
                    "latency_p95_ms must be finite"
                );
                assert!(
                    signals.latency_p95_ms >= 0.0,
                    "latency_p95_ms must be non-negative"
                );

                assert!(
                    signals.latency_p99_ms.is_finite(),
                    "latency_p99_ms must be finite"
                );
                assert!(
                    signals.latency_p99_ms >= 0.0,
                    "latency_p99_ms must be non-negative"
                );
            }

            #[test]
            fn error_rate_in_range() {
                let model = $model_fn();
                let capacity = ServiceCapacity {
                    ready_pods: 1,
                    requests: Resources::new(100, (128 * 1024 * 1024) as u64),
                    limits: Resources::new(1000, (512 * 1024 * 1024) as u64),
                };
                let demand = ServiceDemand {
                    request_rate: 5.0,
                    cpu_millis: 200.0,
                    memory_bytes: f64::from(128 * 1024 * 1024),
                };

                let signals = model.evaluate(&capacity, &demand);

                assert!(
                    signals.error_rate >= 0.0 && signals.error_rate <= 1.0,
                    "error_rate must be in [0,1], got {}",
                    signals.error_rate
                );
            }

            #[test]
            fn throttled_fraction_in_range() {
                let model = $model_fn();
                let capacity = ServiceCapacity {
                    ready_pods: 1,
                    requests: Resources::new(100, (128 * 1024 * 1024) as u64),
                    limits: Resources::new(1000, (512 * 1024 * 1024) as u64),
                };
                let demand = ServiceDemand {
                    request_rate: 5.0,
                    cpu_millis: 200.0,
                    memory_bytes: f64::from(128 * 1024 * 1024),
                };

                let signals = model.evaluate(&capacity, &demand);

                assert!(
                    signals.throttled_fraction >= 0.0 && signals.throttled_fraction <= 1.0,
                    "throttled_fraction must be in [0,1], got {}",
                    signals.throttled_fraction
                );
            }

            #[test]
            fn latency_quantiles_ordered() {
                let model = $model_fn();
                let capacity = ServiceCapacity {
                    ready_pods: 2,
                    requests: Resources::new(100, (128 * 1024 * 1024) as u64),
                    limits: Resources::new(1000, (512 * 1024 * 1024) as u64),
                };
                let demand = ServiceDemand {
                    request_rate: 10.0,
                    cpu_millis: 500.0,
                    memory_bytes: f64::from(256 * 1024 * 1024),
                };

                let signals = model.evaluate(&capacity, &demand);

                assert!(
                    signals.latency_p50_ms <= signals.latency_p95_ms,
                    "p50 <= p95: {} <= {}",
                    signals.latency_p50_ms,
                    signals.latency_p95_ms
                );
                assert!(
                    signals.latency_p95_ms <= signals.latency_p99_ms,
                    "p95 <= p99: {} <= {}",
                    signals.latency_p95_ms,
                    signals.latency_p99_ms
                );
            }

            #[test]
            fn evaluate_deterministic() {
                let model = $model_fn();
                let capacity = ServiceCapacity {
                    ready_pods: 2,
                    requests: Resources::new(100, (128 * 1024 * 1024) as u64),
                    limits: Resources::new(1000, (512 * 1024 * 1024) as u64),
                };
                let demand = ServiceDemand {
                    request_rate: 10.0,
                    cpu_millis: 500.0,
                    memory_bytes: f64::from(256 * 1024 * 1024),
                };

                let signals1 = model.evaluate(&capacity, &demand);
                let signals2 = model.evaluate(&capacity, &demand);

                assert_eq!(
                    signals1, signals2,
                    "evaluate must be deterministic: same inputs must give same outputs"
                );
            }
        }
    };
}
