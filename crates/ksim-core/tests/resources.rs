use ksim_core::resources::{GIB, MIB, Resources};

#[test]
fn fits_in_cpu_and_memory_both_fit() {
    let requested = Resources::new(1000, 2 * GIB);
    let available = Resources::new(4000, 8 * GIB);
    assert!(requested.fits_in(available));
}

#[test]
fn fits_in_cpu_fits_memory_exceeds() {
    let requested = Resources::new(1000, 10 * GIB);
    let available = Resources::new(4000, 8 * GIB);
    assert!(!requested.fits_in(available));
}

#[test]
fn fits_in_cpu_exceeds_memory_fits() {
    let requested = Resources::new(5000, 2 * GIB);
    let available = Resources::new(4000, 8 * GIB);
    assert!(!requested.fits_in(available));
}

#[test]
fn fits_in_both_dimensions_exceed() {
    let requested = Resources::new(5000, 10 * GIB);
    let available = Resources::new(4000, 8 * GIB);
    assert!(!requested.fits_in(available));
}

#[test]
fn fits_in_exact_fit() {
    let requested = Resources::new(4000, 8 * GIB);
    let available = Resources::new(4000, 8 * GIB);
    assert!(requested.fits_in(available));
}

#[test]
fn fits_in_zero_requested() {
    let requested = Resources::ZERO;
    let available = Resources::new(100, 100);
    assert!(requested.fits_in(available));
}

#[test]
fn fits_in_zero_available() {
    let requested = Resources::new(1, 1);
    let available = Resources::ZERO;
    assert!(!requested.fits_in(available));
}

#[test]
fn checked_sub_success() {
    let r1 = Resources::new(1000, 2 * GIB);
    let r2 = Resources::new(400, GIB);
    let result = r1.checked_sub(r2);
    assert_eq!(result, Some(Resources::new(600, GIB)));
}

#[test]
fn checked_sub_cpu_underflow() {
    let r1 = Resources::new(100, 2 * GIB);
    let r2 = Resources::new(200, GIB);
    let result = r1.checked_sub(r2);
    assert_eq!(result, None);
}

#[test]
fn checked_sub_memory_underflow() {
    let r1 = Resources::new(1000, GIB);
    let r2 = Resources::new(400, 2 * GIB);
    let result = r1.checked_sub(r2);
    assert_eq!(result, None);
}

#[test]
fn checked_sub_both_underflow() {
    let r1 = Resources::new(100, GIB);
    let r2 = Resources::new(200, 2 * GIB);
    let result = r1.checked_sub(r2);
    assert_eq!(result, None);
}

#[test]
fn checked_sub_exact_fit() {
    let r1 = Resources::new(1000, 2 * GIB);
    let r2 = Resources::new(1000, 2 * GIB);
    let result = r1.checked_sub(r2);
    assert_eq!(result, Some(Resources::ZERO));
}

#[test]
fn checked_sub_zero() {
    let r1 = Resources::new(1000, 2 * GIB);
    let result = r1.checked_sub(Resources::ZERO);
    assert_eq!(result, Some(r1));
}

#[test]
fn saturating_add_no_overflow() {
    let r1 = Resources::new(1000, 2 * GIB);
    let r2 = Resources::new(500, GIB);
    let result = r1.saturating_add(r2);
    assert_eq!(result, Resources::new(1500, 3 * GIB));
}

#[test]
fn saturating_add_cpu_overflow() {
    let r1 = Resources::new(u64::MAX - 100, GIB);
    let r2 = Resources::new(200, GIB);
    let result = r1.saturating_add(r2);
    assert_eq!(result.cpu_millis, u64::MAX);
    assert_eq!(result.memory_bytes, 2 * GIB);
}

#[test]
fn saturating_add_memory_overflow() {
    let r1 = Resources::new(1000, u64::MAX - 1000);
    let r2 = Resources::new(500, 2000);
    let result = r1.saturating_add(r2);
    assert_eq!(result.cpu_millis, 1500);
    assert_eq!(result.memory_bytes, u64::MAX);
}

#[test]
fn saturating_add_both_overflow() {
    let r1 = Resources::new(u64::MAX - 100, u64::MAX - 100);
    let r2 = Resources::new(200, 200);
    let result = r1.saturating_add(r2);
    assert_eq!(result.cpu_millis, u64::MAX);
    assert_eq!(result.memory_bytes, u64::MAX);
}

#[test]
fn saturating_add_to_zero() {
    let r1 = Resources::ZERO;
    let r2 = Resources::new(1000, GIB);
    let result = r1.saturating_add(r2);
    assert_eq!(result, r2);
}

#[test]
fn saturating_add_zero_to_resource() {
    let r1 = Resources::new(1000, GIB);
    let r2 = Resources::ZERO;
    let result = r1.saturating_add(r2);
    assert_eq!(result, r1);
}

#[test]
fn mib_constant_value() {
    assert_eq!(MIB, 1 << 20);
}

#[test]
fn gib_constant_value() {
    assert_eq!(GIB, 1 << 30);
}

#[test]
fn gib_equals_1024_mibs() {
    assert_eq!(GIB, 1024 * MIB);
}

#[test]
fn resources_zero_constant() {
    assert_eq!(Resources::ZERO.cpu_millis, 0);
    assert_eq!(Resources::ZERO.memory_bytes, 0);
}

#[test]
fn fits_in_with_gib_units() {
    let requested = Resources::new(2000, 4 * GIB);
    let available = Resources::new(8000, 16 * GIB);
    assert!(requested.fits_in(available));
}

#[test]
fn fits_in_with_mib_units() {
    let requested = Resources::new(500, 256 * MIB);
    let available = Resources::new(2000, 512 * MIB);
    assert!(requested.fits_in(available));
}

#[test]
fn checked_sub_with_mib_units() {
    let r1 = Resources::new(1000, 512 * MIB);
    let r2 = Resources::new(300, 128 * MIB);
    let result = r1.checked_sub(r2);
    assert_eq!(result, Some(Resources::new(700, 384 * MIB)));
}

#[test]
fn saturating_add_with_mib_units() {
    let r1 = Resources::new(1000, 256 * MIB);
    let r2 = Resources::new(500, 256 * MIB);
    let result = r1.saturating_add(r2);
    assert_eq!(result, Resources::new(1500, 512 * MIB));
}
