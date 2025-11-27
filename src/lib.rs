//! Composable, metadata-friendly error handling utilities.
//!
//! `error-rail` focuses on three pillars:
//! 1. **Structured context** – enrich any error with layered metadata
//!    using [`context!`] helpers and macros such as [`location!`] and [`tag!`].
//! 2. **Composable collectors** – aggregate successes/failures with the
//!    [`validation`] module and convert between `Result`, `Validation`, and
//!    `ComposableError` via [`convert`].
//! 3. **Ergonomic traits/macros** – glue traits in [`traits`] and shortcuts
//!    in [`macros`] keep the API light-weight.
//!
//! Each submodule re-exports its public surface from here, so consumers can
//! simply depend on `error_rail::*` or pick focused pieces as needed.
//!
//! # Examples
//!
//! ## Basic Error with Context
//!
//! ```
//! use error_rail::{ComposableError, ErrorContext, context, location, tag};
//!
//! let err = ComposableError::new("database connection failed")
//!     .with_context(tag!("db"))
//!     .with_context(location!())
//!     .set_code(500);
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
//!     eprintln!("{}", err.error_chain());
//!     // Output: user_id: 42 -> operation: load_config -> failed
//! }
//! ```
#![no_std]

extern crate alloc;

/// Error context management and accumulation
pub mod context;
/// Error type conversions between Result, Validation, and ComposableError
pub mod convert;
/// Error handling macros for context creation
pub mod macros;
/// Core traits for error handling and composition
pub mod traits;
/// ComposableError and error context structures
pub mod types;
/// Validation type and associated traits for error accumulation
pub mod validation;

pub use context::*;
pub use convert::*;
pub use traits::*;
pub use types::*;
pub use validation::*;
