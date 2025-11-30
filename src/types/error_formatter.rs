//! Error chain formatting utilities.

use crate::types::alloc_type;
use crate::types::ComposableError;
use bitflags::bitflags;
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

bitflags! {
    /// Configuration flags for error formatting.
    ///
    /// This provides a memory-efficient way to configure error formatting
    /// using bit flags instead of individual boolean fields.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct ErrorFormat: u8 {
        /// Show error codes in output (default: enabled).
        const SHOW_CODE = 0b0001;
        /// Use multiline formatting with tree structure (default: disabled).
        const MULTILINE = 0b0010;
        /// Use compact separator style (default: disabled).
        const COMPACT = 0b0100;
        /// Use pretty formatting with tree characters (default: disabled).
        const PRETTY = 0b1000;
        /// Default format: show code, single line.
        const DEFAULT = Self::SHOW_CODE.bits();
        /// All formatting options enabled.
        const ALL = Self::SHOW_CODE.bits() | Self::MULTILINE.bits() | Self::COMPACT.bits() | Self::PRETTY.bits();
    }
}

impl Default for ErrorFormat {
    fn default() -> Self {
        Self::DEFAULT
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
    pub indent: alloc_type::String,
    pub format: ErrorFormat,
}

impl Default for ErrorFormatConfig {
    fn default() -> Self {
        Self {
            separator: " -> ".into(),
            context_prefix: None,
            context_suffix: None,
            root_prefix: None,
            root_suffix: None,
            indent: "  ".into(),
            format: ErrorFormat::default(),
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
            format: ErrorFormat::PRETTY | ErrorFormat::MULTILINE,
            ..Default::default()
        }
    }

    #[inline]
    pub fn compact() -> Self {
        Self {
            separator: " | ".into(),
            format: ErrorFormat::COMPACT,
            ..Default::default()
        }
    }

    #[inline]
    pub fn no_code() -> Self {
        Self {
            format: ErrorFormat::default().difference(ErrorFormat::SHOW_CODE),
            ..Default::default()
        }
    }

    #[inline]
    pub fn with_format(mut self, format: ErrorFormat) -> Self {
        self.format = format;

        // Automatically apply configuration changes based on bitflags
        if self.format.contains(ErrorFormat::COMPACT) {
            self.separator = " | ".into();
        }
        if self.format.contains(ErrorFormat::PRETTY) {
            self.separator = "\n".into();
            self.context_prefix = Some("├─ ".into());
            self.root_prefix = Some("└─ ".into());
        }

        self
    }

    #[inline]
    pub fn show_code(mut self, show: bool) -> Self {
        self.format.set(ErrorFormat::SHOW_CODE, show);
        self
    }

    #[inline]
    pub fn multiline(mut self, multiline: bool) -> Self {
        self.format.set(ErrorFormat::MULTILINE, multiline);
        self
    }

    #[inline]
    pub fn set_compact(mut self, compact: bool) -> Self {
        self.format.set(ErrorFormat::COMPACT, compact);
        self
    }

    #[inline]
    pub fn set_pretty(mut self, pretty: bool) -> Self {
        self.format.set(ErrorFormat::PRETTY, pretty);
        if pretty {
            self.format.insert(ErrorFormat::MULTILINE);
        }
        self
    }

    #[deprecated(
        since = "0.8.0",
        note = "Use with_format() or format-specific methods instead"
    )]
    #[inline]
    pub fn is_show_code(&self) -> bool {
        self.format.contains(ErrorFormat::SHOW_CODE)
    }

    #[deprecated(
        since = "0.8.0",
        note = "Use with_format() or format-specific methods instead"
    )]
    #[inline]
    pub fn is_multiline(&self) -> bool {
        self.format.contains(ErrorFormat::MULTILINE)
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

        let mut result = String::new();
        let item_count = items.len();

        if self.format.contains(ErrorFormat::MULTILINE) && self.context_prefix.is_some() {
            if item_count > 0 {
                result.push_str("┌ ");
                result.push_str(&items[0].to_string());
            }
            for item in items.iter().take(item_count.saturating_sub(1)).skip(1) {
                result.push_str(&self.separator);
                if let Some(prefix) = &self.context_prefix {
                    result.push_str(prefix);
                }
                result.push_str(&item.to_string());
            }
            if item_count > 1 {
                result.push_str(&self.separator);
                if let Some(prefix) = &self.root_prefix {
                    result.push_str(prefix);
                } else if let Some(prefix) = &self.context_prefix {
                    result.push_str(prefix);
                }
                result.push_str(&items[item_count - 1].to_string());
            }
        } else {
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    result.push_str(&self.separator);
                }
                if i == item_count - 1 {
                    if let Some(prefix) = &self.root_prefix {
                        result.push_str(prefix);
                    } else if let Some(prefix) = &self.context_prefix {
                        result.push_str(prefix);
                    }
                } else if let Some(prefix) = &self.context_prefix {
                    result.push_str(prefix);
                }
                result.push_str(&item.to_string());
                if i == item_count - 1 {
                    if let Some(suffix) = &self.root_suffix {
                        result.push_str(suffix);
                    } else if let Some(suffix) = &self.context_suffix {
                        result.push_str(suffix);
                    }
                } else if let Some(suffix) = &self.context_suffix {
                    result.push_str(suffix);
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
        Self {
            error,
            config: ErrorFormatConfig::default(),
            reverse_context: false,
        }
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
        self.config.format.set(ErrorFormat::SHOW_CODE, show);
        self
    }

    pub fn set_pretty(mut self) -> Self {
        self.config = ErrorFormatConfig::pretty();
        self
    }

    pub fn set_compact(mut self) -> Self {
        self.config = ErrorFormatConfig::compact();
        self
    }

    pub fn with_format(mut self, format: ErrorFormat) -> Self {
        self.config = self.config.with_format(format);
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

        if self.config.format.contains(ErrorFormat::SHOW_CODE) {
            if let Some(code) = self.error.error_code() {
                write!(f, " (code: {})", code)?;
            }
        }

        Ok(())
    }
}
