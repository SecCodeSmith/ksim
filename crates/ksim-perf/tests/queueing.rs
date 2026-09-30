use ksim_core::performance::{PerformanceModel, ServiceCapacity};
use ksim_core::resources::Resources;
use ksim_core::workload::ServiceDemand;
use ksim_perf::queueing::{QueueingModel, mmc_wait_quantile};

#[test]
fn mmc_wait_quantile_c_half_p_half() {
    // M/M/1 with offered load 0.5: C(1, 0.5) = 0.5
    // Waiting time distribution: P(Wq > t) = 0.5 * exp(-0.5 * t)
    // For p=0.5: t = -ln(1 - 0.5) / 0.5 = -ln(0.5) / 0.5 = ln(2) / 0.5 ≈ 1.386 s
    // But formula is: P(Wq > t) = C(c, a) * exp(-(c*mu - lambda) * t)
    // For servers=1, lambda=0.5, mu=1: P(Wq > t) = 0.5 * exp(-(1 - 0.5) * t) = 0.5 * exp(-0.5*t)
    // Setting 0.5 = 0.5 * exp(-0.5*t) => 1 = exp(-0.5*t) => t = 0
    // For p=0.9: 0.1 = 0.5 * exp(-0.5*t) => 0.2 = exp(-0.5*t) => t = ln(5) / 0.5 = 2*ln(5) ≈ 3.2189
    let quantile = mmc_wait_quantile(1, 0.5, 1.0, 0.4);
    approx::assert_abs_diff_eq!(quantile, 0.0, epsilon = 1e-6);
}

#[test]
fn mmc_wait_quantile_c_half_p_high() {
    // For p=0.9 (high quantile): 0.1 = 0.5 * exp(-0.5*t)
    // => 0.2 = exp(-0.5*t) => t = -ln(0.2) / 0.5 = -ln(0.2) / 0.5 = 2*ln(5) ≈ 3.2189
    let quantile = mmc_wait_quantile(1, 0.5, 1.0, 0.9);
    let expected = 2.0 * 5.0_f64.ln();
    approx::assert_abs_diff_eq!(quantile, expected, epsilon = 1e-4);
}

#[test]
fn queueing_model_mmc_basic() {
    // QueueingModel::mmc(100, 10_000) creates a model with ca2=cs2=1
    let model = QueueingModel::mmc(100.0, 10_000.0);
    approx::assert_abs_diff_eq!(model.ca2, 1.0, epsilon = 1e-9);
    approx::assert_abs_diff_eq!(model.cs2, 1.0, epsilon = 1e-9);
    approx::assert_abs_diff_eq!(model.cpu_millis_per_request, 100.0, epsilon = 1e-9);
    approx::assert_abs_diff_eq!(model.timeout_ms, 10_000.0, epsilon = 1e-9);
}

#[test]
fn queueing_model_mmc_one_pod_low_load() {
    // 1 pod with 1000m limit: mu = 1000 / 100 = 10 rps/pod
    // Demand: 5 rps, 500m cpu => lambda = 5, mu = 10
    // Load a = 0.5 < 1 => stable
    // Served: all 5 rps
    // Error: 0
    // CPU usage: 500m
    // Latency: service_time (1/mu = 100ms) + wait_quantile
    let model = QueueingModel::mmc(100.0, 10_000.0);
    let capacity = ServiceCapacity {
        ready_pods: 1,
        requests: Resources::new(100, 128 * 1024 * 1024),
        limits: Resources::new(1000, 512 * 1024 * 1024),
    };
    let demand = ServiceDemand {
        request_rate: 5.0,
        cpu_millis: 500.0,
        memory_bytes: f64::from(128 * 1024 * 1024),
    };

    let signals = model.evaluate(&capacity, &demand);

    approx::assert_abs_diff_eq!(signals.served_rps, 5.0, epsilon = 1e-9);
    approx::assert_abs_diff_eq!(signals.error_rate, 0.0, epsilon = 1e-9);
    approx::assert_abs_diff_eq!(signals.cpu_usage_millis, 500.0, epsilon = 1e-9);
    // Latency p50: service_time + wait_quantile(1, 5, 10, 0.5)
    // For M/M/1 with rho=0.5, p50 quantile of waiting time is 0
    // So p50 latency = 100ms
    approx::assert_abs_diff_eq!(signals.latency_p50_ms, 100.0, epsilon = 1e-6);
}

#[test]
fn queueing_model_mmc_overload() {
    // 1 pod with 1000m limit: mu = 10 rps/pod
    // Demand: 20 rps (overload) => capped to 99% utilization = 9.9 rps
    // Timed out: P(Wq > 10 s - 0.1 s) = 0.99 * exp(-0.1 * 9.9) of the 9.9 served rps
    // Error rate: (10.1 dropped + 9.9 * 0.367861 timed out) / 20 ≈ 0.687091
    let model = QueueingModel::mmc(100.0, 10_000.0);
    let capacity = ServiceCapacity {
        ready_pods: 1,
        requests: Resources::new(100, 128 * 1024 * 1024),
        limits: Resources::new(1000, 512 * 1024 * 1024),
    };
    let demand = ServiceDemand {
        request_rate: 20.0,
        cpu_millis: 2000.0,
        memory_bytes: f64::from(256 * 1024 * 1024),
    };

    let signals = model.evaluate(&capacity, &demand);

    approx::assert_abs_diff_eq!(signals.served_rps, 9.9, epsilon = 1e-6);
    let timed_out = 9.9 * 0.99 * (-0.1_f64 * 9.9).exp();
    let expected_error = (20.0 - 9.9 + timed_out) / 20.0;
    approx::assert_abs_diff_eq!(signals.error_rate, expected_error, epsilon = 1e-6);
}

#[test]
fn queueing_model_zero_pods() {
    // Zero ready pods with demand => all requests error
    let model = QueueingModel::mmc(100.0, 10_000.0);
    let capacity = ServiceCapacity {
        ready_pods: 0,
        requests: Resources::new(100, 128 * 1024 * 1024),
        limits: Resources::new(1000, 512 * 1024 * 1024),
    };
    let demand = ServiceDemand {
        request_rate: 5.0,
        cpu_millis: 500.0,
        memory_bytes: f64::from(128 * 1024 * 1024),
    };

    let signals = model.evaluate(&capacity, &demand);

    approx::assert_abs_diff_eq!(signals.error_rate, 1.0, epsilon = 1e-9);
    approx::assert_abs_diff_eq!(signals.latency_p50_ms, 10_000.0, epsilon = 1e-9);
    approx::assert_abs_diff_eq!(signals.latency_p95_ms, 10_000.0, epsilon = 1e-9);
    approx::assert_abs_diff_eq!(signals.latency_p99_ms, 10_000.0, epsilon = 1e-9);
}

#[test]
fn queueing_model_allen_cunneen_mmc_equivalent() {
    // allen_cunneen with ca2=cs2=1 should equal mmc
    let mmc = QueueingModel::mmc(100.0, 10_000.0);
    let ac = QueueingModel::allen_cunneen(100.0, 10_000.0, 1.0, 1.0);

    let capacity = ServiceCapacity {
        ready_pods: 2,
        requests: Resources::new(100, 128 * 1024 * 1024),
        limits: Resources::new(1000, 512 * 1024 * 1024),
    };
    let demand = ServiceDemand {
        request_rate: 10.0,
        cpu_millis: 500.0,
        memory_bytes: f64::from(256 * 1024 * 1024),
    };

    let mmc_signals = mmc.evaluate(&capacity, &demand);
    let ac_signals = ac.evaluate(&capacity, &demand);

    approx::assert_abs_diff_eq!(
        mmc_signals.served_rps,
        ac_signals.served_rps,
        epsilon = 1e-9
    );
    approx::assert_abs_diff_eq!(
        mmc_signals.error_rate,
        ac_signals.error_rate,
        epsilon = 1e-9
    );
    approx::assert_abs_diff_eq!(
        mmc_signals.latency_p50_ms,
        ac_signals.latency_p50_ms,
        epsilon = 1e-6
    );
}

#[test]
fn queueing_model_allen_cunneen_variance_effect() {
    // With ca2=cs2=2, wait component should double vs ca2=cs2=1
    let ac1 = QueueingModel::allen_cunneen(100.0, 10_000.0, 1.0, 1.0);
    let ac2 = QueueingModel::allen_cunneen(100.0, 10_000.0, 2.0, 2.0);

    let capacity = ServiceCapacity {
        ready_pods: 2,
        requests: Resources::new(100, 128 * 1024 * 1024),
        limits: Resources::new(1000, 512 * 1024 * 1024),
    };
    let demand = ServiceDemand {
        request_rate: 10.0,
        cpu_millis: 500.0,
        memory_bytes: f64::from(256 * 1024 * 1024),
    };

    let signals1 = ac1.evaluate(&capacity, &demand);
    let signals2 = ac2.evaluate(&capacity, &demand);

    // With higher variance, latencies should increase
    assert!(
        signals2.latency_p50_ms >= signals1.latency_p50_ms,
        "Higher variance should increase latency"
    );
}

ksim_core::performance_model_contract_tests!(queueing_model_contracts, || {
    ksim_perf::queueing::QueueingModel::mmc(100.0, 10_000.0)
});
