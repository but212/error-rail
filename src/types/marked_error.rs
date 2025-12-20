use crate::traits::TransientError;
use core::fmt::{Debug, Display};

/// An error wrapper that marks an error as transient or permanent based on a closure.
///
/// This type is primarily used by the `.mark_transient_if()` method on pipelines,
/// allowing users to dynamically classify errors without implementing the
/// [`crate::traits::TransientError`] trait for their specific error types.
pub struct MarkedError<E, F> {
    pub(crate) inner: E,
    pub(crate) classifier: F,
}

impl<E, F> MarkedError<E, F> {
    /// Returns a reference to the inner error.
    pub fn inner(&self) -> &E {
        &self.inner
    }

    /// Consumes the marked error, returning the inner error.
    pub fn into_inner(self) -> E {
        self.inner
    }
}

impl<E: Debug, F> Debug for MarkedError<E, F> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("MarkedError")
            .field("inner", &self.inner)
            .finish_non_exhaustive()
    }
}

impl<E: Display, F> Display for MarkedError<E, F> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Display::fmt(&self.inner, f)
    }
}

impl<E, F> TransientError for MarkedError<E, F>
where
    F: Fn(&E) -> bool,
{
    fn is_transient(&self) -> bool {
        (self.classifier)(&self.inner)
    }
}

impl<E, F> core::error::Error for MarkedError<E, F>
where
    E: core::error::Error + 'static,
    F: Fn(&E) -> bool,
{
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        Some(&self.inner)
    }
}
