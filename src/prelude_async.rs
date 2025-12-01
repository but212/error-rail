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

// Re-export everything from sync prelude
pub use crate::prelude::*;

// Async-specific exports
#[cfg(feature = "async")]
pub use crate::async_ext::{AsyncErrorPipeline, ContextFuture, FutureResultExt};

#[cfg(feature = "async")]
pub use crate::{ctx_async, rail_async};
