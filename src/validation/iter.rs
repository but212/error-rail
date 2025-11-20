use crate::validation::core::Validation;
use std::slice::{Iter as SliceIter, IterMut as SliceIterMut};

/// Iterator over the valid value of a Validation.
/// Yields 0 or 1 item.
pub struct Iter<'a, A> {
    inner: Option<&'a A>,
}

impl<'a, A> Iterator for Iter<'a, A> {
    type Item = &'a A;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.take()
    }
}

/// Mutable iterator over the valid value of a Validation.
/// Yields 0 or 1 item.
pub struct IterMut<'a, A> {
    inner: Option<&'a mut A>,
}

impl<'a, A> Iterator for IterMut<'a, A> {
    type Item = &'a mut A;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.take()
    }
}

/// Iterator over the errors of a Validation.
pub enum ErrorsIter<'a, E> {
    Empty,
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
}

/// Mutable iterator over the errors of a Validation.
pub enum ErrorsIterMut<'a, E> {
    Empty,
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
}

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

/// Owning iterator over the valid value of a Validation.
pub struct IntoIter<A> {
    inner: Option<A>,
}

impl<A> Iterator for IntoIter<A> {
    type Item = A;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.take()
    }
}

impl<'a, E, A> IntoIterator for &'a Validation<E, A> {
    type Item = &'a A;
    type IntoIter = Iter<'a, A>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

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
    pub fn iter_mut(&mut self) -> IterMut<'_, A> {
        match self {
            Validation::Valid(a) => IterMut { inner: Some(a) },
            _ => IterMut { inner: None },
        }
    }

    /// Returns an iterator over the errors.
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
    /// ```
    pub fn iter_errors(&self) -> ErrorsIter<'_, E> {
        match self {
            Self::Valid(_) => ErrorsIter::Empty,
            Self::Invalid(errors) => ErrorsIter::Multi(errors.iter()),
        }
    }

    /// Returns a mutable iterator over the errors.
    pub fn iter_errors_mut(&mut self) -> ErrorsIterMut<'_, E> {
        match self {
            Validation::Invalid(es) => ErrorsIterMut::Multi(es.iter_mut()),
            _ => ErrorsIterMut::Empty,
        }
    }
}
