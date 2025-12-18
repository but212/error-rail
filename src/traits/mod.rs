//! Core traits for error handling and composition.
//!
//! This module consolidates the key traits used throughout `error-rail`.

use crate::types::alloc_type::{Box, Cow, String};
use crate::types::{ComposableError, ErrorContext, ErrorVec, LazyContext};
use core::time::Duration;

// ============================================================================
// IntoErrorContext
// ============================================================================

/// Converts a type into an [`ErrorContext`] for error annotation.
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
    #[inline]
    fn into_error_context(self) -> ErrorContext {
        ErrorContext::new(self)
    }
}

impl IntoErrorContext for &'static str {
    #[inline]
    fn into_error_context(self) -> ErrorContext {
        ErrorContext::new(self)
    }
}

impl IntoErrorContext for Cow<'static, str> {
    #[inline]
    fn into_error_context(self) -> ErrorContext {
        ErrorContext::new(self)
    }
}

impl IntoErrorContext for ErrorContext {
    #[inline]
    fn into_error_context(self) -> ErrorContext {
        self
    }
}

// ============================================================================
// TransientError
// ============================================================================

/// Classification of errors as transient or permanent.
pub trait TransientError {
    /// Returns `true` if this error is transient and may succeed on retry.
    fn is_transient(&self) -> bool;

    /// Returns `true` if this error is permanent and should not be retried.
    #[inline]
    fn is_permanent(&self) -> bool {
        !self.is_transient()
    }

    /// Optional hint for how long to wait before retrying.
    #[inline]
    fn retry_after_hint(&self) -> Option<Duration> {
        None
    }

    /// Returns the maximum number of retry attempts for this error.
    #[inline]
    fn max_retries_hint(&self) -> Option<u32> {
        None
    }
}

#[cfg(feature = "std")]
impl TransientError for std::io::Error {
    fn is_transient(&self) -> bool {
        use std::io::ErrorKind;
        matches!(
            self.kind(),
            ErrorKind::ConnectionRefused
                | ErrorKind::ConnectionReset
                | ErrorKind::ConnectionAborted
                | ErrorKind::TimedOut
                | ErrorKind::Interrupted
                | ErrorKind::WouldBlock
        )
    }
}

/// Extension methods for working with transient errors.
pub trait TransientErrorExt<T, E: TransientError> {
    /// Converts a transient error to `Some(Err(e))` for retry, or `None` to stop.
    fn retry_if_transient(self) -> Option<Result<T, E>>;
}

impl<T, E: TransientError> TransientErrorExt<T, E> for Result<T, E> {
    fn retry_if_transient(self) -> Option<Result<T, E>> {
        match &self {
            Ok(_) => None,
            Err(e) if e.is_transient() => Some(self),
            Err(_) => None,
        }
    }
}

// ============================================================================
// WithError
// ============================================================================

/// Abstraction over types that carry an error variant which can be remapped.
pub trait WithError<E> {
    type Success;
    type ErrorOutput<G>;

    /// Maps the error value using `f`, producing a new container with error type `G`.
    fn fmap_error<F, G>(self, f: F) -> Self::ErrorOutput<G>
    where
        F: Fn(E) -> G;

    /// Converts the container into a `Result`, taking only the first error if invalid.
    fn to_result_first(self) -> Result<Self::Success, E>;

    /// Converts the container into a `Result`, preserving all errors if invalid.
    fn to_result_all(self) -> Result<Self::Success, ErrorVec<E>>;
}

impl<T, E> WithError<E> for Result<T, E> {
    type Success = T;
    type ErrorOutput<G> = Result<T, G>;

    fn fmap_error<F, G>(self, f: F) -> Self::ErrorOutput<G>
    where
        F: FnOnce(E) -> G,
    {
        match self {
            Ok(t) => Ok(t),
            Err(e) => Err(f(e)),
        }
    }

    fn to_result_first(self) -> Result<Self::Success, E> {
        self
    }

    fn to_result_all(self) -> Result<Self::Success, ErrorVec<E>> {
        match self {
            Ok(t) => Ok(t),
            Err(e) => {
                let mut error_vec = ErrorVec::new();
                error_vec.push(e);
                Err(error_vec)
            },
        }
    }
}

// ============================================================================
// ErrorOps
// ============================================================================

/// Operations for error recovery and bidirectional mapping.
pub trait ErrorOps<E>: WithError<E> {
    /// Attempts to recover from an error using the provided recovery function.
    fn recover<F>(self, recovery: F) -> Self
    where
        F: FnOnce(E) -> Self,
        Self: Sized;

    /// Maps both success and error cases simultaneously.
    fn bimap_result<B, F, SuccessF, ErrorF>(
        self,
        success_f: SuccessF,
        error_f: ErrorF,
    ) -> Result<B, F>
    where
        SuccessF: FnOnce(Self::Success) -> B,
        ErrorF: FnOnce(E) -> F,
        Self: Sized;
}

impl<T, E> ErrorOps<E> for Result<T, E> {
    #[inline]
    fn recover<F>(self, recovery: F) -> Self
    where
        F: FnOnce(E) -> Self,
    {
        match self {
            Ok(value) => Ok(value),
            Err(error) => recovery(error),
        }
    }

    #[inline]
    fn bimap_result<B, F, SuccessF, ErrorF>(
        self,
        success_f: SuccessF,
        error_f: ErrorF,
    ) -> Result<B, F>
    where
        SuccessF: FnOnce(T) -> B,
        ErrorF: FnOnce(E) -> F,
    {
        match self {
            Ok(value) => Ok(success_f(value)),
            Err(error) => Err(error_f(error)),
        }
    }
}

// ============================================================================
// ResultExt
// ============================================================================

/// Extension trait for ergonomic context addition to `Result` types.
pub trait ResultExt<T, E> {
    /// Adds a static context message to the error.
    fn ctx<C: IntoErrorContext>(self, msg: C) -> Result<T, Box<ComposableError<E>>>;

    /// Adds a lazily-evaluated context message to the error.
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

// ============================================================================
// ErrorCategory
// ============================================================================

/// Trait for types that can lift values and handle errors in a functorial way.
pub trait ErrorCategory<E> {
    type ErrorFunctor<T>: WithError<E>;

    fn lift<T>(value: T) -> Self::ErrorFunctor<T>;
    fn handle_error<T>(error: E) -> Self::ErrorFunctor<T>;
}

impl<E> ErrorCategory<E> for Result<(), E> {
    type ErrorFunctor<T> = Result<T, E>;

    #[inline]
    fn lift<T>(value: T) -> Result<T, E> {
        Ok(value)
    }

    #[inline]
    fn handle_error<T>(error: E) -> Result<T, E> {
        Err(error)
    }
}
