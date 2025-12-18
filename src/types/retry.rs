use crate::traits::TransientError;
use crate::types::ErrorContext;
use crate::types::ErrorPipeline;
use core::time::Duration;

#[cfg(not(feature = "std"))]
use alloc::format;
#[cfg(feature = "std")]
use std::format;

/// Operations for retry policies and transient error handling.
///
/// `RetryOps` provides a fluent API for configuring retry behavior and
/// checking transient error states. It wraps an [`ErrorPipeline`] and
/// adds retry-specific metadata to the error context.
///
/// # Examples
///
/// ```
/// use error_rail::{ErrorPipeline, traits::TransientError};
/// use core::time::Duration;
///
/// #[derive(Debug)]
/// struct NetworkError;
///
/// impl TransientError for NetworkError {
///     fn is_transient(&self) -> bool { true }
/// }
///
/// let pipeline: ErrorPipeline<(), NetworkError> = ErrorPipeline::new(Err(NetworkError));
/// let retry_ops = pipeline.retry()
///     .max_retries(3)
///     .after_hint(Duration::from_secs(1));
/// ```
pub struct RetryOps<T, E> {
    pub(crate) pipeline: ErrorPipeline<T, E>,
}

impl<T, E> RetryOps<T, E> {
    /// Checks if the error is transient and eligible for retry.
    ///
    /// This delegates to the inner error's [`TransientError::is_transient`] implementation.
    /// Returns `false` if the pipeline contains a success value.
    ///
    /// # Examples
    ///
    /// ```
    /// use error_rail::{ErrorPipeline, traits::TransientError};
    ///
    /// #[derive(Debug)]
    /// struct TimeoutError;
    ///
    /// impl TransientError for TimeoutError {
    ///     fn is_transient(&self) -> bool { true }
    /// }
    ///
    /// let pipeline: ErrorPipeline<(), TimeoutError> = ErrorPipeline::new(Err(TimeoutError));
    /// assert!(pipeline.retry().is_transient());
    /// ```
    pub fn is_transient(&self) -> bool
    where
        E: TransientError,
    {
        self.pipeline.is_transient()
    }

    /// Attaches a maximum retry count hint to the error context.
    ///
    /// This adds metadata that can be used by retry logic to limit retry attempts.
    /// The hint is stored as context metadata and does not enforce retry limits itself.
    ///
    /// # Arguments
    ///
    /// * `count` - Maximum number of retry attempts to suggest
    ///
    /// # Examples
    ///
    /// ```
    /// use error_rail::{ErrorPipeline, traits::TransientError};
    ///
    /// #[derive(Debug)]
    /// struct Error;
    ///
    /// impl TransientError for Error {
    ///     fn is_transient(&self) -> bool { true }
    /// }
    ///
    /// let pipeline: ErrorPipeline<(), Error> = ErrorPipeline::new(Err(Error));
    /// let retry_ops = pipeline.retry().max_retries(5);
    /// ```
    pub fn max_retries(mut self, count: u32) -> Self {
        self.pipeline = self
            .pipeline
            .with_context(ErrorContext::metadata("max_retries_hint", format!("{}", count)));
        self
    }

    /// Attaches a retry delay hint to the error context.
    ///
    /// This adds metadata suggesting how long to wait before retrying.
    /// Useful for rate limiting or exponential backoff strategies.
    ///
    /// # Arguments
    ///
    /// * `duration` - Suggested delay before next retry attempt
    ///
    /// # Examples
    ///
    /// ```
    /// use error_rail::{ErrorPipeline, traits::TransientError};
    /// use core::time::Duration;
    ///
    /// #[derive(Debug)]
    /// struct RateLimitError;
    ///
    /// impl TransientError for RateLimitError {
    ///     fn is_transient(&self) -> bool { true }
    /// }
    ///
    /// let pipeline: ErrorPipeline<(), RateLimitError> = ErrorPipeline::new(Err(RateLimitError));
    /// let retry_ops = pipeline.retry().after_hint(Duration::from_secs(60));
    /// ```
    pub fn after_hint(mut self, duration: Duration) -> Self {
        self.pipeline = self
            .pipeline
            .with_context(ErrorContext::metadata("retry_after_hint", format!("{:?}", duration)));
        self
    }

    /// Converts the retry operations back to an error pipeline.
    ///
    /// This consumes the `RetryOps` and returns the underlying [`ErrorPipeline`],
    /// preserving any retry hints that were added to the error context.
    ///
    /// # Examples
    ///
    /// ```
    /// use error_rail::{ErrorPipeline, traits::TransientError};
    /// use core::time::Duration;
    ///
    /// #[derive(Debug)]
    /// struct NetworkError;
    ///
    /// impl TransientError for NetworkError {
    ///     fn is_transient(&self) -> bool { true }
    /// }
    ///
    /// let pipeline: ErrorPipeline<(), NetworkError> = ErrorPipeline::new(Err(NetworkError));
    /// let retry_ops = pipeline.retry().max_retries(3);
    /// let pipeline_again = retry_ops.to_error_pipeline();
    /// ```
    pub fn to_error_pipeline(self) -> ErrorPipeline<T, E> {
        self.pipeline
    }
}
