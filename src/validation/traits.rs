use crate::traits::ErrorCategory;
use crate::traits::WithError;
use crate::validation::core::Validation;

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

impl<T: Clone, E: Clone> WithError<E> for Validation<E, T> {
    type Success = T;
    type ErrorOutput<G> = Validation<G, T>;

    fn fmap_error<F, G>(self, f: F) -> Self::ErrorOutput<G>
    where
        F: Fn(E) -> G,
        G: Clone,
        T: Clone,
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
