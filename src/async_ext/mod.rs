//! Async extensions for error-rail.
//!
//! This module provides async-aware error handling utilities that maintain
//! the same lazy evaluation philosophy as the sync counterparts.
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
//! ```ignore
//! use error_rail::prelude_async::*;
//!
//! async fn fetch_user(id: u64) -> BoxedResult<User, ApiError> {
//!     fetch_from_db(id)
//!         .ctx("fetching user from database")
//!         .await
//! }
//! ```

mod context_future;
mod future_ext;
mod pipeline;

#[cfg(feature = "async-retry")]
mod retry;

#[cfg(feature = "async-validation")]
mod validation;

#[cfg(feature = "async-tokio")]
mod tokio_ext;

#[cfg(feature = "tracing")]
mod tracing_ext;

pub use context_future::ContextFuture;
pub use future_ext::FutureResultExt;
pub use pipeline::AsyncErrorPipeline;

#[cfg(feature = "async-retry")]
pub use retry::{
    retry_with_metadata, retry_with_policy, ExponentialBackoff, FixedDelay, RetryPolicy,
    RetryResult,
};

#[cfg(feature = "async-validation")]
pub use validation::{validate_all_async, validate_seq_async};

#[cfg(feature = "async-tokio")]
pub use tokio_ext::{
    retry_transient, retry_transient_n, try_with_timeout, TimeoutError, TimeoutResult,
};

#[cfg(feature = "tracing")]
pub use tracing_ext::{instrument_error, FutureSpanExt, ResultSpanExt, SpanContextFuture};
