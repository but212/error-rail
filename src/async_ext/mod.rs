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

pub use context_future::ContextFuture;
pub use future_ext::FutureResultExt;
pub use pipeline::AsyncErrorPipeline;
