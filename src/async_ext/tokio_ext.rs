//! Tokio-specific async extensions.
//!
//! This module provides Tokio-optimized utilities that leverage Tokio's
//! runtime features like `task_local!` and `tokio::time::sleep`.
//!
//! # Feature Flag
//!
//! Requires the `async-tokio` feature:
//!
//! ```toml
//! [dependencies]
//! error-rail = { version = "0.8", features = ["async-tokio"] }
//! ```

use core::future::Future;
use core::time::Duration;

use crate::traits::TransientError;
use crate::types::{BoxedComposableError, BoxedComposableResult, ComposableError};

use super::retry::{retry_with_policy, RetryPolicy};

/// Retries an async operation using Tokio's sleep.
///
/// This is a convenience wrapper around [`retry_with_policy`] that uses
/// `tokio::time::sleep` for delays, eliminating the need to pass a sleep function.
///
/// # Arguments
///
/// * `operation` - A closure that returns the future to retry
/// * `policy` - The retry policy to use
///
/// # Example
///
/// ```rust,ignore
/// use error_rail::async_ext::{retry_transient, ExponentialBackoff};
///
/// #[tokio::main]
/// async fn main() {
///     let result = retry_transient(
///         || fetch_data(),
///         ExponentialBackoff::default(),
///     ).await;
/// }
/// ```
pub async fn retry_transient<F, Fut, T, E, P>(
    operation: F,
    policy: P,
) -> BoxedComposableResult<T, E>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, E>>,
    E: TransientError,
    P: RetryPolicy,
{
    retry_with_policy(operation, policy, tokio::time::sleep)
        .await
        .map_err(Box::new)
}

/// Retries an async operation with a simple count limit using Tokio's sleep.
///
/// Uses exponential backoff with sensible defaults.
///
/// # Arguments
///
/// * `operation` - A closure that returns the future to retry
/// * `max_attempts` - Maximum number of retry attempts
///
/// # Example
///
/// ```rust,ignore
/// use error_rail::async_ext::retry_transient_n;
///
/// let result = retry_transient_n(|| fetch_data(), 3).await;
/// ```
pub async fn retry_transient_n<F, Fut, T, E>(
    operation: F,
    max_attempts: u32,
) -> BoxedComposableResult<T, E>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, E>>,
    E: TransientError,
{
    use super::retry::ExponentialBackoff;

    let policy = ExponentialBackoff::new().with_max_attempts(max_attempts);
    retry_transient(operation, policy).await
}

/// Result type for timeout operations that can fail with either
/// the inner error or a timeout.
#[derive(Debug)]
pub enum TimeoutResult<T, E> {
    /// Operation completed successfully.
    Ok(T),
    /// Operation failed with an error.
    Err(BoxedComposableError<E>),
    /// Operation timed out.
    Timeout(Duration),
}

impl<T, E> TimeoutResult<T, E> {
    /// Returns `true` if the result is `Ok`.
    pub fn is_ok(&self) -> bool {
        matches!(self, Self::Ok(_))
    }

    /// Returns `true` if the result is `Err`.
    pub fn is_err(&self) -> bool {
        matches!(self, Self::Err(_))
    }

    /// Returns `true` if the operation timed out.
    pub fn is_timeout(&self) -> bool {
        matches!(self, Self::Timeout(_))
    }

    /// Converts to a standard `Result`, treating timeout as an error message.
    pub fn into_result(self) -> BoxedComposableResult<T, E>
    where
        E: From<TimeoutError>,
    {
        match self {
            Self::Ok(v) => Ok(v),
            Self::Err(e) => Err(e),
            Self::Timeout(d) => Err(Box::new(ComposableError::new(E::from(TimeoutError(d))))),
        }
    }
}

/// Error type representing a timeout.
#[derive(Debug, Clone)]
pub struct TimeoutError(pub Duration);

impl core::fmt::Display for TimeoutError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "operation timed out after {:?}", self.0)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for TimeoutError {}

/// Executes an async operation with a timeout, returning a `TimeoutResult`.
///
/// Unlike [`with_timeout`], this function doesn't panic on timeout but
/// returns a structured result that the caller can handle.
///
/// # Example
///
/// ```rust,ignore
/// use error_rail::async_ext::try_with_timeout;
/// use std::time::Duration;
///
/// match try_with_timeout(Duration::from_secs(5), fetch_data()).await {
///     TimeoutResult::Ok(data) => println!("Got data: {:?}", data),
///     TimeoutResult::Err(e) => println!("Error: {}", e.error_chain()),
///     TimeoutResult::Timeout(d) => println!("Timed out after {:?}", d),
/// }
/// ```
pub async fn try_with_timeout<T, E, Fut>(duration: Duration, future: Fut) -> TimeoutResult<T, E>
where
    Fut: Future<Output = Result<T, E>>,
{
    match tokio::time::timeout(duration, future).await {
        Ok(Ok(value)) => TimeoutResult::Ok(value),
        Ok(Err(e)) => TimeoutResult::Err(Box::new(ComposableError::new(e))),
        Err(_elapsed) => TimeoutResult::Timeout(duration),
    }
}
