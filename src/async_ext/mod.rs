//! Async extensions for error-rail.
//!
//! This module provides async-aware error handling utilities that maintain
//! the same lazy attachment philosophy as the sync counterparts.
//!
//! Note: lazy *string formatting* is provided by `context!` / `.with_ctx(...)`.
//! Passing an already-formatted `String` (e.g. `format!(...)`) is eager.
//!
//! # Feature Flag
//!
//! Requires the `async` feature to be enabled:
//!
//! ```toml
//! [dependencies]
//! error-rail = { version = "0.8", features = ["async"] }
//! ```
//!
//! # Examples
//!
//! ```rust,no_run
//! use error_rail::prelude_async::*;
//!
//! #[derive(Debug)]
//! struct User;
//!
//! #[derive(Debug)]
//! struct ApiError;
//!
//! async fn fetch_from_db(_id: u64) -> Result<User, ApiError> {
//!     Err(ApiError)
//! }
//!
//! async fn fetch_user(id: u64) -> BoxedResult<User, ApiError> {
//!     fetch_from_db(id)
//!         .ctx("fetching user from database")
//!         .await
//!         .map_err(Box::new)
//! }
//! ```

mod context_future;
mod future_ext;
mod pipeline;

#[cfg(feature = "async")]
mod retry;

#[cfg(feature = "async")]
mod validation;

#[cfg(feature = "tokio")]
mod tokio_ext;

#[cfg(feature = "tracing")]
mod tracing_ext;

pub use context_future::ContextFuture;
pub use future_ext::FutureResultExt;
pub use pipeline::AsyncErrorPipeline;

#[cfg(feature = "async")]
pub use retry::{
    retry_with_metadata, retry_with_policy, ExponentialBackoff, FixedDelay, RetryPolicy,
    RetryResult,
};

#[cfg(feature = "async")]
pub use validation::{validate_all_async, validate_seq_async};

#[cfg(feature = "tokio")]
pub use tokio_ext::{
    retry_transient, retry_transient_n, retry_transient_unboxed, try_with_timeout, TimeoutError,
    TimeoutResult,
};

#[cfg(feature = "tracing")]
pub use tracing_ext::{instrument_error, FutureSpanExt, ResultSpanExt, SpanContextFuture};
