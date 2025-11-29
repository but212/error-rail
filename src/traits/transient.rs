//! Transient error classification for retry strategies.
//!
//! This module provides the [`TransientError`] trait for classifying errors
//! as transient (temporary, potentially recoverable by retry) or permanent.
//!
//! # Integration with External Libraries
//!
//! error-rail itself does not implement retry/circuit-breaker logic. Instead,
//! it provides this trait to easily integrate with external resilience libraries
//! like `backoff`, `retry`, or custom implementations.
//!
//! # Examples
//!
//! ```
//! use error_rail::traits::TransientError;
//!
//! #[derive(Debug)]
//! enum MyError {
//!     NetworkTimeout,
//!     RateLimited { retry_after_ms: u64 },
//!     InvalidInput,
//!     DatabaseError,
//! }
//!
//! impl TransientError for MyError {
//!     fn is_transient(&self) -> bool {
//!         matches!(self, MyError::NetworkTimeout | MyError::RateLimited { .. })
//!     }
//!
//!     fn retry_after_hint(&self) -> Option<core::time::Duration> {
//!         match self {
//!             MyError::RateLimited { retry_after_ms } => {
//!                 Some(core::time::Duration::from_millis(*retry_after_ms))
//!             }
//!             _ => None,
//!         }
//!     }
//! }
//! ```

use core::time::Duration;

/// Classification of errors as transient or permanent.
///
/// Transient errors are temporary failures that may succeed if retried,
/// such as network timeouts, rate limiting, or temporary service unavailability.
///
/// This trait enables integration with external retry/circuit-breaker libraries
/// by providing a standard interface for error classification.
///
/// # Design Philosophy
///
/// error-rail follows the principle of composition over built-in features.
/// Rather than implementing full retry logic, we provide this classification
/// trait that works seamlessly with:
///
/// - [`ErrorPipeline::should_retry`](crate::ErrorPipeline::should_retry)
/// - External crates like `backoff`, `retry`, `again`
/// - Custom retry implementations
///
/// # Examples
///
/// ## Basic Implementation
///
/// ```
/// use error_rail::traits::TransientError;
///
/// #[derive(Debug)]
/// struct TimeoutError;
///
/// impl TransientError for TimeoutError {
///     fn is_transient(&self) -> bool {
///         true // Timeouts are always transient
///     }
/// }
/// ```
///
/// ## With Retry Hint
///
/// ```
/// use error_rail::traits::TransientError;
/// use core::time::Duration;
///
/// #[derive(Debug)]
/// struct RateLimitError {
///     retry_after_secs: u64,
/// }
///
/// impl TransientError for RateLimitError {
///     fn is_transient(&self) -> bool {
///         true
///     }
///
///     fn retry_after_hint(&self) -> Option<Duration> {
///         Some(Duration::from_secs(self.retry_after_secs))
///     }
/// }
/// ```
pub trait TransientError {
    /// Returns `true` if this error is transient and may succeed on retry.
    ///
    /// # Guidelines
    ///
    /// Return `true` for:
    /// - Network timeouts
    /// - Rate limiting (HTTP 429)
    /// - Service temporarily unavailable (HTTP 503)
    /// - Connection reset/refused (may indicate temporary overload)
    /// - Deadlock/lock contention errors
    ///
    /// Return `false` for:
    /// - Authentication/authorization failures
    /// - Invalid input/validation errors
    /// - Resource not found
    /// - Business logic violations
    fn is_transient(&self) -> bool;

    /// Returns `true` if this error is permanent and should not be retried.
    ///
    /// Default implementation returns `!self.is_transient()`.
    #[inline]
    fn is_permanent(&self) -> bool {
        !self.is_transient()
    }

    /// Optional hint for how long to wait before retrying.
    ///
    /// This is useful for rate-limiting errors that include a `Retry-After` header.
    /// External retry libraries can use this to implement respectful backoff.
    ///
    /// Returns `None` by default, indicating no specific wait time is suggested.
    #[inline]
    fn retry_after_hint(&self) -> Option<Duration> {
        None
    }

    /// Returns the maximum number of retry attempts for this error.
    ///
    /// Returns `None` by default, indicating no specific limit is suggested.
    /// External libraries should use their own default limits.
    #[inline]
    fn max_retries_hint(&self) -> Option<u32> {
        None
    }
}

/// Blanket implementation for standard I/O errors.
#[cfg(feature = "std")]
impl TransientError for std::io::Error {
    fn is_transient(&self) -> bool {
        use std::io::ErrorKind;
        matches!(
            self.kind(),
            ErrorKind::ConnectionRefused
                | ErrorKind::ConnectionReset
                | ErrorKind::ConnectionAborted
                | ErrorKind::TimedOut
                | ErrorKind::Interrupted
                | ErrorKind::WouldBlock
        )
    }
}

/// Extension methods for working with transient errors.
pub trait TransientErrorExt<T, E: TransientError> {
    /// Converts a transient error to `Some(Err(e))` for retry, or `None` to stop.
    ///
    /// This is useful for integrating with retry libraries that use `Option` to
    /// signal whether to continue retrying.
    ///
    /// # Examples
    ///
    /// ```
    /// use error_rail::traits::{TransientError, TransientErrorExt};
    ///
    /// #[derive(Debug)]
    /// struct MyError { transient: bool }
    /// impl TransientError for MyError {
    ///     fn is_transient(&self) -> bool { self.transient }
    /// }
    ///
    /// let transient_err: Result<(), MyError> = Err(MyError { transient: true });
    /// assert!(transient_err.retry_if_transient().is_some());
    ///
    /// let permanent_err: Result<(), MyError> = Err(MyError { transient: false });
    /// assert!(permanent_err.retry_if_transient().is_none());
    /// ```
    fn retry_if_transient(self) -> Option<Result<T, E>>;
}

impl<T, E: TransientError> TransientErrorExt<T, E> for Result<T, E> {
    fn retry_if_transient(self) -> Option<Result<T, E>> {
        match &self {
            Ok(_) => None, // Success, no retry needed
            Err(e) if e.is_transient() => Some(self),
            Err(_) => None, // Permanent error, stop retrying
        }
    }
}
