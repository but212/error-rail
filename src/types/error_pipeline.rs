use crate::traits::TransientError;
use crate::types::accumulator::Accumulator;
use crate::types::alloc_type::Box;
use crate::types::composable_error::ComposableError;
use crate::types::lazy_context::LazyGroupContext;
use crate::types::marked_error::MarkedError;
use crate::{ComposableResult, ErrorContext, IntoErrorContext};

use crate::types::utils::u32_to_cow;

/// A builder for composing error transformations with accumulated context.
///
/// `ErrorPipeline` allows you to:
/// - Attach multiple contexts that are only materialized on error
/// - Chain operations like `map`, `and_then`, and `recover`
/// - Defer the creation of [`ComposableError`] until finalization
///
/// Contexts are stored in a stack and only applied if the pipeline ends in an error.
///
/// # Type Parameters
///
/// * `T` - The success value type
/// * `E` - The error type
///
/// # Examples
///
/// ```
/// use error_rail::{ErrorPipeline, context};
///
/// let result = ErrorPipeline::<u32, &str>::new(Err("failed"))
///     .with_context(context!("step 1"))
///     .with_context(context!("step 2"))
///     .finish_boxed();
///
/// assert!(result.is_err());
/// ```
#[must_use]
pub struct ErrorPipeline<T, E> {
    result: Result<T, E>,
    pending_contexts: Accumulator<ErrorContext>,
}

impl<T, E> ErrorPipeline<T, E> {
    /// Creates a pipeline from an existing `Result`.
    ///
    /// # Arguments
    ///
    /// * `result` - The result to wrap in a pipeline
    ///
    /// # Examples
    ///
    /// ```
    /// use error_rail::{context, ErrorPipeline};
    ///
    /// fn flaky() -> Result<(), &'static str> {
    ///     Err("boom")
    /// }
    ///
    /// let err = ErrorPipeline::new(flaky())
    ///     .with_context(context!("calling flaky"))
    ///     .finish_boxed()
    ///     .unwrap_err();
    ///
    /// assert!(err.error_chain().contains("calling flaky"));
    /// ```
    #[inline]
    pub fn new(result: Result<T, E>) -> Self {
        Self { result, pending_contexts: Accumulator::new() }
    }

    /// Adds a context entry to the pending context stack.
    ///
    /// If the current result is `Ok`, this is a no-op. Otherwise, the context
    /// is queued to be attached when `finish` is called.
    ///
    /// # Arguments
    ///
    /// * `context` - Context to add
    ///
    /// # Examples
    ///
    /// ```
    /// use error_rail::{ErrorPipeline, ErrorContext};
    ///
    /// let pipeline: ErrorPipeline<&str, &str> = ErrorPipeline::new(Err("error"))
    ///     .with_context(ErrorContext::tag("db"));
    /// ```
    #[inline]
    pub fn with_context<C>(mut self, context: C) -> Self
    where
        C: IntoErrorContext,
    {
        if self.result.is_ok() {
            return self;
        }

        let ctx = context.into_error_context();
        self.pending_contexts.push(ctx);
        self
    }

    /// Alias for `with_context`.
    ///
    /// Adds a context entry to the pending context stack.
    #[inline]
    pub fn context<C>(self, context: C) -> Self
    where
        C: IntoErrorContext,
    {
        self.with_context(context)
    }

    /// Creates a retry operations builder for this pipeline.
    ///
    /// Returns a `RetryHints` wrapper that provides fluent methods for attaching
    /// retry metadata hints and checking transient error states.
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
    #[inline]
    pub fn retry(self) -> crate::types::retry::RetryOps<T, E> {
        crate::types::retry::RetryOps { pipeline: self }
    }

    /// Marks the error as transient or permanent based on a closure.
    ///
    /// This allows for flexible retry control without implementing the [`crate::traits::TransientError`]
    /// trait for the error type.
    ///
    /// # Arguments
    ///
    /// * `classifier` - A closure that returns `true` if the error should be treated as transient
    ///
    /// # Examples
    ///
    /// ```
    /// use error_rail::ErrorPipeline;
    ///
    /// let result = ErrorPipeline::<(), &str>::new(Err("temporary failure"))
    ///     .mark_transient_if(|e| e.contains("temporary"))
    ///     .retry()
    ///     .max_retries(3)
    ///     .to_error_pipeline()
    ///     .finish();
    /// ```
    #[inline]
    pub fn mark_transient_if<F>(self, classifier: F) -> ErrorPipeline<T, MarkedError<E, F>>
    where
        F: Fn(&E) -> bool,
    {
        ErrorPipeline {
            result: self
                .result
                .map_err(|e| MarkedError { inner: e, classifier }),
            pending_contexts: self.pending_contexts,
        }
    }

    /// Transforms the pipeline using a function.
    ///
    /// This is a generic step function that can be used for chaining operations.
    /// Currently behaves like `and_then`.
    #[inline]
    pub fn step<U, F>(self, f: F) -> ErrorPipeline<U, E>
    where
        F: FnOnce(T) -> Result<U, E>,
    {
        self.and_then(f)
    }

    /// Transforms the error type using a mapping function.
    ///
    /// If the current result is `Err`, applies the function. Otherwise, preserves
    /// the success value and pending contexts.
    ///
    /// # Arguments
    ///
    /// * `f` - Function to transform the error
    ///
    /// # Examples
    ///
    /// ```
    /// use error_rail::ErrorPipeline;
    ///
    /// let pipeline = ErrorPipeline::<&str, &str>::new(Err("text error"))
    ///     .map_error(|e| e.len());
    /// ```
    #[inline]
    pub fn map_error<F, NewE>(self, f: F) -> ErrorPipeline<T, NewE>
    where
        F: FnOnce(E) -> NewE,
    {
        ErrorPipeline { result: self.result.map_err(f), pending_contexts: self.pending_contexts }
    }

    /// Attempts to recover from an error using a fallback function.
    ///
    /// If the current result is `Err`, calls the recovery function. If recovery
    /// succeeds, all pending contexts are discarded since the error is resolved.
    /// If recovery fails, pending contexts are preserved.
    ///
    /// **Note:** Successful recovery discards all accumulated contexts from the
    /// error path, as the error has been handled and resolved.
    ///
    /// # Arguments
    ///
    /// * `recovery` - Function that attempts to recover from the error
    ///
    /// # Examples
    ///
    /// ```
    /// use error_rail::ErrorPipeline;
    ///
    /// let pipeline = ErrorPipeline::new(Err("error"))
    ///     .recover(|_| Ok(42));
    /// ```
    #[inline]
    pub fn recover<F>(self, recovery: F) -> ErrorPipeline<T, E>
    where
        F: FnOnce(E) -> Result<T, E>,
    {
        match self.result {
            Ok(v) => ErrorPipeline { result: Ok(v), pending_contexts: self.pending_contexts },
            Err(e) => match recovery(e) {
                Ok(v) => ErrorPipeline { result: Ok(v), pending_contexts: Accumulator::new() },
                Err(e) => ErrorPipeline { result: Err(e), pending_contexts: self.pending_contexts },
            },
        }
    }

    /// Recovers from an error using a default value.
    ///
    /// If the current result is `Err`, replaces it with `Ok(value)`.
    /// All pending contexts are discarded on recovery since the error is resolved.
    ///
    /// **Note:** This method unconditionally discards all accumulated contexts
    /// from the error path when recovery occurs.
    ///
    /// # Arguments
    ///
    /// * `value` - The default value to use in case of error
    ///
    /// # Examples
    ///
    /// ```
    /// use error_rail::ErrorPipeline;
    ///
    /// let pipeline = ErrorPipeline::<i32, &str>::new(Err("error"))
    ///     .fallback(42);
    /// ```
    #[inline]
    pub fn fallback(self, value: T) -> ErrorPipeline<T, E> {
        match self.result {
            Ok(v) => ErrorPipeline { result: Ok(v), pending_contexts: self.pending_contexts },
            Err(_) => ErrorPipeline { result: Ok(value), pending_contexts: Accumulator::new() },
        }
    }

    /// Recovers from an error using a safe function that always returns a value.
    ///
    /// If the current result is `Err`, calls `f` to get a success value.
    /// Pending contexts are discarded on recovery since the error is resolved.
    ///
    /// # Arguments
    ///
    /// * `f` - Function that produces a success value from the error
    ///
    /// # Examples
    ///
    /// ```
    /// use error_rail::ErrorPipeline;
    ///
    /// let pipeline = ErrorPipeline::<i32, &str>::new(Err("error"))
    ///     .recover_safe(|_| 42);
    /// ```
    #[inline]
    pub fn recover_safe<F>(self, f: F) -> ErrorPipeline<T, E>
    where
        F: FnOnce(E) -> T,
    {
        match self.result {
            Ok(v) => ErrorPipeline { result: Ok(v), pending_contexts: self.pending_contexts },
            Err(e) => ErrorPipeline { result: Ok(f(e)), pending_contexts: Accumulator::new() },
        }
    }

    /// Chains a fallible operation on the success value.
    ///
    /// If the current result is `Ok`, applies the function. Otherwise, preserves
    /// the error and pending contexts.
    ///
    /// # Arguments
    ///
    /// * `f` - Function to apply to the success value
    ///
    /// # Examples
    ///
    /// ```
    /// use error_rail::ErrorPipeline;
    ///
    /// let pipeline = ErrorPipeline::<i32, &str>::new(Ok(5))
    ///     .and_then(|x| Ok(x * 2));
    /// ```
    #[inline]
    pub fn and_then<U, F>(self, f: F) -> ErrorPipeline<U, E>
    where
        F: FnOnce(T) -> Result<U, E>,
    {
        ErrorPipeline { result: self.result.and_then(f), pending_contexts: self.pending_contexts }
    }

    /// Transforms the success value using a mapping function.
    ///
    /// If the current result is `Ok`, applies the function. Otherwise, preserves
    /// the error and pending contexts.
    ///
    /// # Arguments
    ///
    /// * `f` - Function to transform the success value
    ///
    /// # Examples
    ///
    /// ```
    /// use error_rail::ErrorPipeline;
    ///
    /// let pipeline = ErrorPipeline::<i32, &str>::new(Ok(5))
    ///     .map(|x| x * 2);
    /// ```
    #[inline]
    pub fn map<U, F>(self, f: F) -> ErrorPipeline<U, E>
    where
        F: FnOnce(T) -> U,
    {
        ErrorPipeline { result: self.result.map(f), pending_contexts: self.pending_contexts }
    }

    /// Finalizes the pipeline into a boxed [`ComposableResult`].
    ///
    /// On `Ok`, returns the success value. On `Err`, creates a [`ComposableError`]
    /// with all pending contexts attached and boxes it to reduce stack size.
    ///
    /// **Note**: This method is used by the [`rail!`](crate::rail) macro and is
    /// recommended for public APIs due to the smaller stack footprint (8 bytes).
    /// For internal code, consider using [`finish()`](Self::finish) to avoid heap allocation.
    ///
    /// # Examples
    ///
    /// ```
    /// use error_rail::{ErrorPipeline, context};
    ///
    /// let result = ErrorPipeline::<u32, &str>::new(Err("failed"))
    ///     .with_context(context!("operation failed"))
    ///     .finish_boxed();
    /// ```
    #[inline]
    pub fn finish_boxed(self) -> crate::types::BoxedComposableResult<T, E> {
        match self.result {
            Ok(v) => Ok(v),
            Err(e) => {
                let composable = ComposableError::new(e).with_contexts(self.pending_contexts);
                Err(Box::new(composable))
            },
        }
    }

    /// Finalizes the pipeline into an unboxed [`ComposableResult`].
    ///
    /// This is the default method that returns errors directly without boxing.
    /// Use this when you need to avoid heap allocation or are working with internal APIs.
    ///
    /// **Note**: This method is used by the [`rail_unboxed!`](crate::rail_unboxed) macro.
    /// For public APIs, consider using [`finish_boxed()`](Self::finish_boxed) instead.
    ///
    /// # Examples
    ///
    /// ```
    /// use error_rail::{ErrorPipeline, context};
    ///
    /// let result = ErrorPipeline::<u32, &str>::new(Err("failed"))
    ///     .with_context(context!("operation failed"))
    ///     .finish();
    /// ```
    #[inline]
    #[allow(clippy::result_large_err)]
    pub fn finish(self) -> ComposableResult<T, E> {
        match self.result {
            Ok(v) => Ok(v),
            Err(e) => {
                let composable = ComposableError::new(e).with_contexts(self.pending_contexts);
                Err(composable)
            },
        }
    }

    /// Checks if the current error (if any) is transient and may be retried.
    ///
    /// This method integrates with the [`crate::traits::TransientError`] trait to help determine
    /// whether a retry operation might succeed.
    ///
    /// # Returns
    ///
    /// - `true` if the pipeline contains a transient error
    /// - `false` if the pipeline is `Ok` or contains a permanent error
    ///
    /// # Examples
    ///
    /// ```
    /// use error_rail::{ErrorPipeline, traits::TransientError};
    ///
    /// #[derive(Debug)]
    /// struct NetworkError { timeout: bool }
    ///
    /// impl TransientError for NetworkError {
    ///     fn is_transient(&self) -> bool { self.timeout }
    /// }
    ///
    /// let timeout_err: ErrorPipeline<(), NetworkError> = ErrorPipeline::new(Err(NetworkError { timeout: true }));
    /// assert!(timeout_err.is_transient());
    ///
    /// let auth_err: ErrorPipeline<(), NetworkError> = ErrorPipeline::new(Err(NetworkError { timeout: false }));
    /// assert!(!auth_err.is_transient());
    /// ```
    #[inline]
    pub fn is_transient(&self) -> bool
    where
        E: TransientError,
    {
        match &self.result {
            Ok(_) => false,
            Err(e) => e.is_transient(),
        }
    }

    /// Attempts recovery only if the error is transient.
    ///
    /// This method combines [`TransientError`] classification with recovery logic,
    /// making it easy to implement retry patterns with external libraries.
    ///
    /// # Arguments
    ///
    /// * `recovery` - Function that attempts to recover from a transient error
    ///
    /// # Behavior
    ///
    /// - If `Ok`: Returns the pipeline unchanged
    /// - If `Err` and transient: Calls recovery function
    /// - If `Err` and permanent: Skips recovery, preserves error
    ///
    /// # Examples
    ///
    /// ```
    /// use error_rail::{ErrorPipeline, traits::TransientError};
    ///
    /// #[derive(Debug, Clone)]
    /// struct ApiError { code: u16 }
    ///
    /// impl TransientError for ApiError {
    ///     fn is_transient(&self) -> bool { self.code == 503 || self.code == 429 }
    /// }
    ///
    /// let retry_count = std::cell::Cell::new(0);
    ///
    /// let result = ErrorPipeline::new(Err(ApiError { code: 503 }))
    ///     .recover_transient(|_| {
    ///         retry_count.set(retry_count.get() + 1);
    ///         Ok(42) // Simulated successful retry
    ///     });
    ///
    /// assert!(result.finish().is_ok());
    /// assert_eq!(retry_count.get(), 1);
    /// ```
    #[inline]
    pub fn recover_transient<F>(self, recovery: F) -> ErrorPipeline<T, E>
    where
        E: TransientError,
        F: FnOnce(E) -> Result<T, E>,
    {
        match self.result {
            Ok(v) => ErrorPipeline { result: Ok(v), pending_contexts: self.pending_contexts },
            Err(e) if e.is_transient() => match recovery(e) {
                Ok(v) => ErrorPipeline { result: Ok(v), pending_contexts: Accumulator::new() },
                Err(e) => ErrorPipeline { result: Err(e), pending_contexts: self.pending_contexts },
            },
            Err(e) => ErrorPipeline { result: Err(e), pending_contexts: self.pending_contexts },
        }
    }

    /// Prepares the error for external retry libraries by classifying it.
    ///
    /// Returns `Some(pipeline)` if retry should be attempted (error is transient),
    /// or `None` if retry should stop (success or permanent error).
    ///
    /// This method is designed for easy integration with retry libraries that
    /// use `Option`-based continuation patterns.
    ///
    /// # Examples
    ///
    /// ```
    /// use error_rail::{ErrorPipeline, traits::TransientError};
    ///
    /// #[derive(Debug)]
    /// struct DbError { is_deadlock: bool }
    ///
    /// impl TransientError for DbError {
    ///     fn is_transient(&self) -> bool { self.is_deadlock }
    /// }
    ///
    /// // Transient error: continue retrying
    /// let deadlock: ErrorPipeline<(), DbError> = ErrorPipeline::new(Err(DbError { is_deadlock: true }));
    /// assert!(deadlock.should_retry().is_some());
    ///
    /// // Permanent error: stop retrying
    /// let constraint: ErrorPipeline<(), DbError> = ErrorPipeline::new(Err(DbError { is_deadlock: false }));
    /// assert!(constraint.should_retry().is_none());
    ///
    /// // Success: stop retrying
    /// let success = ErrorPipeline::<i32, DbError>::new(Ok(42));
    /// assert!(success.should_retry().is_none());
    /// ```
    #[inline]
    pub fn should_retry(self) -> Option<Self>
    where
        E: TransientError,
    {
        match &self.result {
            Ok(_) => None,
            Err(e) if e.is_transient() => Some(self),
            Err(_) => None,
        }
    }

    /// Returns the retry-after hint from the error, if available.
    ///
    /// This is useful for implementing respectful backoff strategies when
    /// dealing with rate-limited APIs.
    ///
    /// # Examples
    ///
    /// ```
    /// use error_rail::{ErrorPipeline, traits::TransientError};
    /// use core::time::Duration;
    ///
    /// #[derive(Debug)]
    /// struct RateLimitError { retry_after_secs: u64 }
    ///
    /// impl TransientError for RateLimitError {
    ///     fn is_transient(&self) -> bool { true }
    ///     fn retry_after_hint(&self) -> Option<Duration> {
    ///         Some(Duration::from_secs(self.retry_after_secs))
    ///     }
    /// }
    ///
    /// let err: ErrorPipeline<(), RateLimitError> = ErrorPipeline::new(Err(RateLimitError { retry_after_secs: 60 }));
    /// assert_eq!(err.retry_after_hint(), Some(Duration::from_secs(60)));
    /// ```
    #[inline]
    pub fn retry_after_hint(&self) -> Option<core::time::Duration>
    where
        E: TransientError,
    {
        match &self.result {
            Ok(_) => None,
            Err(e) => e.retry_after_hint(),
        }
    }

    /// Adds a tag indicating this error was retried.
    ///
    /// This is useful for tracking retry attempts in logs and error reports.
    /// Adds metadata about retry count and whether the error was transient.
    ///
    /// # Arguments
    ///
    /// * `attempt` - The current retry attempt number (1-indexed)
    ///
    /// # Examples
    ///
    /// ```
    /// use error_rail::{ErrorPipeline, context};
    ///
    /// let result = ErrorPipeline::<u32, &str>::new(Err("timeout"))
    ///     .with_retry_context(3)
    ///     .finish_boxed();
    ///
    /// if let Err(err) = result {
    ///     let chain = err.error_chain();
    ///     assert!(chain.contains("retry_attempt=3"));
    /// }
    /// ```
    #[inline]
    pub fn with_retry_context(self, attempt: u32) -> Self {
        if self.result.is_err() {
            // Use lookup table for small numbers to avoid heap allocation
            let attempt_str = u32_to_cow(attempt);
            self.with_context(LazyGroupContext::new(move || {
                ErrorContext::metadata("retry_attempt", attempt_str)
            }))
        } else {
            self
        }
    }
}
