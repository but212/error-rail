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

use crate::types::alloc_type::Vec;
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
                Poll::Ready(Err(Box::new(ComposableError::new(error).with_context(context))))
            },
            Poll::Pending => Poll::Pending,
        }
    }
}

/// Converts a tracing span to an error context.
///
/// Extracts the span's metadata (name, target, level, fields) and creates
/// a structured error context. This provides rich observability when errors
/// occur within instrumented code.
///
/// # Fields Captured
///
/// - **Tag**: The span name (e.g., `fetch_user`)
/// - **Metadata `target`**: The module path where the span was created
/// - **Metadata `level`**: The span's log level (TRACE, DEBUG, INFO, WARN, ERROR)
/// - **Metadata `fields`**: Names of fields defined on the span (values require subscriber)
fn span_to_context(span: &Span) -> ErrorContext {
    let Some(meta) = span.metadata() else {
        return ErrorContext::new("in unknown span");
    };

    let mut builder = ErrorContext::builder().tag(meta.name());

    // Add target module path
    builder = builder.metadata("target", meta.target());

    // Add log level
    builder = builder.metadata("level", meta.level().as_str());

    // Add field names (values not accessible without subscriber integration)
    let field_names: Vec<&str> = meta.fields().iter().map(|f| f.name()).collect();
    if !field_names.is_empty() {
        builder = builder.metadata("fields", field_names.join(", "));
    }

    builder.build()
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
