use crate::types::alloc_type::Box;
use crate::types::composable_error::ComposableError;
use crate::{ComposableResult, ErrorContext, ErrorVec, IntoErrorContext};

/// A builder for composing error transformations with accumulated context.
///
/// `ErrorPipeline` allows you to:
/// - Attach multiple contexts that are only materialized on error
/// - Chain operations like `map`, `and_then`, and `recover`
/// - Defer the creation of [`ComposableError`] until finalization
///
/// Contexts are stored in a stack and only applied if the pipeline ends in an error.
///
/// # Type Parameters
///
/// * `T` - The success value type
/// * `E` - The error type
///
/// # Examples
///
/// ```
/// use error_rail::{ErrorPipeline, context};
///
/// let result = ErrorPipeline::<u32, &str>::new(Err("failed"))
///     .with_context(context!("step 1"))
///     .with_context(context!("step 2"))
///     .finish_boxed();
///
/// assert!(result.is_err());
/// ```
#[must_use]
pub struct ErrorPipeline<T, E> {
    result: Result<T, E>,
    pending_contexts: ErrorVec<ErrorContext>,
}

impl<T, E> ErrorPipeline<T, E> {
    /// Creates a pipeline from an existing `Result`.
    ///
    /// # Arguments
    ///
    /// * `result` - The result to wrap in a pipeline
    ///
    /// # Examples
    ///
    /// ```
    /// use error_rail::{context, ErrorPipeline};
    ///
    /// fn flaky() -> Result<(), &'static str> {
    ///     Err("boom")
    /// }
    ///
    /// let err = ErrorPipeline::new(flaky())
    ///     .with_context(context!("calling flaky"))
    ///     .finish_boxed()
    ///     .unwrap_err();
    ///
    /// assert!(err.error_chain().contains("calling flaky"));
    /// ```
    #[inline]
    pub fn new(result: Result<T, E>) -> Self {
        Self {
            result,
            pending_contexts: ErrorVec::new(),
        }
    }

    /// Adds a context entry to the pending context stack.
    ///
    /// If the current result is `Ok`, this is a no-op. Otherwise, the context
    /// is queued to be attached when `finish` is called.
    ///
    /// # Arguments
    ///
    /// * `context` - Context to add
    ///
    /// # Examples
    ///
    /// ```
    /// use error_rail::{ErrorPipeline, ErrorContext};
    ///
    /// let pipeline: ErrorPipeline<&str, &str> = ErrorPipeline::new(Err("error"))
    ///     .with_context(ErrorContext::tag("db"));
    /// ```
    #[inline]
    pub fn with_context<C>(mut self, context: C) -> Self
    where
        C: IntoErrorContext,
    {
        if self.result.is_ok() {
            return self;
        }

        let ctx = context.into_error_context();
        self.pending_contexts.push(ctx);
        self
    }

    /// Transforms the error type using a mapping function.
    ///
    /// Preserves all pending contexts while converting the error to a new type.
    ///
    /// # Arguments
    ///
    /// * `f` - Function to transform the error
    ///
    /// # Examples
    ///
    /// ```
    /// use error_rail::ErrorPipeline;
    ///
    /// let pipeline = ErrorPipeline::<&str, &str>::new(Err("text error"))
    ///     .map_error(|e| e.len());
    /// ```
    #[inline]
    pub fn map_error<F, NewE>(self, f: F) -> ErrorPipeline<T, NewE>
    where
        F: FnOnce(E) -> NewE,
    {
        ErrorPipeline {
            result: self.result.map_err(f),
            pending_contexts: self.pending_contexts,
        }
    }

    /// Attempts to recover from an error using a fallback function.
    ///
    /// If the current result is `Err`, calls the recovery function. If recovery
    /// succeeds, all pending contexts are discarded since the error is resolved.
    /// If recovery fails, pending contexts are preserved.
    ///
    /// **Note:** Successful recovery discards all accumulated contexts from the
    /// error path, as the error has been handled and resolved.
    ///
    /// # Arguments
    ///
    /// * `recovery` - Function that attempts to recover from the error
    ///
    /// # Examples
    ///
    /// ```
    /// use error_rail::ErrorPipeline;
    ///
    /// let pipeline = ErrorPipeline::new(Err("error"))
    ///     .recover(|_| Ok(42));
    /// ```
    #[inline]
    pub fn recover<F>(self, recovery: F) -> ErrorPipeline<T, E>
    where
        F: FnOnce(E) -> Result<T, E>,
    {
        match self.result {
            Ok(v) => ErrorPipeline {
                result: Ok(v),
                pending_contexts: self.pending_contexts,
            },
            Err(e) => match recovery(e) {
                Ok(v) => ErrorPipeline {
                    result: Ok(v),
                    pending_contexts: ErrorVec::new(),
                },
                Err(e) => ErrorPipeline {
                    result: Err(e),
                    pending_contexts: self.pending_contexts,
                },
            },
        }
    }

    /// Recovers from an error using a default value.
    ///
    /// If the current result is `Err`, replaces it with `Ok(value)`.
    /// All pending contexts are discarded on recovery since the error is resolved.
    ///
    /// **Note:** This method unconditionally discards all accumulated contexts
    /// from the error path when recovery occurs.
    ///
    /// # Arguments
    ///
    /// * `value` - The default value to use in case of error
    ///
    /// # Examples
    ///
    /// ```
    /// use error_rail::ErrorPipeline;
    ///
    /// let pipeline = ErrorPipeline::<i32, &str>::new(Err("error"))
    ///     .fallback(42);
    /// ```
    #[inline]
    pub fn fallback(self, value: T) -> ErrorPipeline<T, E> {
        match self.result {
            Ok(v) => ErrorPipeline {
                result: Ok(v),
                pending_contexts: self.pending_contexts,
            },
            Err(_) => ErrorPipeline {
                result: Ok(value),
                pending_contexts: ErrorVec::new(),
            },
        }
    }

    /// Recovers from an error using a safe function that always returns a value.
    ///
    /// If the current result is `Err`, calls `f` to get a success value.
    /// Pending contexts are discarded on recovery since the error is resolved.
    ///
    /// # Arguments
    ///
    /// * `f` - Function that produces a success value from the error
    ///
    /// # Examples
    ///
    /// ```
    /// use error_rail::ErrorPipeline;
    ///
    /// let pipeline = ErrorPipeline::<i32, &str>::new(Err("error"))
    ///     .recover_safe(|_| 42);
    /// ```
    #[inline]
    pub fn recover_safe<F>(self, f: F) -> ErrorPipeline<T, E>
    where
        F: FnOnce(E) -> T,
    {
        match self.result {
            Ok(v) => ErrorPipeline {
                result: Ok(v),
                pending_contexts: self.pending_contexts,
            },
            Err(e) => ErrorPipeline {
                result: Ok(f(e)),
                pending_contexts: ErrorVec::new(),
            },
        }
    }

    /// Chains a fallible operation on the success value.
    ///
    /// If the current result is `Ok`, applies the function. Otherwise, preserves
    /// the error and pending contexts.
    ///
    /// # Arguments
    ///
    /// * `f` - Function to apply to the success value
    ///
    /// # Examples
    ///
    /// ```
    /// use error_rail::ErrorPipeline;
    ///
    /// let pipeline = ErrorPipeline::<i32, &str>::new(Ok(5))
    ///     .and_then(|x| Ok(x * 2));
    /// ```
    #[inline]
    pub fn and_then<U, F>(self, f: F) -> ErrorPipeline<U, E>
    where
        F: FnOnce(T) -> Result<U, E>,
    {
        ErrorPipeline {
            result: self.result.and_then(f),
            pending_contexts: self.pending_contexts,
        }
    }

    /// Transforms the success value using a mapping function.
    ///
    /// If the current result is `Ok`, applies the function. Otherwise, preserves
    /// the error and pending contexts.
    ///
    /// # Arguments
    ///
    /// * `f` - Function to transform the success value
    ///
    /// # Examples
    ///
    /// ```
    /// use error_rail::ErrorPipeline;
    ///
    /// let pipeline = ErrorPipeline::<i32, &str>::new(Ok(5))
    ///     .map(|x| x * 2);
    /// ```
    #[inline]
    pub fn map<U, F>(self, f: F) -> ErrorPipeline<U, E>
    where
        F: FnOnce(T) -> U,
    {
        ErrorPipeline {
            result: self.result.map(f),
            pending_contexts: self.pending_contexts,
        }
    }

    /// Finalizes the pipeline into a boxed [`ComposableResult`].
    ///
    /// On `Ok`, returns the success value. On `Err`, creates a [`ComposableError`]
    /// with all pending contexts attached and boxes it.
    ///
    /// # Examples
    ///
    /// ```
    /// use error_rail::{ErrorPipeline, context};
    ///
    /// let result = ErrorPipeline::<u32, &str>::new(Err("failed"))
    ///     .with_context(context!("operation failed"))
    ///     .finish_boxed();
    /// ```
    #[inline]
    pub fn finish_boxed(self) -> crate::types::BoxedComposableResult<T, E> {
        match self.result {
            Ok(v) => Ok(v),
            Err(e) => {
                let composable = ComposableError::new(e).with_contexts(self.pending_contexts);
                Err(Box::new(composable))
            }
        }
    }

    /// Finalizes the pipeline into a [`ComposableResult`].
    ///
    /// Similar to `finish_boxed`, but returns the error directly without boxing.
    /// Use this when you need to avoid heap allocation.
    ///
    /// # Examples
    ///
    /// ```
    /// use error_rail::{ErrorPipeline, context};
    ///
    /// let result = ErrorPipeline::<u32, &str>::new(Err("failed"))
    ///     .with_context(context!("operation failed"))
    ///     .finish();
    /// ```
    #[inline]
    #[allow(clippy::result_large_err)]
    pub fn finish(self) -> ComposableResult<T, E> {
        match self.result {
            Ok(v) => Ok(v),
            Err(e) => {
                let composable = ComposableError::new(e).with_contexts(self.pending_contexts);
                Err(composable)
            }
        }
    }
}
