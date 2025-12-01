//! Async prelude - all async utilities in one import.
//!
//! This module provides all the async-related items needed for async error handling.
//! It re-exports everything from the sync [`prelude`](crate::prelude) plus async-specific items.
//!
//! # Usage
//!
//! ```ignore
//! use error_rail::prelude_async::*;
//!
//! async fn fetch_user(id: u64) -> BoxedResult<User, ApiError> {
//!     fetch_from_db(id)
//!         .ctx("fetching user from database")
//!         .await
//!         .map_err(Box::new)
//! }
//! ```
//!
//! # What's Included
//!
//! ## From Sync Prelude
//!
//! - **Macros**: [`context!`], [`group!`], [`rail!`], [`rail_unboxed!`]
//! - **Types**: [`ComposableError`], [`ErrorContext`], [`ErrorPipeline`]
//! - **Traits**: [`ResultExt`], [`BoxedResultExt`], [`IntoErrorContext`]
//! - **Type Alias**: [`BoxedResult`]
//!
//! ## Async-Specific
//!
//! - **Traits**: [`FutureResultExt`](crate::async_ext::FutureResultExt) - `.ctx()` and `.with_ctx()` for futures
//! - **Types**: [`AsyncErrorPipeline`](crate::async_ext::AsyncErrorPipeline), [`ContextFuture`](crate::async_ext::ContextFuture)
//! - **Macros**: [`rail_async!`], [`ctx_async!`]
//!
//! ## Async Retry (requires `async-retry` feature)
//!
//! - **Traits**: [`RetryPolicy`](crate::async_ext::RetryPolicy)
//! - **Types**: [`ExponentialBackoff`](crate::async_ext::ExponentialBackoff), [`FixedDelay`](crate::async_ext::FixedDelay)
//! - **Functions**: [`retry_with_policy`](crate::async_ext::retry_with_policy)
//!
//! ## Async Validation (requires `async-validation` feature)
//!
//! - **Functions**: [`validate_all_async`](crate::async_ext::validate_all_async), [`validate_seq_async`](crate::async_ext::validate_seq_async)
//!
//! ## Tokio Integration (requires `async-tokio` feature)
//!
//! - **Functions**: [`retry_transient`](crate::async_ext::retry_transient), [`retry_transient_n`](crate::async_ext::retry_transient_n), [`try_with_timeout`](crate::async_ext::try_with_timeout)
//! - **Types**: [`TimeoutResult`](crate::async_ext::TimeoutResult), [`TimeoutError`](crate::async_ext::TimeoutError)
//!
//! ## Tracing Integration (requires `tracing` feature)
//!
//! - **Traits**: [`FutureSpanExt`](crate::async_ext::FutureSpanExt), [`ResultSpanExt`](crate::async_ext::ResultSpanExt)
//! - **Functions**: [`instrument_error`](crate::async_ext::instrument_error)

// Re-export everything from sync prelude
pub use crate::prelude::*;

// Async-specific exports
#[cfg(feature = "async")]
pub use crate::async_ext::{AsyncErrorPipeline, ContextFuture, FutureResultExt};

#[cfg(feature = "async")]
pub use crate::{ctx_async, rail_async};

// Async retry exports
#[cfg(feature = "async-retry")]
pub use crate::async_ext::{
    retry_with_metadata, retry_with_policy, ExponentialBackoff, FixedDelay, RetryPolicy,
    RetryResult,
};

// Async validation exports
#[cfg(feature = "async-validation")]
pub use crate::async_ext::{validate_all_async, validate_seq_async};

// Tokio integration exports
#[cfg(feature = "async-tokio")]
pub use crate::async_ext::{
    retry_transient, retry_transient_n, try_with_timeout, TimeoutError, TimeoutResult,
};

// Tracing integration exports
#[cfg(feature = "tracing")]
pub use crate::async_ext::{instrument_error, FutureSpanExt, ResultSpanExt, SpanContextFuture};
