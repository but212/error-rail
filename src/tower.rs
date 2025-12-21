//! Tower integration for error-rail.
//!
//! This module provides Tower `Layer` and `Service` implementations
//! that automatically attach error context to service errors.
//!
//! # Feature Flag
//!
//! Requires the `tower` feature:
//!
//! ```toml
//! [dependencies]
//! error-rail = { version = "0.8", features = ["tower"] }
//! ```
//!
//! # Example
//!
//! ```rust,ignore
//! use error_rail::tower::ErrorRailLayer;
//! use tower::ServiceBuilder;
//!
//! let service = ServiceBuilder::new()
//!     .layer(ErrorRailLayer::new("api-gateway"))
//!     .service(my_service);
//! ```

use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};

use futures_core::future::FusedFuture;
use pin_project_lite::pin_project;
use tower::{Layer, Service};

use crate::traits::IntoErrorContext;
use crate::types::ComposableError;

/// A Tower [`Layer`] that wraps service errors in [`ComposableError`] with context.
///
/// This layer intercepts errors from the wrapped service and adds the configured
/// context, making it easy to add consistent error context at service boundaries.
///
/// # Type Parameters
///
/// * `C` - The context type, must implement [`IntoErrorContext`] and [`Clone`]
///
/// # Example
///
/// ```rust,ignore
/// use error_rail::tower::ErrorRailLayer;
/// use tower::ServiceBuilder;
///
/// // Add static context
/// let layer = ErrorRailLayer::new("user-service");
///
/// // Or use structured context
/// let layer = ErrorRailLayer::new(error_rail::group!(
///     tag("service"),
///     metadata("version", "1.0")
/// ));
/// ```
#[derive(Clone, Debug)]
pub struct ErrorRailLayer<C> {
    context: C,
}

impl<C> ErrorRailLayer<C> {
    /// Creates a new `ErrorRailLayer` with the given context.
    ///
    /// The context will be attached to all errors from the wrapped service.
    #[inline]
    pub const fn new(context: C) -> Self {
        Self { context }
    }

    /// Returns a reference to the context.
    #[inline]
    pub const fn context(&self) -> &C {
        &self.context
    }
}

impl<S, C: Clone> Layer<S> for ErrorRailLayer<C> {
    type Service = ErrorRailService<S, C>;

    #[inline]
    fn layer(&self, inner: S) -> Self::Service {
        ErrorRailService { inner, context: self.context.clone() }
    }
}

/// A Tower [`Service`] that wraps errors in [`ComposableError`] with context.
///
/// This is created by [`ErrorRailLayer`] and wraps an inner service,
/// adding error context to any errors it produces.
#[derive(Clone, Debug)]
pub struct ErrorRailService<S, C> {
    inner: S,
    context: C,
}

impl<S, C> ErrorRailService<S, C> {
    /// Creates a new `ErrorRailService` wrapping the given service.
    #[inline]
    pub const fn new(inner: S, context: C) -> Self {
        Self { inner, context }
    }

    /// Returns a reference to the inner service.
    #[inline]
    pub const fn inner(&self) -> &S {
        &self.inner
    }

    /// Returns a mutable reference to the inner service.
    #[inline]
    pub fn inner_mut(&mut self) -> &mut S {
        &mut self.inner
    }

    /// Consumes the wrapper and returns the inner service.
    #[inline]
    pub fn into_inner(self) -> S {
        self.inner
    }

    /// Returns a reference to the context.
    #[inline]
    pub const fn context(&self) -> &C {
        &self.context
    }
}

impl<S, C, Request> Service<Request> for ErrorRailService<S, C>
where
    S: Service<Request>,
    S::Error: core::fmt::Debug,
    C: IntoErrorContext + Clone,
{
    type Response = S::Response;
    type Error = ComposableError<S::Error>;
    type Future = ErrorRailFuture<S::Future, C>;

    #[inline]
    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner
            .poll_ready(cx)
            .map_err(|e| ComposableError::new(e).with_context(self.context.clone()))
    }

    #[inline]
    fn call(&mut self, request: Request) -> Self::Future {
        ErrorRailFuture::new(self.inner.call(request), self.context.clone())
    }
}

pin_project! {
    /// Future returned by [`ErrorRailService`].
    ///
    /// Wraps the inner service's future and adds context on error.
    #[must_use = "futures do nothing unless polled"]
    pub struct ErrorRailFuture<F, C> {
        #[pin]
        inner: F,
        context: Option<C>,
    }
}

impl<F, C> ErrorRailFuture<F, C> {
    /// Creates a new `ErrorRailFuture` with the given inner future and context.
    #[inline]
    fn new(inner: F, context: C) -> Self {
        Self { inner, context: Some(context) }
    }
}

impl<F, T, E, C> Future for ErrorRailFuture<F, C>
where
    F: Future<Output = Result<T, E>>,
    C: IntoErrorContext,
{
    type Output = Result<T, ComposableError<E>>;

    #[inline]
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();

        match this.inner.poll(cx) {
            Poll::Ready(Ok(response)) => Poll::Ready(Ok(response)),
            Poll::Ready(Err(error)) => {
                // SAFETY: context is always Some until first Ready result
                let context = this.context.take().expect("polled after completion");
                Poll::Ready(Err(ComposableError::new(error).with_context(context)))
            },
            Poll::Pending => Poll::Pending,
        }
    }
}

impl<F, T, E, C> FusedFuture for ErrorRailFuture<F, C>
where
    F: FusedFuture<Output = Result<T, E>>,
    C: IntoErrorContext,
{
    #[inline]
    fn is_terminated(&self) -> bool {
        self.context.is_none() || self.inner.is_terminated()
    }
}

/// Extension trait for easily wrapping services with error context.
pub trait ServiceErrorExt<Request>: Service<Request> + Sized {
    /// Wraps this service to add error context to all errors.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use error_rail::tower::ServiceErrorExt;
    ///
    /// let wrapped = my_service.with_error_context("database-layer");
    /// ```
    fn with_error_context<C>(self, context: C) -> ErrorRailService<Self, C>
    where
        C: IntoErrorContext + Clone,
    {
        ErrorRailService::new(self, context)
    }
}

impl<S, Request> ServiceErrorExt<Request> for S where S: Service<Request> {}
