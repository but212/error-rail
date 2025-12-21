//! Error chain formatting utilities.

use crate::types::alloc_type;
use crate::types::ComposableError;
use core::fmt::Display;

#[cfg(not(feature = "std"))]
use alloc::string::ToString;

use crate::types::alloc_type::String;
use crate::types::alloc_type::Vec;

/// Trait for customizing error chain formatting.
pub trait ErrorFormatter {
    #[inline]
    fn format_item(&self, item: &dyn Display) -> String {
        item.to_string()
    }

    #[inline]
    fn separator(&self) -> &str {
        " -> "
    }

    fn format_chain<'a>(&self, chain: impl Iterator<Item = &'a dyn Display>) -> String {
        let items: Vec<_> = chain.map(|item| self.format_item(item)).collect();
        items.join(self.separator())
    }
}

/// Configuration-based error formatter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ErrorFormatConfig {
    pub separator: String,
    pub context_prefix: Option<String>,
    pub context_suffix: Option<String>,
    pub root_prefix: Option<String>,
    pub root_suffix: Option<String>,
    pub multiline: bool,
    pub indent: String,
    pub show_code: bool,
    pub cascade: bool,
}

impl Default for ErrorFormatConfig {
    #[inline]
    fn default() -> Self {
        Self {
            separator: " -> ".into(),
            context_prefix: None,
            context_suffix: None,
            root_prefix: None,
            root_suffix: None,
            multiline: false,
            indent: "  ".into(),
            show_code: true,
            cascade: false,
        }
    }
}

impl ErrorFormatConfig {
    /// Tree-style formatting with box-drawing characters.
    #[inline]
    pub fn pretty() -> Self {
        Self {
            separator: "\n".into(),
            context_prefix: Some("├─ ".into()),
            root_prefix: Some("└─ ".into()),
            multiline: true,
            ..Default::default()
        }
    }

    /// Indented cascade formatting.
    #[inline]
    pub fn cascaded() -> Self {
        Self { separator: "\n".into(), multiline: true, cascade: true, ..Default::default() }
    }

    #[inline]
    pub fn compact() -> Self {
        Self { separator: " | ".into(), ..Default::default() }
    }

    #[inline]
    pub fn no_code() -> Self {
        Self { show_code: false, ..Default::default() }
    }

    #[inline]
    fn get_prefix(&self, is_last: bool) -> Option<&str> {
        if is_last {
            self.root_prefix
                .as_deref()
                .or(self.context_prefix.as_deref())
        } else {
            self.context_prefix.as_deref()
        }
    }

    #[inline]
    fn get_suffix(&self, is_last: bool) -> Option<&str> {
        if is_last {
            self.root_suffix
                .as_deref()
                .or(self.context_suffix.as_deref())
        } else {
            self.context_suffix.as_deref()
        }
    }

    fn format_multiline_tree(&self, items: &[&dyn Display]) -> String {
        let len = items.len();
        let mut result = String::with_capacity(len * 40);

        result.push_str("┌ ");
        result.push_str(&items[0].to_string());

        for (i, item) in items.iter().enumerate().skip(1) {
            result.push_str(&self.separator);
            let is_last = i == len - 1;
            if let Some(p) = self.get_prefix(is_last) {
                result.push_str(p);
            }
            result.push_str(&item.to_string());
        }

        result
    }

    fn format_cascade(&self, items: &[&dyn Display]) -> String {
        let len = items.len();
        let mut result = String::with_capacity(len * 32);

        for (i, item) in items.iter().enumerate() {
            if i > 0 {
                result.push_str(&self.separator);
                for _ in 0..i {
                    result.push_str(&self.indent);
                }
            }
            result.push_str(&item.to_string());
        }

        result
    }

    fn format_linear(&self, items: &[&dyn Display]) -> String {
        let len = items.len();
        let last_idx = len - 1;
        let mut result = String::with_capacity(len * 32);

        for (i, item) in items.iter().enumerate() {
            if i > 0 {
                result.push_str(&self.separator);
            }

            let is_last = i == last_idx;
            if let Some(p) = self.get_prefix(is_last) {
                result.push_str(p);
            }
            result.push_str(&item.to_string());
            if let Some(s) = self.get_suffix(is_last) {
                result.push_str(s);
            }
        }

        result
    }
}

impl ErrorFormatter for ErrorFormatConfig {
    #[inline]
    fn format_item(&self, item: &dyn Display) -> String {
        match (&self.context_prefix, &self.context_suffix) {
            (None, None) => item.to_string(),
            (Some(p), None) => {
                let s = item.to_string();
                let mut result = String::with_capacity(p.len() + s.len());
                result.push_str(p);
                result.push_str(&s);
                result
            },
            (None, Some(suf)) => {
                let s = item.to_string();
                let mut result = String::with_capacity(s.len() + suf.len());
                result.push_str(&s);
                result.push_str(suf);
                result
            },
            (Some(p), Some(suf)) => {
                let s = item.to_string();
                let mut result = String::with_capacity(p.len() + s.len() + suf.len());
                result.push_str(p);
                result.push_str(&s);
                result.push_str(suf);
                result
            },
        }
    }

    #[inline]
    fn separator(&self) -> &str {
        &self.separator
    }

    fn format_chain<'a>(&self, chain: impl Iterator<Item = &'a dyn Display>) -> String {
        let items: Vec<_> = chain.collect();
        if items.is_empty() {
            return String::new();
        }

        if self.multiline && self.context_prefix.is_some() {
            self.format_multiline_tree(&items)
        } else if self.cascade {
            self.format_cascade(&items)
        } else {
            self.format_linear(&items)
        }
    }
}

/// Builder for customizing error display output.
pub struct ErrorFormatBuilder<'a, E> {
    pub(crate) error: &'a ComposableError<E>,
    pub(crate) config: ErrorFormatConfig,
    pub(crate) reverse_context: bool,
}

impl<'a, E> ErrorFormatBuilder<'a, E> {
    #[inline]
    pub fn new(error: &'a ComposableError<E>) -> Self {
        Self { error, config: ErrorFormatConfig::default(), reverse_context: false }
    }

    #[inline]
    pub fn with_separator(mut self, separator: impl Into<alloc_type::String>) -> Self {
        self.config.separator = separator.into();
        self
    }

    #[inline]
    pub fn reverse_context(mut self, reverse: bool) -> Self {
        self.reverse_context = reverse;
        self
    }

    #[inline]
    pub fn show_code(mut self, show: bool) -> Self {
        self.config.show_code = show;
        self
    }

    #[inline]
    pub fn pretty(mut self) -> Self {
        self.config = ErrorFormatConfig::pretty();
        self
    }

    #[inline]
    pub fn compact(mut self) -> Self {
        self.config = ErrorFormatConfig::compact();
        self
    }

    #[inline]
    pub fn cascade(mut self, enabled: bool) -> Self {
        self.config.cascade = enabled;
        if enabled {
            self.config.multiline = true;
            if self.config.separator == " -> " {
                self.config.separator = "\n".into();
            }
        }
        self
    }

    #[inline]
    pub fn cascaded(mut self) -> Self {
        self.config = ErrorFormatConfig::cascaded();
        self
    }
}

impl<'a, E> Display for ErrorFormatBuilder<'a, E>
where
    E: Display,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let ctx_count = self.error.context_iter().count();
        let mut items: Vec<&dyn Display> = Vec::with_capacity(ctx_count + 1);

        if self.reverse_context {
            items.extend(
                self.error
                    .context_iter()
                    .rev()
                    .map(|ctx| ctx as &dyn Display),
            );
        } else {
            items.extend(self.error.context_iter().map(|ctx| ctx as &dyn Display));
        }
        items.push(self.error.core_error());

        let formatted = self.config.format_chain(items.iter().copied());
        f.write_str(&formatted)?;

        if self.config.show_code {
            if let Some(code) = self.error.error_code() {
                write!(f, " (code: {})", code)?;
            }
        }

        Ok(())
    }
}
