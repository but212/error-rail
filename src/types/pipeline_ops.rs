use crate::traits::TransientError;
use crate::types::ErrorPipeline;

/// A builder/wrapper for error operations in the pipeline.
///
/// This struct is passed to the closure in `ErrorPipeline::error_ops`.
pub struct PipelineErrorOps<T, E> {
    pub(crate) pipeline: ErrorPipeline<T, E>,
}

impl<T, E> PipelineErrorOps<T, E> {
    /// Recovers from an error using a fallback function.
    pub fn recover<F>(self, recovery: F) -> Self
    where
        F: FnOnce(E) -> Result<T, E>,
    {
        Self {
            pipeline: self.pipeline.recover(recovery),
        }
    }

    /// Recovers from an error using a default value.
    pub fn fallback(self, value: T) -> Self {
        Self {
            pipeline: self.pipeline.fallback(value),
        }
    }

    /// Recovers only if the error is transient.
    pub fn recover_transient<F>(self, recovery: F) -> Self
    where
        E: TransientError,
        F: FnOnce(E) -> Result<T, E>,
    {
        Self {
            pipeline: self.pipeline.recover_transient(recovery),
        }
    }
}
