use crate::types::ErrorVec;
use serde::{Deserialize, Serialize};
use smallvec::smallvec;

#[derive(Clone, PartialEq, PartialOrd, Eq, Ord, Debug, Hash, Serialize, Deserialize)]
pub enum Validation<E, A> {
    Valid(A),
    Invalid(ErrorVec<E>),
}

impl<E, A> Validation<E, A> {
    #[inline]
    pub fn valid(value: A) -> Self {
        Self::Valid(value)
    }

    #[inline]
    pub fn invalid(error: E) -> Self
    where
        E: Clone,
    {
        Self::Invalid(smallvec![error])
    }

    #[inline]
    pub fn invalid_many<I>(errors: I) -> Self
    where
        I: IntoIterator<Item = E>,
    {
        Self::Invalid(errors.into_iter().collect())
    }

    #[inline]
    pub fn is_valid(&self) -> bool {
        matches!(self, Self::Valid(_))
    }

    #[inline]
    pub fn is_invalid(&self) -> bool {
        !self.is_valid()
    }

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

    #[inline]
    pub fn map_err<F, G>(self, f: F) -> Validation<G, A>
    where
        F: Fn(E) -> G,
        A: Clone,
    {
        match self {
            Self::Valid(value) => Validation::Valid(value),
            Self::Invalid(errors) => Validation::Invalid(errors.into_iter().map(f).collect()),
        }
    }

    #[inline]
    pub fn to_result(self) -> Result<A, ErrorVec<E>> {
        match self {
            Self::Valid(value) => Ok(value),
            Self::Invalid(errors) => Err(errors),
        }
    }

    #[inline]
    pub fn from_result(result: Result<A, E>) -> Self
    where
        E: Clone,
    {
        match result {
            Ok(value) => Self::Valid(value),
            Err(error) => Self::invalid(error),
        }
    }

    #[inline]
    pub fn into_errors(self) -> Option<ErrorVec<E>> {
        match self {
            Self::Valid(_) => None,
            Self::Invalid(errors) => Some(errors),
        }
    }

    #[inline]
    pub fn into_value(self) -> Option<A> {
        match self {
            Self::Valid(value) => Some(value),
            Self::Invalid(_) => None,
        }
    }
}
