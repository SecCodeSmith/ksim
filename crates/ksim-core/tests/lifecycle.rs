use ksim_core::{
    lifecycle::{crash_loop_backoff, DefaultLatencies, FixedLatencies, LatencyModel},
    rng::{Concern, RngStreams},
    time::SimDuration,
};

#[test]
fn crash_loop_backoff_10s_at_0() {
    assert_eq!(crash_loop_backoff(0), SimDuration::from_secs(10));
}

#[test]
fn crash_loop_backoff_20s_at_1() {
    assert_eq!(crash_loop_backoff(1), SimDuration::from_secs(20));
}

#[test]
fn crash_loop_backoff_40s_at_2() {
    assert_eq!(crash_loop_backoff(2), SimDuration::from_secs(40));
}

#[test]
fn crash_loop_backoff_300s_cap_at_5() {
    assert_eq!(crash_loop_backoff(5), SimDuration::from_secs(300));
}

#[test]
fn crash_loop_backoff_300s_cap_at_20() {
    assert_eq!(crash_loop_backoff(20), SimDuration::from_secs(300));
}

#[test]
fn fixed_latencies_returns_its_fields() {
    let fixed = FixedLatencies {
        pod_startup: SimDuration::from_secs(5),
        node_provisioning: SimDuration::from_secs(60),
        termination: SimDuration::from_secs(30),
    };

    let mut rng = RngStreams::new(42).stream(Concern::Lifecycle);

    assert_eq!(fixed.pod_startup(&mut rng), SimDuration::from_secs(5));
    assert_eq!(
        fixed.node_provisioning(&mut rng),
        SimDuration::from_secs(60)
    );
    assert_eq!(fixed.termination(&mut rng), SimDuration::from_secs(30));
}

#[test]
fn default_latencies_default_values() {
    let defaults = DefaultLatencies::default();

    assert_eq!(defaults.image_pull, SimDuration::ZERO);
    assert_eq!(defaults.pod_start_min, SimDuration::from_secs(1));
    assert_eq!(defaults.pod_start_max, SimDuration::from_secs(5));
    assert_eq!(defaults.node_provisioning_min, SimDuration::from_secs(60));
    assert_eq!(defaults.node_provisioning_max, SimDuration::from_secs(180));
    assert_eq!(defaults.termination, SimDuration::from_secs(30));
}

#[test]
fn default_latencies_pod_startup_within_bounds() {
    let defaults = DefaultLatencies::default();
    let mut rng = RngStreams::new(42).stream(Concern::Lifecycle);

    for _ in 0..1000 {
        let duration = defaults.pod_startup(&mut rng);
        let millis = duration.as_millis();
        assert!((1_000..=5_000).contains(&millis));
    }
}

#[test]
fn default_latencies_node_provisioning_within_bounds() {
    let defaults = DefaultLatencies::default();
    let mut rng = RngStreams::new(42).stream(Concern::Lifecycle);

    for _ in 0..1000 {
        let duration = defaults.node_provisioning(&mut rng);
        let millis = duration.as_millis();
        assert!((60_000..=180_000).contains(&millis));
    }
}

#[test]
fn default_latencies_termination_fixed() {
    let defaults = DefaultLatencies::default();
    let mut rng = RngStreams::new(42).stream(Concern::Lifecycle);

    for _ in 0..100 {
        let duration = defaults.termination(&mut rng);
        assert_eq!(duration, SimDuration::from_secs(30));
    }
}

#[test]
fn rng_streams_lifecycle_deterministic() {
    let mut rng1 = RngStreams::new(42).stream(Concern::Lifecycle);
    let mut rng2 = RngStreams::new(42).stream(Concern::Lifecycle);

    let defaults = DefaultLatencies::default();

    for _ in 0..100 {
        let val1 = defaults.pod_startup(&mut rng1);
        let val2 = defaults.pod_startup(&mut rng2);
        assert_eq!(val1, val2);
    }
}

#[test]
fn rng_streams_different_seeds_produce_different_sequences() {
    let mut rng1 = RngStreams::new(42).stream(Concern::Lifecycle);
    let mut rng2 = RngStreams::new(43).stream(Concern::Lifecycle);

    let defaults = DefaultLatencies::default();

    let mut same_count = 0;
    for _ in 0..100 {
        let val1 = defaults.pod_startup(&mut rng1);
        let val2 = defaults.pod_startup(&mut rng2);
        if val1 == val2 {
            same_count += 1;
        }
    }

    assert!(same_count < 50);
}

#[test]
fn rng_streams_independent_concerns() {
    let rng = RngStreams::new(42);
    let mut lifecycle_rng = rng.stream(Concern::Lifecycle);
    let mut workload_rng = rng.stream(Concern::Workload);

    let defaults = DefaultLatencies::default();

    let lifecycle_val = defaults.pod_startup(&mut lifecycle_rng);
    let workload_val = defaults.pod_startup(&mut workload_rng);

    assert_ne!(lifecycle_val, workload_val);
}

#[test]
fn default_latencies_image_pull_zero() {
    let defaults = DefaultLatencies::default();
    assert_eq!(defaults.image_pull, SimDuration::ZERO);
}
