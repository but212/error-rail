use super::ComposableError;
use core::fmt::Display;

/// Legacy formatter for customizing error display output.
#[deprecated(since = "0.9.0", note = "Use ErrorFormatBuilder via ComposableError::fmt() instead")]
#[doc(hidden)]
pub struct LegacyErrorFormatter<'a, E> {
    pub(crate) error: &'a ComposableError<E>,
    pub(crate) separator: &'a str,
    pub(crate) reverse_context: bool,
    pub(crate) show_code: bool,
}

#[allow(deprecated)]
impl<'a, E> LegacyErrorFormatter<'a, E> {
    /// Sets the separator between context elements (default: " -> ").
    pub fn with_separator(mut self, separator: &'a str) -> Self {
        self.separator = separator;
        self
    }

    /// If true, displays contexts in FIFO order (oldest first) instead of LIFO.
    pub fn reverse_context(mut self, reverse: bool) -> Self {
        self.reverse_context = reverse;
        self
    }

    /// Whether to include the error code in the output (default: true).
    pub fn show_code(mut self, show: bool) -> Self {
        self.show_code = show;
        self
    }
}

#[allow(deprecated)]
impl<'a, E> Display for LegacyErrorFormatter<'a, E>
where
    E: Display,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let contexts = &self.error.context;
        let mut first = true;

        if self.reverse_context {
            for ctx in contexts.iter() {
                if !first {
                    f.write_str(self.separator)?;
                }
                first = false;
                f.write_str(ctx.message().as_ref())?;
            }
        } else {
            for ctx in contexts.iter().rev() {
                if !first {
                    f.write_str(self.separator)?;
                }
                first = false;
                f.write_str(ctx.message().as_ref())?;
            }
        }

        if !first {
            f.write_str(self.separator)?;
        }
        Display::fmt(&self.error.core_error, f)?;

        if self.show_code {
            if let Some(code) = &self.error.error_code {
                write!(f, " (code: {})", code)?;
            }
        }

        Ok(())
    }
}
