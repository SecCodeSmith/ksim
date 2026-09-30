use ksim_core::performance::{PerformanceModel, ServiceCapacity};
use ksim_core::resources::Resources;
use ksim_core::workload::ServiceDemand;
use ksim_perf::utilization::UtilizationModel;

#[test]
fn utilization_model_zero_demand_zero_usage() {
    // Zero demand with ready pods => zero cpu usage, no throttling
    let model = UtilizationModel {
        base_latency_ms: 20.0,
    };
    let capacity = ServiceCapacity {
        ready_pods: 2,
        requests: Resources::new(500, 256 * 1024 * 1024),
        limits: Resources::new(1000, 512 * 1024 * 1024),
    };
    let demand = ServiceDemand {
        request_rate: 0.0,
        cpu_millis: 0.0,
        memory_bytes: 0.0,
    };

    let signals = model.evaluate(&capacity, &demand);

    approx::assert_abs_diff_eq!(signals.cpu_usage_millis, 0.0, epsilon = 1e-9);
    approx::assert_abs_diff_eq!(signals.throttled_fraction, 0.0, epsilon = 1e-9);
    approx::assert_abs_diff_eq!(signals.error_rate, 0.0, epsilon = 1e-9);
}

#[test]
fn utilization_model_no_throttle_under_limit() {
    // 2 pods with 1000m limit each = 2000m total
    // Demand: 1000m => no throttling
    let model = UtilizationModel {
        base_latency_ms: 20.0,
    };
    let capacity = ServiceCapacity {
        ready_pods: 2,
        requests: Resources::new(500, 256 * 1024 * 1024),
        limits: Resources::new(1000, 512 * 1024 * 1024),
    };
    let demand = ServiceDemand {
        request_rate: 10.0,
        cpu_millis: 1000.0,
        memory_bytes: f64::from(256 * 1024 * 1024),
    };

    let signals = model.evaluate(&capacity, &demand);

    approx::assert_abs_diff_eq!(signals.cpu_usage_millis, 1000.0, epsilon = 1e-9);
    approx::assert_abs_diff_eq!(signals.throttled_fraction, 0.0, epsilon = 1e-9);
    approx::assert_abs_diff_eq!(signals.error_rate, 0.0, epsilon = 1e-9);
    // Latency should be base_latency with no throttle inflation
    approx::assert_abs_diff_eq!(signals.latency_p50_ms, 20.0, epsilon = 1e-9);
}

#[test]
fn utilization_model_throttle_excess() {
    // 2 pods with 1000m limit each = 2000m total
    // Demand: 4000m => 4000 - 2000 = 2000m throttled
    // Throttled fraction: 2000 / 4000 = 0.5
    // Latency inflation: base * (demand / capacity) = 20 * (4000 / 2000) = 40
    let model = UtilizationModel {
        base_latency_ms: 20.0,
    };
    let capacity = ServiceCapacity {
        ready_pods: 2,
        requests: Resources::new(500, 256 * 1024 * 1024),
        limits: Resources::new(1000, 512 * 1024 * 1024),
    };
    let demand = ServiceDemand {
        request_rate: 40.0,
        cpu_millis: 4000.0,
        memory_bytes: f64::from(256 * 1024 * 1024),
    };

    let signals = model.evaluate(&capacity, &demand);

    // CPU usage capped at pod limits
    approx::assert_abs_diff_eq!(signals.cpu_usage_millis, 2000.0, epsilon = 1e-9);
    // Throttled fraction: excess / demand = 2000 / 4000
    approx::assert_abs_diff_eq!(signals.throttled_fraction, 0.5, epsilon = 1e-9);
    // Latency inflates by throttling factor
    approx::assert_abs_diff_eq!(signals.latency_p50_ms, 40.0, epsilon = 1e-9);
}

#[test]
fn utilization_model_memory_oom() {
    // Per-pod memory above limit sets oom
    // 2 pods with 512MiB limit each
    // Demand: 1GiB per pod = above limit => oom=true
    let model = UtilizationModel {
        base_latency_ms: 20.0,
    };
    let capacity = ServiceCapacity {
        ready_pods: 2,
        requests: Resources::new(500, 256 * 1024 * 1024),
        limits: Resources::new(1000, 512 * 1024 * 1024),
    };
    let per_pod_demand = f64::from(512 * 1024 * 1024) + 1.0; // Just over limit
    let total_demand = per_pod_demand * 2.0;
    let demand = ServiceDemand {
        request_rate: 10.0,
        cpu_millis: 500.0,
        memory_bytes: total_demand,
    };

    let signals = model.evaluate(&capacity, &demand);

    assert!(
        signals.oom,
        "OOM should be set when per-pod memory exceeds limit"
    );
}

#[test]
fn utilization_model_zero_pods_with_demand() {
    // Zero ready pods with non-zero demand => error_rate = 1
    let model = UtilizationModel {
        base_latency_ms: 20.0,
    };
    let capacity = ServiceCapacity {
        ready_pods: 0,
        requests: Resources::new(500, 256 * 1024 * 1024),
        limits: Resources::new(1000, 512 * 1024 * 1024),
    };
    let demand = ServiceDemand {
        request_rate: 5.0,
        cpu_millis: 500.0,
        memory_bytes: f64::from(128 * 1024 * 1024),
    };

    let signals = model.evaluate(&capacity, &demand);

    approx::assert_abs_diff_eq!(signals.error_rate, 1.0, epsilon = 1e-9);
}

#[test]
fn utilization_model_zero_pods_zero_demand() {
    // Zero ready pods with zero demand => error_rate = 0
    let model = UtilizationModel {
        base_latency_ms: 20.0,
    };
    let capacity = ServiceCapacity {
        ready_pods: 0,
        requests: Resources::new(500, 256 * 1024 * 1024),
        limits: Resources::new(1000, 512 * 1024 * 1024),
    };
    let demand = ServiceDemand {
        request_rate: 0.0,
        cpu_millis: 0.0,
        memory_bytes: 0.0,
    };

    let signals = model.evaluate(&capacity, &demand);

    approx::assert_abs_diff_eq!(signals.error_rate, 0.0, epsilon = 1e-9);
}

ksim_core::performance_model_contract_tests!(utilization_model_contracts, || {
    ksim_perf::utilization::UtilizationModel {
        base_latency_ms: 20.0,
    }
});
