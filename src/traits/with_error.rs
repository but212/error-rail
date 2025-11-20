/// Abstraction over types that carry an error variant which can be remapped.
///
/// This trait provides a generic interface for types that contain both success and error cases,
/// allowing transformation of the error type while preserving the success value.
///
/// # Type Parameters
///
/// * `E` - The current error type contained in the implementor
///
/// # Associated Types
///
/// * `Success` - The success value type when no error is present
/// * `ErrorOutput<G>` - The output type after mapping the error to type `G`
///
/// # Examples
///
/// ```
/// use error_rail::traits::WithError;
///
/// let result: Result<i32, &str> = Err("original error");
/// let mapped = result.fmap_error(|e| format!("Error: {}", e));
/// assert_eq!(mapped, Err("Error: original error".to_string()));
/// ```
pub trait WithError<E> {
    type Success;

    type ErrorOutput<G>;

    /// Maps the error value using `f`, producing a new container with error type `G`.
    ///
    /// This operation leaves the success case untouched and only transforms the error.
    ///
    /// # Arguments
    ///
    /// * `f` - A function that transforms the error from type `E` to type `G`
    ///
    /// # Examples
    ///
    /// ```
    /// use error_rail::traits::WithError;
    ///
    /// let result: Result<i32, u32> = Err(404);
    /// let mapped = result.fmap_error(|code| format!("HTTP {}", code));
    /// assert_eq!(mapped, Err("HTTP 404".to_string()));
    /// ```
    fn fmap_error<F, G>(self, f: F) -> Self::ErrorOutput<G>
    where
        F: Fn(E) -> G,
        G: Clone;

    /// Converts the container into a `Result`.
    ///
    /// For types that are already `Result`, this is a no-op.
    /// For other types, this extracts the success/error into standard Result form.
    ///
    /// # Examples
    ///
    /// ```
    /// use error_rail::traits::WithError;
    ///
    /// let result: Result<i32, &str> = Ok(42);
    /// assert_eq!(result.to_result(), Ok(42));
    /// ```
    fn to_result(self) -> Result<Self::Success, E>;
}

impl<T, E: Clone> WithError<E> for Result<T, E> {
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

    fn to_result(self) -> Result<Self::Success, E> {
        self
    }
}
