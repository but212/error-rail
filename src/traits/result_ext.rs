//! Extension trait for ergonomic context addition to `Result` types.
//!
//! This module provides [`ResultExt`], which adds convenient methods for
//! attaching context to errors without verbose `.map_err()` chains.
//!
//! # Examples
//!
//! ```
//! use error_rail::traits::ResultExt;
//!
//! fn load_config() -> Result<String, Box<error_rail::ComposableError<std::io::Error>>> {
//!     std::fs::read_to_string("config.toml")
//!         .ctx("loading configuration file")
//! }
//!
//! assert!(load_config().is_err());
//! ```

use crate::traits::IntoErrorContext;
use crate::types::alloc_type::{Box, String};
use crate::types::{ComposableError, LazyContext};

/// Extension trait for adding context to `Result` types ergonomically.
///
/// This trait provides a more natural API for error context compared to
/// manual `.map_err()` chains, reducing boilerplate while maintaining
/// full type safety.
///
/// # Performance
///
/// The [`ctx_with`](ResultExt::ctx_with) method uses lazy evaluation,
/// meaning the closure is only executed when an error actually occurs.
/// This provides the same 2.1x performance benefit as the `context!` macro.
///
/// # Examples
///
/// ## Basic Usage
///
/// ```
/// use error_rail::traits::ResultExt;
/// use error_rail::ComposableError;
///
/// fn read_file() -> Result<String, Box<ComposableError<std::io::Error>>> {
///     std::fs::read_to_string("data.txt")
///         .ctx("reading data file")
/// }
/// ```
///
/// ## Lazy Context (Recommended for Performance)
///
/// ```
/// use error_rail::traits::ResultExt;
/// use error_rail::ComposableError;
///
/// fn process(user_id: u64) -> Result<(), Box<ComposableError<&'static str>>> {
///     let result: Result<(), &str> = Err("not found");
///     result.ctx_with(|| format!("processing user {}", user_id))
/// }
/// ```
///
/// ## Chaining Multiple Contexts
///
/// ```
/// use error_rail::traits::ResultExt;
/// use error_rail::ComposableError;
///
/// fn complex_operation() -> Result<String, Box<ComposableError<std::io::Error>>> {
///     std::fs::read_to_string("config.toml")
///         .ctx("loading config")
///         .map(|s| s.to_uppercase())
/// }
/// ```
pub trait ResultExt<T, E> {
    /// Adds a static context message to the error.
    ///
    /// This method wraps the error in a [`ComposableError`] with the given
    /// context message, then boxes it for ergonomic return types.
    ///
    /// # Arguments
    ///
    /// * `msg` - A static string describing what operation was being performed.
    ///
    /// # Examples
    ///
    /// ```
    /// use error_rail::traits::ResultExt;
    ///
    /// let result: Result<(), &str> = Err("failed");
    /// let with_context = result.ctx("performing operation");
    /// assert!(with_context.is_err());
    /// ```
    fn ctx<C: IntoErrorContext>(self, msg: C) -> Result<T, Box<ComposableError<E>>>;

    /// Adds a lazily-evaluated context message to the error.
    ///
    /// The closure is only called if the `Result` is an `Err`, providing
    /// optimal performance on success paths (2.1x faster than eager evaluation).
    ///
    /// # Arguments
    ///
    /// * `f` - A closure that produces the context message.
    ///
    /// # Examples
    ///
    /// ```
    /// use error_rail::traits::ResultExt;
    ///
    /// let user_id = 42;
    /// let result: Result<(), &str> = Err("not found");
    /// let with_context = result.ctx_with(|| format!("user_id: {}", user_id));
    /// ```
    fn ctx_with<F>(self, f: F) -> Result<T, Box<ComposableError<E>>>
    where
        F: FnOnce() -> String;
}

impl<T, E> ResultExt<T, E> for Result<T, E> {
    #[inline]
    fn ctx<C: IntoErrorContext>(self, msg: C) -> Result<T, Box<ComposableError<E>>> {
        self.map_err(|e| Box::new(ComposableError::new(e).with_context(msg)))
    }

    #[inline]
    fn ctx_with<F>(self, f: F) -> Result<T, Box<ComposableError<E>>>
    where
        F: FnOnce() -> String,
    {
        self.map_err(|e| Box::new(ComposableError::new(e).with_context(LazyContext::new(f))))
    }
}

/// Extension trait for adding context to already-boxed `ComposableError` results.
///
/// This trait allows chaining `.ctx()` calls on results that already contain
/// a boxed `ComposableError`.
///
/// # Examples
///
/// ```
/// use error_rail::traits::{ResultExt, BoxedResultExt};
///
/// fn inner() -> Result<(), Box<error_rail::ComposableError<&'static str>>> {
///     Err("inner error").ctx("inner operation")
/// }
///
/// fn outer() -> Result<(), Box<error_rail::ComposableError<&'static str>>> {
///     inner().ctx_boxed("outer operation")
/// }
/// ```
pub trait BoxedResultExt<T, E> {
    /// Adds additional context to an already-boxed `ComposableError`.
    fn ctx_boxed<C: IntoErrorContext>(self, msg: C) -> Self;

    /// Adds lazily-evaluated context to an already-boxed `ComposableError`.
    fn ctx_boxed_with<F>(self, f: F) -> Self
    where
        F: FnOnce() -> String;
}

impl<T, E> BoxedResultExt<T, E> for Result<T, Box<ComposableError<E>>> {
    #[inline]
    fn ctx_boxed<C: IntoErrorContext>(self, msg: C) -> Self {
        self.map_err(|e| {
            let inner = *e;
            Box::new(inner.with_context(msg))
        })
    }

    #[inline]
    fn ctx_boxed_with<F>(self, f: F) -> Self
    where
        F: FnOnce() -> String,
    {
        self.map_err(|e| {
            let inner = *e;
            Box::new(inner.with_context(LazyContext::new(f)))
        })
    }
}
