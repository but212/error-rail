//! Deferred context generation for performance-critical paths.
//!
//! This module provides [`LazyContext`], a wrapper that delays the creation
//! of error context strings until they are actually needed. This is useful
//! when constructing context messages is expensive (e.g., formatting large
//! data structures) and you want to avoid the overhead unless an error occurs.
//!
//! # Examples
//!
//! ```
//! use error_rail::{LazyContext, ErrorPipeline};
//!
//! fn expensive_debug_info() -> String {
//!     // Simulate expensive operation
//!     format!("computed value: {}", 42)
//! }
//!
//! let result: Result<(), &str> = Err("failed");
//! let pipeline = ErrorPipeline::new(result)
//!     .with_context(LazyContext::new(expensive_debug_info))
//!     .finish_boxed();
//! ```
use crate::{
    traits::IntoErrorContext, types::alloc_type::String, types::error_context::ErrorContext,
};

/// A lazily-evaluated error context that defers string generation until needed.
///
/// Wraps a closure that produces a `String` only when the error actually occurs,
/// avoiding unnecessary allocations and computations on the success path.
///
/// # Type Parameters
///
/// * `F` - A closure type that implements `FnOnce() -> String`.
///
/// # Examples
///
/// ```
/// use error_rail::LazyContext;
///
/// let lazy = LazyContext::new(|| format!("user_id: {}", 123));
/// // The closure is not called until `into_error_context` is invoked
/// ```
#[repr(transparent)]
pub struct LazyContext<F> {
    generator: F,
}

impl<F> LazyContext<F> {
    /// Creates a new `LazyContext` from a closure.
    ///
    /// The provided closure will be invoked only when the context is converted
    /// into an [`ErrorContext`], typically when an error is being processed.
    ///
    /// # Arguments
    ///
    /// * `generator` - A closure that returns a `String` when called.
    ///
    /// # Examples
    ///
    /// ```
    /// use error_rail::LazyContext;
    ///
    /// let ctx = LazyContext::new(|| "deferred message".to_string());
    /// ```
    #[inline]
    pub fn new(generator: F) -> Self {
        Self { generator }
    }
}

/// A lazily-evaluated grouped error context that defers GroupContext creation until needed.
///
/// Wraps a closure that produces an `ErrorContext` only when the error actually occurs,
/// avoiding unnecessary allocations and computations on the success path while
/// providing the benefits of grouped context display.
///
/// # Type Parameters
///
/// * `F` - A closure type that implements `FnOnce() -> ErrorContext`.
///
/// # Examples
///
/// ```rust
/// use error_rail::{LazyGroupContext, ErrorContext};
///
/// let lazy = LazyGroupContext::new(|| {
///     ErrorContext::builder()
///         .tag("database")
///         .metadata("host", "localhost")
///         .build()
/// });
/// // The closure is not called until `into_error_context` is invoked
/// ```
#[repr(transparent)]
pub struct LazyGroupContext<F> {
    generator: F,
}

impl<F> LazyGroupContext<F> {
    /// Creates a new `LazyGroupContext` from a closure.
    ///
    /// The provided closure will be invoked only when the context is converted
    /// into an [`ErrorContext`], typically when an error is being processed.
    ///
    /// # Arguments
    ///
    /// * `generator` - A closure that returns an `ErrorContext` when called.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use error_rail::{LazyGroupContext, ErrorContext};
    ///
    /// let ctx = LazyGroupContext::new(|| ErrorContext::tag("lazy"));
    /// ```
    #[inline]
    pub fn new(generator: F) -> Self {
        Self { generator }
    }
}

impl<F> IntoErrorContext for LazyGroupContext<F>
where
    F: FnOnce() -> ErrorContext,
{
    /// Evaluates the lazy closure and returns the resulting [`ErrorContext`].
    ///
    /// This is called automatically by the error pipeline when an error occurs.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use error_rail::{LazyGroupContext, traits::IntoErrorContext, ErrorContext};
    ///
    /// let lazy = LazyGroupContext::new(|| ErrorContext::tag("computed"));
    /// let ctx = lazy.into_error_context();
    /// ```
    #[inline]
    fn into_error_context(self) -> ErrorContext {
        (self.generator)()
    }
}

impl<F> IntoErrorContext for LazyContext<F>
where
    F: FnOnce() -> String,
{
    /// Evaluates the lazy closure and converts the result into an [`ErrorContext`].
    ///
    /// This is called automatically by the error pipeline when an error occurs.
    ///
    /// # Examples
    ///
    /// ```
    /// use error_rail::{LazyContext, traits::IntoErrorContext};
    ///
    /// let lazy = LazyContext::new(|| "computed".to_string());
    /// let ctx = lazy.into_error_context();
    /// assert_eq!(ctx.message(), "computed");
    /// ```
    #[inline]
    fn into_error_context(self) -> ErrorContext {
        ErrorContext::new((self.generator)())
    }
}
