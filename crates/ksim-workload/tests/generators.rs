use ksim_core::ids::ServiceId;
use ksim_core::time::{SimDuration, SimTime};
use ksim_core::workload::WorkloadSource;
use ksim_workload::synthetic::{
    Constant, CurveWorkload, DemandCurve, DemandProfile, FlashCrowd, Ramp, Sinusoid, Step,
};
use proptest::prelude::*;

#[test]
fn constant_load() {
    let constant = Constant { rate: 100.0 };
    approx::assert_abs_diff_eq!(constant.load(SimTime::ZERO), 100.0, epsilon = 1e-9);
    approx::assert_abs_diff_eq!(
        constant.load(SimTime::from_millis(3_600_000)),
        100.0,
        epsilon = 1e-9
    );
}

#[test]
fn step_before_transition() {
    let step = Step {
        before: 50.0,
        after: 150.0,
        at: SimTime::from_millis(3_600_000), // 1 hour
    };
    approx::assert_abs_diff_eq!(step.load(SimTime::ZERO), 50.0, epsilon = 1e-9);
    approx::assert_abs_diff_eq!(
        step.load(SimTime::from_millis(1_800_000)),
        50.0,
        epsilon = 1e-9
    );
}

#[test]
fn step_at_transition() {
    let step = Step {
        before: 50.0,
        after: 150.0,
        at: SimTime::from_millis(3_600_000), // 1 hour
    };
    approx::assert_abs_diff_eq!(
        step.load(SimTime::from_millis(3_600_000)),
        150.0,
        epsilon = 1e-9
    );
}

#[test]
fn step_after_transition() {
    let step = Step {
        before: 50.0,
        after: 150.0,
        at: SimTime::from_millis(3_600_000), // 1 hour
    };
    approx::assert_abs_diff_eq!(
        step.load(SimTime::from_millis(7_200_000)),
        150.0,
        epsilon = 1e-9
    );
}

#[test]
fn ramp_before_start() {
    let ramp = Ramp {
        from: 0.0,
        to: 100.0,
        start: SimTime::from_millis(3_600_000), // 1 hour
        end: SimTime::from_millis(7_200_000),   // 2 hours
    };
    approx::assert_abs_diff_eq!(ramp.load(SimTime::ZERO), 0.0, epsilon = 1e-9);
}

#[test]
fn ramp_at_start() {
    let ramp = Ramp {
        from: 0.0,
        to: 100.0,
        start: SimTime::from_millis(3_600_000), // 1 hour
        end: SimTime::from_millis(7_200_000),   // 2 hours
    };
    approx::assert_abs_diff_eq!(
        ramp.load(SimTime::from_millis(3_600_000)),
        0.0,
        epsilon = 1e-9
    );
}

#[test]
fn ramp_midpoint() {
    let ramp = Ramp {
        from: 0.0,
        to: 100.0,
        start: SimTime::from_millis(3_600_000), // 1 hour
        end: SimTime::from_millis(7_200_000),   // 2 hours
    };
    let midpoint = SimTime::from_millis(5_400_000); // 1.5 hours
    let load = ramp.load(midpoint);
    approx::assert_abs_diff_eq!(load, 50.0, epsilon = 1e-9);
}

#[test]
fn ramp_at_end() {
    let ramp = Ramp {
        from: 0.0,
        to: 100.0,
        start: SimTime::from_millis(3_600_000), // 1 hour
        end: SimTime::from_millis(7_200_000),   // 2 hours
    };
    approx::assert_abs_diff_eq!(
        ramp.load(SimTime::from_millis(7_200_000)),
        100.0,
        epsilon = 1e-9
    );
}

#[test]
fn ramp_after_end() {
    let ramp = Ramp {
        from: 0.0,
        to: 100.0,
        start: SimTime::from_millis(3_600_000), // 1 hour
        end: SimTime::from_millis(7_200_000),   // 2 hours
    };
    approx::assert_abs_diff_eq!(
        ramp.load(SimTime::from_millis(10_800_000)),
        100.0,
        epsilon = 1e-9
    );
}

#[test]
fn flash_crowd_before_start() {
    let flash = FlashCrowd {
        base: 100.0,
        peak: 200.0,
        start: SimTime::from_millis(3_600_000), // 1 hour
        decay: SimDuration::from_secs(1800),    // 30 minutes
    };
    approx::assert_abs_diff_eq!(flash.load(SimTime::ZERO), 100.0, epsilon = 1e-9);
    approx::assert_abs_diff_eq!(
        flash.load(SimTime::from_millis(1_800_000)),
        100.0,
        epsilon = 1e-9
    );
}

#[test]
fn flash_crowd_at_start() {
    let flash = FlashCrowd {
        base: 100.0,
        peak: 200.0,
        start: SimTime::from_millis(3_600_000), // 1 hour
        decay: SimDuration::from_secs(1800),    // 30 minutes
    };
    approx::assert_abs_diff_eq!(
        flash.load(SimTime::from_millis(3_600_000)),
        200.0,
        epsilon = 1e-9
    );
}

#[test]
fn flash_crowd_after_one_decay() {
    let flash = FlashCrowd {
        base: 100.0,
        peak: 200.0,
        start: SimTime::from_millis(3_600_000),
        decay: SimDuration::from_secs(1800), // 30 minutes
    };
    // After one decay: base + (peak - base) * exp(-1) = 100 + 100 * exp(-1)
    let expected = 100.0 + (200.0 - 100.0) * (-1.0_f64).exp();
    let at_one_decay = SimTime::from_millis(3_600_000 + 1_800_000);
    approx::assert_abs_diff_eq!(flash.load(at_one_decay), expected, epsilon = 1e-6);
}

#[test]
fn sinusoid_noiseless_at_zero() {
    let sin = Sinusoid {
        mean: 50.0,
        amplitude: 25.0,
        period: SimDuration::from_secs(3600), // 1 hour
        noise_sd: 0.0,
        seed: 0,
    };
    approx::assert_abs_diff_eq!(sin.load(SimTime::ZERO), 50.0, epsilon = 1e-9);
}

#[test]
fn sinusoid_noiseless_quarter_period() {
    let sin = Sinusoid {
        mean: 50.0,
        amplitude: 25.0,
        period: SimDuration::from_secs(3600), // 1 hour
        noise_sd: 0.0,
        seed: 0,
    };
    // At quarter period: mean + amplitude * sin(π/2) = mean + amplitude
    let quarter_period = SimTime::from_millis(900_000); // 15 minutes
    approx::assert_abs_diff_eq!(sin.load(quarter_period), 75.0, epsilon = 1e-6);
}

#[test]
fn sinusoid_noiseless_periodic() {
    let sin = Sinusoid {
        mean: 50.0,
        amplitude: 25.0,
        period: SimDuration::from_secs(3600), // 1 hour
        noise_sd: 0.0,
        seed: 0,
    };
    let at_zero = sin.load(SimTime::ZERO);
    let at_period = sin.load(SimTime::from_millis(3_600_000)); // 1 hour
    approx::assert_abs_diff_eq!(at_zero, at_period, epsilon = 1e-9);
}

#[test]
fn sinusoid_noisy_same_seed() {
    let sin1 = Sinusoid {
        mean: 50.0,
        amplitude: 25.0,
        period: SimDuration::from_secs(3600),
        noise_sd: 10.0,
        seed: 42,
    };
    let sin2 = Sinusoid {
        mean: 50.0,
        amplitude: 25.0,
        period: SimDuration::from_secs(3600),
        noise_sd: 10.0,
        seed: 42,
    };
    let t = SimTime::from_millis(1_800_000);
    approx::assert_abs_diff_eq!(sin1.load(t), sin2.load(t), epsilon = 1e-12);
}

#[test]
fn sinusoid_noisy_clamped() {
    let sin = Sinusoid {
        mean: 50.0,
        amplitude: 25.0,
        period: SimDuration::from_secs(3600),
        noise_sd: 100.0, // Large noise
        seed: 0,
    };
    for ms in (0..=3_600_000).step_by(100_000) {
        let load = sin.load(SimTime::from_millis(ms));
        assert!(
            load >= 0.0,
            "Noisy sinusoid must be clamped at 0, got {load}"
        );
    }
}

#[test]
fn demand_profile_formula() {
    let profile = DemandProfile {
        cpu_millis_per_request: 50.0,
        memory_base_bytes: 1_000_000.0,
        memory_bytes_per_rps: 1000.0,
    };
    let demand = profile.demand(100.0);
    approx::assert_abs_diff_eq!(demand.request_rate, 100.0, epsilon = 1e-9);
    approx::assert_abs_diff_eq!(demand.cpu_millis, 5000.0, epsilon = 1e-9);
    approx::assert_abs_diff_eq!(demand.memory_bytes, 1_100_000.0, epsilon = 1e-9);
}

#[test]
fn curve_workload_services_count() {
    let curves: Vec<(Box<dyn DemandCurve>, DemandProfile)> = vec![
        (
            Box::new(Constant { rate: 10.0 }),
            DemandProfile {
                cpu_millis_per_request: 50.0,
                memory_base_bytes: 1_000_000.0,
                memory_bytes_per_rps: 1000.0,
            },
        ),
        (
            Box::new(Constant { rate: 20.0 }),
            DemandProfile {
                cpu_millis_per_request: 100.0,
                memory_base_bytes: 2_000_000.0,
                memory_bytes_per_rps: 500.0,
            },
        ),
    ];
    let workload = CurveWorkload::new(curves);
    assert_eq!(
        workload.services(),
        2,
        "CurveWorkload should have 2 services"
    );
}

#[test]
fn curve_workload_demand_per_service() {
    let curves: Vec<(Box<dyn DemandCurve>, DemandProfile)> = vec![
        (
            Box::new(Constant { rate: 10.0 }),
            DemandProfile {
                cpu_millis_per_request: 50.0,
                memory_base_bytes: 100_000.0,
                memory_bytes_per_rps: 1000.0,
            },
        ),
        (
            Box::new(Constant { rate: 20.0 }),
            DemandProfile {
                cpu_millis_per_request: 100.0,
                memory_base_bytes: 200_000.0,
                memory_bytes_per_rps: 500.0,
            },
        ),
    ];
    let workload = CurveWorkload::new(curves);

    let demand0 = workload.demand(ServiceId(0), SimTime::ZERO);
    approx::assert_abs_diff_eq!(demand0.request_rate, 10.0, epsilon = 1e-9);
    approx::assert_abs_diff_eq!(demand0.cpu_millis, 500.0, epsilon = 1e-9);
    approx::assert_abs_diff_eq!(demand0.memory_bytes, 110_000.0, epsilon = 1e-9);

    let demand1 = workload.demand(ServiceId(1), SimTime::ZERO);
    approx::assert_abs_diff_eq!(demand1.request_rate, 20.0, epsilon = 1e-9);
    approx::assert_abs_diff_eq!(demand1.cpu_millis, 2000.0, epsilon = 1e-9);
    approx::assert_abs_diff_eq!(demand1.memory_bytes, 210_000.0, epsilon = 1e-9);
}

proptest! {
    #[test]
    fn constant_always_nonnegative(rate in 0.0_f64..1000.0) {
        let constant = Constant { rate };
        prop_assert!(constant.load(SimTime::ZERO) >= 0.0);
    }

    #[test]
    fn step_always_nonnegative(before in 0.0_f64..1000.0, after in 0.0_f64..1000.0) {
        let step = Step {
            before,
            after,
            at: SimTime::from_millis(3_600_000),
        };
        prop_assert!(step.load(SimTime::ZERO) >= 0.0);
        prop_assert!(step.load(SimTime::from_millis(7_200_000)) >= 0.0);
    }

    #[test]
    fn ramp_always_nonnegative(
        from in 0.0_f64..1000.0,
        to in 0.0_f64..1000.0,
        start_ms in 0u64..1_000_000,
        end_ms in 2_000_000u64..3_000_000
    ) {
        let ramp = Ramp {
            from,
            to,
            start: SimTime::from_millis(start_ms),
            end: SimTime::from_millis(end_ms),
        };
        prop_assert!(ramp.load(SimTime::from_millis(start_ms)) >= 0.0);
        prop_assert!(ramp.load(SimTime::from_millis(u64::midpoint(start_ms, end_ms))) >= 0.0);
        prop_assert!(ramp.load(SimTime::from_millis(end_ms)) >= 0.0);
    }

    #[test]
    fn flash_crowd_always_nonnegative(
        base in 0.0_f64..1000.0,
        peak in 0.0_f64..2000.0
    ) {
        let flash = FlashCrowd {
            base,
            peak,
            start: SimTime::from_millis(3_600_000),
            decay: SimDuration::from_secs(1800),
        };
        prop_assert!(flash.load(SimTime::ZERO) >= 0.0);
        prop_assert!(flash.load(SimTime::from_millis(3_600_000)) >= 0.0);
        prop_assert!(flash.load(SimTime::from_millis(7_200_000)) >= 0.0);
    }

    #[test]
    fn sinusoid_always_nonnegative(
        mean in 50.0_f64..1000.0,
        amplitude in 0.0_f64..100.0,
        time_ms in 0u64..86_400_000
    ) {
        let sin = Sinusoid {
            mean,
            amplitude,
            period: SimDuration::from_secs(3600),
            noise_sd: 0.0,
            seed: 0,
        };
        prop_assert!(sin.load(SimTime::from_millis(time_ms)) >= 0.0);
    }
}

ksim_core::workload_source_contract_tests!(curve_workload_contracts, || {
    use ksim_workload::synthetic::{Constant, CurveWorkload, DemandCurve, DemandProfile};
    let curves: Vec<(Box<dyn DemandCurve>, DemandProfile)> = vec![(
        Box::new(Constant { rate: 10.0 }),
        DemandProfile {
            cpu_millis_per_request: 50.0,
            memory_base_bytes: 100_000.0,
            memory_bytes_per_rps: 1000.0,
        },
    )];
    CurveWorkload::new(curves)
});
