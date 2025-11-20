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
use crate::types::{BoxedComposableResult, ErrorContext, ErrorPipeline};
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
