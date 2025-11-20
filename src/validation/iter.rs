use crate::{ErrorVec, validation::core::Validation};
use std::slice::{Iter as SliceIter, IterMut as SliceIterMut};

/// Iterator over the valid value of a [`Validation`].
///
/// This iterator yields at most one item (the valid value if present).
/// It is created by the [`Validation::iter`] method.
///
/// # Examples
///
/// ```
/// use error_rail::validation::Validation;
///
/// let valid: Validation<String, i32> = Validation::Valid(5);
/// let mut iter = valid.iter();
/// assert_eq!(iter.next(), Some(&5));
/// assert_eq!(iter.next(), None);
/// ```
pub struct Iter<'a, A> {
    inner: Option<&'a A>,
}

impl<'a, A> Iterator for Iter<'a, A> {
    type Item = &'a A;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.take()
    }
}

impl<'a, A> ExactSizeIterator for Iter<'a, A> {
    fn len(&self) -> usize {
        self.inner.is_some() as usize
    }
}

/// Mutable iterator over the valid value of a [`Validation`].
///
/// This iterator yields at most one mutable reference to the valid value if present.
/// It is created by the [`Validation::iter_mut`] method.
///
/// # Examples
///
/// ```
/// use error_rail::validation::Validation;
///
/// let mut valid: Validation<String, i32> = Validation::Valid(5);
/// for value in valid.iter_mut() {
///     *value += 10;
/// }
/// assert_eq!(valid, Validation::Valid(15));
/// ```
pub struct IterMut<'a, A> {
    inner: Option<&'a mut A>,
}

impl<'a, A> Iterator for IterMut<'a, A> {
    type Item = &'a mut A;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.take()
    }
}

impl<'a, A> ExactSizeIterator for IterMut<'a, A> {
    fn len(&self) -> usize {
        self.inner.is_some() as usize
    }
}

/// Iterator over the errors of a [`Validation`].
///
/// This iterator yields references to all accumulated errors in an invalid validation.
/// For valid validations, it yields no items.
///
/// # Examples
///
/// ```
/// use error_rail::validation::Validation;
///
/// let invalid: Validation<&str, i32> = Validation::invalid_many(["error1", "error2"]);
/// let errors: Vec<_> = invalid.iter_errors().collect();
/// assert_eq!(errors, vec![&"error1", &"error2"]);
/// ```
pub enum ErrorsIter<'a, E> {
    /// No errors present (valid validation)
    Empty,
    /// Multiple errors present (invalid validation)
    Multi(SliceIter<'a, E>),
}

impl<'a, E> Iterator for ErrorsIter<'a, E> {
    type Item = &'a E;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            ErrorsIter::Empty => None,
            ErrorsIter::Multi(it) => it.next(),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            ErrorsIter::Empty => (0, Some(0)),
            ErrorsIter::Multi(it) => it.size_hint(),
        }
    }
}

impl<'a, E> ExactSizeIterator for ErrorsIter<'a, E> {
    fn len(&self) -> usize {
        match self {
            ErrorsIter::Empty => 0,
            ErrorsIter::Multi(it) => it.len(),
        }
    }
}

/// Mutable iterator over the errors of a [`Validation`].
///
/// This iterator yields mutable references to all accumulated errors in an invalid validation.
/// For valid validations, it yields no items.
///
/// # Examples
///
/// ```
/// use error_rail::validation::Validation;
///
/// let mut invalid: Validation<String, i32> = Validation::invalid_many([
///     "error1".to_string(),
///     "error2".to_string()
/// ]);
///
/// for error in invalid.iter_errors_mut() {
///     error.push_str(" [modified]");
/// }
/// ```
pub enum ErrorsIterMut<'a, E> {
    /// No errors present (valid validation)
    Empty,
    /// Multiple errors present (invalid validation)
    Multi(SliceIterMut<'a, E>),
}

impl<'a, E> Iterator for ErrorsIterMut<'a, E> {
    type Item = &'a mut E;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            ErrorsIterMut::Empty => None,
            ErrorsIterMut::Multi(it) => it.next(),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            ErrorsIterMut::Empty => (0, Some(0)),
            ErrorsIterMut::Multi(it) => it.size_hint(),
        }
    }
}

impl<'a, E> ExactSizeIterator for ErrorsIterMut<'a, E> {
    fn len(&self) -> usize {
        match self {
            ErrorsIterMut::Empty => 0,
            ErrorsIterMut::Multi(it) => it.len(),
        }
    }
}

/// Converts a [`Validation`] into an iterator over its valid value.
///
/// This consumes the validation and yields at most one item.
///
/// # Examples
///
/// ```
/// use error_rail::validation::Validation;
///
/// let valid: Validation<String, i32> = Validation::Valid(42);
/// let values: Vec<_> = valid.into_iter().collect();
/// assert_eq!(values, vec![42]);
///
/// let invalid: Validation<String, i32> = Validation::invalid("error".to_string());
/// let values: Vec<_> = invalid.into_iter().collect();
/// assert!(values.is_empty());
/// ```
impl<E, A> IntoIterator for Validation<E, A> {
    type Item = A;
    type IntoIter = IntoIter<A>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            Validation::Valid(a) => IntoIter { inner: Some(a) },
            _ => IntoIter { inner: None },
        }
    }
}

/// Owning iterator over the valid value of a [`Validation`].
///
/// This iterator takes ownership of the validation and yields at most one item.
/// It is created by calling [`IntoIterator::into_iter`] on a [`Validation`].
///
/// # Examples
///
/// ```
/// use error_rail::validation::Validation;
///
/// let valid: Validation<String, i32> = Validation::Valid(100);
/// let mut iter = valid.into_iter();
/// assert_eq!(iter.next(), Some(100));
/// assert_eq!(iter.next(), None);
/// ```
pub struct IntoIter<A> {
    inner: Option<A>,
}

impl<A> Iterator for IntoIter<A> {
    type Item = A;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.take()
    }
}

impl<A> ExactSizeIterator for IntoIter<A> {
    fn len(&self) -> usize {
        self.inner.is_some() as usize
    }
}

/// Converts a reference to a [`Validation`] into an iterator.
///
/// This allows for convenient iteration over the valid value without consuming the validation.
///
/// # Examples
///
/// ```
/// use error_rail::validation::Validation;
///
/// let valid: Validation<String, i32> = Validation::Valid(5);
/// for value in &valid {
///     assert_eq!(value, &5);
/// }
/// // valid is still usable here
/// ```
impl<'a, E, A> IntoIterator for &'a Validation<E, A> {
    type Item = &'a A;
    type IntoIter = Iter<'a, A>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// Converts a mutable reference to a [`Validation`] into an iterator.
///
/// This allows for convenient mutable iteration over the valid value without consuming the validation.
///
/// # Examples
///
/// ```
/// use error_rail::validation::Validation;
///
/// let mut valid: Validation<String, i32> = Validation::Valid(5);
/// for value in &mut valid {
///     *value *= 2;
/// }
/// assert_eq!(valid, Validation::Valid(10));
/// ```
impl<'a, E, A> IntoIterator for &'a mut Validation<E, A> {
    type Item = &'a mut A;
    type IntoIter = IterMut<'a, A>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<E, A> Validation<E, A> {
    /// Returns an iterator over the valid value.
    ///
    /// The iterator yields at most one item - the valid value if present.
    /// For invalid validations, the iterator yields nothing.
    ///
    /// # Examples
    ///
    /// ```
    /// use error_rail::validation::Validation;
    ///
    /// let valid: Validation<String, i32> = Validation::Valid(5);
    /// assert_eq!(valid.iter().next(), Some(&5));
    ///
    /// let invalid: Validation<String, i32> = Validation::invalid("error".to_string());
    /// assert_eq!(invalid.iter().next(), None);
    /// ```
    pub fn iter(&self) -> Iter<'_, A> {
        match self {
            Validation::Valid(a) => Iter { inner: Some(a) },
            _ => Iter { inner: None },
        }
    }

    /// Returns a mutable iterator over the valid value.
    ///
    /// The iterator yields at most one mutable reference - to the valid value if present.
    /// For invalid validations, the iterator yields nothing.
    ///
    /// # Examples
    ///
    /// ```
    /// use error_rail::validation::Validation;
    ///
    /// let mut valid: Validation<String, i32> = Validation::Valid(5);
    /// if let Some(value) = valid.iter_mut().next() {
    ///     *value = 10;
    /// }
    /// assert_eq!(valid, Validation::Valid(10));
    /// ```
    pub fn iter_mut(&mut self) -> IterMut<'_, A> {
        match self {
            Validation::Valid(a) => IterMut { inner: Some(a) },
            _ => IterMut { inner: None },
        }
    }

    /// Returns an iterator over the errors.
    ///
    /// For invalid validations, yields references to all accumulated errors.
    /// For valid validations, yields nothing.
    ///
    /// # Examples
    ///
    /// ```
    /// use error_rail::validation::Validation;
    ///
    /// let valid: Validation<String, i32> = Validation::Valid(5);
    /// assert_eq!(valid.iter_errors().next(), None);
    ///
    /// let invalid: Validation<String, i32> = Validation::invalid("error".to_string());
    /// assert_eq!(invalid.iter_errors().next(), Some(&"error".to_string()));
    ///
    /// let multi_invalid: Validation<&str, i32> = Validation::invalid_many(["err1", "err2"]);
    /// assert_eq!(multi_invalid.iter_errors().count(), 2);
    /// ```
    pub fn iter_errors(&self) -> ErrorsIter<'_, E> {
        match self {
            Self::Valid(_) => ErrorsIter::Empty,
            Self::Invalid(errors) => ErrorsIter::Multi(errors.iter()),
        }
    }

    /// Returns a mutable iterator over the errors.
    ///
    /// For invalid validations, yields mutable references to all accumulated errors.
    /// For valid validations, yields nothing.
    ///
    /// # Examples
    ///
    /// ```
    /// use error_rail::validation::Validation;
    ///
    /// let mut invalid: Validation<String, i32> = Validation::invalid("error".to_string());
    /// for error in invalid.iter_errors_mut() {
    ///     error.push_str(" [updated]");
    /// }
    /// assert_eq!(
    ///     invalid.iter_errors().next(),
    ///     Some(&"error [updated]".to_string())
    /// );
    /// ```
    pub fn iter_errors_mut(&mut self) -> ErrorsIterMut<'_, E> {
        match self {
            Validation::Invalid(es) => ErrorsIterMut::Multi(es.iter_mut()),
            _ => ErrorsIterMut::Empty,
        }
    }
}

/// Collects an iterator of `Result`s into a single `Validation`, aggregating all errors.
///
/// If all results are `Ok`, returns `Valid` with a vector of all success values.
/// If any results are `Err`, returns `Invalid` with all accumulated errors.
///
/// This is useful for validating multiple operations and collecting all failures
/// instead of short-circuiting on the first error.
///
/// # Examples
///
/// ```
/// use error_rail::validation::Validation;
///
/// let inputs = vec![Ok(1), Err("oops"), Ok(2)];
/// let collected: Validation<&str, Vec<i32>> = inputs.into_iter().collect();
/// assert!(collected.is_invalid());
/// assert_eq!(collected.into_errors().unwrap()[0], "oops");
///
/// let all_ok = vec![Ok(1), Ok(2), Ok(3)];
/// let collected: Validation<&str, Vec<i32>> = all_ok.into_iter().collect();
/// assert_eq!(collected, Validation::Valid(vec![1, 2, 3]));
/// ```
impl<E, A> FromIterator<Result<A, E>> for Validation<E, Vec<A>> {
    fn from_iter<T: IntoIterator<Item = Result<A, E>>>(iter: T) -> Self {
        let mut values = Vec::new();
        let mut errors = ErrorVec::new();

        for item in iter {
            match item {
                Ok(v) => values.push(v),
                Err(e) => errors.push(e),
            }
        }

        if errors.is_empty() {
            Validation::Valid(values)
        } else {
            Validation::Invalid(errors)
        }
    }
}

/// Collects an iterator of `Validation`s into a single `Validation`, aggregating all errors.
///
/// If all validations are valid, returns `Valid` with a vector of all success values.
/// If any validations are invalid, returns `Invalid` with all accumulated errors from all
/// invalid validations.
///
/// This is the primary way to combine multiple validations while preserving all error information.
///
/// # Examples
///
/// ```
/// use error_rail::validation::Validation;
///
/// let items = vec![Validation::valid(1), Validation::invalid("bad")];
/// let collected: Validation<&str, Vec<i32>> = items.into_iter().collect();
/// assert!(collected.is_invalid());
/// ```
impl<E, A> FromIterator<Validation<E, A>> for Validation<E, Vec<A>> {
    fn from_iter<T: IntoIterator<Item = Validation<E, A>>>(iter: T) -> Self {
        let mut values = Vec::new();
        let mut errors = ErrorVec::new();

        for item in iter {
            match item {
                Validation::Valid(v) => values.push(v),
                Validation::Invalid(es) => errors.extend(es),
            }
        }

        if errors.is_empty() {
            Validation::Valid(values)
        } else {
            Validation::Invalid(errors)
        }
    }
}
