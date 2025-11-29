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
//!
//! assert_eq!(ctx1.message(), "simple message");
//! assert_eq!(ctx2.message(), "owned message");
//! assert!(ctx3.message().contains("[network]"));
//! ```
use crate::types::alloc_type::{Cow, String};
use crate::types::error_context::ErrorContext;

/// Converts a type into an [`ErrorContext`] for error annotation.
///
/// This trait is used throughout the error handling pipeline to accept
/// flexible context types when building composable errors.
///
/// # Implementing for Custom Types
///
/// If you need to use a custom type as error context, you have two options:
///
/// 1. Use the [`impl_error_context!`](crate::impl_error_context) macro for types implementing `Display`:
///    ```ignore
///    impl_error_context!(MyCustomError);
///    ```
///
/// 2. Implement the trait manually:
///    ```
///    use error_rail::{traits::IntoErrorContext, ErrorContext};
///
///    struct MyContext { user_id: u64 }
///
///    impl IntoErrorContext for MyContext {
///        fn into_error_context(self) -> ErrorContext {
///            ErrorContext::metadata("user_id", self.user_id.to_string())
///        }
///    }
///    ```
#[diagnostic::on_unimplemented(
    message = "`{Self}` cannot be used as error context",
    label = "this type does not implement `IntoErrorContext`",
    note = "implement `IntoErrorContext` manually or use `impl_error_context!({Self})` macro",
    note = "see: https://docs.rs/error-rail/latest/error_rail/macro.impl_error_context.html"
)]
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

impl IntoErrorContext for &'static str {
    /// Converts a static string slice into a message context.
    #[inline]
    fn into_error_context(self) -> ErrorContext {
        ErrorContext::new(self)
    }
}

impl IntoErrorContext for Cow<'static, str> {
    /// Converts a Cow string into a message context.
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
