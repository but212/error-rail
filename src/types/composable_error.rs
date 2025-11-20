use std::fmt::{Debug, Display};

use crate::traits::IntoErrorContext;
use crate::types::{ErrorContext, ErrorVec};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComposableError<E, C = u32> {
    pub core_error: E,
    context: ErrorVec<ErrorContext>,
    pub error_code: Option<C>,
}

impl<E, C> ComposableError<E, C> {
    #[inline]
    pub fn new(error: E) -> Self {
        Self {
            core_error: error,
            context: ErrorVec::new(),
            error_code: None,
        }
    }

    #[inline]
    pub fn with_code(error: E, code: C) -> Self {
        Self {
            core_error: error,
            context: ErrorVec::new(),
            error_code: Some(code),
        }
    }

    #[inline]
    pub fn with_context<Ctx>(mut self, ctx: Ctx) -> Self
    where
        Ctx: IntoErrorContext,
    {
        self.context.push(ctx.into_error_context());
        self
    }

    #[inline]
    pub fn with_contexts<I>(mut self, contexts: I) -> Self
    where
        I: IntoIterator<Item = ErrorContext>,
    {
        self.context.extend(contexts);
        self
    }

    #[inline]
    pub fn core_error(&self) -> &E {
        &self.core_error
    }

    #[inline]
    pub fn context(&self) -> Vec<ErrorContext> {
        self.context.iter().rev().cloned().collect()
    }

    #[inline]
    pub fn context_iter(&self) -> std::iter::Rev<std::slice::Iter<'_, ErrorContext>> {
        self.context.iter().rev()
    }

    #[inline]
    pub fn error_code(&self) -> Option<&C> {
        self.error_code.as_ref()
    }

    #[inline]
    pub fn set_code(mut self, code: C) -> Self {
        self.error_code = Some(code);
        self
    }

    #[inline]
    pub fn map_core<F, T>(self, f: F) -> ComposableError<T, C>
    where
        F: FnOnce(E) -> T,
    {
        ComposableError {
            core_error: f(self.core_error),
            context: self.context,
            error_code: self.error_code,
        }
    }

    pub fn error_chain(&self) -> String
    where
        E: Display,
        C: Display,
    {
        let mut chain = String::new();

        // Iterate in reverse order (most recent first)
        for (i, ctx) in self.context.iter().rev().enumerate() {
            if i > 0 {
                chain.push_str(" -> ");
            }
            chain.push_str(&ctx.message());
        }

        if !self.context.is_empty() {
            chain.push_str(" -> ");
        }

        chain.push_str(&format!("{}", self.core_error));

        if let Some(code) = &self.error_code {
            chain.push_str(&format!(" (code: {})", code));
        }

        chain
    }
}

impl<E: Display, C: Display> Display for ComposableError<E, C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.error_chain())
    }
}

impl<E: Debug + Display, C: Debug + Display> std::error::Error for ComposableError<E, C> {}

impl<E, C> From<E> for ComposableError<E, C> {
    #[inline]
    fn from(error: E) -> Self {
        Self::new(error)
    }
}
