use ksim_core::time::{SimDuration, SimTime};

#[test]
fn from_secs_converts_to_millis() {
    let time = SimTime::from_secs(15);
    assert_eq!(time.as_millis(), 15000);
}

#[test]
fn duration_from_secs_converts_to_millis() {
    let dur = SimDuration::from_secs(15);
    assert_eq!(dur.as_millis(), 15000);
}

#[test]
fn from_millis_returns_same_value() {
    let time = SimTime::from_millis(15000);
    assert_eq!(time.as_millis(), 15000);
}

#[test]
fn duration_from_millis_returns_same_value() {
    let dur = SimDuration::from_millis(15000);
    assert_eq!(dur.as_millis(), 15000);
}

#[test]
fn simtime_add_duration() {
    let time = SimTime::from_secs(10);
    let dur = SimDuration::from_secs(5);
    let result = time + dur;
    assert!((result.as_secs_f64() - 15.0).abs() < 1e-9);
}

#[test]
fn simtime_add_zero_duration() {
    let time = SimTime::from_secs(10);
    let dur = SimDuration::ZERO;
    let result = time + dur;
    assert!((result.as_secs_f64() - 10.0).abs() < 1e-9);
}

#[test]
fn simtime_subtract_earlier_time_gives_duration() {
    let time1 = SimTime::from_secs(10);
    let time2 = SimTime::from_secs(4);
    let dur = time1 - time2;
    assert!((dur.as_secs_f64() - 6.0).abs() < 1e-9);
}

#[test]
fn simtime_subtract_later_time_saturates_to_zero() {
    let time1 = SimTime::from_secs(4);
    let time2 = SimTime::from_secs(10);
    let dur = time1 - time2;
    assert_eq!(dur, SimDuration::ZERO);
}

#[test]
fn simtime_subtract_same_time_gives_zero() {
    let time = SimTime::from_secs(10);
    let dur = time - time;
    assert_eq!(dur, SimDuration::ZERO);
}

#[test]
fn as_secs_f64_for_time() {
    let time = SimTime::from_secs(10);
    assert!((time.as_secs_f64() - 10.0).abs() < 1e-9);
}

#[test]
fn as_secs_f64_for_duration() {
    let dur = SimDuration::from_secs(10);
    assert!((dur.as_secs_f64() - 10.0).abs() < 1e-9);
}

#[test]
fn as_secs_f64_with_millis() {
    let time = SimTime::from_millis(5500);
    assert!((time.as_secs_f64() - 5.5).abs() < 0.0001);
}

#[test]
fn duration_saturating_mul_no_overflow() {
    let dur = SimDuration::from_secs(10);
    let result = dur.saturating_mul(5);
    assert!((result.as_secs_f64() - 50.0).abs() < 1e-9);
}

#[test]
fn duration_saturating_mul_with_overflow() {
    let dur = SimDuration::from_secs(u64::MAX / 2 + 1);
    let result = dur.saturating_mul(3);
    assert_eq!(result.as_millis(), u64::MAX);
}

#[test]
fn duration_saturating_mul_by_zero() {
    let dur = SimDuration::from_secs(100);
    let result = dur.saturating_mul(0);
    assert_eq!(result, SimDuration::ZERO);
}

#[test]
fn duration_saturating_mul_by_one() {
    let dur = SimDuration::from_secs(42);
    let result = dur.saturating_mul(1);
    assert!((result.as_secs_f64() - 42.0).abs() < 1e-9);
}

#[test]
fn simtime_zero_constant() {
    assert_eq!(SimTime::ZERO.as_millis(), 0);
}

#[test]
fn simduration_zero_constant() {
    assert_eq!(SimDuration::ZERO.as_millis(), 0);
}

#[test]
fn add_large_durations() {
    let time = SimTime::from_millis(1000);
    let dur = SimDuration::from_millis(999);
    let result = time + dur;
    assert_eq!(result.as_millis(), 1999);
}
