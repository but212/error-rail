use crate::traits::ErrorCategory;
use crate::traits::WithError;
use crate::validation::core::Validation;
use crate::ErrorVec;

/// Implementation of [`ErrorCategory`] for [`Validation`] types.
///
/// This allows `Validation<E, ()>` to act as an error category, where:
/// - `lift` creates `Valid` values
/// - `handle_error` creates `Invalid` values with a single error
///
/// # Examples
///
/// ```
/// use error_rail::traits::ErrorCategory;
/// use error_rail::validation::Validation;
///
/// let valid: Validation<String, i32> = <Validation<String, ()>>::lift(42);
/// assert!(valid.is_valid());
///
/// let invalid: Validation<String, i32> = <Validation<String, ()>>::handle_error("error".to_string());
/// assert!(invalid.is_invalid());
/// ```
impl<E> ErrorCategory<E> for Validation<E, ()> {
    type ErrorFunctor<T> = Validation<E, T>;

    #[inline]
    fn lift<T>(value: T) -> Validation<E, T> {
        Validation::Valid(value)
    }

    #[inline]
    fn handle_error<T>(error: E) -> Validation<E, T> {
        Validation::invalid(error)
    }
}

/// Implementation of `WithError` for `Validation` types.
///
/// This allows transforming the error type of a validation while preserving
/// the success value and accumulating all errors through the transformation.
///
/// # Examples
///
/// ```
/// use error_rail::traits::WithError;
/// use error_rail::validation::Validation;
///
/// let validation: Validation<&str, i32> = Validation::invalid_many(vec!["err1", "err2"]);
/// let mapped = validation.fmap_error(|e| format!("Error: {}", e));
/// assert_eq!(mapped.iter_errors().count(), 2);
///
/// let valid: Validation<&str, i32> = Validation::valid(42);
/// let result = valid.to_result();
/// assert_eq!(result, Ok(42));
/// ```
impl<T, E> WithError<E> for Validation<E, T> {
    type Success = T;
    type ErrorOutput<G> = Validation<G, T>;

    fn fmap_error<F, G>(self, f: F) -> Self::ErrorOutput<G>
    where
        F: Fn(E) -> G,
    {
        match self {
            Validation::Valid(t) => Validation::Valid(t),
            Validation::Invalid(e) => Validation::Invalid(e.into_iter().map(f).collect()),
        }
    }

    /// Converts the validation to a result, taking only the first error if invalid.
    ///
    /// **⚠️ DEPRECATED**: Use [`to_result_first()`](Self::to_result_first) or
    /// [`to_result_all()`](Self::to_result_all) for explicit error handling.
    /// This method loses additional errors in multi-error scenarios.
    ///
    /// # Returns
    ///
    /// * `Ok(value)` if validation is valid
    /// * `Err(first_error)` if validation is invalid (only the first error)
    fn to_result(self) -> Result<Self::Success, E> {
        self.to_result_first()
    }

    /// Converts the validation to a result, taking only the first error if invalid.
    ///
    /// This method explicitly indicates that only the first error will be returned,
    /// potentially losing additional errors in multi-error scenarios.
    ///
    /// # Returns
    ///
    /// * `Ok(value)` if validation is valid
    /// * `Err(first_error)` if validation is invalid (only the first error)
    ///
    /// # Examples
    ///
    /// ```
    /// use error_rail::validation::Validation;
    ///
    /// let valid = Validation::<&str, i32>::valid(42);
    /// assert_eq!(valid.to_result_first(), Ok(42));
    ///
    /// let invalid = Validation::<&str, i32>::invalid_many(vec!["error1", "error2"]);
    /// assert_eq!(invalid.to_result_first(), Err("error1"));
    /// ```
    fn to_result_first(self) -> Result<Self::Success, E> {
        match self {
            Validation::Valid(t) => Ok(t),
            Validation::Invalid(e) => Err(e.into_iter().next().unwrap()),
        }
    }

    /// Converts the validation to a result, preserving all errors if invalid.
    ///
    /// This method returns all accumulated errors in a `Vec<E>`, ensuring no error
    /// information is lost during the conversion.
    ///
    /// # Returns
    ///
    /// * `Ok(value)` if validation is valid
    /// * `Err(all_errors)` if validation is invalid (all errors in a Vec)
    ///
    /// # Examples
    ///
    /// ```
    /// use error_rail::validation::Validation;
    ///
    /// let valid = Validation::<&str, i32>::valid(42);
    /// assert_eq!(valid.to_result_all(), Ok(42));
    ///
    /// let invalid = Validation::<&str, i32>::invalid_many(vec!["error1", "error2"]);
    /// assert_eq!(invalid.to_result_all(), Err(vec!["error1", "error2"]));
    /// ```
    fn to_result_all(self) -> Result<Self::Success, ErrorVec<E>> {
        match self {
            Validation::Valid(t) => Ok(t),
            Validation::Invalid(e) => Err(e),
        }
    }
}
