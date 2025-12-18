//! Core traits for error handling and composition.
//!
//! This module consolidates the key traits used throughout `error-rail`.

use crate::types::alloc_type::{Box, Cow, String};
use crate::types::{ComposableError, ErrorContext, ErrorVec, LazyContext};
use core::time::Duration;

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

/// Extension trait for ergonomic context addition to `Result` types.
///
/// This trait provides methods for enriching errors with contextual information,
/// wrapping them in a [`ComposableError`] for structured error handling.
///
/// # Examples
///
/// ```
/// use error_rail::{ResultExt, ErrorContext};
///
/// fn read_config() -> Result<String, Box<error_rail::ComposableError<std::io::Error>>> {
///     std::fs::read_to_string("config.toml")
///         .ctx("failed to read configuration file")
/// }
///
/// fn parse_config() -> Result<i32, Box<error_rail::ComposableError<&'static str>>> {
///     Err("invalid format")
///         .ctx_with(|| format!("parsing failed at line {}", 42))
/// }
/// ```
pub trait ResultExt<T, E> {
    /// Adds a static context message to the error.
    ///
    /// Wraps the error in a [`ComposableError`] with the provided context.
    /// Use this when the context message is cheap to construct or already available.
    ///
    /// # Arguments
    ///
    /// * `msg` - Any type implementing [`IntoErrorContext`], such as `&str`, `String`, or [`ErrorContext`]
    ///
    /// # Examples
    ///
    /// ```
    /// use error_rail::ResultExt;
    ///
    /// let result: Result<i32, &str> = Err("connection failed");
    /// let enriched = result.ctx("database operation failed");
    /// assert!(enriched.is_err());
    /// ```
    fn ctx<C: IntoErrorContext>(self, msg: C) -> Result<T, Box<ComposableError<E>>>;

    /// Adds a lazily-evaluated context message to the error.
    ///
    /// The context is only computed if the result is an error, making this
    /// suitable for expensive context generation (e.g., formatting, I/O).
    ///
    /// # Arguments
    ///
    /// * `f` - A closure that produces the context string when called
    ///
    /// # Examples
    ///
    /// ```
    /// use error_rail::ResultExt;
    ///
    /// let result: Result<i32, &str> = Err("timeout");
    /// let enriched = result.ctx_with(|| format!("request failed after {} retries", 3));
    /// assert!(enriched.is_err());
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
/// This trait provides methods for enriching errors that are already wrapped in
/// `Box<ComposableError<E>>`, allowing additional context to be added without
/// re-boxing the error.
///
/// # Examples
///
/// ```
/// use error_rail::{BoxedResultExt, ResultExt, ErrorContext};
///
/// fn inner_operation() -> Result<i32, Box<error_rail::ComposableError<&'static str>>> {
///     Err("inner error").ctx("inner context")
/// }
///
/// fn outer_operation() -> Result<i32, Box<error_rail::ComposableError<&'static str>>> {
///     inner_operation().ctx_boxed("outer context")
/// }
/// ```
pub trait BoxedResultExt<T, E> {
    /// Adds additional context to an already-boxed `ComposableError`.
    ///
    /// This method is useful when you have a `Result<T, Box<ComposableError<E>>>`
    /// and want to add more context without changing the error type.
    ///
    /// # Arguments
    ///
    /// * `msg` - The context message to add, which must implement `IntoErrorContext`
    ///
    /// # Examples
    ///
    /// ```
    /// use error_rail::{BoxedResultExt, ResultExt};
    ///
    /// let result: Result<i32, _> = Err("error").ctx("first context");
    /// let enriched = result.ctx_boxed("second context");
    /// ```
    fn ctx_boxed<C: IntoErrorContext>(self, msg: C) -> Self;

    /// Adds lazily-evaluated context to an already-boxed `ComposableError`.
    ///
    /// The context message is only computed if the result is an error,
    /// which can improve performance when context generation is expensive.
    ///
    /// # Arguments
    ///
    /// * `f` - A closure that produces the context message
    ///
    /// # Examples
    ///
    /// ```
    /// use error_rail::{BoxedResultExt, ResultExt};
    ///
    /// let result: Result<i32, _> = Err("error").ctx("first context");
    /// let enriched = result.ctx_boxed_with(|| format!("context at {}", "runtime"));
    /// ```
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

/// Trait for types that can lift values and handle errors in a functorial way.
///
/// This trait provides a category-theoretic abstraction for error handling,
/// allowing types to be lifted into an error-handling context and errors
/// to be properly propagated.
///
/// # Type Parameters
///
/// * `E` - The error type that this category handles
///
/// # Associated Types
///
/// * `ErrorFunctor<T>` - The functor type that wraps values of type `T` with error handling
///
/// # Examples
///
/// ```
/// use error_rail::traits::ErrorCategory;
///
/// fn example<E>() {
///     let success: Result<i32, E> = <Result<(), E> as ErrorCategory<E>>::lift(42);
///     assert!(success.is_ok());
/// }
/// ```
pub trait ErrorCategory<E> {
    /// The functor type that provides error handling for values of type `T`.
    type ErrorFunctor<T>: WithError<E>;

    /// Lifts a pure value into the error-handling context.
    ///
    /// # Arguments
    ///
    /// * `value` - The value to lift into the functor
    ///
    /// # Returns
    ///
    /// The value wrapped in a successful error-handling context.
    fn lift<T>(value: T) -> Self::ErrorFunctor<T>;

    /// Creates an error-handling context representing a failure.
    ///
    /// # Arguments
    ///
    /// * `error` - The error to wrap
    ///
    /// # Returns
    ///
    /// An error-handling context containing the error.
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
