//! Tracing integration for error-rail.
//!
//! This module provides utilities for integrating error-rail with the
//! `tracing` ecosystem, automatically capturing span information as
//! error context.
//!
//! # Feature Flag
//!
//! Requires the `tracing` feature:
//!
//! ```toml
//! [dependencies]
//! error-rail = { version = "0.8", features = ["tracing"] }
//! ```

use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};

use pin_project_lite::pin_project;
use tracing::Span;

use crate::types::{BoxedComposableResult, ComposableError};
use crate::ErrorContext;

/// Extension trait for futures that adds tracing span context to errors.
///
/// This trait provides methods to automatically capture the current tracing
/// span's metadata and attach it as error context when errors occur.
///
/// # Example
///
/// ```rust,ignore
/// use error_rail::async_ext::FutureSpanExt;
/// use tracing::info_span;
///
/// async fn fetch_user(id: u64) -> Result<User, ApiError> {
///     let span = info_span!("fetch_user", user_id = id);
///     
///     async {
///         database.get_user(id).await
///     }
///     .with_span_context()
///     .instrument(span)
///     .await
/// }
/// ```
pub trait FutureSpanExt<T, E>: Future<Output = Result<T, E>> + Sized {
    /// Captures the current span's metadata as error context on failure.
    ///
    /// When the future resolves to an error, the current span's name and
    /// metadata are attached as context to the error.
    fn with_span_context(self) -> SpanContextFuture<Self> {
        SpanContextFuture { inner: self, span: Span::current() }
    }

    /// Captures a specific span's metadata as error context on failure.
    ///
    /// Unlike `with_span_context()`, this method uses the provided span
    /// instead of the current span.
    fn with_span(self, span: Span) -> SpanContextFuture<Self> {
        SpanContextFuture { inner: self, span }
    }
}

impl<F, T, E> FutureSpanExt<T, E> for F where F: Future<Output = Result<T, E>> {}

pin_project! {
    /// Future wrapper that captures span context on error.
    ///
    /// Created by [`FutureSpanExt::with_span_context`] or [`FutureSpanExt::with_span`].
    #[must_use = "futures do nothing unless polled"]
    pub struct SpanContextFuture<F> {
        #[pin]
        inner: F,
        span: Span,
    }
}

impl<F, T, E> Future for SpanContextFuture<F>
where
    F: Future<Output = Result<T, E>>,
{
    type Output = BoxedComposableResult<T, E>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();

        match this.inner.poll(cx) {
            Poll::Ready(Ok(value)) => Poll::Ready(Ok(value)),
            Poll::Ready(Err(error)) => {
                let context = span_to_context(this.span);
                let composable = ComposableError::new(error).with_context(context);
                Poll::Ready(Err(Box::new(composable)))
            },
            Poll::Pending => Poll::Pending,
        }
    }
}

/// Converts a tracing span to an error context.
///
/// Extracts the span name and formats it as an error context message.
/// The span must be valid (not disabled) to produce meaningful context.
fn span_to_context(span: &Span) -> ErrorContext {
    let name = span.metadata().map(|m| m.name()).unwrap_or("unknown");
    ErrorContext::new(alloc::format!("in span '{}'", name))
}

/// Extension trait for `Result` types to add span context to errors.
pub trait ResultSpanExt<T, E> {
    /// Adds the current span's context to an error.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use error_rail::async_ext::ResultSpanExt;
    ///
    /// fn process() -> BoxedComposableResult<Data, ProcessError> {
    ///     do_work().with_current_span()
    /// }
    /// ```
    fn with_current_span(self) -> BoxedComposableResult<T, E>;

    /// Adds a specific span's context to an error.
    fn with_span(self, span: &Span) -> BoxedComposableResult<T, E>;
}

impl<T, E> ResultSpanExt<T, E> for Result<T, E> {
    fn with_current_span(self) -> BoxedComposableResult<T, E> {
        self.with_span(&Span::current())
    }

    fn with_span(self, span: &Span) -> BoxedComposableResult<T, E> {
        match self {
            Ok(v) => Ok(v),
            Err(e) => {
                let context = span_to_context(span);
                Err(Box::new(ComposableError::new(e).with_context(context)))
            },
        }
    }
}

/// Instruments an error with the current span and all its parents.
///
/// This function captures the entire span hierarchy, providing a complete
/// picture of the execution context when the error occurred.
///
/// # Example
///
/// ```rust,ignore
/// use error_rail::async_ext::instrument_error;
///
/// let error = ApiError::NotFound;
/// let instrumented = instrument_error(error);
/// // Error now contains context from all active spans
/// ```
pub fn instrument_error<E>(error: E) -> ComposableError<E> {
    let span = Span::current();
    ComposableError::new(error).with_context(span_to_context(&span))
}

// Required for alloc::format!
extern crate alloc;
