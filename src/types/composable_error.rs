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
//!     .set_code(500);
//!
//! assert!(err.to_string().contains("database connection failed"));
//! assert_eq!(err.error_code(), Some(500));
//! ```
use core::fmt::{Debug, Display};

use crate::traits::IntoErrorContext;
use crate::types::alloc_type::String;
use crate::types::{ErrorContext, ErrorVec};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(not(feature = "std"))]
use alloc::format;
#[cfg(not(feature = "std"))]
use alloc::string::ToString;
#[cfg(feature = "std")]
use std::format;
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
        Self { core_error: error, context: ErrorVec::new(), error_code: None }
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
        Self { core_error: error, context: ErrorVec::new(), error_code: Some(code) }
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
    ///
    /// **Note**: This method allocates a new `ErrorVec` on each call.
    /// For zero-allocation iteration, prefer [`context_iter()`](Self::context_iter) instead.
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
    pub fn fmt(&self) -> crate::types::error_formatter::ErrorFormatBuilder<'_, E> {
        crate::types::error_formatter::ErrorFormatBuilder::new(self)
    }

    /// Formats the error using a closure to configure the builder.
    ///
    /// # Examples
    ///
    /// ```
    /// use error_rail::ComposableError;
    ///
    /// let err = ComposableError::new("error");
    /// let s = err.format_with(|fmt| fmt.pretty().show_code(false));
    /// ```
    #[must_use]
    pub fn format_with<F>(&self, f: F) -> String
    where
        E: Display,
        F: FnOnce(
            crate::types::error_formatter::ErrorFormatBuilder<'_, E>,
        ) -> crate::types::error_formatter::ErrorFormatBuilder<'_, E>,
    {
        let builder = self.fmt();
        f(builder).to_string()
    }

    /// Formats the error chain using a custom formatter.
    ///
    /// This method allows you to customize how the error chain is formatted
    /// by providing a custom formatter implementation or using
    /// one of the built-in configurations.
    ///
    /// # Arguments
    ///
    /// * `formatter` - A closure that configures the error formatter
    ///
    /// # Examples
    ///
    /// ```
    /// use error_rail::{ComposableError, ErrorFormatConfig};
    ///
    /// let err = ComposableError::new("database error")
    ///     .with_context("fetching user");
    ///
    /// // Use pretty format
    /// println!("{}", err.error_chain_with(ErrorFormatConfig::pretty()));
    /// ```
    #[must_use]
    pub fn error_chain_with<F>(&self, formatter: F) -> String
    where
        E: Display,
        F: crate::types::error_formatter::ErrorFormatter,
    {
        use crate::types::alloc_type::Vec;

        // Collect contexts and error as Display trait objects
        let mut items: Vec<&dyn Display> = Vec::new();

        // Add contexts in LIFO order (most recent first)
        for ctx in self.context.iter().rev() {
            items.push(ctx as &dyn Display);
        }

        // Add the main error
        items.push(&self.core_error);

        // Format using the provided formatter
        formatter.format_chain(items.iter().copied())
    }

    /// Returns the complete error chain as a formatted string.
    ///
    /// This is a convenience method that uses the default formatter to create
    /// a human-readable representation of the entire error chain, including
    /// all contexts and the original error.
    ///
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

    /// Generates a unique fingerprint for this error.
    ///
    /// The fingerprint is computed from:
    /// - All tags attached to the error
    /// - The error code (if present)
    /// - The core error message
    ///
    /// This fingerprint can be used for:
    /// - Deduplicating similar errors in error tracking systems (e.g., Sentry)
    /// - Grouping related errors in logs
    /// - Creating stable error identifiers for alerting
    ///
    /// # Algorithm
    ///
    /// Uses a simple but stable hash algorithm (FNV-1a inspired) to generate
    /// a 64-bit fingerprint. The fingerprint is deterministic for the same
    /// input values.
    ///
    /// # Examples
    ///
    /// ```
    /// use error_rail::{ComposableError, ErrorContext};
    ///
    /// let err1 = ComposableError::new("database error")
    ///     .with_context(ErrorContext::tag("db"))
    ///     .set_code(500);
    ///
    /// let err2 = ComposableError::new("database error")
    ///     .with_context(ErrorContext::tag("db"))
    ///     .set_code(500);
    ///
    /// // Same configuration produces same fingerprint
    /// assert_eq!(err1.fingerprint(), err2.fingerprint());
    ///
    /// let err3 = ComposableError::new("different error")
    ///     .with_context(ErrorContext::tag("db"))
    ///     .set_code(500);
    ///
    /// // Different message produces different fingerprint
    /// assert_ne!(err1.fingerprint(), err3.fingerprint());
    /// ```
    #[must_use]
    pub fn fingerprint(&self) -> u64
    where
        E: Display,
    {
        self.compute_fingerprint()
    }

    /// Generates a hex string representation of the fingerprint.
    ///
    /// This is useful for logging and integration with external systems
    /// that expect string-based identifiers.
    ///
    /// # Examples
    ///
    /// ```
    /// use error_rail::{ComposableError, ErrorContext};
    ///
    /// let err = ComposableError::new("timeout")
    ///     .with_context(ErrorContext::tag("network"))
    ///     .set_code(504);
    ///
    /// let fp = err.fingerprint_hex();
    /// assert_eq!(fp.len(), 16); // 64-bit hex = 16 characters
    /// println!("Error fingerprint: {}", fp);
    /// ```
    #[must_use]
    pub fn fingerprint_hex(&self) -> String
    where
        E: Display,
    {
        format!("{:016x}", self.fingerprint())
    }

    /// Internal fingerprint computation delegating to FingerprintConfig.
    ///
    /// Uses FingerprintConfig with default settings (include tags, code, message).
    #[inline]
    fn compute_fingerprint(&self) -> u64
    where
        E: Display,
    {
        self.fingerprint_config().compute()
    }

    /// Creates a fingerprint configuration for customizing fingerprint generation.
    ///
    /// This allows fine-grained control over which components are included
    /// in the fingerprint calculation.
    ///
    /// # Examples
    ///
    /// ```
    /// use error_rail::{ComposableError, ErrorContext};
    ///
    /// let err = ComposableError::new("timeout")
    ///     .with_context(ErrorContext::tag("network"))
    ///     .set_code(504);
    ///
    /// // Generate fingerprint with only tags and code (ignoring message)
    /// let fp = err.fingerprint_config()
    ///     .include_message(false)
    ///     .compute();
    /// ```
    #[must_use]
    pub fn fingerprint_config(&self) -> FingerprintConfig<'_, E> {
        FingerprintConfig {
            error: self,
            include_tags: true,
            include_code: true,
            include_message: true,
            include_metadata: false,
        }
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

        // Write contexts
        if self.reverse_context {
            for ctx in contexts.iter() {
                if !first {
                    f.write_str(self.separator)?;
                }
                first = false;
                f.write_str(ctx.message().as_ref())?;
            }
        } else {
            for ctx in contexts.iter().rev() {
                if !first {
                    f.write_str(self.separator)?;
                }
                first = false;
                f.write_str(ctx.message().as_ref())?;
            }
        }

        // Write core error
        if !first {
            f.write_str(self.separator)?;
        }
        Display::fmt(&self.error.core_error, f)?;

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
        if !f.alternate() {
            return Display::fmt(&self.fmt(), f);
        }

        write!(f, "Error: {}", self.core_error)?;
        if let Some(code) = &self.error_code {
            write!(f, " (code: {})", code)?;
        }

        if self.context.is_empty() {
            return Ok(());
        }

        writeln!(f)?;
        writeln!(f, "Context:")?;
        for ctx in self.context.iter().rev() {
            let message = ctx.message();
            let mut lines = message.lines();
            if let Some(first) = lines.next() {
                writeln!(f, "  - {}", first)?;
                for line in lines {
                    writeln!(f, "    {}", line)?;
                }
            }
        }
        Ok(())
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

/// Configuration builder for customizing fingerprint generation.
///
/// This struct allows fine-grained control over which error components
/// are included when computing the fingerprint.
///
/// # Examples
///
/// ```
/// use error_rail::{ComposableError, ErrorContext};
///
/// let err = ComposableError::new("database timeout")
///     .with_context(ErrorContext::tag("db"))
///     .with_context(ErrorContext::metadata("table", "users"))
///     .set_code(504);
///
/// // Include only tags and code, ignore message for broader grouping
/// let fp1 = err.fingerprint_config()
///     .include_message(false)
///     .compute();
///
/// // Include everything for precise matching
/// let fp2 = err.fingerprint_config()
///     .include_metadata(true)
///     .compute();
///
/// // Different configurations produce different fingerprints
/// assert_ne!(fp1, fp2);
/// ```
pub struct FingerprintConfig<'a, E> {
    error: &'a ComposableError<E>,
    include_tags: bool,
    include_code: bool,
    include_message: bool,
    include_metadata: bool,
}

impl<'a, E> FingerprintConfig<'a, E> {
    /// Whether to include tags in the fingerprint (default: true).
    ///
    /// Tags are useful for categorizing errors by subsystem (e.g., "db", "network").
    #[must_use]
    pub fn include_tags(mut self, include: bool) -> Self {
        self.include_tags = include;
        self
    }

    /// Whether to include the error code in the fingerprint (default: true).
    ///
    /// Error codes provide semantic meaning (e.g., HTTP status codes).
    #[must_use]
    pub fn include_code(mut self, include: bool) -> Self {
        self.include_code = include;
        self
    }

    /// Whether to include the core error message in the fingerprint (default: true).
    ///
    /// Excluding the message creates broader groupings useful when error
    /// messages contain variable data (e.g., timestamps, IDs).
    #[must_use]
    pub fn include_message(mut self, include: bool) -> Self {
        self.include_message = include;
        self
    }

    /// Whether to include metadata in the fingerprint (default: false).
    ///
    /// Metadata often contains variable data, so it's excluded by default.
    #[must_use]
    pub fn include_metadata(mut self, include: bool) -> Self {
        self.include_metadata = include;
        self
    }

    /// Computes the fingerprint using the configured options.
    ///
    /// Uses FNV-1a hash algorithm to generate a stable 64-bit fingerprint
    /// based on the configured components (tags, code, message, metadata).
    ///
    /// # Algorithm Details
    ///
    /// The fingerprint is computed using FNV-1a (Fowler-Noll-Vo) hash:
    /// - Deterministic: Same input always produces same output
    /// - Fast: Simple byte-by-byte computation
    /// - Good distribution: Minimizes collisions for similar inputs
    ///
    /// Components are hashed in a fixed order with prefixes to prevent
    /// collision between different component types:
    /// 1. Tags (sorted alphabetically, prefixed with "tag:")
    /// 2. Error code (prefixed with "code:")
    /// 3. Core message (prefixed with "msg:")
    /// 4. Metadata (sorted by key, prefixed with "meta:")
    ///
    /// # Returns
    ///
    /// A 64-bit unsigned integer representing the fingerprint.
    ///
    /// # Examples
    ///
    /// ```
    /// use error_rail::{ComposableError, ErrorContext};
    ///
    /// let err = ComposableError::new("timeout")
    ///     .with_context(ErrorContext::tag("network"))
    ///     .set_code(504);
    ///
    /// let fp = err.fingerprint_config().compute();
    /// assert_ne!(fp, 0); // Fingerprint is computed
    /// ```
    #[must_use]
    pub fn compute(&self) -> u64
    where
        E: Display,
    {
        // FNV-1a 64-bit offset basis
        // This is the standard starting value for FNV-1a hash
        const FNV_OFFSET: u64 = 0xcbf29ce484222325;

        let mut hash = FNV_OFFSET;

        /// Hashes a byte slice into the running hash using FNV-1a algorithm.
        ///
        /// FNV-1a works by XORing each byte with the hash, then multiplying
        /// by the FNV prime. This order (XOR then multiply) is what makes
        /// it "1a" variant, which has better avalanche characteristics.
        #[inline(always)]
        fn hash_bytes(hash: &mut u64, bytes: &[u8]) {
            // FNV-1a 64-bit prime
            const FNV_PRIME: u64 = 0x100000001b3;
            for &byte in bytes {
                *hash ^= byte as u64;
                *hash = hash.wrapping_mul(FNV_PRIME);
            }
        }

        // Include tags in sorted order for deterministic fingerprinting
        // Sorting ensures ["a", "b"] and ["b", "a"] produce the same fingerprint
        if self.include_tags {
            let mut tags: crate::types::alloc_type::Vec<_> = self
                .error
                .context
                .iter()
                .filter_map(|ctx| match ctx {
                    ErrorContext::Group(g) => Some(g.tags.iter()),
                    _ => None,
                })
                .flatten()
                .collect();
            tags.sort_unstable();

            for tag in tags {
                hash_bytes(&mut hash, b"tag:");
                hash_bytes(&mut hash, tag.as_bytes());
            }
        }

        // Include error code as little-endian bytes
        if self.include_code {
            if let Some(code) = self.error.error_code {
                hash_bytes(&mut hash, b"code:");
                hash_bytes(&mut hash, &code.to_le_bytes());
            }
        }

        // Include core error message
        if self.include_message {
            hash_bytes(&mut hash, b"msg:");
            hash_bytes(&mut hash, self.error.core_error.to_string().as_bytes());
        }

        // Include metadata in sorted order by key for deterministic fingerprinting
        if self.include_metadata {
            let mut metadata: crate::types::alloc_type::Vec<_> = self
                .error
                .context
                .iter()
                .filter_map(|ctx| match ctx {
                    ErrorContext::Group(g) => Some(g.metadata.iter()),
                    _ => None,
                })
                .flatten()
                .collect();
            metadata.sort_unstable_by(|a, b| a.0.cmp(&b.0));

            for (key, value) in metadata {
                hash_bytes(&mut hash, b"meta:");
                hash_bytes(&mut hash, key.as_bytes());
                hash_bytes(&mut hash, b"=");
                hash_bytes(&mut hash, value.as_bytes());
            }
        }

        hash
    }

    /// Computes the fingerprint and returns it as a hex string.
    #[must_use]
    pub fn compute_hex(&self) -> String
    where
        E: Display,
    {
        format!("{:016x}", self.compute())
    }
}
