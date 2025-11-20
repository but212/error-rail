use crate::traits::*;
pub trait ErrorCategory<E> {
    type ErrorFunctor<T: Clone>: WithError<E>;

    fn lift<T: Clone>(value: T) -> Self::ErrorFunctor<T>;

    fn handle_error<T: Clone>(error: E) -> Self::ErrorFunctor<T>;
}

impl<E: Clone> ErrorCategory<E> for Result<(), E> {
    type ErrorFunctor<T: Clone> = Result<T, E>;

    #[inline]
    fn lift<T>(value: T) -> Result<T, E> {
        Ok(value)
    }

    #[inline]
    fn handle_error<T>(error: E) -> Result<T, E> {
        Err(error)
    }
}
