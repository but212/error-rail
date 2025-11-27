//! Composable error type with structured context and error codes.
//!
//! This module provides [`ComposableError`], a wrapper that enriches any error type with:
//! - Multiple [`ErrorContext`] entries for structured metadata
//! - Optional error codes (defaults to `u32`)
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
use core::fmt::{Debug, Display};

use crate::traits::IntoErrorContext;
use crate::types::alloc_type::String;
use crate::types::{ErrorContext, ErrorVec};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(not(feature = "std"))]
use alloc::string::ToString;
#[cfg(feature = "std")]
use std::string::ToString;

/// Error wrapper that stores the original error plus structured contexts and an optional code.
///
/// `ComposableError` wraps any error type `E` and allows you to attach:
/// - Multiple [`ErrorContext`] entries (file location, tags, metadata, messages)
/// - An optional error code (currently fixed to `u32`)
///
/// Contexts are stored in a stack (LIFO) so the most recently added context appears first
/// when formatting or iterating.
///
/// # Type Parameters
///
/// * `E` - The underlying error type being wrapped
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
/// assert_eq!(err.error_code(), Some(500));
/// assert!(err.error_chain().contains("database error"));
/// ```
///
/// # Interoperability
///
/// When the wrapped error implements [`core::error::Error`], the composable error
/// exposes it through [`core::error::Error::source`], preserving compatibility with
/// libraries that inspect chained errors.
///
/// ```
/// use error_rail::ComposableError;
/// use std::{error::Error as StdError, io};
///
/// let err = ComposableError::new(io::Error::new(io::ErrorKind::Other, "boom"));
/// assert!(StdError::source(&err).is_some());
/// ```
#[must_use]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComposableError<E> {
    core_error: E,
    context: ErrorVec<ErrorContext>,
    error_code: Option<u32>,
}

impl<E> ComposableError<E> {
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
    /// let err = ComposableError::new("something failed");
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
    /// assert_eq!(err.error_code(), Some(404));
    /// ```
    #[inline]
    pub fn with_code(error: E, code: u32) -> Self {
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
    /// let err = ComposableError::new("io error")
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
    /// let err = ComposableError::new("unauthorized")
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
    /// let err = ComposableError::new("network timeout");
    /// assert_eq!(err.core_error(), &"network timeout");
    /// ```
    #[inline]
    pub fn core_error(&self) -> &E {
        &self.core_error
    }

    /// Returns the context stack in LIFO order (most recent first).
    #[inline]
    pub fn context(&self) -> ErrorVec<ErrorContext> {
        self.context.iter().rev().cloned().collect()
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
    /// let err = ComposableError::new("error")
    ///     .with_context("first")
    ///     .with_context("second");
    ///```
    /// Returns an iterator in LIFO order (most recent first) that borrows the contexts.
    ///
    /// # Examples
    ///
    /// ```
    /// use error_rail::{ComposableError, ErrorContext};
    ///
    /// let err = ComposableError::new("error")
    ///     .with_context("ctx1")
    ///     .with_context("ctx2");
    ///
    /// let mut iter = err.context_iter();
    /// assert_eq!(iter.next().unwrap().message(), "ctx2");
    /// assert_eq!(iter.next().unwrap().message(), "ctx1");
    /// ```
    #[inline]
    pub fn context_iter(&self) -> core::iter::Rev<core::slice::Iter<'_, ErrorContext>> {
        self.context.iter().rev()
    }

    /// Returns the optional error code.
    ///
    /// # Examples
    ///
    /// ```
    /// use error_rail::ComposableError;
    ///
    /// let err = ComposableError::with_code("forbidden", 403);
    /// assert_eq!(err.error_code(), Some(403));
    ///
    /// let err2 = ComposableError::new("no code");
    /// assert_eq!(err2.error_code(), None);
    /// ```
    #[inline]
    pub fn error_code(&self) -> Option<u32> {
        self.error_code
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
    /// let err = ComposableError::new("server error")
    ///     .set_code(500);
    ///
    /// assert_eq!(err.error_code(), Some(500));
    /// ```
    #[inline]
    pub fn set_code(mut self, code: u32) -> Self {
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
    /// let err = ComposableError::new("io error")
    ///     .with_context(ErrorContext::tag("fs"))
    ///     .set_code(500);
    ///
    /// let mapped = err.map_core(|e| format!("wrapped: {}", e));
    ///
    /// assert_eq!(mapped.core_error(), "wrapped: io error");
    /// assert_eq!(mapped.error_code(), Some(500));
    /// assert_eq!(mapped.context().len(), 1);
    /// ```
    #[inline]
    pub fn map_core<F, T>(self, f: F) -> ComposableError<T>
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
    /// let err = ComposableError::new("error")
    ///     .with_context("ctx1")
    ///     .set_code(500);
    ///
    /// println!("{}", err.fmt().with_separator(" | ").show_code(false));
    /// // Output: ctx1 | error
    /// ```
    #[must_use]
    #[inline]
    pub fn fmt(&self) -> ErrorFormatter<'_, E> {
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
    /// use error_rail::{ComposableError, ErrorContext, group};
    ///
    /// let err = ComposableError::new("database error")
    ///     .with_context(group!(
    ///         message("fetching user"),
    ///         tag("db")
    ///     ))
    ///     .set_code(500);
    ///
    /// let chain = err.error_chain();
    /// assert!(chain.contains("[db]"));
    /// assert!(chain.contains("fetching user"));
    /// assert!(chain.contains("database error"));
    /// assert!(chain.contains("(code: 500)"));
    /// ```
    #[must_use]
    pub fn error_chain(&self) -> String
    where
        E: Display,
    {
        self.fmt().to_string()
    }
}

/// Formatter for customizing error display output.
///
/// This struct provides fine-grained control over how [`ComposableError`] instances
/// are formatted as strings, including separator style, context ordering, and
/// whether to display error codes.
///
/// Created via [`ComposableError::fmt`], it follows the builder pattern for
/// configuring output options.
///
/// # Examples
///
/// ```
/// use error_rail::{ComposableError, ErrorContext};
///
/// let err = ComposableError::new("connection failed")
///     .with_context(ErrorContext::tag("network"))
///     .with_context("retry exhausted")
///     .set_code(503);
///
/// // Customize formatting
/// let formatted = err.fmt()
///     .with_separator(" | ")
///     .show_code(false)
///     .to_string();
///
/// assert_eq!(formatted, "retry exhausted | [network] | connection failed");
/// ```
pub struct ErrorFormatter<'a, E> {
    error: &'a ComposableError<E>,
    separator: &'a str,
    reverse_context: bool,
    show_code: bool,
}

impl<'a, E> ErrorFormatter<'a, E> {
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

impl<'a, E> Display for ErrorFormatter<'a, E>
where
    E: Display,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let contexts = &self.error.context;
        let mut first = true;

        // Helper to write separator
        let mut write_sep = |f: &mut core::fmt::Formatter<'_>| -> core::fmt::Result {
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

impl<E: Display> Display for ComposableError<E> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if f.alternate() {
            // Multi-line format
            // Error: core error (code: 500)
            // Context:
            //   - ctx1
            //   - ctx2
            write!(f, "Error: {}", self.core_error)?;
            if let Some(code) = &self.error_code {
                write!(f, " (code: {})", code)?;
            }

            if !self.context.is_empty() {
                writeln!(f)?;
                writeln!(f, "Context:")?;
                for ctx in self.context.iter().rev() {
                    let msg = ctx.message();
                    let mut lines = msg.lines();
                    if let Some(first_line) = lines.next() {
                        writeln!(f, "  - {}", first_line)?;
                    }
                    for line in lines {
                        writeln!(f, "    {}", line)?;
                    }
                }
            }
            Ok(())
        } else {
            Display::fmt(&self.fmt(), f)
        }
    }
}

impl<E> core::error::Error for ComposableError<E>
where
    E: core::error::Error + Send + Sync + 'static,
{
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        Some(self.core_error())
    }
}

impl<E> From<E> for ComposableError<E> {
    #[inline]
    fn from(error: E) -> Self {
        Self::new(error)
    }
}
