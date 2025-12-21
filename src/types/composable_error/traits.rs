use super::ComposableError;
use core::fmt::Display;

impl<E: Display> Display for ComposableError<E> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if !f.alternate() {
            return Display::fmt(&self.fmt(), f);
        }
        Display::fmt(&self.fmt().cascaded(), f)
    }
}

impl<E> core::error::Error for ComposableError<E>
where
    E: core::error::Error + Send + Sync + 'static,
{
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        Some(self.core_error())
    }
}

impl<E> From<E> for ComposableError<E> {
    #[inline]
    fn from(error: E) -> Self {
        Self::new(error)
    }
}
