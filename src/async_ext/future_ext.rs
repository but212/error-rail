//! Extension trait for `Future<Output = Result<T, E>>`.
//!
//! Provides `.ctx()` and `.with_ctx()` methods for futures, mirroring
//! the sync `ResultExt` trait.

use core::future::Future;

use crate::traits::IntoErrorContext;

use super::context_future::ContextFuture;

/// Extension trait for attaching error context to async Result-returning futures.
///
/// This trait mirrors the sync [`ResultExt`](crate::traits::ResultExt) trait,
/// providing the same `.ctx()` and `.with_ctx()` ergonomics for async code.
///
/// # Design Principles
///
/// - **Lazy evaluation**: Context is only evaluated when an error occurs
/// - **Zero-cost success path**: No allocation or context computation on success
/// - **Familiar syntax**: Same method names as sync counterparts
///
/// # Examples
///
/// ## Basic Usage
///
/// ```rust,no_run
/// use error_rail::prelude_async::*;
///
/// #[derive(Debug)]
/// struct User;
///
/// #[derive(Debug)]
/// struct ApiError;
///
/// async fn fetch_from_db(_id: u64) -> Result<User, ApiError> {
///     Err(ApiError)
/// }
///
/// async fn fetch_user(id: u64) -> BoxedResult<User, ApiError> {
///     fetch_from_db(id)
///         .ctx("fetching user")
///         .await
///         .map_err(Box::new)
/// }
/// ```
///
/// ## With Lazy Context
///
/// ```rust,no_run
/// use error_rail::prelude_async::*;
///
/// #[derive(Debug)]
/// struct Order;
///
/// #[derive(Debug)]
/// struct OrderError;
///
/// async fn validate_order(_order_id: u64) -> Result<Order, OrderError> {
///     Err(OrderError)
/// }
///
/// async fn process_order(order_id: u64) -> BoxedResult<Order, OrderError> {
///     validate_order(order_id)
///         .with_ctx(|| format!("validating order {}", order_id))
///         .await
///         .map_err(Box::new)
/// }
/// ```
pub trait FutureResultExt<T, E>: Future<Output = Result<T, E>> + Sized {
    /// Attaches a static context to the future's error.
    ///
    /// The context is converted to an error context only when the future
    /// resolves to an error, maintaining lazy evaluation.
    ///
    /// # Arguments
    ///
    /// * `context` - Any type implementing `IntoErrorContext`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use error_rail::prelude_async::*;
    ///
    /// async fn example() {
    ///     let result = async { Err::<(), _>("failed") }
    ///         .ctx("operation failed")
    ///         .await;
    ///     assert!(result.is_err());
    /// }
    /// ```
    fn ctx<C>(self, context: C) -> ContextFuture<Self, impl FnOnce() -> C>
    where
        C: IntoErrorContext,
    {
        self.with_ctx(move || context)
    }

    /// Attaches a lazily-evaluated context to the future's error.
    ///
    /// The closure is only called when the future resolves to an error,
    /// avoiding any computation on the success path.
    ///
    /// # Arguments
    ///
    /// * `f` - A closure that produces the error context
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use error_rail::prelude_async::*;
    ///
    /// #[derive(Debug)]
    /// struct User;
    ///
    /// #[derive(Debug)]
    /// struct ApiError;
    ///
    /// async fn fetch_from_db(_id: u64) -> Result<User, ApiError> {
    ///     Err(ApiError)
    /// }
    ///
    /// async fn fetch_user(id: u64) -> BoxedResult<User, ApiError> {
    ///     fetch_from_db(id)
    ///         .with_ctx(|| format!("fetching user {}", id))
    ///         .await
    ///         .map_err(Box::new)
    /// }
    /// ```
    fn with_ctx<F, C>(self, f: F) -> ContextFuture<Self, F>
    where
        F: FnOnce() -> C,
        C: IntoErrorContext;
}

impl<Fut, T, E> FutureResultExt<T, E> for Fut
where
    Fut: Future<Output = Result<T, E>>,
{
    #[inline]
    fn with_ctx<F, C>(self, f: F) -> ContextFuture<Self, F>
    where
        F: FnOnce() -> C,
        C: IntoErrorContext,
    {
        ContextFuture::new(self, f)
    }
}
