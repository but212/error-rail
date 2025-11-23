//! Operations for error recovery and transformation.
//!
//! This module provides [`ErrorOps`], a trait that extends [`WithError`] with
//! additional combinators for recovering from errors and mapping both success
//! and error cases simultaneously.
//!
//! # Examples
//!
//! ```
//! use error_rail::traits::ErrorOps;
//!
//! let result: Result<i32, &str> = Err("failed");
//! let recovered = result.recover(|_| Ok(42));
//! assert_eq!(recovered, Ok(42));
//! ```
use crate::traits::with_error::WithError;

/// Operations for error recovery and bidirectional mapping.
///
/// This trait provides methods to:
/// - Recover from errors by providing a fallback computation
/// - Transform both success and error cases in a single operation
///
/// # Type Parameters
///
/// * `E` - The error type contained in the implementor
///
/// # Examples
///
/// ```
/// use error_rail::traits::ErrorOps;
///
/// let result: Result<i32, &str> = Err("error");
/// let mapped = result.bimap_result(|x| x * 2, |e| format!("Error: {}", e));
/// assert_eq!(mapped, Err("Error: error".to_string()));
/// ```
pub trait ErrorOps<E>: WithError<E> {
    /// Attempts to recover from an error using the provided recovery function.
    ///
    /// If the value is an error, the recovery function is called with the error
    /// and its result is returned. Otherwise, the success value is preserved.
    ///
    /// # Arguments
    ///
    /// * `recovery` - A function that takes the error and returns a new result
    ///
    /// # Examples
    ///
    /// ```
    /// use error_rail::traits::ErrorOps;
    ///
    /// let result: Result<i32, &str> = Err("failed");
    /// let recovered = result.recover(|_| Ok(0));
    /// assert_eq!(recovered, Ok(0));
    /// ```
    fn recover<F>(self, recovery: F) -> Self
    where
        F: FnOnce(E) -> Self,
        Self: Sized;

    /// Maps both success and error cases simultaneously.
    ///
    /// This is equivalent to calling `map` followed by `map_err`, but more efficient
    /// as it only matches once.
    ///
    /// # Arguments
    ///
    /// * `success_f` - Function to transform the success value
    /// * `error_f` - Function to transform the error value
    ///
    /// # Examples
    ///
    /// ```
    /// use error_rail::traits::ErrorOps;
    ///
    /// let result: Result<i32, &str> = Ok(21);
    /// let mapped = result.bimap_result(|x| x * 2, |e| e.to_uppercase());
    /// assert_eq!(mapped, Ok(42));
    /// ```
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
