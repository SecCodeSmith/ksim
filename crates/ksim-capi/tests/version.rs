#[test]
fn test_version_returns_correct_cstr() {
    assert_eq!(ksim::version().to_str().unwrap(), env!("CARGO_PKG_VERSION"));
}

#[test]
fn test_c_version_returns_same_pointer() {
    assert_eq!(ksim::ksim_version(), ksim::version().as_ptr());
}
