use crate::{traits::IntoErrorContext, types::error_context::ErrorContext};

#[repr(transparent)]
pub struct LazyContext<F> {
    generator: F,
}

impl<F> LazyContext<F> {
    #[inline]
    pub fn new(generator: F) -> Self {
        Self { generator }
    }
}

impl<F> IntoErrorContext for LazyContext<F>
where
    F: FnOnce() -> String,
{
    #[inline]
    fn into_error_context(self) -> ErrorContext {
        ErrorContext::new((self.generator)())
    }
}
