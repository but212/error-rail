//! Tests for Tokio integration.

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;

use error_rail::prelude_async::*;
use error_rail::traits::TransientError;

#[derive(Debug, Clone)]
enum TestError {
    Transient(String),
    #[allow(dead_code)]
    Permanent(String),
}

impl std::fmt::Display for TestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TestError::Transient(msg) => write!(f, "transient: {}", msg),
            TestError::Permanent(msg) => write!(f, "permanent: {}", msg),
        }
    }
}

impl TransientError for TestError {
    fn is_transient(&self) -> bool {
        matches!(self, TestError::Transient(_))
    }
}

#[tokio::test]
async fn retry_transient_succeeds_immediately() {
    let counter = Arc::new(AtomicU32::new(0));
    let counter_clone = counter.clone();

    let result = retry_transient(
        move || {
            let c = counter_clone.clone();
            async move {
                c.fetch_add(1, Ordering::SeqCst);
                Ok::<_, TestError>(42)
            }
        },
        ExponentialBackoff::new().with_max_attempts(3),
    )
    .await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 42);
    assert_eq!(counter.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn retry_transient_succeeds_after_failures() {
    let counter = Arc::new(AtomicU32::new(0));
    let counter_clone = counter.clone();

    let result = retry_transient(
        move || {
            let c = counter_clone.clone();
            async move {
                let count = c.fetch_add(1, Ordering::SeqCst);
                if count < 2 {
                    Err(TestError::Transient("temporary".into()))
                } else {
                    Ok(42)
                }
            }
        },
        ExponentialBackoff::new().with_max_attempts(5),
    )
    .await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 42);
    assert_eq!(counter.load(Ordering::SeqCst), 3);
}

#[tokio::test]
async fn retry_transient_n_works() {
    let counter = Arc::new(AtomicU32::new(0));
    let counter_clone = counter.clone();

    let result = retry_transient_n(
        move || {
            let c = counter_clone.clone();
            async move {
                let count = c.fetch_add(1, Ordering::SeqCst);
                if count < 1 {
                    Err(TestError::Transient("temporary".into()))
                } else {
                    Ok(42)
                }
            }
        },
        3,
    )
    .await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 42);
}

#[tokio::test]
async fn try_with_timeout_success() {
    let result: TimeoutResult<i32, &str> =
        try_with_timeout(Duration::from_secs(1), async { Ok(42) }).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn try_with_timeout_error() {
    let result: TimeoutResult<i32, &str> =
        try_with_timeout(Duration::from_secs(1), async { Err("failed") }).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn try_with_timeout_timeout() {
    let result: TimeoutResult<i32, &str> = try_with_timeout(Duration::from_millis(10), async {
        tokio::time::sleep(Duration::from_secs(1)).await;
        Ok(42)
    })
    .await;

    assert!(result.is_timeout());
}

#[test]
fn timeout_result_variants() {
    let ok: TimeoutResult<i32, &str> = TimeoutResult::Ok(42);
    assert!(ok.is_ok());

    let err: TimeoutResult<i32, &str> =
        TimeoutResult::Err(Box::new(error_rail::ComposableError::new("error")));
    assert!(err.is_err());

    let timeout: TimeoutResult<i32, &str> = TimeoutResult::Timeout(Duration::from_secs(1));
    assert!(timeout.is_timeout());
}

#[test]
fn timeout_error_display() {
    let err = TimeoutError(Duration::from_secs(5));
    assert!(err.to_string().contains("5s"));
}
