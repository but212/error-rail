use crate::traits::IntoErrorContext;
use crate::types::composable_error::ComposableError;
use crate::types::{BoxedComposableResult, ComposableResult};
use smallvec::SmallVec;
use std::fmt::Display;

#[inline]
pub fn with_context<E, C>(error: E, context: C) -> ComposableError<E>
where
    C: IntoErrorContext,
{
    ComposableError::new(error).with_context(context)
}

#[inline]
pub fn with_context_result<T, E, C>(result: Result<T, E>, context: C) -> BoxedComposableResult<T, E>
where
    C: IntoErrorContext,
{
    result.map_err(|e| Box::new(with_context(e, context)))
}

#[inline]
pub fn context_fn<E, C>(context: C) -> impl Fn(E) -> ComposableError<E>
where
    C: IntoErrorContext + Clone,
{
    move |error| with_context(error, context.clone())
}

pub struct ErrorPipeline<T, E> {
    result: Result<T, E>,
    pending_contexts: SmallVec<[String; 4]>,
}

impl<T, E> ErrorPipeline<T, E> {
    #[inline]
    pub fn new(result: Result<T, E>) -> Self {
        Self {
            result,
            pending_contexts: SmallVec::new(),
        }
    }

    #[inline]
    pub fn with_context<C>(mut self, context: C) -> Self
    where
        C: IntoErrorContext,
    {
        if self.result.is_ok() {
            return self;
        }

        let ctx_str = context.into_error_context().message().to_string();
        self.pending_contexts.push(ctx_str);
        self
    }

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

    #[inline]
    pub fn recover<F>(self, recovery: F) -> ErrorPipeline<T, E>
    where
        F: FnOnce(E) -> Result<T, E>,
    {
        ErrorPipeline {
            result: self.result.or_else(recovery),
            pending_contexts: self.pending_contexts,
        }
    }

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

    #[inline]
    pub fn finish(self) -> BoxedComposableResult<T, E> {
        match self.result {
            Ok(v) => Ok(v),
            Err(e) => {
                let composable = ComposableError::new(e).with_contexts(self.pending_contexts);
                Err(Box::new(composable))
            }
        }
    }

    #[inline]
    #[allow(clippy::result_large_err)]
    pub fn finish_without_box(self) -> ComposableResult<T, E> {
        match self.result {
            Ok(v) => Ok(v),
            Err(e) => {
                let composable = ComposableError::new(e).with_contexts(self.pending_contexts);
                Err(composable)
            }
        }
    }
}

#[inline]
pub fn error_pipeline<T, E>(result: Result<T, E>) -> ErrorPipeline<T, E> {
    ErrorPipeline::new(result)
}

pub fn accumulate_context<E, I, C>(error: E, contexts: I) -> ComposableError<E>
where
    I: IntoIterator<Item = C>,
    C: IntoErrorContext,
{
    let context_strings: Vec<String> = contexts
        .into_iter()
        .map(|c| c.into_error_context().message().to_string())
        .collect();

    ComposableError::new(error).with_contexts(context_strings)
}

pub fn context_accumulator<E, I, C>(contexts: I) -> impl Fn(E) -> ComposableError<E>
where
    I: IntoIterator<Item = C> + Clone,
    C: IntoErrorContext + Clone,
{
    move |error| accumulate_context(error, contexts.clone())
}

pub fn format_error_chain<E>(error: &ComposableError<E>) -> String
where
    E: Display,
{
    error.error_chain()
}

pub fn extract_context<E>(error: &ComposableError<E>) -> Vec<String> {
    error.context()
}
