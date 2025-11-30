use core::error::Error;
use error_rail::ComposableError;
use std::io;

#[test]
fn test_composable_error_with_code() {
    let err = ComposableError::with_code("error", 500);
    assert_eq!(err.core_error(), &"error");
    assert_eq!(err.error_code(), Some(500));
}

#[test]
fn test_composable_error_context_iter() {
    let err = ComposableError::<&str>::new("error")
        .with_context("ctx1")
        .with_context("ctx2");

    let mut iter = err.context_iter();
    assert_eq!(iter.next().map(|c| c.message()), Some("ctx2".into()));
    assert_eq!(iter.next().map(|c| c.message()), Some("ctx1".into()));
    assert_eq!(iter.next(), None);
}

#[test]
fn test_composable_error_display_multiline() {
    let err = ComposableError::<&str>::new("error")
        .with_context("ctx1\nmultiline")
        .set_code(500);

    let formatted = format!("{:#}", err);
    assert!(formatted.contains("Error: error (code: 500)"));
    assert!(formatted.contains("Context:"));
    assert!(formatted.contains("- ctx1"));
    assert!(formatted.contains("multiline"));
}

#[test]
fn test_error_trait_impl() {
    // Verify ComposableError<io::Error> implements core::error::Error
    let io_err = io::Error::other("root cause");
    let err = ComposableError::<io::Error>::new(io_err).with_context("context");

    assert!(err.source().is_some());
    let source = err.source().unwrap();
    assert_eq!(source.to_string(), "root cause");
}

#[test]
fn test_display_format() {
    let err = ComposableError::<&str>::new("core error")
        .with_context("ctx1")
        .with_context("ctx2")
        .set_code(500);

    // Standard display
    assert_eq!(format!("{}", err), "ctx2 -> ctx1 -> core error (code: 500)");

    // Alternate display
    let alternate = format!("{:#}", err);
    let expected = "Error: core error (code: 500)\nContext:\n  - ctx2\n  - ctx1\n";
    assert_eq!(alternate, expected);
}

#[test]
fn test_display_format_no_context() {
    let err = ComposableError::<&str>::new("core error");

    // Standard
    assert_eq!(format!("{}", err), "core error");

    // Alternate
    let alternate = format!("{:#}", err);
    assert_eq!(alternate, "Error: core error");
}

// ============================================================================
// Fingerprint Tests
// ============================================================================

#[test]
fn test_fingerprint_same_errors_same_fingerprint() {
    use error_rail::ErrorContext;

    let err1 = ComposableError::new("database error")
        .with_context(ErrorContext::tag("db"))
        .set_code(500);

    let err2 = ComposableError::new("database error")
        .with_context(ErrorContext::tag("db"))
        .set_code(500);

    assert_eq!(err1.fingerprint(), err2.fingerprint());
    assert_eq!(err1.fingerprint_hex(), err2.fingerprint_hex());
}

#[test]
fn test_fingerprint_different_message_different_fingerprint() {
    use error_rail::ErrorContext;

    let err1 = ComposableError::new("database error")
        .with_context(ErrorContext::tag("db"))
        .set_code(500);

    let err2 = ComposableError::new("different error")
        .with_context(ErrorContext::tag("db"))
        .set_code(500);

    assert_ne!(err1.fingerprint(), err2.fingerprint());
}

#[test]
fn test_fingerprint_different_code_different_fingerprint() {
    use error_rail::ErrorContext;

    let err1 = ComposableError::new("error")
        .with_context(ErrorContext::tag("db"))
        .set_code(500);

    let err2 = ComposableError::new("error")
        .with_context(ErrorContext::tag("db"))
        .set_code(404);

    assert_ne!(err1.fingerprint(), err2.fingerprint());
}

#[test]
fn test_fingerprint_different_tags_different_fingerprint() {
    use error_rail::ErrorContext;

    let err1 = ComposableError::new("error")
        .with_context(ErrorContext::tag("db"))
        .set_code(500);

    let err2 = ComposableError::new("error")
        .with_context(ErrorContext::tag("network"))
        .set_code(500);

    assert_ne!(err1.fingerprint(), err2.fingerprint());
}

#[test]
fn test_fingerprint_hex_format() {
    use error_rail::ErrorContext;

    let err = ComposableError::new("timeout")
        .with_context(ErrorContext::tag("network"))
        .set_code(504);

    let fp = err.fingerprint_hex();
    assert_eq!(fp.len(), 16); // 64-bit hex = 16 characters
    assert!(fp.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn test_fingerprint_config_exclude_message() {
    use error_rail::ErrorContext;

    let err1 = ComposableError::new("error message 1")
        .with_context(ErrorContext::tag("db"))
        .set_code(500);

    let err2 = ComposableError::new("error message 2")
        .with_context(ErrorContext::tag("db"))
        .set_code(500);

    // With message included (default), different fingerprints
    assert_ne!(err1.fingerprint(), err2.fingerprint());

    // With message excluded, same fingerprints (same tags and code)
    let fp1 = err1.fingerprint_config().include_message(false).compute();
    let fp2 = err2.fingerprint_config().include_message(false).compute();
    assert_eq!(fp1, fp2);
}

#[test]
fn test_fingerprint_config_include_metadata() {
    use error_rail::ErrorContext;

    let err1 = ComposableError::new("error")
        .with_context(ErrorContext::metadata("user_id", "123"))
        .set_code(500);

    let err2 = ComposableError::new("error")
        .with_context(ErrorContext::metadata("user_id", "456"))
        .set_code(500);

    // Without metadata (default), same fingerprints
    assert_eq!(err1.fingerprint(), err2.fingerprint());

    // With metadata included, different fingerprints
    let fp1 = err1.fingerprint_config().include_metadata(true).compute();
    let fp2 = err2.fingerprint_config().include_metadata(true).compute();
    assert_ne!(fp1, fp2);
}

#[test]
fn test_fingerprint_tag_order_independence() {
    use error_rail::ErrorContext;

    // Tags should be sorted before hashing for consistency
    let err1 = ComposableError::new("error")
        .with_context(ErrorContext::tag("alpha"))
        .with_context(ErrorContext::tag("beta"));

    let err2 = ComposableError::new("error")
        .with_context(ErrorContext::tag("beta"))
        .with_context(ErrorContext::tag("alpha"));

    assert_eq!(err1.fingerprint(), err2.fingerprint());
}

#[test]
fn test_error_chain_with_custom_formatter() {
    use error_rail::types::error_formatter::ErrorFormatter;

    struct CustomFormatter;
    impl ErrorFormatter for CustomFormatter {
        fn separator(&self) -> &str {
            " >> "
        }
    }

    let err = ComposableError::new("error").with_context("ctx");

    let chain = err.error_chain_with(CustomFormatter);
    assert_eq!(chain, "ctx >> error");
}
