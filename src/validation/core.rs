use crate::types::ErrorVec;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use smallvec::smallvec;

/// Applicative-style validation that accumulates many errors instead of failing fast.
///
/// `Validation<E, A>` represents a computation that either succeeds with a value of type `A`
/// or fails with one or more errors of type `E`. Unlike `Result`, which fails fast on the first
/// error, `Validation` accumulates all errors, making it ideal for form validation and other
/// scenarios where you want to collect all problems at once.
///
/// # Serde Support
///
/// `Validation` implements `Serialize` and `Deserialize` when `E` and `A` do.
/// This makes it easy to use in API responses or configuration files.
///
/// # Type Parameters
///
/// * `E` - The error type
/// * `A` - The success value type
///
/// # Variants
///
/// * `Valid(A)` - Contains a successful value
/// * `Invalid(ErrorVec<E>)` - Contains one or more errors
///
/// # Examples
///
/// ```
/// use error_rail::validation::Validation;
///
/// let valid = Validation::<&str, i32>::valid(42);
/// assert!(valid.is_valid());
///
/// let invalid = Validation::<&str, i32>::invalid("error");
/// assert!(invalid.is_invalid());
/// ```
#[must_use]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
pub enum Validation<E, A> {
    Valid(A),
    Invalid(ErrorVec<E>),
}

impl<E, A> Validation<E, A> {
    /// Creates a valid value.
    ///
    /// # Arguments
    ///
    /// * `value` - The success value to wrap
    ///
    /// # Examples
    ///
    /// ```
    /// use error_rail::validation::Validation;
    ///
    /// let v = Validation::<&str, i32>::valid(42);
    /// assert_eq!(v.into_value(), Some(42));
    /// ```
    #[must_use]
    #[inline]
    pub fn valid(value: A) -> Self {
        Self::Valid(value)
    }

    /// Creates an invalid value from a single error.
    ///
    /// # Arguments
    ///
    /// * `error` - The error to wrap
    ///
    /// # Examples
    ///
    /// ```
    /// use error_rail::validation::Validation;
    ///
    /// let v = Validation::<&str, ()>::invalid("missing field");
    /// assert!(v.is_invalid());
    /// ```
    #[must_use]
    #[inline]
    pub fn invalid(error: E) -> Self {
        Self::Invalid(smallvec![error])
    }

    /// Creates an invalid value from an iterator of errors.
    ///
    /// # Arguments
    ///
    /// * `errors` - An iterator of errors to collect
    ///
    /// # Examples
    ///
    /// ```
    /// use error_rail::validation::Validation;
    ///
    /// let v = Validation::<&str, ()>::invalid_many(["missing", "invalid"]);
    /// assert!(v.is_invalid());
    /// assert_eq!(v.into_errors().unwrap().len(), 2);
    /// ```
    #[must_use]
    #[inline]
    pub fn invalid_many<I>(errors: I) -> Self
    where
        I: IntoIterator<Item = E>,
    {
        Self::Invalid(errors.into_iter().collect())
    }

    /// Returns `true` if the validation contains a value.
    ///
    /// # Examples
    ///
    /// ```
    /// use error_rail::validation::Validation;
    ///
    /// let v = Validation::<&str, i32>::valid(42);
    /// assert!(v.is_valid());
    /// ```
    #[must_use]
    #[inline]
    pub fn is_valid(&self) -> bool {
        matches!(self, Self::Valid(_))
    }

    /// Returns `true` if the validation contains errors.
    ///
    /// # Examples
    ///
    /// ```
    /// use error_rail::validation::Validation;
    ///
    /// let v = Validation::<&str, i32>::invalid("error");
    /// assert!(v.is_invalid());
    /// ```
    #[must_use]
    #[inline]
    pub fn is_invalid(&self) -> bool {
        !self.is_valid()
    }

    /// Maps the valid value using the provided function.
    ///
    /// If the validation is invalid, the errors are preserved unchanged.
    ///
    /// # Arguments
    ///
    /// * `f` - A function that transforms the success value from type `A` to type `B`
    ///
    /// # Examples
    ///
    /// ```
    /// use error_rail::validation::Validation;
    ///
    /// let v = Validation::<&str, i32>::valid(21);
    /// let doubled = v.map(|x| x * 2);
    /// assert_eq!(doubled.into_value(), Some(42));
    /// ```
    #[must_use]
    #[inline]
    pub fn map<B, F>(self, f: F) -> Validation<E, B>
    where
        F: FnOnce(A) -> B,
    {
        match self {
            Self::Valid(value) => Validation::Valid(f(value)),
            Self::Invalid(errors) => Validation::Invalid(errors),
        }
    }

    /// Chains computations that may produce additional validation errors.
    ///
    /// Behaves like [`Result::and_then`], propagating invalid states while
    /// invoking `f` only when the current validation is valid.
    ///
    /// # Arguments
    ///
    /// * `f` - Function producing the next validation step
    ///
    /// # Examples
    ///
    /// ```
    /// use error_rail::validation::Validation;
    ///
    /// fn parse_even(input: i32) -> Validation<&'static str, i32> {
    ///     if input % 2 == 0 {
    ///         Validation::valid(input)
    ///     } else {
    ///         Validation::invalid("not even")
    ///     }
    /// }
    ///
    /// let result = Validation::valid(4).and_then(parse_even);
    /// assert_eq!(result.into_value(), Some(4));
    ///
    /// let invalid = Validation::valid(3).and_then(parse_even);
    /// assert!(invalid.is_invalid());
    /// ```
    #[must_use]
    #[inline]
    pub fn and_then<B, F>(self, f: F) -> Validation<E, B>
    where
        F: FnOnce(A) -> Validation<E, B>,
    {
        match self {
            Self::Valid(value) => f(value),
            Self::Invalid(errors) => Validation::Invalid(errors),
        }
    }

    /// Calls `op` if the validation is invalid, otherwise returns the `Valid` value.
    ///
    /// This function can be used for control flow based on validation results.
    ///
    /// # Arguments
    ///
    /// * `op` - The function to call if the validation is invalid.
    ///
    /// # Examples
    ///
    /// ```
    /// use error_rail::validation::Validation;
    ///
    /// let v = Validation::<&str, i32>::invalid("error");
    /// let res = v.or_else(|_errs| Validation::valid(42));
    /// assert_eq!(res.into_value(), Some(42));
    /// ```
    #[must_use]
    #[inline]
    pub fn or_else<F>(self, op: F) -> Validation<E, A>
    where
        F: FnOnce(ErrorVec<E>) -> Validation<E, A>,
    {
        match self {
            Self::Valid(value) => Validation::Valid(value),
            Self::Invalid(errors) => op(errors),
        }
    }

    /// Combines two validations into a tuple, accumulating all errors.
    ///
    /// If both validations are valid, returns a tuple of both values.
    /// If either or both are invalid, accumulates all errors from both.
    ///
    /// # Arguments
    ///
    /// * `other` - Another validation to combine with this one
    ///
    /// # Examples
    ///
    /// ```
    /// use error_rail::validation::Validation;
    ///
    /// let v1 = Validation::<&str, i32>::valid(42);
    /// let v2 = Validation::<&str, i32>::valid(21);
    /// let result = v1.zip(v2);
    /// assert_eq!(result.into_value(), Some((42, 21)));
    ///
    /// let v3 = Validation::<&str, i32>::invalid("error1");
    /// let v4 = Validation::<&str, i32>::invalid("error2");
    /// let result = v3.zip(v4);
    /// assert_eq!(result.into_errors().unwrap().len(), 2);
    /// ```
    #[must_use]
    #[inline]
    pub fn zip<B>(self, other: Validation<E, B>) -> Validation<E, (A, B)> {
        match (self, other) {
            (Validation::Valid(a), Validation::Valid(b)) => Validation::Valid((a, b)),
            (Validation::Invalid(e), Validation::Valid(_)) => Validation::Invalid(e),
            (Validation::Valid(_), Validation::Invalid(e)) => Validation::Invalid(e),
            (Validation::Invalid(mut e1), Validation::Invalid(e2)) => {
                e1.extend(e2);
                Validation::Invalid(e1)
            }
        }
    }

    /// Maps each error while preserving the success branch.
    ///
    /// Transforms all accumulated errors using the provided function,
    /// leaving valid values unchanged.
    ///
    /// # Arguments
    ///
    /// * `f` - A function that transforms errors from type `E` to type `G`
    ///
    /// # Examples
    ///
    /// ```
    /// use error_rail::validation::Validation;
    ///
    /// let v = Validation::<&str, i32>::invalid("error");
    /// let mapped = v.map_err(|e| format!("Error: {}", e));
    /// assert!(mapped.is_invalid());
    /// ```
    #[must_use]
    #[inline]
    pub fn map_err<F, G>(self, f: F) -> Validation<G, A>
    where
        F: Fn(E) -> G,
    {
        match self {
            Self::Valid(value) => Validation::Valid(value),
            Self::Invalid(errors) => Validation::Invalid(errors.into_iter().map(f).collect()),
        }
    }

    /// Converts into a `Result`, losing error accumulation if invalid.
    ///
    /// The success value becomes `Ok`, and all accumulated errors become `Err`.
    ///
    /// # Examples
    ///
    /// ```
    /// use error_rail::validation::Validation;
    ///
    /// let v = Validation::<&str, i32>::valid(42);
    /// assert_eq!(v.to_result(), Ok(42));
    ///
    /// let v = Validation::<&str, i32>::invalid("error");
    /// assert!(v.to_result().is_err());
    /// ```
    #[must_use]
    #[inline]
    pub fn to_result(self) -> Result<A, ErrorVec<E>> {
        match self {
            Self::Valid(value) => Ok(value),
            Self::Invalid(errors) => Err(errors),
        }
    }

    /// Wraps a normal `Result` into a `Validation`, turning the error side into a singleton vec.
    ///
    /// # Arguments
    ///
    /// * `result` - The result to convert
    ///
    /// # Examples
    ///
    /// ```
    /// use error_rail::validation::Validation;
    ///
    /// let result: Result<i32, &str> = Ok(42);
    /// let v = Validation::from_result(result);
    /// assert!(v.is_valid());
    /// ```
    #[must_use]
    #[inline]
    pub fn from_result(result: Result<A, E>) -> Self {
        match result {
            Ok(value) => Self::Valid(value),
            Err(error) => Self::invalid(error),
        }
    }

    /// Extracts the error list, if any.
    ///
    /// Returns `Some(errors)` if invalid, `None` if valid.
    ///
    /// # Examples
    ///
    /// ```
    /// use error_rail::validation::Validation;
    ///
    /// let v = Validation::<&str, i32>::invalid("error");
    /// assert_eq!(v.into_errors().unwrap().len(), 1);
    /// ```
    #[must_use]
    #[inline]
    pub fn into_errors(self) -> Option<ErrorVec<E>> {
        match self {
            Self::Valid(_) => None,
            Self::Invalid(errors) => Some(errors),
        }
    }

    /// Extracts the value, if valid.
    ///
    /// Returns `Some(value)` if valid, `None` if invalid.
    ///
    /// # Examples
    ///
    /// ```
    /// use error_rail::validation::Validation;
    ///
    /// let v = Validation::<&str, i32>::valid(42);
    /// assert_eq!(v.into_value(), Some(42));
    /// ```
    #[must_use]
    #[inline]
    pub fn into_value(self) -> Option<A> {
        match self {
            Self::Valid(value) => Some(value),
            Self::Invalid(_) => None,
        }
    }
}
