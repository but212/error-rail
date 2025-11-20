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
//!
//! ## Choosing an error code type
//!
//! `ComposableError<E, C>` defaults `C` to `u32`, but you can pin any
//! type that implements your domain rules. Use the provided aliases as
//! starting points or create project-specific ones:
//!
//! ```
//! use error_rail::{
//!     ErrorContext,
//!     SimpleComposableError,
//!     TaggedComposableError,
//!     ComposableError,
//! };
//!
//! let http_error: SimpleComposableError<&str> = ComposableError::new("oops").set_code(500);
//! let tagged: TaggedComposableError<&str> =
//!     ComposableError::new("missing feature").set_code("feature_disabled");
//! ```
use smallvec::SmallVec;

pub mod composable_error;
pub mod error_context;
pub mod lazy_context;

pub use composable_error::*;
pub use error_context::*;
pub use lazy_context::*;

/// SmallVec-backed collection used for accumulating contexts/errors.
///
/// Uses inline storage for up to 8 elements to avoid heap allocations
/// in common cases where only a few contexts are attached.
pub type ErrorVec<E> = SmallVec<[E; 8]>;

/// Result alias that wraps failures in [`ComposableError`].
///
/// # Type Parameters
///
/// * `T` - The success value type
/// * `E` - The core error type
/// * `C` - The error code type (defaults to `u32`)
pub type ComposableResult<T, E, C = u32> = Result<T, ComposableError<E, C>>;

/// Convenience alias for `ComposableError` with the default numeric code type.
pub type SimpleComposableError<E> = ComposableError<E, u32>;

/// Convenience alias for `ComposableError` with static string codes.
pub type TaggedComposableError<E> = ComposableError<E, &'static str>;

/// Boxed [`ComposableError`] for reduced stack size.
///
/// # Type Parameters
///
/// * `E` - The core error type
/// * `C` - The error code type (defaults to `u32`)
pub type BoxedComposableError<E, C = u32> = Box<ComposableError<E, C>>;

/// Result alias with boxed [`ComposableError`] for reduced stack size.
///
/// # Type Parameters
///
/// * `T` - The success value type
/// * `E` - The core error type
/// * `C` - The error code type (defaults to `u32`)
pub type BoxedComposableResult<T, E, C = u32> = Result<T, BoxedComposableError<E, C>>;
