//! Tests for FutureResultExt trait.

use error_rail::prelude_async::*;
use std::sync::atomic::{AtomicU32, Ordering};

#[test]
fn context_future_is_send_sync() {
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}

    assert_send::<
        error_rail::async_ext::ContextFuture<
            std::future::Ready<Result<(), ()>>,
            fn() -> &'static str,
        >,
    >();
    assert_sync::<
        error_rail::async_ext::ContextFuture<
            std::future::Ready<Result<(), ()>>,
            fn() -> &'static str,
        >,
    >();
}

#[tokio::test]
async fn ctx_does_not_evaluate_on_success() {
    let call_count = AtomicU32::new(0);

    let fut = async { Ok::<_, &str>(42) };
    let result = fut
        .with_ctx(|| {
            call_count.fetch_add(1, Ordering::SeqCst);
            "should not be called"
        })
        .await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 42);
    assert_eq!(call_count.load(Ordering::SeqCst), 0); // Not called on success
}

#[tokio::test]
async fn ctx_evaluates_only_on_error() {
    let call_count = AtomicU32::new(0);

    let fut = async { Err::<i32, _>("failed") };
    let result = fut
        .with_ctx(|| {
            call_count.fetch_add(1, Ordering::SeqCst);
            "operation failed"
        })
        .await;

    assert!(result.is_err());
    assert_eq!(call_count.load(Ordering::SeqCst), 1); // Called exactly once
}

#[tokio::test]
async fn ctx_with_static_message() {
    let fut = async { Err::<i32, _>("inner error") };
    let result = fut.ctx("static context").await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.error_chain().contains("static context"));
    assert!(err.error_chain().contains("inner error"));
}

#[tokio::test]
async fn ctx_with_formatted_message() {
    let user_id = 42u64;
    let fut = async { Err::<i32, _>("not found") };
    let result = fut.with_ctx(|| format!("fetching user {}", user_id)).await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.error_chain().contains("fetching user 42"));
    assert!(err.error_chain().contains("not found"));
}

#[tokio::test]
async fn multiple_ctx_chains() {
    let fut = async { Err::<i32, _>("base error") };
    let result = fut.ctx("layer 1").ctx("layer 2").ctx("layer 3").await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    let chain = err.error_chain();
    assert!(chain.contains("layer 1"));
    assert!(chain.contains("layer 2"));
    assert!(chain.contains("layer 3"));
    assert!(chain.contains("base error"));
}

#[tokio::test]
async fn success_path_returns_value() {
    let fut = async { Ok::<_, &str>(vec![1, 2, 3]) };
    let result = fut.ctx("should not appear").await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), vec![1, 2, 3]);
}
