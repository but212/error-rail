//! Helpers for attaching rich context metadata to errors.
//!
//! Key features:
//! - [`with_context`] / [`with_context_result`] wrap any `Result` while preserving
//!   structured [`ErrorContext`] values.
//! - [`ErrorPipeline`] enables chaining operations, collecting pending contexts,
//!   and mapping/recovering errors before finalizing into a `ComposableError`.
//! - Convenience builders such as [`context_fn`], [`accumulate_context`], and
//!   [`context_accumulator`] let you compose context-aware closures.
//!
//! See the crate-level docs for a high-level overview of when to prefer these
//! utilities over bare `Result` transformations.

use crate::traits::IntoErrorContext;
use crate::types::composable_error::ComposableError;
use crate::types::{BoxedComposableResult, ComposableResult, ErrorContext, ErrorVec};
use std::fmt::Display;

/// Wraps an error with a single context entry.
///
/// Creates a new [`ComposableError`] containing the given error and context.
///
/// # Arguments
///
/// * `error` - The core error to wrap
/// * `context` - Context information to attach
///
/// # Examples
///
/// ```
/// use error_rail::{with_context, ErrorContext};
///
/// let err = with_context("io failed", ErrorContext::tag("disk"));
/// assert_eq!(err.context().len(), 1);
/// ```
#[inline]
pub fn with_context<E, C>(error: E, context: C) -> ComposableError<E>
where
    C: IntoErrorContext,
{
    ComposableError::new(error).with_context(context)
}

/// Transforms a `Result` by adding context to any error.
///
/// On `Err`, wraps the error in a boxed [`ComposableError`] with the provided context.
/// On `Ok`, returns the success value unchanged.
///
/// # Arguments
///
/// * `result` - The result to transform
/// * `context` - Context to attach on error
///
/// # Examples
///
/// ```
/// use error_rail::{with_context_result, ErrorContext};
///
/// let result: Result<i32, &str> = Err("failed");
/// let enriched = with_context_result(result, ErrorContext::tag("auth"));
/// assert!(enriched.is_err());
/// ```
#[inline]
pub fn with_context_result<T, E, C>(result: Result<T, E>, context: C) -> BoxedComposableResult<T, E>
where
    C: IntoErrorContext,
{
    result.map_err(|e| Box::new(with_context(e, context)))
}

/// Creates a reusable closure that wraps errors with a fixed context.
///
/// Returns a function that can be used with `map_err` to attach the same
/// context to multiple error paths.
///
/// # Arguments
///
/// * `context` - The context to attach (must be `Clone`)
///
/// # Examples
///
/// ```
/// use error_rail::{context_fn, ErrorContext};
///
/// let add_tag = context_fn(ErrorContext::tag("network"));
/// let err = add_tag("timeout");
/// assert_eq!(err.context().len(), 1);
/// ```
#[inline]
pub fn context_fn<E, C>(context: C) -> impl Fn(E) -> ComposableError<E>
where
    C: IntoErrorContext + Clone,
{
    move |error| with_context(error, context.clone())
}

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
///     .finish();
/// ```
pub struct ErrorPipeline<T, E> {
    result: Result<T, E>,
    pending_contexts: ErrorVec<ErrorContext>,
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
    ///     .finish()
    ///     .unwrap_err();
    ///
    /// assert!(err.error_chain().contains("calling flaky"));
    /// ```
    #[inline]
    pub fn new(result: Result<T, E>) -> Self {
        Self {
            result,
            pending_contexts: ErrorVec::new(),
        }
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

    /// Transforms the error type using a mapping function.
    ///
    /// Preserves all pending contexts while converting the error to a new type.
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
        ErrorPipeline {
            result: self.result.map_err(f),
            pending_contexts: self.pending_contexts,
        }
    }

    /// Attempts to recover from an error using a fallback function.
    ///
    /// If the current result is `Err`, calls the recovery function. All pending
    /// contexts are preserved regardless of recovery success.
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
        ErrorPipeline {
            result: self.result.or_else(recovery),
            pending_contexts: self.pending_contexts,
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
        ErrorPipeline {
            result: self.result.and_then(f),
            pending_contexts: self.pending_contexts,
        }
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
        ErrorPipeline {
            result: self.result.map(f),
            pending_contexts: self.pending_contexts,
        }
    }

    /// Finalizes the pipeline into a boxed [`ComposableResult`].
    ///
    /// On `Ok`, returns the success value. On `Err`, creates a [`ComposableError`]
    /// with all pending contexts attached and boxes it.
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
    pub fn finish(self) -> BoxedComposableResult<T, E> {
        match self.result {
            Ok(v) => Ok(v),
            Err(e) => {
                let composable = ComposableError::new(e).with_contexts(self.pending_contexts);
                Err(Box::new(composable))
            }
        }
    }

    /// Finalizes the pipeline into an unboxed [`ComposableResult`].
    ///
    /// Similar to `finish`, but returns the error directly without boxing.
    /// Use this when you need to avoid heap allocation.
    ///
    /// # Examples
    ///
    /// ```
    /// use error_rail::{ErrorPipeline, context};
    ///
    /// let result = ErrorPipeline::<u32, &str>::new(Err("failed"))
    ///     .with_context(context!("operation failed"))
    ///     .finish_without_box();
    /// ```
    #[inline]
    #[allow(clippy::result_large_err)]
    pub fn finish_without_box(self) -> ComposableResult<T, E> {
        match self.result {
            Ok(v) => Ok(v),
            Err(e) => {
                let composable = ComposableError::new(e).with_contexts(self.pending_contexts);
                Err(composable)
            }
        }
    }
}

/// Creates an [`ErrorPipeline`] from a result.
///
/// Convenience function equivalent to `ErrorPipeline::new(result)`.
///
/// # Arguments
///
/// * `result` - The result to wrap
///
/// # Examples
///
/// ```
/// use error_rail::{error_pipeline, context};
///
/// let pipeline = error_pipeline::<u32, &str>(Err("failed"))
///     .with_context(context!("step failed"));
/// ```
#[inline]
pub fn error_pipeline<T, E>(result: Result<T, E>) -> ErrorPipeline<T, E> {
    ErrorPipeline::new(result)
}

/// Wraps an error with multiple context entries at once.
///
/// Creates a [`ComposableError`] and attaches all provided contexts.
///
/// # Arguments
///
/// * `error` - The core error to wrap
/// * `contexts` - Iterator of contexts to attach
///
/// # Examples
///
/// ```
/// use error_rail::{accumulate_context, ErrorContext};
///
/// let contexts = vec![
///     ErrorContext::tag("db"),
///     ErrorContext::new("connection failed"),
/// ];
/// let err = accumulate_context("timeout", contexts);
/// assert_eq!(err.context().len(), 2);
/// ```
pub fn accumulate_context<E, I, C>(error: E, contexts: I) -> ComposableError<E>
where
    I: IntoIterator<Item = C>,
    C: IntoErrorContext,
{
    let context_vec: Vec<ErrorContext> = contexts
        .into_iter()
        .map(|c| c.into_error_context())
        .collect();

    ComposableError::new(error).with_contexts(context_vec)
}

/// Creates a reusable closure that wraps errors with multiple contexts.
///
/// Returns a function that attaches all provided contexts to any error.
///
/// # Arguments
///
/// * `contexts` - Iterator of contexts to attach (must be `Clone`)
///
/// # Examples
///
/// ```
/// use error_rail::{context_accumulator, ErrorContext};
///
/// let contexts = vec![ErrorContext::tag("auth"), ErrorContext::tag("api")];
/// let add_contexts = context_accumulator(contexts);
/// let err = add_contexts("unauthorized");
/// assert_eq!(err.context().len(), 2);
/// ```
pub fn context_accumulator<E, I, C>(contexts: I) -> impl Fn(E) -> ComposableError<E>
where
    I: IntoIterator<Item = C> + Clone,
    C: IntoErrorContext + Clone,
{
    move |error| accumulate_context(error, contexts.clone())
}

/// Formats a [`ComposableError`] as a human-readable error chain.
///
/// Returns a string representation showing all contexts and the core error.
///
/// # Arguments
///
/// * `error` - The composable error to format
///
/// # Examples
///
/// ```
/// use error_rail::{ComposableError, ErrorContext, format_error_chain};
///
/// let err = ComposableError::new("failed")
///     .with_context(ErrorContext::tag("db"));
/// let chain = format_error_chain(&err);
/// assert!(chain.contains("failed"));
/// ```
pub fn format_error_chain<E>(error: &ComposableError<E>) -> String
where
    E: Display,
{
    error.error_chain()
}

/// Extracts all context entries from a [`ComposableError`].
///
/// Returns a vector containing all attached contexts.
///
/// # Arguments
///
/// * `error` - The composable error to extract from
///
/// # Examples
///
/// ```
/// use error_rail::{ComposableError, ErrorContext, extract_context};
///
/// let err = ComposableError::new("failed")
///     .with_context(ErrorContext::tag("db"));
/// let contexts = extract_context(&err);
/// assert_eq!(contexts.len(), 1);
/// ```
pub fn extract_context<E>(error: &ComposableError<E>) -> Vec<ErrorContext> {
    error.context()
}
