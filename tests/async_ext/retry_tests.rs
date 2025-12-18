//! Tests for async retry functionality.

use core::time::Duration;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

use error_rail::prelude_async::*;
use error_rail::traits::TransientError;

#[derive(Debug, Clone)]
enum TestError {
    Transient(String),
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

// Mock sleep that doesn't actually sleep (for fast tests)
async fn mock_sleep(_: Duration) {}

#[tokio::test]
async fn retry_succeeds_immediately() {
    let counter = Arc::new(AtomicU32::new(0));
    let counter_clone = counter.clone();

    let result = retry_with_policy(
        move || {
            let c = counter_clone.clone();
            async move {
                c.fetch_add(1, Ordering::SeqCst);
                Ok::<_, TestError>(42)
            }
        },
        ExponentialBackoff::default(),
        mock_sleep,
    )
    .await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 42);
    assert_eq!(counter.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn retry_succeeds_after_transient_failures() {
    let counter = Arc::new(AtomicU32::new(0));
    let counter_clone = counter.clone();

    let result = retry_with_policy(
        move || {
            let c = counter_clone.clone();
            async move {
                let count = c.fetch_add(1, Ordering::SeqCst);
                if count < 2 {
                    Err(TestError::Transient("temporary failure".into()))
                } else {
                    Ok(42)
                }
            }
        },
        ExponentialBackoff::default(),
        mock_sleep,
    )
    .await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 42);
    assert_eq!(counter.load(Ordering::SeqCst), 3); // 2 failures + 1 success
}

#[tokio::test]
async fn retry_fails_on_permanent_error() {
    let counter = Arc::new(AtomicU32::new(0));
    let counter_clone = counter.clone();

    let result = retry_with_policy(
        move || {
            let c = counter_clone.clone();
            async move {
                c.fetch_add(1, Ordering::SeqCst);
                Err::<i32, _>(TestError::Permanent("fatal error".into()))
            }
        },
        ExponentialBackoff::default(),
        mock_sleep,
    )
    .await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.error_chain().contains("permanent error, no retry"));
    assert_eq!(counter.load(Ordering::SeqCst), 1); // No retry for permanent
}

#[tokio::test]
async fn retry_exhausts_attempts() {
    let counter = Arc::new(AtomicU32::new(0));
    let counter_clone = counter.clone();

    let policy = ExponentialBackoff::new().with_max_attempts(3);

    let result = retry_with_policy(
        move || {
            let c = counter_clone.clone();
            async move {
                c.fetch_add(1, Ordering::SeqCst);
                Err::<i32, _>(TestError::Transient("always fails".into()))
            }
        },
        policy,
        mock_sleep,
    )
    .await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.error_chain().contains("exhausted"));
    assert_eq!(counter.load(Ordering::SeqCst), 4); // Initial attempt + 3 retries
}

#[test]
fn exponential_backoff_builder() {
    let policy = ExponentialBackoff::new()
        .with_initial_delay(Duration::from_millis(50))
        .with_max_delay(Duration::from_secs(5))
        .with_max_attempts(10)
        .with_multiplier(3.0);

    assert_eq!(policy.initial_delay, Duration::from_millis(50));
    assert_eq!(policy.max_delay, Duration::from_secs(5));
    assert_eq!(policy.max_attempts, 10);
    assert!((policy.multiplier - 3.0).abs() < f64::EPSILON);
}

#[test]
fn fixed_delay_policy() {
    let mut policy = FixedDelay::new(Duration::from_millis(100), 3);

    assert_eq!(policy.next_delay(0), Some(Duration::from_millis(100)));
    assert_eq!(policy.next_delay(1), Some(Duration::from_millis(100)));
    assert_eq!(policy.next_delay(2), Some(Duration::from_millis(100)));
    assert_eq!(policy.next_delay(3), None);
}
