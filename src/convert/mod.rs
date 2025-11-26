//! Conversion helpers between `Result`, `Validation`, and `ComposableError`.
//!
//! These adapters make it straightforward to incrementally adopt `error-rail`
//! by wrapping legacy results or by flattening composable errors back into core
//! types when interacting with external APIs.
//!
//! # Examples
//!
//! ```
//! use error_rail::convert::*;
//! use error_rail::validation::Validation;
//!
//! // Convert between Result and Validation
//! let result: Result<i32, &str> = Ok(42);
//! let validation = result_to_validation(result);
//! assert!(validation.is_valid());
//!
//! // Wrap errors in ComposableError
//! let result: Result<i32, &str> = Err("failed");
//! let composable = wrap_in_composable_result(result);
//! ```

use crate::types::BoxedComposableResult;
use crate::validation::core::Validation;
use crate::{types::composable_error::ComposableError, ErrorVec};
use core::iter::FusedIterator;

/// Converts a `Validation` to a `Result`, taking the first error if invalid.
///
/// # Arguments
///
/// * `validation` - The validation to convert
///
/// # Returns
///
/// * `Ok(value)` if validation is valid
/// * `Err(first_error)` if validation is invalid
///
/// # Panics
///
/// Panics if the `Validation::Invalid` variant contains no errors (should never happen).
///
/// # Examples
///
/// ```
/// use error_rail::convert::validation_to_result;
/// use error_rail::validation::Validation;
///
/// let valid = Validation::<&str, i32>::Valid(42);
/// assert_eq!(validation_to_result(valid), Ok(42));
///
/// let invalid = Validation::<&str, i32>::invalid("error");
/// assert_eq!(validation_to_result(invalid), Err("error"));
/// ```
#[inline]
pub fn validation_to_result<T, E>(validation: Validation<E, T>) -> Result<T, E> {
    match validation {
        Validation::Valid(value) => Ok(value),
        Validation::Invalid(mut errors) => {
            let error = errors
                .pop()
                .expect("Validation::Invalid must contain at least one error");
            Err(error)
        }
    }
}

/// Converts a `Result` to a `Validation`.
///
/// # Arguments
///
/// * `result` - The result to convert
///
/// # Returns
///
/// * `Validation::Valid(value)` if result is `Ok`
/// * `Validation::Invalid([error])` if result is `Err`
///
/// # Examples
///
/// ```
/// use error_rail::convert::result_to_validation;
/// use error_rail::validation::Validation;
///
/// let ok_result: Result<i32, &str> = Ok(42);
/// let validation = result_to_validation(ok_result);
/// assert!(validation.is_valid());
///
/// let err_result: Result<i32, &str> = Err("failed");
/// let validation = result_to_validation(err_result);
/// assert!(validation.is_invalid());
/// ```
#[inline]
pub fn result_to_validation<T, E>(result: Result<T, E>) -> Validation<E, T> {
    match result {
        Ok(value) => Validation::Valid(value),
        Err(error) => Validation::invalid(error),
    }
}

/// Extracts the core error from a `ComposableError`, discarding all context.
///
/// # Arguments
///
/// * `composable` - The composable error to unwrap
///
/// # Returns
///
/// The underlying core error value
///
/// # Examples
///
/// ```
/// use error_rail::{ComposableError, convert::composable_to_core};
///
/// let composable = ComposableError::<&str>::new("error")
///     .with_context("additional context");
/// let core = composable_to_core(composable);
/// assert_eq!(core, "error");
/// ```
#[inline]
pub fn composable_to_core<E>(composable: ComposableError<E>) -> E {
    composable.into_core()
}

/// Wraps a core error in a `ComposableError` with no context.
///
/// # Arguments
///
/// * `error` - The core error to wrap
///
/// # Returns
///
/// A new `ComposableError` containing the error
///
/// # Examples
///
/// ```
/// use error_rail::{ComposableError, convert::core_to_composable};
///
/// let core_error = "something failed";
/// let composable = core_to_composable(core_error);
/// assert_eq!(composable.core_error(), &"something failed");
/// ```
#[inline]
pub fn core_to_composable<E>(error: E) -> ComposableError<E> {
    error.into()
}

/// Flattens a `Result<T, ComposableError<E>>` into `Result<T, E>`.
///
/// Strips all context and error codes, returning only the core error.
///
/// # Arguments
///
/// * `result` - The result with composable error to flatten
///
/// # Returns
///
/// A result containing only the core error type
///
/// # Examples
///
/// ```
/// use error_rail::{ComposableError, convert::flatten_composable_result};
///
/// let composable_result: Result<i32, ComposableError<&str>> =
///     Err(ComposableError::new("error").with_context("context"));
/// let flattened = flatten_composable_result(composable_result);
/// assert_eq!(flattened, Err("error"));
/// ```
#[inline]
pub fn flatten_composable_result<T, E>(result: Result<T, ComposableError<E>>) -> Result<T, E> {
    result.map_err(composable_to_core)
}

/// Wraps a plain `Result<T, E>` into `Result<T, ComposableError<E>>`.
///
/// Converts the error variant into a `ComposableError` with no context.
///
/// # Arguments
///
/// * `result` - The plain result to wrap
///
/// # Returns
///
/// A result with composable error type
///
/// # Examples
///
/// ```
/// use error_rail::convert::wrap_in_composable_result;
///
/// let plain_result: Result<i32, &str> = Err("error");
/// let wrapped = wrap_in_composable_result(plain_result);
/// assert!(wrapped.is_err());
/// ```
#[inline]
#[allow(clippy::result_large_err)]
pub fn wrap_in_composable_result<T, E>(result: Result<T, E>) -> Result<T, ComposableError<E>> {
    result.map_err(core_to_composable)
}

/// Wraps a plain `Result<T, E>` into a boxed `ComposableError`.
///
/// Similar to [`wrap_in_composable_result`] but boxes the error to reduce stack size.
///
/// # Arguments
///
/// * `result` - The plain result to wrap
///
/// # Returns
///
/// A result with boxed composable error
///
/// # Examples
///
/// ```
/// use error_rail::convert::wrap_in_composable_result_boxed;
///
/// let plain_result: Result<i32, &str> = Err("error");
/// let boxed = wrap_in_composable_result_boxed(plain_result);
/// assert!(boxed.is_err());
/// ```
#[inline]
pub fn wrap_in_composable_result_boxed<T, E>(result: Result<T, E>) -> BoxedComposableResult<T, E> {
    result.map_err(|e| Box::new(core_to_composable(e)))
}

/// Collects multiple errors into a single `Validation`.
///
/// # Arguments
///
/// * `errors` - An iterator of errors to collect
///
/// # Returns
///
/// * `Validation::Valid(())` if no errors
/// * `Validation::Invalid(errors)` if any errors present
///
/// # Examples
///
/// ```
/// use error_rail::convert::collect_errors;
/// use error_rail::validation::Validation;
///
/// let errors = vec!["error1", "error2"];
/// let validation = collect_errors(errors);
/// assert!(validation.is_invalid());
///
/// let no_errors: Vec<&str> = vec![];
/// let validation = collect_errors(no_errors);
/// assert!(validation.is_valid());
/// ```
#[inline]
pub fn collect_errors<E, I>(errors: I) -> Validation<E, ()>
where
    I: IntoIterator<Item = E>,
{
    let error_vec: ErrorVec<E> = errors.into_iter().collect();
    if error_vec.is_empty() {
        Validation::Valid(())
    } else {
        Validation::invalid_many(error_vec)
    }
}

/// Iterator returned by [`split_validation_errors`].
pub enum SplitValidationIter<T, E> {
    Valid(Option<T>),
    Invalid(<ErrorVec<E> as IntoIterator>::IntoIter),
}

impl<T, E> Iterator for SplitValidationIter<T, E> {
    type Item = Result<T, E>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Valid(opt) => opt.take().map(Ok),
            Self::Invalid(iter) => iter.next().map(Err),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            Self::Valid(opt) => {
                let len = if opt.is_some() { 1 } else { 0 };
                (len, Some(len))
            }
            Self::Invalid(iter) => iter.size_hint(),
        }
    }
}

impl<T, E> ExactSizeIterator for SplitValidationIter<T, E> {}
impl<T, E> FusedIterator for SplitValidationIter<T, E> {}

/// Splits a `Validation` into individual `Result` values.
///
/// # Arguments
///
/// * `validation` - The validation to split
///
/// # Returns
///
/// An iterator that yields:
/// * `Ok(value)` if validation is valid
/// * `Err(e)` for each error if validation is invalid
///
/// # Examples
///
/// ```
/// use error_rail::convert::split_validation_errors;
/// use error_rail::validation::Validation;
///
/// let valid = Validation::<&str, i32>::Valid(42);
/// let results: Vec<_> = split_validation_errors(valid).collect();
/// assert_eq!(results, vec![Ok(42)]);
///
/// let invalid = Validation::<&str, i32>::invalid_many(vec!["err1", "err2"]);
/// let results: Vec<_> = split_validation_errors(invalid).collect();
/// assert_eq!(results, vec![Err("err1"), Err("err2")]);
/// ```
pub fn split_validation_errors<T, E>(validation: Validation<E, T>) -> SplitValidationIter<T, E> {
    match validation {
        Validation::Valid(value) => SplitValidationIter::Valid(Some(value)),
        Validation::Invalid(errors) => SplitValidationIter::Invalid(errors.into_iter()),
    }
}
