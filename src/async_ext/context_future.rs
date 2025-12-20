//! Future wrappers for lazy context evaluation.
//!
//! This module provides `ContextFuture`, which wraps a `Future<Output = Result<T, E>>`
//! and attaches error context only when the future resolves to an error.

use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};

use futures_core::future::FusedFuture;

use pin_project_lite::pin_project;

use crate::traits::IntoErrorContext;
use crate::types::ComposableError;

pin_project! {
    /// A Future wrapper that attaches error context lazily.
    ///
    /// The context is only evaluated when the inner future resolves to an error,
    /// maintaining zero-cost on the success path.
    ///
    /// # Cancel Safety
    ///
    /// `ContextFuture` is cancel-safe if the inner future is cancel-safe.
    /// The `context_fn` is only called when `poll` returns `Poll::Ready(Err(_))`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use error_rail::prelude_async::*;
    ///
    /// async fn example() -> BoxedResult<i32, &'static str> {
    ///     async { Err("failed") }
    ///         .ctx("operation context")
    ///         .await
    ///         .map_err(Box::new)
    /// }
    /// ```
    #[must_use = "futures do nothing unless polled"]
    pub struct ContextFuture<Fut, F> {
        #[pin]
        future: Fut,
        context_fn: Option<F>,
    }
}

impl<Fut, F> ContextFuture<Fut, F> {
    /// Creates a new `ContextFuture` with the given future and context generator.
    #[inline]
    pub fn new(future: Fut, context_fn: F) -> Self {
        Self { future, context_fn: Some(context_fn) }
    }
}

impl<Fut, F, C, T, E> Future for ContextFuture<Fut, F>
where
    Fut: Future<Output = Result<T, E>>,
    F: FnOnce() -> C,
    C: IntoErrorContext,
{
    type Output = Result<T, ComposableError<E>>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();

        this.future.poll(cx).map(|res| {
            res.map_err(|err| {
                let context_fn = this
                    .context_fn
                    .take()
                    .expect("ContextFuture polled after completion; this is a bug");
                ComposableError::new(err).with_context(context_fn())
            })
        })
    }
}

impl<Fut, F, C, T, E> FusedFuture for ContextFuture<Fut, F>
where
    Fut: FusedFuture<Output = Result<T, E>>,
    F: FnOnce() -> C,
    C: IntoErrorContext,
{
    fn is_terminated(&self) -> bool {
        // Also check context_fn since it's taken on error completion
        self.context_fn.is_none() || self.future.is_terminated()
    }
}
