use crate::traits::with_error::WithError;

pub trait ErrorOps<E>: WithError<E> {
    fn recover<F>(self, recovery: F) -> Self
    where
        F: FnOnce(E) -> Self,
        Self: Sized;

    fn bimap_result<B, F, SuccessF, ErrorF>(
        self,
        success_f: SuccessF,
        error_f: ErrorF,
    ) -> Result<B, F>
    where
        SuccessF: FnOnce(Self::Success) -> B,
        ErrorF: FnOnce(E) -> F,
        Self: Sized;
}

impl<T: Clone, E: Clone> ErrorOps<E> for Result<T, E> {
    #[inline]
    fn recover<F>(self, recovery: F) -> Self
    where
        F: FnOnce(E) -> Self,
    {
        match self {
            Ok(value) => Ok(value),
            Err(error) => recovery(error),
        }
    }

    #[inline]
    fn bimap_result<B, F, SuccessF, ErrorF>(
        self,
        success_f: SuccessF,
        error_f: ErrorF,
    ) -> Result<B, F>
    where
        SuccessF: FnOnce(T) -> B,
        ErrorF: FnOnce(E) -> F,
    {
        match self {
            Ok(value) => Ok(success_f(value)),
            Err(error) => Err(error_f(error)),
        }
    }
}
