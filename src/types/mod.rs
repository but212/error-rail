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
//!     .with_context(ErrorContext::location(file!(), line!()))
//!     .set_code(500);
//!
//! println!("{}", err.error_chain());
//! // Output: [db] -> main.rs:42 -> database connection failed (code: 500)
//! ```
use smallvec::SmallVec;

pub mod composable_error;
pub mod error_context;
pub mod error_pipeline;
pub mod lazy_context;

pub use composable_error::*;
pub use error_context::*;
pub use error_pipeline::*;
pub use lazy_context::*;

/// SmallVec-backed collection used for accumulating contexts/errors.
///
/// Uses inline storage for up to 1 elements to avoid heap allocations
/// in common cases where only a few contexts are attached.
pub type ErrorVec<E> = SmallVec<[E; 1]>;

/// Result alias that wraps failures in [`ComposableError`].
///
/// # Type Parameters
///
/// * `T` - The success value type
/// * `E` - The core error type
pub type ComposableResult<T, E> = Result<T, ComposableError<E>>;

/// Convenience alias for `ComposableError`.
///
/// Kept for backward compatibility.
pub type SimpleComposableError<E> = ComposableError<E>;

/// Convenience alias for `ComposableError`.
///
/// Kept for backward compatibility. Note that string codes are no longer supported via generic C.
/// Use `ErrorContext::tag` instead.
pub type TaggedComposableError<E> = ComposableError<E>;

/// Boxed [`ComposableError`] for reduced stack size.
///
/// # Type Parameters
///
/// * `E` - The core error type
pub type BoxedComposableError<E> = Box<ComposableError<E>>;

/// Result alias with boxed [`ComposableError`] for reduced stack size.
///
/// # Type Parameters
///
/// * `T` - The success value type
/// * `E` - The core error type
pub type BoxedComposableResult<T, E> = Result<T, BoxedComposableError<E>>;
