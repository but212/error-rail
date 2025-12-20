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
    ///
    /// Default implementation does nothing, suitable for stateless policies.
    fn reset(&mut self) {}
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
    // Uses default reset() - ExponentialBackoff is stateless
}

/// Fixed delay retry policy.
///
/// Waits the same duration between each retry attempt. This is simpler than
/// exponential backoff but may not be suitable for services under heavy load.
///
/// # Example
///
/// ```rust
/// use error_rail::async_ext::FixedDelay;
/// use core::time::Duration;
///
/// let policy = FixedDelay::new(Duration::from_millis(500), 3);
///
/// // Delays: 500ms, 500ms, 500ms (then stops)
/// ```
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
        Self { delay, max_attempts }
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
    // Uses default reset() - FixedDelay is stateless
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
    policy.reset();
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
                // Exhausted all attempts
                return Err(ComposableError::new(e)
                    .with_context(crate::context!("exhausted after {} attempts", attempt + 1)));
            },
            Err(e) => {
                // Permanent error, no retry
                return Err(ComposableError::new(e)
                    .with_context(crate::context!("permanent error, no retry")));
            },
        }
    }
}

/// Result of a retry operation with metadata about attempts.
///
/// This struct provides detailed information about a retry operation,
/// including the final result and statistics about the retry process.
///
/// # Type Parameters
///
/// * `T` - The success type of the operation
/// * `E` - The error type of the operation
///
/// # Example
///
/// ```rust,ignore
/// use error_rail::async_ext::{retry_with_metadata, ExponentialBackoff, RetryResult};
///
/// let retry_result: RetryResult<Data, ApiError> = retry_with_metadata(
///     || fetch_data(),
///     ExponentialBackoff::default(),
///     |d| tokio::time::sleep(d),
/// ).await;
///
/// if retry_result.attempts > 1 {
///     log::warn!(
///         "Operation succeeded after {} attempts (waited {:?})",
///         retry_result.attempts,
///         retry_result.total_wait_time
///     );
/// }
/// ```
#[derive(Debug)]
pub struct RetryResult<T, E> {
    /// The final result of the operation.
    ///
    /// Contains `Ok(T)` if the operation eventually succeeded, or
    /// `Err(ComposableError<E>)` if all retry attempts were exhausted
    /// or a permanent error occurred.
    pub result: Result<T, ComposableError<E>>,

    /// Total number of attempts made.
    ///
    /// This is always at least 1 (the initial attempt). A value greater
    /// than 1 indicates that retries occurred.
    pub attempts: u32,

    /// Total time spent waiting between retries.
    ///
    /// This does not include the time spent executing the operation itself,
    /// only the delays between retry attempts. A value of `Duration::ZERO`
    /// indicates either immediate success or immediate permanent failure.
    pub total_wait_time: Duration,
}

/// Retries an operation with detailed result metadata.
///
/// Similar to [`retry_with_policy`], but returns additional information about
/// the retry process, including the number of attempts made and total wait time.
///
/// # Arguments
///
/// * `operation` - A closure that returns the future to retry
/// * `policy` - The retry policy to use
/// * `sleep_fn` - A function that returns a sleep future for the given duration
///
/// # Returns
///
/// A [`RetryResult`] containing:
/// - The final result (success or error with context)
/// - Total number of attempts made
/// - Total time spent waiting between retries
///
/// # Example
///
/// ```rust,ignore
/// use error_rail::async_ext::{retry_with_metadata, ExponentialBackoff};
///
/// let retry_result = retry_with_metadata(
///     || fetch_data(),
///     ExponentialBackoff::default(),
///     |d| tokio::time::sleep(d),
/// ).await;
///
/// println!("Attempts: {}", retry_result.attempts);
/// println!("Total wait time: {:?}", retry_result.total_wait_time);
///
/// match retry_result.result {
///     Ok(data) => println!("Success: {:?}", data),
///     Err(e) => println!("Failed after retries: {:?}", e),
/// }
/// ```
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
    policy.reset();
    let mut attempt = 0u32;
    let mut total_wait_time = Duration::ZERO;

    let result = loop {
        match operation().await {
            Ok(value) => break Ok(value),
            Err(e) if e.is_transient() => {
                if let Some(delay) = policy.next_delay(attempt) {
                    total_wait_time += delay;
                    sleep_fn(delay).await;
                    attempt += 1;
                    continue;
                }
                break Err(ComposableError::new(e)
                    .with_context(crate::context!("exhausted after {} attempts", attempt + 1)));
            },
            Err(e) => {
                break Err(ComposableError::new(e)
                    .with_context(crate::context!("permanent error, no retry")));
            },
        }
    };

    RetryResult { result, attempts: attempt + 1, total_wait_time }
}
