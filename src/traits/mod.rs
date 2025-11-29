//! Core traits for error handling and composition.
//!
//! This module defines the fundamental traits that enable error-rail's composable
//! error handling patterns:
//!
//! - [`ErrorCategory`]: Categorical abstraction for lifting values and handling errors
//! - [`ErrorOps`]: Operations for error recovery and bidirectional mapping
//! - [`IntoErrorContext`]: Conversion trait for creating structured error contexts
//! - [`WithError`]: Abstraction for types that carry remappable error variants
//!
//! # Examples
//!
//! ```
//! use error_rail::traits::{ErrorCategory, IntoErrorContext};
//! use error_rail::{ComposableError, ErrorContext};
//!
//! // Using ErrorCategory to lift values
//! let success: Result<i32, String> = <Result<(), String>>::lift(42);
//! assert_eq!(success, Ok(42));
//!
//! // Using IntoErrorContext for structured contexts
//! let err = ComposableError::<&str>::new("failed")
//!     .with_context("operation context");
//! assert_eq!(err.context().len(), 1);
//! ```

pub mod error_category;
pub mod error_ops;
pub mod into_error_context;
pub mod result_ext;
pub mod transient;
pub mod with_error;

pub use error_category::ErrorCategory;
pub use error_ops::ErrorOps;
pub use into_error_context::IntoErrorContext;
pub use result_ext::{BoxedResultExt, ResultExt};
pub use transient::{TransientError, TransientErrorExt};
pub use with_error::WithError;
