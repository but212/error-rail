//! Each submodule re-exports its public surface from here, so consumers can
//! simply depend on `error_rail::*` or pick focused pieces as needed.
//!
//! # Examples
//!
//! ## Basic Error with Context
//!
//! ```
//! use error_rail::{ComposableError, ErrorContext, group};
//!
//! let err = ComposableError::new("database connection failed")
//!     .with_context(group!(
//!         tag("db"),
//!         metadata("retry_count", "3")
//!     ))
//!     .set_code(500);
//!
//! assert!(err.to_string().contains("database connection failed"));
//! assert_eq!(err.error_code(), Some(500));
//! ```
//!
//! ## Validation Accumulation
//!
//! ```
//! use error_rail::validation::Validation;
//!
//! let v1: Validation<&str, i32> = Validation::Valid(10);
//! let v2: Validation<&str, i32> = Validation::invalid("error");
//! let combined: Validation<&str, Vec<i32>> = vec![v1, v2].into_iter().collect();
//!
//! assert!(combined.is_invalid());
//! ```
//!
//! ## Error Pipeline
//!
//! ```
//! use error_rail::{ErrorPipeline, context};
//!
//! let result = ErrorPipeline::<i32, &str>::new(Err("failed"))
//!     .with_context(context!("operation: load_config"))
//!     .with_context(context!("user_id: 42"))
//!     .finish_boxed();
//!
//! if let Err(err) = result {
//!     let chain = err.error_chain();
//!     assert!(chain.contains("user_id: 42"));
//!     assert!(chain.contains("operation: load_config"));
//!     assert!(chain.contains("failed"));
//! }
//! ```
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

/// Error context management and accumulation
pub mod context;
/// Error type conversions between Result, Validation, and ComposableError
pub mod convert;
/// Error handling macros for context creation
pub mod macros;
/// Convenience re-exports for quick starts
pub mod prelude;
/// Core traits for error handling and composition
pub mod traits;
/// ComposableError and error context structures
pub mod types;
/// Validation type and associated traits for error accumulation
pub mod validation;

/// Advanced API level for library authors
pub mod advanced;
/// Intermediate API level for service developers
pub mod intermediate;

/// Async extensions for error handling (requires `async` feature)
#[cfg(feature = "async")]
pub mod async_ext;

/// Async prelude - all async utilities in one import (requires `async` feature)
#[cfg(feature = "async")]
pub mod prelude_async;

/// Tower integration - Layer and Service implementations (requires `tower` feature)
#[cfg(feature = "tower")]
pub mod tower;

// Re-export common types that might be needed at root,
// but encourage using prelude/intermediate/advanced modules.
pub use context::*;
pub use convert::*;
pub use traits::*;
pub use types::{
    error_formatter::ErrorFormatConfig, BoxedComposableResult, BoxedResult, ComposableError,
    ComposableResult, ErrorContext, ErrorPipeline, ErrorVec, GroupContext, LazyContext,
    LazyGroupContext,
};
pub use validation::*;
