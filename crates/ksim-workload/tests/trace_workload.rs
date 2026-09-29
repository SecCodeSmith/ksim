use std::sync::Arc;

use ksim_core::ids::ServiceId;
use ksim_core::time::{SimDuration, SimTime};
use ksim_core::workload::WorkloadSource;
use ksim_workload::trace::{
    encode, Interpolation, ServiceSeries, TraceError, TraceFile, TraceOptions, TraceWorkload,
};

fn make_trace() -> Arc<TraceFile> {
    // Service "a": cpu [100,200,300,400], rate [1,2,3,4], memory [10,20,30,40]
    // Service "b": cpu all 50, rate all 5, memory all 7
    let services = vec![
        ServiceSeries {
            name: "a".to_string(),
            request_rate: vec![1.0, 2.0, 3.0, 4.0],
            cpu_millis: vec![100.0, 200.0, 300.0, 400.0],
            memory_bytes: vec![10.0, 20.0, 30.0, 40.0],
        },
        ServiceSeries {
            name: "b".to_string(),
            request_rate: vec![5.0, 5.0, 5.0, 5.0],
            cpu_millis: vec![50.0, 50.0, 50.0, 50.0],
            memory_bytes: vec![7.0, 7.0, 7.0, 7.0],
        },
    ];

    let bytes = encode(60, &services).expect("encode should succeed");
    let file = TraceFile::from_bytes(bytes).expect("TraceFile::from_bytes should succeed");
    Arc::new(file)
}

#[test]
fn default_options_services_count() {
    let file = make_trace();
    let options = TraceOptions::default();
    let workload = TraceWorkload::new(file, options).expect("TraceWorkload::new should succeed");
    assert_eq!(workload.services(), 2, "default options should have 2 services");
}

#[test]
fn demand_at_t_zero() {
    let file = make_trace();
    let options = TraceOptions::default();
    let workload = TraceWorkload::new(file, options).expect("TraceWorkload::new should succeed");

    let demand = workload.demand(ServiceId(0), SimTime::ZERO);
    // Tick 0 for service "a": cpu 100, rate 1, memory 10
    approx::assert_abs_diff_eq!(demand.cpu_millis, 100.0, epsilon = 1e-6);
    approx::assert_abs_diff_eq!(demand.request_rate, 1.0, epsilon = 1e-6);
    approx::assert_abs_diff_eq!(demand.memory_bytes, 10.0, epsilon = 1e-6);
}

#[test]
fn step_interpolation_at_t_59s() {
    let file = make_trace();
    let options = TraceOptions::default();
    let workload = TraceWorkload::new(file, options).expect("TraceWorkload::new should succeed");

    // With Step interpolation and tick_seconds=60, at t=59s we still read tick 0
    let demand = workload.demand(ServiceId(0), SimTime::from_millis(59_000));
    approx::assert_abs_diff_eq!(demand.cpu_millis, 100.0, epsilon = 1e-6);
    approx::assert_abs_diff_eq!(demand.request_rate, 1.0, epsilon = 1e-6);
}

#[test]
fn step_interpolation_at_t_60s() {
    let file = make_trace();
    let options = TraceOptions::default();
    let workload = TraceWorkload::new(file, options).expect("TraceWorkload::new should succeed");

    // At t=60s, we switch to tick 1
    let demand = workload.demand(ServiceId(0), SimTime::from_millis(60_000));
    approx::assert_abs_diff_eq!(demand.cpu_millis, 200.0, epsilon = 1e-6);
    approx::assert_abs_diff_eq!(demand.request_rate, 2.0, epsilon = 1e-6);
}

#[test]
fn linear_interpolation_at_t_90s() {
    let file = make_trace();
    let options = TraceOptions {
        services: None,
        start: SimDuration::ZERO,
        compress: 1.0,
        scale: 1.0,
        interpolation: Interpolation::Linear,
        wrap: false,
    };
    let workload = TraceWorkload::new(file, options).expect("TraceWorkload::new should succeed");

    // At t=90s, we are between ticks 1 and 2 (at 1.5)
    // Tick 1: (200, 2, 20), Tick 2: (300, 3, 30)
    // Linear interpolation at 0.5 factor: (250, 2.5, 25)
    let demand = workload.demand(ServiceId(0), SimTime::from_millis(90_000));
    approx::assert_abs_diff_eq!(demand.cpu_millis, 250.0, epsilon = 1e-6);
    approx::assert_abs_diff_eq!(demand.request_rate, 2.5, epsilon = 1e-6);
    approx::assert_abs_diff_eq!(demand.memory_bytes, 25.0, epsilon = 1e-6);
}

#[test]
fn hold_past_end_without_wrap() {
    let file = make_trace();
    let options = TraceOptions::default();
    let workload = TraceWorkload::new(file, options).expect("TraceWorkload::new should succeed");

    // At t=600s (10 min), we are past the last tick (tick 3 at t=180s)
    // Without wrap, we hold the last value (tick 3: 400, 4, 40)
    let demand = workload.demand(ServiceId(0), SimTime::from_millis(600_000));
    approx::assert_abs_diff_eq!(demand.cpu_millis, 400.0, epsilon = 1e-6);
    approx::assert_abs_diff_eq!(demand.request_rate, 4.0, epsilon = 1e-6);
}

#[test]
fn wrap_at_t_240s() {
    let file = make_trace();
    let options = TraceOptions {
        services: None,
        start: SimDuration::ZERO,
        compress: 1.0,
        scale: 1.0,
        interpolation: Interpolation::Step,
        wrap: true,
    };
    let workload = TraceWorkload::new(file, options).expect("TraceWorkload::new should succeed");

    // At t=240s, trace_index = 240/60 = 4, with wrap we get 4 % 4 = 0 (tick 0)
    let demand = workload.demand(ServiceId(0), SimTime::from_millis(240_000));
    approx::assert_abs_diff_eq!(demand.cpu_millis, 100.0, epsilon = 1e-6);
    approx::assert_abs_diff_eq!(demand.request_rate, 1.0, epsilon = 1e-6);
}

#[test]
fn wrap_at_t_300s() {
    let file = make_trace();
    let options = TraceOptions {
        services: None,
        start: SimDuration::ZERO,
        compress: 1.0,
        scale: 1.0,
        interpolation: Interpolation::Step,
        wrap: true,
    };
    let workload = TraceWorkload::new(file, options).expect("TraceWorkload::new should succeed");

    // At t=300s, trace_index = 300/60 = 5, with wrap we get 5 % 4 = 1 (tick 1)
    let demand = workload.demand(ServiceId(0), SimTime::from_millis(300_000));
    approx::assert_abs_diff_eq!(demand.cpu_millis, 200.0, epsilon = 1e-6);
    approx::assert_abs_diff_eq!(demand.request_rate, 2.0, epsilon = 1e-6);
}

#[test]
fn start_offset_120s() {
    let file = make_trace();
    let options = TraceOptions {
        services: None,
        start: SimDuration::from_secs(120),
        compress: 1.0,
        scale: 1.0,
        interpolation: Interpolation::Step,
        wrap: false,
    };
    let workload = TraceWorkload::new(file, options).expect("TraceWorkload::new should succeed");

    // At t=0, trace_time = 120 + 0*1.0 = 120, trace_index = 120/60 = 2 (tick 2)
    let demand = workload.demand(ServiceId(0), SimTime::ZERO);
    approx::assert_abs_diff_eq!(demand.cpu_millis, 300.0, epsilon = 1e-6);
}

#[test]
fn compress_2_0() {
    let file = make_trace();
    let options = TraceOptions {
        services: None,
        start: SimDuration::ZERO,
        compress: 2.0,
        scale: 1.0,
        interpolation: Interpolation::Step,
        wrap: false,
    };
    let workload = TraceWorkload::new(file, options).expect("TraceWorkload::new should succeed");

    // At t=60s, trace_time = 0 + 60*2.0 = 120, trace_index = 120/60 = 2 (tick 2)
    let demand = workload.demand(ServiceId(0), SimTime::from_millis(60_000));
    approx::assert_abs_diff_eq!(demand.cpu_millis, 300.0, epsilon = 1e-6);
}

#[test]
fn scale_2_0() {
    let file = make_trace();
    let options = TraceOptions {
        services: None,
        start: SimDuration::ZERO,
        compress: 1.0,
        scale: 2.0,
        interpolation: Interpolation::Step,
        wrap: false,
    };
    let workload = TraceWorkload::new(file, options).expect("TraceWorkload::new should succeed");

    // At t=0, tick 0: (100, 1, 10)
    // With scale=2.0: cpu and rate are scaled, but memory is not
    // Result: (200, 2, 10)
    let demand = workload.demand(ServiceId(0), SimTime::ZERO);
    approx::assert_abs_diff_eq!(demand.cpu_millis, 200.0, epsilon = 1e-6);
    approx::assert_abs_diff_eq!(demand.request_rate, 2.0, epsilon = 1e-6);
    approx::assert_abs_diff_eq!(demand.memory_bytes, 10.0, epsilon = 1e-6);
}

#[test]
fn services_reordered() {
    let file = make_trace();
    let options = TraceOptions {
        services: Some(vec!["b".to_string(), "a".to_string()]),
        start: SimDuration::ZERO,
        compress: 1.0,
        scale: 1.0,
        interpolation: Interpolation::Step,
        wrap: false,
    };
    let workload = TraceWorkload::new(file, options).expect("TraceWorkload::new should succeed");

    // ServiceId(0) should now be "b", which has cpu=50 at tick 0
    let demand = workload.demand(ServiceId(0), SimTime::ZERO);
    approx::assert_abs_diff_eq!(demand.cpu_millis, 50.0, epsilon = 1e-6);
}

#[test]
fn unknown_service_name() {
    let file = make_trace();
    let options = TraceOptions {
        services: Some(vec!["zzz".to_string()]),
        start: SimDuration::ZERO,
        compress: 1.0,
        scale: 1.0,
        interpolation: Interpolation::Step,
        wrap: false,
    };
    let result = TraceWorkload::new(file, options);
    assert!(result.is_err(), "unknown service should error");
    match result.unwrap_err() {
        TraceError::UnknownService(name) => {
            assert_eq!(name, "zzz");
        }
        other => panic!("expected UnknownService, got {other:?}"),
    }
}

#[test]
fn compress_zero_is_malformed() {
    let file = make_trace();
    let options = TraceOptions {
        services: None,
        start: SimDuration::ZERO,
        compress: 0.0,
        scale: 1.0,
        interpolation: Interpolation::Step,
        wrap: false,
    };
    let result = TraceWorkload::new(file, options);
    assert!(result.is_err(), "compress=0 should error");
    match result.unwrap_err() {
        TraceError::Malformed(_) => {
            // expected
        }
        other => panic!("expected Malformed, got {other:?}"),
    }
}

#[test]
fn scale_negative_is_malformed() {
    let file = make_trace();
    let options = TraceOptions {
        services: None,
        start: SimDuration::ZERO,
        compress: 1.0,
        scale: -1.0,
        interpolation: Interpolation::Step,
        wrap: false,
    };
    let result = TraceWorkload::new(file, options);
    assert!(result.is_err(), "scale=-1 should error");
    match result.unwrap_err() {
        TraceError::Malformed(_) => {
            // expected
        }
        other => panic!("expected Malformed, got {other:?}"),
    }
}

#[test]
fn scale_nan_is_malformed() {
    let file = make_trace();
    let options = TraceOptions {
        services: None,
        start: SimDuration::ZERO,
        compress: 1.0,
        scale: f64::NAN,
        interpolation: Interpolation::Step,
        wrap: false,
    };
    let result = TraceWorkload::new(file, options);
    assert!(result.is_err(), "scale=NaN should error");
    match result.unwrap_err() {
        TraceError::Malformed(_) => {
            // expected
        }
        other => panic!("expected Malformed, got {other:?}"),
    }
}

ksim_core::workload_source_contract_tests!(trace_workload_contracts, || {
    use std::sync::Arc;
    use ksim_workload::trace::{encode, ServiceSeries, TraceFile, TraceOptions, TraceWorkload};

    // Helper inline to ensure scope access
    let services = vec![
        ServiceSeries {
            name: "a".to_string(),
            request_rate: vec![1.0, 2.0, 3.0, 4.0],
            cpu_millis: vec![100.0, 200.0, 300.0, 400.0],
            memory_bytes: vec![10.0, 20.0, 30.0, 40.0],
        },
        ServiceSeries {
            name: "b".to_string(),
            request_rate: vec![5.0, 5.0, 5.0, 5.0],
            cpu_millis: vec![50.0, 50.0, 50.0, 50.0],
            memory_bytes: vec![7.0, 7.0, 7.0, 7.0],
        },
    ];

    let bytes = encode(60, &services).expect("encode should succeed");
    let file = TraceFile::from_bytes(bytes).expect("TraceFile::from_bytes should succeed");
    let file = Arc::new(file);
    let options = TraceOptions::default();
    TraceWorkload::new(file, options).expect("TraceWorkload::new should succeed")
});
