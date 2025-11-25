use error_rail::ComposableError;
use core::error::Error;
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
    let io_err = io::Error::new(io::ErrorKind::Other, "root cause");
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
