//! Async error pipeline for chainable error handling.
//!
//! Provides `AsyncErrorPipeline`, the async counterpart to [`ErrorPipeline`](crate::ErrorPipeline).

use core::future::Future;

use crate::traits::IntoErrorContext;
use crate::types::alloc_type::Box;
use crate::types::ComposableError;

use super::future_ext::FutureResultExt;

/// Async error pipeline for chainable error handling.
///
/// This is the async counterpart to [`ErrorPipeline`](crate::ErrorPipeline),
/// providing fluent, chainable error context accumulation for async operations.
///
/// # Examples
///
/// ## Basic Usage
///
/// ```ignore
/// use error_rail::prelude_async::*;
///
/// async fn example() -> BoxedResult<Data, ApiError> {
///     AsyncErrorPipeline::new(fetch_data(id))
///         .with_context("fetching data")
///         .finish_boxed()
///         .await
/// }
/// ```
///
/// ## With Multiple Contexts
///
/// ```ignore
/// use error_rail::prelude_async::*;
///
/// async fn process_order(order_id: u64) -> BoxedResult<Order, OrderError> {
///     AsyncErrorPipeline::new(load_order(order_id))
///         .with_context("loading order")
///         .with_context(format!("order_id: {}", order_id))
///         .finish_boxed()
///         .await
/// }
/// ```
pub struct AsyncErrorPipeline<Fut> {
    future: Fut,
}

impl<Fut> AsyncErrorPipeline<Fut> {
    /// Creates a new async error pipeline from a future.
    ///
    /// # Arguments
    ///
    /// * `future` - A future that returns a `Result<T, E>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use error_rail::async_ext::AsyncErrorPipeline;
    ///
    /// let pipeline = AsyncErrorPipeline::new(async { Ok::<_, &str>(42) });
    /// ```
    #[inline]
    pub fn new(future: Fut) -> Self {
        Self { future }
    }
}

impl<Fut, T, E> AsyncErrorPipeline<Fut>
where
    Fut: Future<Output = Result<T, E>>,
{
    /// Adds a context that will be attached to any error.
    ///
    /// The context is only evaluated when an error occurs (lazy evaluation).
    ///
    /// # Arguments
    ///
    /// * `context` - Any type implementing `IntoErrorContext`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use error_rail::async_ext::AsyncErrorPipeline;
    ///
    /// let pipeline = AsyncErrorPipeline::new(async { Err::<(), _>("error") })
    ///     .with_context("operation context");
    /// ```
    #[inline]
    pub fn with_context<C>(
        self,
        context: C,
    ) -> AsyncErrorPipeline<impl Future<Output = Result<T, ComposableError<E>>>>
    where
        C: IntoErrorContext,
    {
        AsyncErrorPipeline {
            future: self.future.ctx(context),
        }
    }

    /// Adds a lazily-evaluated context using a closure.
    ///
    /// The closure is only called when an error occurs, avoiding
    /// any computation on the success path.
    ///
    /// # Arguments
    ///
    /// * `f` - A closure that produces the error context
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use error_rail::async_ext::AsyncErrorPipeline;
    ///
    /// let id = 42u64;
    /// let pipeline = AsyncErrorPipeline::new(fetch_user(id))
    ///     .with_context_fn(|| format!("user_id: {}", id));
    /// ```
    #[inline]
    pub fn with_context_fn<F, C>(
        self,
        f: F,
    ) -> AsyncErrorPipeline<impl Future<Output = Result<T, ComposableError<E>>>>
    where
        F: FnOnce() -> C,
        C: IntoErrorContext,
    {
        AsyncErrorPipeline {
            future: self.future.with_ctx(f),
        }
    }

    /// Completes the pipeline and returns the inner future.
    ///
    /// This method consumes the pipeline and returns a future that
    /// produces the original `Result<T, E>`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use error_rail::async_ext::AsyncErrorPipeline;
    ///
    /// async fn example() -> Result<i32, &'static str> {
    ///     AsyncErrorPipeline::new(async { Ok(42) })
    ///         .finish()
    ///         .await
    /// }
    /// ```
    #[inline]
    pub fn finish(self) -> Fut {
        self.future
    }
}

impl<Fut, T, E> AsyncErrorPipeline<Fut>
where
    Fut: Future<Output = Result<T, ComposableError<E>>>,
{
    /// Completes the pipeline and returns a boxed error result.
    ///
    /// This is the recommended way to finish a pipeline when returning
    /// from a function, as it provides minimal stack footprint.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use error_rail::prelude_async::*;
    ///
    /// async fn example() -> BoxedResult<i32, &'static str> {
    ///     AsyncErrorPipeline::new(async { Err("error") })
    ///         .with_context("operation failed")
    ///         .finish_boxed()
    ///         .await
    /// }
    /// ```
    #[inline]
    pub async fn finish_boxed(self) -> Result<T, Box<ComposableError<E>>> {
        self.future.await.map_err(Box::new)
    }

    /// Maps the error type using a transformation function.
    ///
    /// # Arguments
    ///
    /// * `f` - A function that transforms `ComposableError<E>` to `ComposableError<E2>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use error_rail::async_ext::AsyncErrorPipeline;
    ///
    /// let pipeline = AsyncErrorPipeline::new(async { Err::<(), _>("error") })
    ///     .with_context("context")
    ///     .map_err(|e| e.map_core(|_| "new error"));
    /// ```
    #[inline]
    pub fn map_err<F, E2>(
        self,
        f: F,
    ) -> AsyncErrorPipeline<impl Future<Output = Result<T, ComposableError<E2>>>>
    where
        F: FnOnce(ComposableError<E>) -> ComposableError<E2>,
    {
        AsyncErrorPipeline {
            future: async move { self.future.await.map_err(f) },
        }
    }
}
