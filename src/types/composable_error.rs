use std::fmt::{Debug, Display};

use crate::traits::IntoErrorContext;
use crate::types::ErrorVec;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComposableError<E> {
    pub core_error: E,
    context: ErrorVec<String>,
    pub error_code: Option<u32>,
}

impl<E> ComposableError<E> {
    #[inline]
    pub fn new(error: E) -> Self {
        Self {
            core_error: error,
            context: ErrorVec::new(),
            error_code: None,
        }
    }

    #[inline]
    pub fn with_code(error: E, code: u32) -> Self {
        Self {
            core_error: error,
            context: ErrorVec::new(),
            error_code: Some(code),
        }
    }

    #[inline]
    pub fn with_context<C>(mut self, ctx: C) -> Self
    where
        C: IntoErrorContext,
    {
        self.context
            .push(ctx.into_error_context().message().to_string());
        self
    }

    #[inline]
    pub fn with_contexts<I>(mut self, contexts: I) -> Self
    where
        I: IntoIterator<Item = String>,
    {
        self.context.extend(contexts); // O(1) per element
        self
    }

    #[inline]
    pub fn core_error(&self) -> &E {
        &self.core_error
    }

    #[inline]
    pub fn context(&self) -> Vec<String> {
        self.context.iter().rev().cloned().collect()
    }

    #[inline]
    pub fn context_iter(&self) -> std::iter::Rev<std::slice::Iter<'_, String>> {
        self.context.iter().rev()
    }

    #[inline]
    pub fn error_code(&self) -> Option<u32> {
        self.error_code
    }

    #[inline]
    pub fn set_code(mut self, code: u32) -> Self {
        self.error_code = Some(code);
        self
    }

    #[inline]
    pub fn map_core<F, T>(self, f: F) -> ComposableError<T>
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
    {
        let mut chain = String::new();

        // Iterate in reverse order (most recent first)
        for (i, ctx) in self.context.iter().rev().enumerate() {
            if i > 0 {
                chain.push_str(" -> ");
            }
            chain.push_str(ctx);
        }

        if !self.context.is_empty() {
            chain.push_str(" -> ");
        }

        chain.push_str(&format!("{}", self.core_error));

        if let Some(code) = self.error_code {
            chain.push_str(&format!(" (code: {})", code));
        }

        chain
    }
}

impl<E: Display> Display for ComposableError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.error_chain())
    }
}

impl<E: Debug + Display> std::error::Error for ComposableError<E> {}

impl<E> From<E> for ComposableError<E> {
    #[inline]
    fn from(error: E) -> Self {
        Self::new(error)
    }
}
