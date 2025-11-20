//! Trait for converting types into structured error context.
//!
//! This trait provides a unified interface for types that can be converted into
//! [`ErrorContext`], enabling flexible context attachment in error handling pipelines.
//!
//! # Implementations
//!
//! The trait is implemented for common types:
//! - `String` - Converts to `ErrorContext::Message`
//! - `&str` - Converts to `ErrorContext::Message`
//! - `ErrorContext` - Identity conversion (no-op)
//!
//! # Examples
//!
//! ```
//! use error_rail::{traits::IntoErrorContext, ErrorContext};
//!
//! let ctx1 = "simple message".into_error_context();
//! let ctx2 = String::from("owned message").into_error_context();
//! let ctx3 = ErrorContext::tag("network").into_error_context();
//! ```
use crate::types::error_context::ErrorContext;

/// Converts a type into an [`ErrorContext`] for error annotation.
///
/// This trait is used throughout the error handling pipeline to accept
/// flexible context types when building composable errors.
pub trait IntoErrorContext {
    /// Converts `self` into an [`ErrorContext`].
    fn into_error_context(self) -> ErrorContext;
}

impl IntoErrorContext for String {
    /// Converts an owned `String` into a message context.
    #[inline]
    fn into_error_context(self) -> ErrorContext {
        ErrorContext::new(self)
    }
}

impl IntoErrorContext for &str {
    /// Converts a string slice into a message context.
    #[inline]
    fn into_error_context(self) -> ErrorContext {
        ErrorContext::new(self)
    }
}

impl IntoErrorContext for ErrorContext {
    /// Identity conversion for `ErrorContext` (no-op).
    #[inline]
    fn into_error_context(self) -> ErrorContext {
        self
    }
}
