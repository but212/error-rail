//! Composable error type with structured context and error codes.
//!
//! This module provides [`ComposableError`], a wrapper that enriches any error type with:
//! - Multiple [`ErrorContext`] entries for structured metadata
//! - Optional error codes of any type (defaults to `u32`)
//! - Builder pattern for incremental context accumulation
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
use std::fmt::{Debug, Display};

use crate::traits::IntoErrorContext;
use crate::types::{ErrorContext, ErrorVec};

/// Error wrapper that stores the original error plus structured contexts and an optional code `C`.
///
/// `ComposableError` wraps any error type `E` and allows you to attach:
/// - Multiple [`ErrorContext`] entries (file location, tags, metadata, messages)
/// - An optional error code of type `C` (defaults to `u32`)
///
/// Contexts are stored in a stack (LIFO) so the most recently added context appears first
/// when formatting or iterating.
///
/// # Type Parameters
///
/// * `E` - The underlying error type being wrapped
/// * `C` - The error code type (defaults to `u32`)
///
/// # Examples
///
/// ```
/// use error_rail::{ComposableError, ErrorContext};
///
/// let err = ComposableError::new("database error")
///     .with_context(ErrorContext::tag("db"))
///     .with_context(ErrorContext::location(file!(), line!()))
///     .set_code(500);
///
/// assert_eq!(err.error_code(), Some(&500));
/// assert!(err.error_chain().contains("database error"));
/// ```
///
/// # Interoperability
///
/// When the wrapped error implements [`std::error::Error`], the composable error
/// exposes it through [`std::error::Error::source`], preserving compatibility with
/// libraries that inspect chained errors.
///
/// ```
/// use error_rail::ComposableError;
/// use std::{error::Error as StdError, io};
///
/// let err = ComposableError::<io::Error, u32>::new(io::Error::new(io::ErrorKind::Other, "boom"));
/// assert!(StdError::source(&err).is_some());
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComposableError<E, C = u32> {
    core_error: E,
    context: ErrorVec<ErrorContext>,
    error_code: Option<C>,
}

impl<E, C> ComposableError<E, C> {
    /// Creates a composable error without context or code.
    ///
    /// This is the simplest constructor - it wraps the error without any additional metadata.
    ///
    /// # Arguments
    ///
    /// * `error` - The core error to wrap
    ///
    /// # Examples
    ///
    /// ```
    /// use error_rail::ComposableError;
    ///
    /// let err = ComposableError::<&str, u32>::new("something failed");
    /// assert_eq!(err.core_error(), &"something failed");
    /// assert!(err.context().is_empty());
    /// ```
    #[inline]
    pub fn new(error: E) -> Self {
        Self {
            core_error: error,
            context: ErrorVec::new(),
            error_code: None,
        }
    }

    /// Creates a composable error with a pre-set error code.
    ///
    /// Use this when you want to immediately assign an error code without chaining.
    ///
    /// # Arguments
    ///
    /// * `error` - The core error to wrap
    /// * `code` - The error code to assign
    ///
    /// # Examples
    ///
    /// ```
    /// use error_rail::ComposableError;
    ///
    /// let err = ComposableError::with_code("not found", 404);
    /// assert_eq!(err.error_code(), Some(&404));
    /// ```
    #[inline]
    pub fn with_code(error: E, code: C) -> Self {
        Self {
            core_error: error,
            context: ErrorVec::new(),
            error_code: Some(code),
        }
    }

    /// Adds a single context entry produced by `IntoErrorContext`.
    ///
    /// This method chains, allowing you to build up context incrementally.
    /// Contexts are stored in LIFO order (most recent first).
    ///
    /// # Arguments
    ///
    /// * `ctx` - Any type implementing [`IntoErrorContext`]
    ///
    /// # Examples
    ///
    /// ```
    /// use error_rail::{ComposableError, ErrorContext};
    ///
    /// let err = ComposableError::<&str, u32>::new("io error")
    ///     .with_context(ErrorContext::tag("filesystem"))
    ///     .with_context("reading config file");
    ///
    /// assert_eq!(err.context().len(), 2);
    /// ```
    #[inline]
    pub fn with_context<Ctx>(mut self, ctx: Ctx) -> Self
    where
        Ctx: IntoErrorContext,
    {
        self.context.push(ctx.into_error_context());
        self
    }

    /// Extends the context stack with a pre-built iterator.
    ///
    /// Useful when you have multiple contexts to add at once from an existing collection.
    ///
    /// # Arguments
    ///
    /// * `contexts` - An iterator of [`ErrorContext`] entries
    ///
    /// # Examples
    ///
    /// ```
    /// use error_rail::{ComposableError, ErrorContext};
    ///
    /// let contexts = vec![
    ///     ErrorContext::tag("auth"),
    ///     ErrorContext::new("token expired"),
    /// ];
    ///
    /// let err = ComposableError::<&str, u32>::new("unauthorized")
    ///     .with_contexts(contexts);
    ///
    /// assert_eq!(err.context().len(), 2);
    /// ```
    #[inline]
    pub fn with_contexts<I>(mut self, contexts: I) -> Self
    where
        I: IntoIterator<Item = ErrorContext>,
    {
        self.context.extend(contexts);
        self
    }

    /// Returns a reference to the underlying error.
    ///
    /// # Examples
    ///
    /// ```
    /// use error_rail::ComposableError;
    ///
    /// let err = ComposableError::<&str, u32>::new("network timeout");
    /// assert_eq!(err.core_error(), &"network timeout");
    /// ```
    #[inline]
    pub fn core_error(&self) -> &E {
        &self.core_error
    }

    /// Consumes the composable error, returning the underlying core error.
    #[inline]
    pub fn into_core(self) -> E {
        self.core_error
    }

    /// Returns contexts in LIFO order (most recent first).
    ///
    /// This allocates a new `Vec` with cloned contexts. For zero-allocation iteration,
    /// use [`context_iter`](Self::context_iter) instead.
    ///
    /// # Examples
    ///
    /// ```
    /// use error_rail::{ComposableError, ErrorContext};
    ///
    /// let err = ComposableError::<&str, u32>::new("error")
    ///     .with_context("first")
    ///     .with_context("second");
    ///
    /// let contexts = err.context();
    /// assert_eq!(contexts[0].message(), "second");
    /// assert_eq!(contexts[1].message(), "first");
    /// ```
    #[inline]
    pub fn context(&self) -> Vec<ErrorContext> {
        self.context.iter().rev().cloned().collect()
    }

    /// Efficient iterator over contexts without allocation.
    ///
    /// Returns an iterator in LIFO order (most recent first) that borrows the contexts.
    ///
    /// # Examples
    ///
    /// ```
    /// use error_rail::{ComposableError, ErrorContext};
    ///
    /// let err = ComposableError::<&str, u32>::new("error")
    ///     .with_context("ctx1")
    ///     .with_context("ctx2");
    ///
    /// let mut iter = err.context_iter();
    /// assert_eq!(iter.next().unwrap().message(), "ctx2");
    /// assert_eq!(iter.next().unwrap().message(), "ctx1");
    /// ```
    #[inline]
    pub fn context_iter(&self) -> std::iter::Rev<std::slice::Iter<'_, ErrorContext>> {
        self.context.iter().rev()
    }

    /// Returns the optional error code reference.
    ///
    /// # Examples
    ///
    /// ```
    /// use error_rail::ComposableError;
    ///
    /// let err = ComposableError::<&str, u32>::with_code("forbidden", 403);
    /// assert_eq!(err.error_code(), Some(&403));
    ///
    /// let err2 = ComposableError::<&str, u32>::new("no code");
    /// assert_eq!(err2.error_code(), None);
    /// ```
    #[inline]
    pub fn error_code(&self) -> Option<&C> {
        self.error_code.as_ref()
    }

    /// Sets (or overrides) the error code.
    ///
    /// This method chains, allowing you to set the code after construction.
    ///
    /// # Arguments
    ///
    /// * `code` - The error code to set
    ///
    /// # Examples
    ///
    /// ```
    /// use error_rail::ComposableError;
    ///
    /// let err = ComposableError::<&str, u32>::new("server error")
    ///     .set_code(500);
    ///
    /// assert_eq!(err.error_code(), Some(&500));
    /// ```
    #[inline]
    pub fn set_code(mut self, code: C) -> Self {
        self.error_code = Some(code);
        self
    }

    /// Maps the core error type while preserving context/code.
    ///
    /// Useful for converting between error types while maintaining all attached metadata.
    ///
    /// # Arguments
    ///
    /// * `f` - A function that transforms the core error from type `E` to type `T`
    ///
    /// # Examples
    ///
    /// ```
    /// use error_rail::{ComposableError, ErrorContext};
    ///
    /// let err = ComposableError::<&str, u32>::new("io error")
    ///     .with_context(ErrorContext::tag("fs"))
    ///     .set_code(500);
    ///
    /// let mapped = err.map_core(|e| format!("wrapped: {}", e));
    ///
    /// assert_eq!(mapped.core_error(), "wrapped: io error");
    /// assert_eq!(mapped.error_code(), Some(&500));
    /// assert_eq!(mapped.context().len(), 1);
    /// ```
    #[inline]
    pub fn map_core<F, T>(self, f: F) -> ComposableError<T, C>
    where
        F: FnOnce(E) -> T,
    {
        ComposableError {
            core_error: f(self.core_error),
            context: self.context,
            error_code: self.error_code,
        }
    }

    /// Returns a builder for customizing the error formatting.
    ///
    /// This allows you to change the separator, order, and visibility of components
    /// without allocating a new string immediately.
    ///
    /// # Examples
    ///
    /// ```
    /// use error_rail::{ComposableError, ErrorContext};
    ///
    /// let err = ComposableError::<&str, u32>::new("error")
    ///     .with_context("ctx1")
    ///     .set_code(500);
    ///
    /// println!("{}", err.fmt().with_separator(" | ").show_code(false));
    /// // Output: ctx1 | error
    /// ```
    #[inline]
    pub fn fmt(&self) -> ErrorFormatter<'_, E, C> {
        ErrorFormatter {
            error: self,
            separator: " -> ",
            reverse_context: false,
            show_code: true,
        }
    }

    /// Formats the error chain as `ctx1 -> ctx2 -> core_error (code: ...)`.
    ///
    /// Contexts are displayed in LIFO order (most recent first), followed by the core error.
    /// If an error code is present, it's appended at the end.
    ///
    /// # Examples
    ///
    /// ```
    /// use error_rail::{ComposableError, ErrorContext};
    ///
    /// let err = ComposableError::<&str, u32>::new("database error")
    ///     .with_context("fetching user")
    ///     .with_context(ErrorContext::tag("db"))
    ///     .set_code(500);
    ///
    /// let chain = err.error_chain();
    /// assert!(chain.contains("[db] -> fetching user -> database error (code: 500)"));
    /// ```
    pub fn error_chain(&self) -> String
    where
        E: Display,
        C: Display,
    {
        self.fmt().to_string()
    }
}

/// Helper struct for customizing error formatting.
///
/// Created via [`ComposableError::fmt`].
pub struct ErrorFormatter<'a, E, C> {
    error: &'a ComposableError<E, C>,
    separator: &'a str,
    reverse_context: bool,
    show_code: bool,
}

impl<'a, E, C> ErrorFormatter<'a, E, C> {
    /// Sets the separator between context elements (default: " -> ").
    pub fn with_separator(mut self, separator: &'a str) -> Self {
        self.separator = separator;
        self
    }

    /// If true, displays contexts in FIFO order (oldest first) instead of LIFO.
    pub fn reverse_context(mut self, reverse: bool) -> Self {
        self.reverse_context = reverse;
        self
    }

    /// Whether to include the error code in the output (default: true).
    pub fn show_code(mut self, show: bool) -> Self {
        self.show_code = show;
        self
    }
}

impl<'a, E, C> Display for ErrorFormatter<'a, E, C>
where
    E: Display,
    C: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let contexts = &self.error.context;
        let mut first = true;

        // Helper to write separator
        let mut write_sep = |f: &mut std::fmt::Formatter<'_>| -> std::fmt::Result {
            if !first {
                write!(f, "{}", self.separator)?;
            }
            first = false;
            Ok(())
        };

        // Write contexts
        if self.reverse_context {
            // FIFO order (oldest first)
            for ctx in contexts.iter() {
                write_sep(f)?;
                write!(f, "{}", ctx.message())?;
            }
        } else {
            // LIFO order (newest first) - default
            for ctx in contexts.iter().rev() {
                write_sep(f)?;
                write!(f, "{}", ctx.message())?;
            }
        }

        // Write core error
        write_sep(f)?;
        write!(f, "{}", self.error.core_error)?;

        // Write error code
        if self.show_code {
            if let Some(code) = &self.error.error_code {
                write!(f, " (code: {})", code)?;
            }
        }

        Ok(())
    }
}

impl<E: Display, C: Display> Display for ComposableError<E, C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.fmt(), f)
    }
}

impl<E, C> std::error::Error for ComposableError<E, C>
where
    E: std::error::Error + 'static,
    C: Debug + Display,
{
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(self.core_error())
    }
}

impl<E, C> From<E> for ComposableError<E, C> {
    #[inline]
    fn from(error: E) -> Self {
        Self::new(error)
    }
}
