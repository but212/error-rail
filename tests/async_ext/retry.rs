use std::time::Duration;

use error_rail::async_ext::{ExponentialBackoff, FixedDelay, RetryPolicy};

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
