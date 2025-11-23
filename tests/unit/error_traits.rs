use error_rail::ComposableError;
use std::error::Error;
use std::io;

#[test]
fn test_error_trait_impl() {
    // Verify ComposableError<io::Error> implements std::error::Error
    let io_err = io::Error::new(io::ErrorKind::Other, "root cause");
    let err = ComposableError::<io::Error, u32>::new(io_err).with_context("context");

    assert!(err.source().is_some());
    let source = err.source().unwrap();
    assert_eq!(source.to_string(), "root cause");
}

#[test]
fn test_display_format() {
    let err = ComposableError::<&str, u32>::new("core error")
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
    let err = ComposableError::<&str, u32>::new("core error");

    // Standard
    assert_eq!(format!("{}", err), "core error");

    // Alternate
    let alternate = format!("{:#}", err);
    assert_eq!(alternate, "Error: core error");
}
