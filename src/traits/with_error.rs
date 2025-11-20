pub trait WithError<E> {
    type Success;

    type ErrorOutput<G>;

    fn fmap_error<F, G>(self, f: F) -> Self::ErrorOutput<G>
    where
        F: Fn(E) -> G,
        G: Clone;

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
