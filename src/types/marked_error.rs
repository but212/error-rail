use crate::traits::TransientError;
use core::fmt::{Debug, Display};

/// An error wrapper that marks an error as transient or permanent based on a closure.
///
/// This type is primarily used by the [`ErrorPipeline::mark_transient_if()`](crate::ErrorPipeline::mark_transient_if)
/// method, allowing users to dynamically classify errors without implementing the
/// [`TransientError`] trait for their specific error types.
///
/// # Type Parameters
///
/// * `E` - The inner error type being wrapped
/// * `F` - A classifier closure that determines if the error is transient
///
/// # Examples
///
/// ```
/// use error_rail::ErrorPipeline;
///
/// let result = ErrorPipeline::<(), &str>::new(Err("temporary failure"))
///     .mark_transient_if(|e| e.contains("temporary"))
///     .retry()
///     .max_retries(3)
///     .to_error_pipeline()
///     .finish();
/// ```
///
/// # Design Notes
///
/// The classifier closure is stored alongside the error, enabling lazy evaluation
/// of transient status. This avoids the need for users to implement [`TransientError`]
/// on their error types while still integrating with the retry infrastructure.
pub struct MarkedError<E, F> {
    pub(crate) inner: E,
    pub(crate) classifier: F,
}

impl<E, F> MarkedError<E, F> {
    /// Returns a reference to the inner error.
    ///
    /// # Examples
    ///
    /// ```
    /// use error_rail::ErrorPipeline;
    ///
    /// let pipeline = ErrorPipeline::<(), &str>::new(Err("error"))
    ///     .mark_transient_if(|_| true);
    ///
    /// // Access inner error through the pipeline's error
    /// ```
    #[inline]
    pub const fn inner(&self) -> &E {
        &self.inner
    }

    /// Consumes the marked error, returning the inner error.
    ///
    /// This is useful when you need to extract the original error
    /// after processing through a pipeline.
    #[inline]
    pub fn into_inner(self) -> E {
        self.inner
    }

    /// Returns a reference to the classifier closure.
    ///
    /// This can be useful for debugging or introspection purposes.
    #[inline]
    pub const fn classifier(&self) -> &F {
        &self.classifier
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
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Display::fmt(&self.inner, f)
    }
}

impl<E, F> TransientError for MarkedError<E, F>
where
    F: Fn(&E) -> bool,
{
    /// Determines if the wrapped error is transient by invoking the classifier closure.
    #[inline]
    fn is_transient(&self) -> bool {
        (self.classifier)(&self.inner)
    }
}

impl<E, F> core::error::Error for MarkedError<E, F>
where
    E: core::error::Error + 'static,
    F: Fn(&E) -> bool,
{
    /// Returns the inner error as the source, enabling error chain traversal.
    #[inline]
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        Some(&self.inner)
    }
}
