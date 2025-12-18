//! Tests for tracing integration.

use std::future::Future;

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

#[tokio::test]
async fn test_span_context_future_pending() {
    use core::pin::Pin;
    use core::task::{Context, Poll};

    struct PendingFuture;
    impl Future for PendingFuture {
        type Output = Result<i32, &'static str>;
        fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
            Poll::Pending
        }
    }

    let fut = PendingFuture;
    let mut wrapped = fut.with_span_context();

    // Manual waker
    use core::task::{RawWaker, RawWakerVTable, Waker};
    fn noop(_: *const ()) {}
    fn clone(p: *const ()) -> RawWaker {
        RawWaker::new(p, &VTABLE)
    }
    static VTABLE: RawWakerVTable = RawWakerVTable::new(clone, noop, noop, noop);
    let raw_waker = RawWaker::new(core::ptr::null(), &VTABLE);
    let waker = unsafe { Waker::from_raw(raw_waker) };

    let mut cx = Context::from_waker(&waker);
    let mut wrapped = Pin::new(&mut wrapped);

    assert_eq!(wrapped.as_mut().poll(&mut cx), Poll::Pending);
}
