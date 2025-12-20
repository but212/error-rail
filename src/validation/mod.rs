//! Validation types and utilities for accumulating errors.
//!
//! This module provides the [`Validation`] type, which can accumulate multiple
//! errors while preserving success values. It's particularly useful for validating
//! complex data structures where you want to collect all validation errors at once
//! rather than failing on the first error.
//!
//! # Key Components
//!
//! - [`Validation`] - Core type that represents either a valid value or accumulated errors
//! - Iterator adapters for traversing errors
//! - Trait implementations for composing validations
//!
//! # Examples
//!
//! ```
//! use error_rail::validation::Validation;
//!
//! let valid: Validation<String, i32> = Validation::Valid(42);
//! assert!(valid.is_valid());
//!
//! let invalid: Validation<&str, i32> = Validation::invalid_many(["err1", "err2"]);
//! assert_eq!(invalid.iter_errors().count(), 2);
//! ```
pub mod core;
pub mod iter;
pub mod prelude;
pub mod traits;

pub use self::core::*;
pub use self::iter::*;
// Note: traits module provides impl blocks for WithError and ErrorCategory
// which are automatically available when this module is compiled
