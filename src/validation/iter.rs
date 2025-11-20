use crate::validation::core::Validation;

pub struct Iter<'a, A> {
    inner: Option<&'a A>,
}

impl<'a, A> Iterator for Iter<'a, A> {
    type Item = &'a A;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.take()
    }
}

pub struct IterMut<'a, A> {
    inner: Option<&'a mut A>,
}

impl<'a, A> Iterator for IterMut<'a, A> {
    type Item = &'a mut A;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.take()
    }
}

pub enum ErrorsIter<'a, E> {
    Empty,
    Multi(smallvec::alloc::slice::Iter<'a, E>),
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

pub enum ErrorsIterMut<'a, E> {
    Empty,
    Multi(smallvec::alloc::slice::IterMut<'a, E>),
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
    pub fn iter(&self) -> Iter<'_, A> {
        match self {
            Validation::Valid(a) => Iter { inner: Some(a) },
            _ => Iter { inner: None },
        }
    }

    pub fn iter_mut(&mut self) -> IterMut<'_, A> {
        match self {
            Validation::Valid(a) => IterMut { inner: Some(a) },
            _ => IterMut { inner: None },
        }
    }

    pub fn iter_errors(&self) -> impl Iterator<Item = &E> {
        match self {
            Self::Valid(_) => [].iter(),
            Self::Invalid(errors) => errors.iter(),
        }
    }

    pub fn iter_errors_mut(&mut self) -> ErrorsIterMut<'_, E> {
        match self {
            Validation::Invalid(es) => ErrorsIterMut::Multi(es.iter_mut()),
            _ => ErrorsIterMut::Empty,
        }
    }
}
