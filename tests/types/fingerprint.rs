use error_rail::{ComposableError, ErrorContext};

#[test]
fn test_fingerprint_metadata_filtering() {
    let err = ComposableError::new("core error")
        .with_context(ErrorContext::metadata("key1", "val1"))
        .with_context(ErrorContext::metadata("key2", "val2"));

    let fp_all = err.fingerprint_config().include_metadata(true).compute();
    let fp_key1 = err
        .fingerprint_config()
        .include_metadata_keys(&["key1"])
        .compute();
    let fp_key2 = err
        .fingerprint_config()
        .include_metadata_keys(&["key2"])
        .compute();
    let fp_no_key1 = err
        .fingerprint_config()
        .exclude_metadata_keys(&["key1"])
        .compute();

    // Should be Different
    assert_ne!(fp_all, fp_key1);
    assert_ne!(fp_key1, fp_key2);
    assert_ne!(fp_all, fp_no_key1);

    // Verify key1 filter actually matches the same core + key1
    let err_only_key1 =
        ComposableError::new("core error").with_context(ErrorContext::metadata("key1", "val1"));
    let fp_only_key1 = err_only_key1
        .fingerprint_config()
        .include_metadata(true)
        .compute();

    assert_eq!(fp_key1, fp_only_key1);
}
