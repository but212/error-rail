//! Error chain formatting utilities.

use crate::types::alloc_type;
use crate::types::ComposableError;
use core::fmt::Display;

#[cfg(not(feature = "std"))]
use alloc::string::{String, ToString};
#[cfg(feature = "std")]
use std::string::{String, ToString};

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;
#[cfg(feature = "std")]
use std::vec::Vec;

/// Trait for customizing error chain formatting.
pub trait ErrorFormatter {
    fn format_item(&self, item: &dyn Display) -> String {
        item.to_string()
    }

    fn separator(&self) -> &str {
        " -> "
    }

    fn format_chain<'a>(&self, chain: impl Iterator<Item = &'a dyn Display>) -> String {
        chain
            .map(|item| self.format_item(item))
            .collect::<Vec<_>>()
            .join(self.separator())
    }
}

/// Configuration-based error formatter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ErrorFormatConfig {
    pub separator: alloc_type::String,
    pub context_prefix: Option<alloc_type::String>,
    pub context_suffix: Option<alloc_type::String>,
    pub root_prefix: Option<alloc_type::String>,
    pub root_suffix: Option<alloc_type::String>,
    pub multiline: bool,
    pub indent: alloc_type::String,
    pub show_code: bool,
    pub cascade: bool,
}

impl Default for ErrorFormatConfig {
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
    #[inline]
    pub fn pretty() -> Self {
        Self {
            separator: "\n".into(),
            context_prefix: Some("├─ ".into()),
            root_prefix: Some("└─ ".into()),
            multiline: true,
            cascade: true,
            ..Default::default()
        }
    }

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
}

impl ErrorFormatter for ErrorFormatConfig {
    fn format_item(&self, item: &dyn Display) -> String {
        let mut result = String::new();
        if let Some(prefix) = &self.context_prefix {
            result.push_str(prefix);
        }
        result.push_str(&item.to_string());
        if let Some(suffix) = &self.context_suffix {
            result.push_str(suffix);
        }
        result
    }

    fn separator(&self) -> &str {
        &self.separator
    }

    fn format_chain<'a>(&self, chain: impl Iterator<Item = &'a dyn Display>) -> String {
        let items: Vec<_> = chain.collect();
        if items.is_empty() {
            return String::new();
        }

        let item_count = items.len();
        let mut result = String::with_capacity(item_count * 32);

        if self.multiline && self.context_prefix.is_some() {
            result.push_str("┌ ");
            result.push_str(&items[0].to_string());

            for item in items.iter().take(item_count.saturating_sub(1)).skip(1) {
                result.push_str(&self.separator);
                if let Some(prefix) = &self.context_prefix {
                    result.push_str(prefix);
                }
                result.push_str(&item.to_string());
            }

            if item_count > 1 {
                result.push_str(&self.separator);
                let prefix = self.root_prefix.as_ref().or(self.context_prefix.as_ref());
                if let Some(p) = prefix {
                    result.push_str(p);
                }
                result.push_str(&items[item_count - 1].to_string());
            }
        } else if self.cascade {
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    result.push_str(&self.separator);
                    for _ in 0..i {
                        result.push_str(&self.indent);
                    }
                }
                result.push_str(&item.to_string());
            }
        } else {
            let last_idx = item_count - 1;
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    result.push_str(&self.separator);
                }

                let is_last = i == last_idx;
                let prefix = if is_last {
                    self.root_prefix.as_ref().or(self.context_prefix.as_ref())
                } else {
                    self.context_prefix.as_ref()
                };
                if let Some(p) = prefix {
                    result.push_str(p);
                }

                result.push_str(&item.to_string());

                let suffix = if is_last {
                    self.root_suffix.as_ref().or(self.context_suffix.as_ref())
                } else {
                    self.context_suffix.as_ref()
                };
                if let Some(s) = suffix {
                    result.push_str(s);
                }
            }
        }
        result
    }
}

/// Builder for customizing error display output.
pub struct ErrorFormatBuilder<'a, E> {
    pub(crate) error: &'a ComposableError<E>,
    pub(crate) config: ErrorFormatConfig,
    pub(crate) reverse_context: bool,
}

impl<'a, E> ErrorFormatBuilder<'a, E> {
    pub fn new(error: &'a ComposableError<E>) -> Self {
        Self { error, config: ErrorFormatConfig::default(), reverse_context: false }
    }

    pub fn with_separator(mut self, separator: impl Into<alloc_type::String>) -> Self {
        self.config.separator = separator.into();
        self
    }

    pub fn reverse_context(mut self, reverse: bool) -> Self {
        self.reverse_context = reverse;
        self
    }

    pub fn show_code(mut self, show: bool) -> Self {
        self.config.show_code = show;
        self
    }

    pub fn pretty(mut self) -> Self {
        self.config = ErrorFormatConfig::pretty();
        self
    }

    pub fn compact(mut self) -> Self {
        self.config = ErrorFormatConfig::compact();
        self
    }

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
        use crate::types::alloc_type::Vec;

        let mut items: Vec<&dyn Display> = Vec::new();

        if self.reverse_context {
            for ctx in self.error.context_iter().rev() {
                items.push(ctx as &dyn Display);
            }
        } else {
            for ctx in self.error.context_iter() {
                items.push(ctx as &dyn Display);
            }
        }

        items.push(self.error.core_error());

        let s = self.config.format_chain(items.iter().copied());
        write!(f, "{}", s)?;

        if self.config.show_code {
            if let Some(code) = self.error.error_code() {
                write!(f, " (code: {})", code)?;
            }
        }

        Ok(())
    }
}
