//! Async retry utilities with runtime-neutral design.
//!
//! This module provides retry policies and functions that work with any async
//! runtime by accepting a sleep function as a parameter.

use core::future::Future;
use core::time::Duration;

use crate::traits::TransientError;
use crate::types::ComposableError;

/// Defines a retry policy for async operations.
///
/// Implementations determine when and how long to wait between retry attempts.
pub trait RetryPolicy: Clone {
    /// Returns the delay before the next retry attempt, or `None` to stop retrying.
    ///
    /// # Arguments
    ///
    /// * `attempt` - The current attempt number (0-indexed)
    ///
    /// # Returns
    ///
    /// - `Some(Duration)` - Wait this duration before retrying
    /// - `None` - Stop retrying (max attempts reached or policy exhausted)
    fn next_delay(&mut self, attempt: u32) -> Option<Duration>;

    /// Resets the policy to its initial state.
    fn reset(&mut self);
}

/// Exponential backoff retry policy.
///
/// Each retry waits exponentially longer than the previous one, up to a maximum
/// delay. This is the recommended policy for most network operations.
///
/// # Example
///
/// ```rust
/// use error_rail::async_ext::ExponentialBackoff;
/// use core::time::Duration;
///
/// let policy = ExponentialBackoff {
///     initial_delay: Duration::from_millis(100),
///     max_delay: Duration::from_secs(10),
///     max_attempts: 5,
///     multiplier: 2.0,
/// };
///
/// // Delays: 100ms, 200ms, 400ms, 800ms, 1600ms (capped at 10s)
/// ```
#[derive(Clone, Debug)]
pub struct ExponentialBackoff {
    /// Initial delay before first retry.
    pub initial_delay: Duration,
    /// Maximum delay between retries.
    pub max_delay: Duration,
    /// Maximum number of retry attempts.
    pub max_attempts: u32,
    /// Multiplier applied to delay after each attempt.
    pub multiplier: f64,
}

impl Default for ExponentialBackoff {
    fn default() -> Self {
        Self {
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(30),
            max_attempts: 5,
            multiplier: 2.0,
        }
    }
}

impl ExponentialBackoff {
    /// Creates a new exponential backoff policy with default settings.
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the initial delay.
    #[inline]
    pub fn with_initial_delay(mut self, delay: Duration) -> Self {
        self.initial_delay = delay;
        self
    }

    /// Sets the maximum delay.
    #[inline]
    pub fn with_max_delay(mut self, delay: Duration) -> Self {
        self.max_delay = delay;
        self
    }

    /// Sets the maximum number of attempts.
    #[inline]
    pub fn with_max_attempts(mut self, attempts: u32) -> Self {
        self.max_attempts = attempts;
        self
    }

    /// Sets the multiplier.
    #[inline]
    pub fn with_multiplier(mut self, multiplier: f64) -> Self {
        self.multiplier = multiplier;
        self
    }
}

impl RetryPolicy for ExponentialBackoff {
    fn next_delay(&mut self, attempt: u32) -> Option<Duration> {
        if attempt >= self.max_attempts {
            return None;
        }
        let delay = Duration::from_secs_f64(
            self.initial_delay.as_secs_f64() * self.multiplier.powi(attempt as i32),
        );
        Some(delay.min(self.max_delay))
    }

    fn reset(&mut self) {
        // ExponentialBackoff is stateless, nothing to reset
    }
}

/// Fixed delay retry policy.
///
/// Waits the same duration between each retry attempt.
#[derive(Clone, Debug)]
pub struct FixedDelay {
    /// Delay between retry attempts.
    pub delay: Duration,
    /// Maximum number of retry attempts.
    pub max_attempts: u32,
}

impl FixedDelay {
    /// Creates a new fixed delay policy.
    #[inline]
    pub fn new(delay: Duration, max_attempts: u32) -> Self {
        Self {
            delay,
            max_attempts,
        }
    }
}

impl RetryPolicy for FixedDelay {
    fn next_delay(&mut self, attempt: u32) -> Option<Duration> {
        if attempt >= self.max_attempts {
            None
        } else {
            Some(self.delay)
        }
    }

    fn reset(&mut self) {}
}

/// Retries an async operation according to a policy when transient errors occur.
///
/// This function is **runtime-neutral**: it accepts a `sleep_fn` parameter that
/// performs the actual sleeping, allowing it to work with any async runtime.
///
/// # Arguments
///
/// * `operation` - A closure that returns the future to retry
/// * `policy` - The retry policy to use
/// * `sleep_fn` - A function that returns a sleep future for the given duration
///
/// # Example
///
/// ```rust,ignore
/// use error_rail::async_ext::{retry_with_policy, ExponentialBackoff};
///
/// // With Tokio
/// let result = retry_with_policy(
///     || fetch_data(),
///     ExponentialBackoff::default(),
///     |d| tokio::time::sleep(d),
/// ).await;
///
/// // With async-std
/// let result = retry_with_policy(
///     || fetch_data(),
///     ExponentialBackoff::default(),
///     |d| async_std::task::sleep(d),
/// ).await;
/// ```
pub async fn retry_with_policy<F, Fut, T, E, P, S, SFut>(
    mut operation: F,
    mut policy: P,
    sleep_fn: S,
) -> Result<T, ComposableError<E>>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, E>>,
    E: TransientError,
    P: RetryPolicy,
    S: Fn(Duration) -> SFut,
    SFut: Future<Output = ()>,
{
    let mut attempt = 0u32;

    loop {
        match operation().await {
            Ok(value) => return Ok(value),
            Err(e) if e.is_transient() => {
                if let Some(delay) = policy.next_delay(attempt) {
                    sleep_fn(delay).await;
                    attempt += 1;
                    continue;
                }
                // Exhausted retry attempts
                return Err(ComposableError::new(e)
                    .with_context(crate::context!("exhausted {} retry attempts", attempt + 1)));
            }
            Err(e) => {
                // Permanent error, no retry
                return Err(ComposableError::new(e)
                    .with_context(crate::context!("permanent error, no retry")));
            }
        }
    }
}

/// Result of a retry operation with metadata about attempts.
#[derive(Debug)]
pub struct RetryResult<T, E> {
    /// The final result.
    pub result: Result<T, ComposableError<E>>,
    /// Total number of attempts made.
    pub attempts: u32,
    /// Total time spent waiting (not including operation time).
    pub total_wait_time: Duration,
}

/// Retries an operation with detailed result metadata.
///
/// Similar to [`retry_with_policy`], but returns additional information about
/// the retry process.
pub async fn retry_with_metadata<F, Fut, T, E, P, S, SFut>(
    mut operation: F,
    mut policy: P,
    sleep_fn: S,
) -> RetryResult<T, E>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, E>>,
    E: TransientError,
    P: RetryPolicy,
    S: Fn(Duration) -> SFut,
    SFut: Future<Output = ()>,
{
    let mut attempt = 0u32;
    let mut total_wait_time = Duration::ZERO;

    loop {
        match operation().await {
            Ok(value) => {
                return RetryResult {
                    result: Ok(value),
                    attempts: attempt + 1,
                    total_wait_time,
                }
            }
            Err(e) if e.is_transient() => {
                if let Some(delay) = policy.next_delay(attempt) {
                    total_wait_time += delay;
                    sleep_fn(delay).await;
                    attempt += 1;
                    continue;
                }
                return RetryResult {
                    result: Err(ComposableError::new(e)
                        .with_context(crate::context!("exhausted {} retry attempts", attempt + 1))),
                    attempts: attempt + 1,
                    total_wait_time,
                };
            }
            Err(e) => {
                return RetryResult {
                    result: Err(ComposableError::new(e)
                        .with_context(crate::context!("permanent error, no retry"))),
                    attempts: attempt + 1,
                    total_wait_time,
                };
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
