//! Composable error type with structured context and error codes.
//!
//! This module provides [`ComposableError`], a wrapper that enriches any error type with:
//! - Multiple [`ErrorContext`] entries for structured metadata
//! - Optional error codes (defaults to `u32`)
//! - Builder pattern for incremental context accumulation

use crate::traits::IntoErrorContext;
use crate::types::alloc_type::String;
use crate::types::{ErrorContext, ErrorVec};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

mod fingerprint;
mod legacy;
mod traits;

pub use fingerprint::FingerprintConfig;
#[allow(deprecated)]
pub use legacy::LegacyErrorFormatter;

/// Error wrapper that stores the original error plus structured contexts and an optional code.
#[must_use]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComposableError<E> {
    pub(crate) core_error: E,
    pub(crate) context: ErrorVec<ErrorContext>,
    pub(crate) error_code: Option<u32>,
}

impl<E> ComposableError<E> {
    /// Creates a composable error without context or code.
    #[inline]
    pub fn new(error: E) -> Self {
        Self { core_error: error, context: ErrorVec::new(), error_code: None }
    }

    /// Creates a composable error with a pre-set error code.
    #[inline]
    pub fn with_code(error: E, code: u32) -> Self {
        Self { core_error: error, context: ErrorVec::new(), error_code: Some(code) }
    }

    /// Adds a single context entry produced by `IntoErrorContext`.
    #[inline]
    pub fn with_context<Ctx>(mut self, ctx: Ctx) -> Self
    where
        Ctx: IntoErrorContext,
    {
        self.context.push(ctx.into_error_context());
        self
    }

    /// Extends the context stack with a pre-built iterator.
    #[inline]
    pub fn with_contexts<I>(mut self, contexts: I) -> Self
    where
        I: IntoIterator<Item = ErrorContext>,
    {
        self.context.extend(contexts);
        self
    }

    /// Returns a reference to the underlying error.
    #[inline]
    pub fn core_error(&self) -> &E {
        &self.core_error
    }

    /// Returns the context stack in LIFO order (most recent first).
    #[inline]
    pub fn context(&self) -> ErrorVec<ErrorContext> {
        let mut result = ErrorVec::with_capacity(self.context.len());
        result.extend(self.context.iter().rev().cloned());
        result
    }

    /// Consumes the composable error, returning the underlying core error.
    #[inline]
    pub fn into_core(self) -> E {
        self.core_error
    }

    /// Returns an iterator in LIFO order (most recent first) that borrows the contexts.
    #[inline]
    pub fn context_iter(&self) -> core::iter::Rev<core::slice::Iter<'_, ErrorContext>> {
        self.context.iter().rev()
    }

    /// Returns the optional error code.
    #[inline]
    pub fn error_code(&self) -> Option<u32> {
        self.error_code
    }

    /// Sets (or overrides) the error code.
    #[inline]
    pub fn set_code(mut self, code: u32) -> Self {
        self.error_code = Some(code);
        self
    }

    /// Maps the core error type while preserving context/code.
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
    #[must_use]
    #[inline]
    pub fn fmt(&self) -> crate::types::error_formatter::ErrorFormatBuilder<'_, E> {
        crate::types::error_formatter::ErrorFormatBuilder::new(self)
    }

    /// Formats the error using a closure to configure the builder.
    #[must_use]
    pub fn format_with<F>(&self, f: F) -> String
    where
        E: core::fmt::Display,
        F: FnOnce(
            crate::types::error_formatter::ErrorFormatBuilder<'_, E>,
        ) -> crate::types::error_formatter::ErrorFormatBuilder<'_, E>,
    {
        let builder = self.fmt();
        f(builder).to_string()
    }

    /// Formats the error chain using a custom formatter.
    #[must_use]
    pub fn error_chain_with<F>(&self, formatter: F) -> String
    where
        E: core::fmt::Display,
        F: crate::types::error_formatter::ErrorFormatter,
    {
        use crate::types::alloc_type::Vec;
        use core::fmt::Display;

        let mut items: Vec<&dyn Display> = Vec::with_capacity(self.context.len() + 1);

        for ctx in self.context.iter().rev() {
            items.push(ctx as &dyn Display);
        }

        items.push(&self.core_error);

        formatter.format_chain(items.iter().copied())
    }

    /// Returns the complete error chain as a formatted string.
    #[must_use]
    pub fn error_chain(&self) -> String
    where
        E: core::fmt::Display,
    {
        self.fmt().to_string()
    }

    /// Generates a unique fingerprint for this error.
    #[must_use]
    pub fn fingerprint(&self) -> u64
    where
        E: core::fmt::Display,
    {
        self.compute_fingerprint()
    }

    /// Generates a hex string representation of the fingerprint.
    #[must_use]
    pub fn fingerprint_hex(&self) -> String
    where
        E: core::fmt::Display,
    {
        let mut result = String::with_capacity(16);
        let fp = self.fingerprint();
        use core::fmt::Write;
        let _ = write!(result, "{:016x}", fp);
        result
    }

    /// Internal fingerprint computation delegating to FingerprintConfig.
    #[inline]
    fn compute_fingerprint(&self) -> u64
    where
        E: core::fmt::Display,
    {
        self.fingerprint_config().compute()
    }

    /// Creates a fingerprint configuration for customizing fingerprint generation.
    #[must_use]
    pub fn fingerprint_config(&self) -> FingerprintConfig<'_, E> {
        FingerprintConfig::new(self)
    }
}
