use crate::traits::TransientError;
use crate::types::ErrorContext;
use crate::types::ErrorPipeline;
use core::time::Duration;

#[cfg(not(feature = "std"))]
use alloc::format;
#[cfg(feature = "std")]
use std::format;

/// Operations for retry policies and transient error handling.
pub struct RetryOps<T, E> {
    pub(crate) pipeline: ErrorPipeline<T, E>,
}

impl<T, E> RetryOps<T, E> {
    /// Checks if the error is transient.
    ///
    /// This is a pass-through to the inner error's `TransientError` implementation.
    /// It does not modify the pipeline.
    pub fn is_transient(&self) -> bool
    where
        E: TransientError,
    {
        self.pipeline.is_transient()
    }

    /// Adds a hint for the maximum number of retries.
    ///
    /// This attaches metadata to the error context.
    pub fn max_retries(mut self, count: u32) -> Self {
        self.pipeline = self.pipeline.with_context(ErrorContext::metadata(
            "max_retries_hint",
            format!("{}", count),
        ));
        self
    }

    /// Adds a hint for the retry delay.
    ///
    /// This attaches metadata to the error context.
    pub fn after_hint(mut self, duration: Duration) -> Self {
        self.pipeline = self.pipeline.with_context(ErrorContext::metadata(
            "retry_after_hint",
            format!("{:?}", duration),
        ));
        self
    }

    // Add other retry-related methods here
}
