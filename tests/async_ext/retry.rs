use core::fmt;
use std::time::Duration;

use error_rail::{
    async_ext::{retry_with_metadata, ExponentialBackoff, FixedDelay, RetryPolicy},
    TransientError,
};

#[test]
fn exponential_backoff_delays() {
    let mut policy = ExponentialBackoff {
        initial_delay: Duration::from_millis(100),
        max_delay: Duration::from_secs(10),
        max_attempts: 5,
        multiplier: 2.0,
    };

    assert_eq!(policy.next_delay(0), Some(Duration::from_millis(100)));
    assert_eq!(policy.next_delay(1), Some(Duration::from_millis(200)));
    assert_eq!(policy.next_delay(2), Some(Duration::from_millis(400)));
    assert_eq!(policy.next_delay(3), Some(Duration::from_millis(800)));
    assert_eq!(policy.next_delay(4), Some(Duration::from_millis(1600)));
    assert_eq!(policy.next_delay(5), None); // max_attempts reached
}

#[test]
fn exponential_backoff_caps_at_max() {
    let mut policy = ExponentialBackoff {
        initial_delay: Duration::from_secs(1),
        max_delay: Duration::from_secs(5),
        max_attempts: 10,
        multiplier: 10.0,
    };

    // 1s * 10^2 = 100s, but capped at 5s
    assert_eq!(policy.next_delay(2), Some(Duration::from_secs(5)));
}

#[test]
fn fixed_delay_consistent() {
    let mut policy = FixedDelay::new(Duration::from_millis(500), 3);

    assert_eq!(policy.next_delay(0), Some(Duration::from_millis(500)));
    assert_eq!(policy.next_delay(1), Some(Duration::from_millis(500)));
    assert_eq!(policy.next_delay(2), Some(Duration::from_millis(500)));
    assert_eq!(policy.next_delay(3), None);
}

#[derive(Debug, Clone, Copy)]
struct MyTransient;
impl fmt::Display for MyTransient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "transient")
    }
}
impl TransientError for MyTransient {
    fn is_transient(&self) -> bool {
        true
    }
}

#[derive(Debug, Clone, Copy)]
struct MyPermanent;
impl fmt::Display for MyPermanent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "permanent")
    }
}
impl TransientError for MyPermanent {
    fn is_transient(&self) -> bool {
        false
    }
}

#[test]
fn test_fixed_delay_reset() {
    let mut policy = FixedDelay::new(Duration::from_millis(100), 3);
    policy.reset(); // Call reset for coverage
    assert_eq!(policy.next_delay(0), Some(Duration::from_millis(100)));
}

#[tokio::test]
async fn test_retry_with_metadata_success() {
    let result = retry_with_metadata(
        || async { Ok::<_, MyTransient>(42) },
        FixedDelay::new(Duration::from_millis(1), 3),
        |_| async {},
    )
    .await;

    assert_eq!(result.attempts, 1);
    assert!(result.result.is_ok());
}

#[tokio::test]
async fn test_retry_with_metadata_retry_success() {
    let mut attempts = 0;
    let result = retry_with_metadata(
        || {
            attempts += 1;
            async move {
                if attempts < 2 {
                    Err(MyTransient)
                } else {
                    Ok(42)
                }
            }
        },
        FixedDelay::new(Duration::from_millis(1), 3),
        |_| async {},
    )
    .await;

    assert_eq!(result.attempts, 2);
    assert!(result.result.is_ok());
    assert_eq!(result.total_wait_time, Duration::from_millis(1));
}

#[tokio::test]
async fn test_retry_with_metadata_exhausted() {
    let result = retry_with_metadata(
        || async { Err::<i32, _>(MyTransient) },
        FixedDelay::new(Duration::from_millis(1), 1),
        |_| async {},
    )
    .await;

    assert_eq!(result.attempts, 2); // 1 initial + 1 retry
    assert!(result.result.is_err());
    assert!(result
        .result
        .unwrap_err()
        .error_chain()
        .contains("exhausted 2 retry attempts"));
}

#[tokio::test]
async fn test_retry_with_metadata_permanent() {
    let result = retry_with_metadata(
        || async { Err::<i32, _>(MyPermanent) },
        FixedDelay::new(Duration::from_millis(1), 3),
        |_| async {},
    )
    .await;

    assert_eq!(result.attempts, 1);
    assert!(result.result.is_err());
    assert!(result
        .result
        .unwrap_err()
        .error_chain()
        .contains("permanent error, no retry"));
}
