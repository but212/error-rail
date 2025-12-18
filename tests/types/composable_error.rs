use core::error::Error;
use error_rail::{ComposableError, ErrorContext};
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

#[test]
fn test_composable_error_format_with() {
    let err = ComposableError::new("error").with_context("ctx");
    let s = err.format_with(|fmt| fmt.with_separator(" | "));
    assert_eq!(s, "ctx | error");
}

#[test]
fn test_error_formatter_options() {
    let err = ComposableError::new("error")
        .with_context("c1")
        .with_context("c2")
        .set_code(123);

    let s1 = err.fmt().with_separator(" - ").to_string();
    assert_eq!(s1, "c2 - c1 - error (code: 123)");

    let s2 = err.fmt().reverse_context(true).to_string();
    assert_eq!(s2, "c1 -> c2 -> error (code: 123)");

    let s3 = err.fmt().show_code(false).to_string();
    assert_eq!(s3, "c2 -> c1 -> error");
}

#[test]
fn test_fingerprint_config() {
    let err = ComposableError::with_code("error", 500)
        .with_context(ErrorContext::tag("t1"))
        .with_context(ErrorContext::metadata("k1", "v1"));

    let fp_full = err.fingerprint();

    let fp_no_msg = err.fingerprint_config().include_message(false).compute();
    assert_ne!(fp_full, fp_no_msg);

    let fp_no_code = err.fingerprint_config().include_code(false).compute();
    assert_ne!(fp_full, fp_no_code);

    let fp_no_tags = err.fingerprint_config().include_tags(false).compute();
    assert_ne!(fp_full, fp_no_tags);

    let fp_with_meta = err.fingerprint_config().include_metadata(true).compute();
    assert_ne!(fp_full, fp_with_meta);

    // Hex version
    assert_eq!(err.fingerprint_config().compute_hex().len(), 16);
}

#[test]
fn test_error_formatter_with_separator() {
    let err = ComposableError::new("core")
        .with_context("ctx1")
        .with_context("ctx2");

    let formatted = err.fmt().with_separator(" | ").to_string();
    assert_eq!(formatted, "ctx2 | ctx1 | core");
}

#[test]
fn test_error_formatter_reverse_context() {
    let err = ComposableError::new("core")
        .with_context("ctx1")
        .with_context("ctx2");

    let formatted = err.fmt().reverse_context(true).to_string();
    assert_eq!(formatted, "ctx1 -> ctx2 -> core");
}

#[test]
fn test_error_formatter_show_code_false() {
    let err = ComposableError::new("core")
        .with_context("ctx")
        .set_code(500);

    let formatted = err.fmt().show_code(false).to_string();
    assert_eq!(formatted, "ctx -> core");
    assert!(!formatted.contains("500"));
}

#[test]
fn test_error_formatter_show_code_true_with_code() {
    let err = ComposableError::new("core")
        .with_context("ctx")
        .set_code(404);

    let formatted = err.fmt().show_code(true).to_string();
    assert!(formatted.contains("(code: 404)"));
}

#[test]
fn test_error_formatter_all_options() {
    let err = ComposableError::new("error")
        .with_context("first")
        .with_context("second")
        .set_code(123);

    let formatted = err
        .fmt()
        .with_separator(" :: ")
        .reverse_context(true)
        .show_code(false)
        .to_string();

    assert_eq!(formatted, "first :: second :: error");
}

#[test]
fn test_error_formatter_no_context() {
    let err = ComposableError::<&str>::new("error");

    let formatted = err.fmt().to_string();
    assert_eq!(formatted, "error");
}

#[test]
fn test_error_formatter_no_context_with_code() {
    let err = ComposableError::<&str>::new("error").set_code(500);

    let formatted = err.fmt().to_string();
    assert_eq!(formatted, "error (code: 500)");
}

#[test]
fn test_fingerprint_with_simple_context() {
    let err = ComposableError::new("error")
        .with_context("simple string context")
        .set_code(500);

    let fp = err.fingerprint();
    assert!(fp > 0);

    let fp_hex = err.fingerprint_hex();
    assert_eq!(fp_hex.len(), 16);
}

#[test]
fn test_fingerprint_simple_context_no_tags() {
    let err1 = ComposableError::new("error").with_context("context1");

    let err2 = ComposableError::new("error").with_context("context2");

    let fp1 = err1.fingerprint_config().include_tags(true).compute();
    let fp2 = err2.fingerprint_config().include_tags(true).compute();

    assert_eq!(fp1, fp2);
}

#[test]
fn test_fingerprint_simple_context_no_metadata() {
    let err1 = ComposableError::new("error").with_context("ctx");

    let err2 = ComposableError::new("error").with_context("ctx");

    let fp1 = err1.fingerprint_config().include_metadata(true).compute();
    let fp2 = err2.fingerprint_config().include_metadata(true).compute();

    assert_eq!(fp1, fp2);
}

#[test]
fn test_fingerprint_mixed_context_types() {
    let err = ComposableError::new("error")
        .with_context("simple context")
        .with_context(ErrorContext::tag("network"))
        .with_context(ErrorContext::metadata("key", "value"));

    let fp = err.fingerprint();
    assert!(fp > 0);

    let fp_with_meta = err.fingerprint_config().include_metadata(true).compute();
    assert_ne!(fp, fp_with_meta);
}
