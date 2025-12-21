//! Error types and utilities.
//!
//! This module provides a set of types and utilities for handling errors
//! in a structured and composable way.
//!
//! # Examples
//!
//! ```
//! use error_rail::{ComposableError, ErrorContext};
//!
//! let err = ComposableError::new("database connection failed")
//!     .with_context(ErrorContext::tag("db"))
//!     .set_code(500);
//!
//! assert!(err.to_string().contains("database connection failed"));
//! assert_eq!(err.error_code(), Some(500));
//! ```
use smallvec::SmallVec;

pub mod accumulator;
pub mod alloc_type;
pub mod composable_error;
pub mod error_context;
pub mod error_formatter;
pub mod error_pipeline;
pub mod lazy_context;
pub mod marked_error;
pub mod retry;
pub(crate) mod utils;

pub use alloc_type::*;
pub use composable_error::{ComposableError, FingerprintConfig};
pub use error_context::*;
pub use error_pipeline::*;
pub use lazy_context::*;
pub use marked_error::MarkedError;
pub use retry::RetryOps;

/// SmallVec-backed collection used for accumulating contexts/errors.
///
/// Uses inline storage for up to 2 elements to avoid heap allocations
/// in common cases where only a few contexts are attached.
pub type ErrorVec<E> = SmallVec<[E; 2]>;

/// Result alias that wraps failures in [`ComposableError`].
///
/// # Type Parameters
///
/// * `T` - The success value type
/// * `E` - The core error type
pub type ComposableResult<T, E> = Result<T, ComposableError<E>>;

/// Boxed [`ComposableError`] for reduced stack size.
///
/// # Type Parameters
///
/// * `E` - The core error type
pub type BoxedComposableError<E> = alloc_type::Box<ComposableError<E>>;

/// Result alias with boxed [`ComposableError`] for reduced stack size.
///
/// This is identical to [`crate::prelude::BoxedResult`] but with a more explicit name.
/// For new code, prefer using [`crate::prelude::BoxedResult`] for brevity.
///
/// # Type Parameters
///
/// * `T` - The success value type
/// * `E` - The core error type
///
/// # See Also
///
/// * [`crate::prelude::BoxedResult`] - Shorter alias (recommended)
pub type BoxedComposableResult<T, E> = Result<T, BoxedComposableError<E>>;
