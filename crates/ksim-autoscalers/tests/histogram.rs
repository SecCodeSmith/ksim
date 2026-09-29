use ksim_autoscalers::histogram::{DecayingHistogram, HistogramOptions};
use ksim_core::time::{SimDuration, SimTime};

#[test]
fn cpu_millis_options() {
    let opts = HistogramOptions::cpu_millis();
    assert!((opts.first_bucket_size - 10.0).abs() < f64::EPSILON);
    assert!((opts.ratio - 1.05).abs() < f64::EPSILON);
    assert!((opts.max_value - 1_000_000.0).abs() < f64::EPSILON);
}

#[test]
fn memory_bytes_options() {
    let opts = HistogramOptions::memory_bytes();
    assert!((opts.first_bucket_size - 1e7).abs() < 1.0);
    assert!((opts.ratio - 1.05).abs() < f64::EPSILON);
    assert!((opts.max_value - 1e12).abs() < 1.0);
}

#[test]
fn bucket_start_values() {
    let opts = HistogramOptions::cpu_millis();
    assert!((opts.bucket_start(0) - 0.0).abs() < f64::EPSILON);
    assert!((opts.bucket_start(1) - 10.0).abs() < f64::EPSILON);
    // bucket 2 starts at 10 + 10*1.05 = 10 + 10.5 = 20.5
    assert!((opts.bucket_start(2) - 20.5).abs() < 0.01);
}

#[test]
fn find_bucket_within_range() {
    let opts = HistogramOptions::cpu_millis();
    assert_eq!(opts.find_bucket(15.0), 1);
    assert_eq!(opts.find_bucket(-3.0), 0);
    let huge = 2_000_000.0;
    assert_eq!(opts.find_bucket(huge), opts.bucket_count() - 1);
}

#[test]
fn find_bucket_invariant() {
    let opts = HistogramOptions::cpu_millis();
    for value in [0.0, 1.0, 50.0, 100.0, 10_000.0, 100_000.0] {
        let bucket = opts.find_bucket(value);
        let start = opts.bucket_start(bucket);
        let end = if bucket + 1 < opts.bucket_count() {
            opts.bucket_start(bucket + 1)
        } else {
            opts.max_value
        };
        assert!(
            start <= value,
            "value {value} not >= bucket_start({bucket})"
        );
        assert!(
            value < end,
            "value {value} not < bucket_end({bucket}) = {end}"
        );
    }
}

#[test]
fn bucket_start_is_increasing() {
    let opts = HistogramOptions::cpu_millis();
    for i in 0..opts.bucket_count() - 1 {
        assert!(
            opts.bucket_start(i) < opts.bucket_start(i + 1),
            "bucket_start({}) >= bucket_start({})",
            i,
            i + 1
        );
    }
}

#[test]
fn empty_histogram_percentile_and_is_empty() {
    let opts = HistogramOptions::cpu_millis();
    let hist = DecayingHistogram::new(opts, SimDuration::from_secs(24 * 3600));
    assert!(hist.is_empty());
    assert!((hist.percentile(0.0) - 0.0).abs() < f64::EPSILON);
    assert!((hist.percentile(0.5) - 0.0).abs() < f64::EPSILON);
    assert!((hist.percentile(1.0) - 0.0).abs() < f64::EPSILON);
}

#[test]
fn single_sample_percentile() {
    let opts = HistogramOptions::cpu_millis();
    let mut hist = DecayingHistogram::new(opts, SimDuration::from_secs(24 * 3600));
    let now = SimTime::from_secs(0);
    hist.add_sample(100.0, 1.0, now);

    // percentile(0.5) should return bucket_start(find_bucket(100)+1)
    let bucket = opts.find_bucket(100.0);
    let expected = opts.bucket_start(bucket + 1);
    let p50 = hist.percentile(0.5);
    assert!(
        p50 > 100.0 && p50 < 106.0,
        "percentile(0.5)={p50} should be in (100, 106), bucket_end={expected}"
    );
}

#[test]
fn decay_with_two_samples() {
    let opts = HistogramOptions::cpu_millis();
    let half_life = SimDuration::from_secs(24 * 3600);
    let mut hist = DecayingHistogram::new(opts, half_life);

    let t0 = SimTime::from_secs(0);
    let t1 = SimTime::from_secs(24 * 3600); // 24 hours later

    hist.add_sample(50.0, 1.0, t0);
    hist.add_sample(500.0, 1.0, t1);

    // After 24h with half_life 24h, sample at t0 has weight 1 * 2^((0 - 24h) / 24h) = 0.5
    // Sample at t1 has weight 1 * 2^((24h - 24h) / 24h) = 1.0
    // Total weight = 1.5
    // Threshold for p50 = 0.75
    // Only the 500 sample has enough cumulative weight >= 0.75
    let p50 = hist.percentile(0.5);
    let bucket_500 = opts.find_bucket(500.0);
    let bucket_end_500 = opts.bucket_start(bucket_500 + 1);

    // p50 should be the bucket end for 500
    assert!(
        (p50 - bucket_end_500).abs() < 1.0,
        "percentile(0.5)={p50} should be bucket_end_500={bucket_end_500}"
    );
}

#[test]
fn subtract_sample_removes_sample() {
    let opts = HistogramOptions::cpu_millis();
    let mut hist = DecayingHistogram::new(opts, SimDuration::from_secs(24 * 3600));
    let now = SimTime::from_secs(0);

    hist.add_sample(100.0, 1.0, now);
    assert!(!hist.is_empty());

    hist.subtract_sample(100.0, 1.0, now);
    assert!(hist.is_empty());
}
