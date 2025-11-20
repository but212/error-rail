use crate::traits::*;

/// Trait for types that can lift values and handle errors in a functorial way.
///
/// This trait provides a categorical abstraction over error-handling types,
/// allowing them to:
/// - Lift pure values into the error context (`lift`)
/// - Construct error cases from error values (`handle_error`)
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
/// let success: Result<i32, &str> = <Result<(), &str>>::lift(42);
/// assert_eq!(success, Ok(42));
///
/// let failure: Result<i32, &str> = <Result<(), &str>>::handle_error("error");
/// assert_eq!(failure, Err("error"));
/// ```
pub trait ErrorCategory<E> {
    /// The functor type that wraps values with error handling capability.
    type ErrorFunctor<T>: WithError<E>;

    /// Lifts a pure value into the error functor context.
    ///
    /// # Arguments
    ///
    /// * `value` - The value to lift into the error context
    ///
    /// # Examples
    ///
    /// ```
    /// use error_rail::traits::ErrorCategory;
    ///
    /// let result: Result<i32, &str> = <Result<(), &str>>::lift(42);
    /// assert_eq!(result, Ok(42));
    /// ```
    fn lift<T>(value: T) -> Self::ErrorFunctor<T>;

    /// Constructs an error case from an error value.
    ///
    /// # Arguments
    ///
    /// * `error` - The error value to wrap
    ///
    /// # Examples
    ///
    /// ```
    /// use error_rail::traits::ErrorCategory;
    ///
    /// let result: Result<i32, &str> = <Result<(), &str>>::handle_error("failed");
    /// assert_eq!(result, Err("failed"));
    /// ```
    fn handle_error<T>(error: E) -> Self::ErrorFunctor<T>;
}

/// Implementation of `ErrorCategory` for `Result` types.
///
/// This allows `Result<(), E>` to act as an error category, where:
/// - `lift` creates `Ok` values
/// - `handle_error` creates `Err` values
///
/// # Examples
///
/// ```
/// use error_rail::traits::ErrorCategory;
///
/// let ok_value: Result<i32, String> = <Result<(), String>>::lift(100);
/// assert_eq!(ok_value, Ok(100));
///
/// let err_value: Result<i32, String> = <Result<(), String>>::handle_error("error".to_string());
/// assert_eq!(err_value, Err("error".to_string()));
/// ```
impl<E: Clone> ErrorCategory<E> for Result<(), E> {
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
