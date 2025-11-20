use crate::traits::ErrorCategory;
use crate::traits::WithError;
use crate::validation::core::Validation;

/// Implementation of `ErrorCategory` for `Validation` types.
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
impl<E: Clone> ErrorCategory<E> for Validation<E, ()> {
    type ErrorFunctor<T: Clone> = Validation<E, T>;

    #[inline]
    fn lift<T: Clone>(value: T) -> Validation<E, T> {
        Validation::Valid(value)
    }

    #[inline]
    fn handle_error<T: Clone>(error: E) -> Validation<E, T> {
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
/// let validation: Validation<&str, i32> = Validation::invalid_many(["err1", "err2"]);
/// let mapped = validation.fmap_error(|e| format!("Error: {}", e));
/// assert_eq!(mapped.iter_errors().count(), 2);
///
/// let valid: Validation<&str, i32> = Validation::valid(42);
/// let result = valid.to_result();
/// assert_eq!(result, Ok(42));
/// ```
impl<T: Clone, E: Clone> WithError<E> for Validation<E, T> {
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

    fn to_result(self) -> Result<Self::Success, E> {
        match self {
            Validation::Valid(t) => Ok(t),
            Validation::Invalid(e) => Err(e.into_iter().next().unwrap()),
        }
    }
}
