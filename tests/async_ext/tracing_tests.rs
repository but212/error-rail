//! Tests for tracing integration.

use error_rail::prelude_async::*;
use tracing::Span;

#[test]
fn result_span_ext_with_current_span() {
    let result: Result<i32, &str> = Err("failed");
    let wrapped = result.with_current_span();

    assert!(wrapped.is_err());
    let err = wrapped.unwrap_err();
    // Should have span context attached
    assert!(err.error_chain().contains("span"));
}

#[test]
fn result_span_ext_ok_passes_through() {
    let result: Result<i32, &str> = Ok(42);
    let wrapped = result.with_current_span();

    assert!(wrapped.is_ok());
    assert_eq!(wrapped.unwrap(), 42);
}

#[test]
fn instrument_error_adds_context() {
    let error = "something went wrong";
    let instrumented = instrument_error(error);

    assert!(instrumented.error_chain().contains("span"));
    assert!(instrumented.error_chain().contains("something went wrong"));
}

#[tokio::test]
async fn future_span_ext_success() {
    let result = async { Ok::<_, &str>(42) }.with_span_context().await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 42);
}

#[tokio::test]
async fn future_span_ext_error() {
    let result = async { Err::<i32, _>("failed") }.with_span_context().await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.error_chain().contains("span"));
    assert!(err.error_chain().contains("failed"));
}

#[test]
fn with_span_none_handles_gracefully() {
    let span = Span::none();
    let result: Result<i32, &str> = Err("error");
    let wrapped = result.with_span(&span);

    assert!(wrapped.is_err());
    let err = wrapped.unwrap_err();
    // Should handle the none span gracefully
    assert!(err.error_chain().contains("unknown") || err.error_chain().contains("span"));
}
