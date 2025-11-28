//! Error chain formatting utilities.
//!
//! This module provides flexible formatting for error chains through two mechanisms:
//!
//! 1. **Simple configuration** via [`ErrorFormatConfig`] - good for most use cases
//! 2. **Custom formatters** via [`ErrorFormatter`] trait - for advanced customization
//!
//! # Examples
//!
//! ## Using ErrorFormatConfig (Simple)
//!
//! ```
//! use error_rail::{ComposableError, ErrorFormatConfig};
//!
//! let err = ComposableError::new("database error")
//!     .with_context("fetching user")
//!     .set_code(500);
//!
//! // Use built-in pretty format
//! println!("{}", err.error_chain_with(ErrorFormatConfig::pretty()));
//! ```
//!
//! ## Custom Formatter (Advanced)
//!
//! ```
//! use error_rail::{ComposableError, ErrorFormatter};
//! use core::fmt::Display;
//!
//! struct JsonFormatter;
//!
//! impl ErrorFormatter for JsonFormatter {
//!     fn format_item(&self, item: &dyn Display) -> String {
//!         format!("\"{}\"", item.to_string().replace("\"", "\\\""))
//!     }
//!     
//!     fn separator(&self) -> &str {
//!         ","
//!     }
//! }
//!
//! let err = ComposableError::new("error")
//!     .with_context("context");
//!
//! println!("{}", err.error_chain_with(JsonFormatter));
//! ```

use core::fmt::Display;

#[cfg(not(feature = "std"))]
use alloc::string::{String, ToString};
#[cfg(feature = "std")]
use std::string::{String, ToString};

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;
#[cfg(feature = "std")]
use std::vec::Vec;

use crate::types::alloc_type;

/// Trait for customizing error chain formatting.
///
/// Implement this trait to create custom formatters for error chains.
/// For simple use cases, use [`ErrorFormatConfig`] instead.
///
/// # Examples
///
/// ```
/// use error_rail::{ComposableError, ErrorFormatter};
/// use core::fmt::Display;
///
/// struct UppercaseFormatter;
///
/// impl ErrorFormatter for UppercaseFormatter {
///     fn format_item(&self, item: &dyn Display) -> String {
///         item.to_string().to_uppercase()
///     }
/// }
///
/// let err = ComposableError::new("error")
///     .with_context("context");
///
/// let chain = err.error_chain_with(UppercaseFormatter);
/// assert!(chain.contains("ERROR"));
/// assert!(chain.contains("CONTEXT"));
/// ```
pub trait ErrorFormatter {
    /// Formats a single item (context or root error).
    ///
    /// Override this to customize how individual items are displayed.
    ///
    /// # Arguments
    ///
    /// * `item` - A displayable item (context or error)
    ///
    /// # Examples
    ///
    /// ```
    /// use error_rail::ErrorFormatter;
    /// use core::fmt::Display;
    ///
    /// struct PrefixFormatter;
    ///
    /// impl ErrorFormatter for PrefixFormatter {
    ///     fn format_item(&self, item: &dyn Display) -> String {
    ///         format!("[ERROR] {}", item)
    ///     }
    /// }
    /// ```
    fn format_item(&self, item: &dyn Display) -> String {
        item.to_string()
    }

    /// Returns the separator between items.
    ///
    /// Default is `" -> "`.
    fn separator(&self) -> &str {
        " -> "
    }

    /// Formats the entire error chain.
    ///
    /// Override this for complete control over chain formatting.
    /// The default implementation joins items with the separator.
    ///
    /// # Arguments
    ///
    /// * `chain` - Iterator of displayable items (contexts then root error)
    fn format_chain<'a>(&self, chain: impl Iterator<Item = &'a dyn Display>) -> String {
        chain
            .map(|item| self.format_item(item))
            .collect::<Vec<_>>()
            .join(self.separator())
    }
}

/// Configuration-based error formatter for common formatting needs.
///
/// This struct provides a simple way to customize error chain formatting
/// without implementing the [`ErrorFormatter`] trait. It automatically
/// implements `ErrorFormatter` based on the configuration.
///
/// # Examples
///
/// ## Default Format
///
/// ```
/// use error_rail::{ComposableError, ErrorFormatConfig};
///
/// let err = ComposableError::new("error")
///     .with_context("context");
///
/// let chain = err.error_chain_with(ErrorFormatConfig::default());
/// assert_eq!(chain, "context -> error");
/// ```
///
/// ## Pretty Format (Multiline with Tree)
///
/// ```
/// use error_rail::{ComposableError, ErrorFormatConfig};
///
/// let err = ComposableError::new("error")
///     .with_context("context1")
///     .with_context("context2");
///
/// let chain = err.error_chain_with(ErrorFormatConfig::pretty());
/// // Output:
/// // ┌ context2
/// // ├─ context1
/// // └─ error
/// ```
///
/// ## Custom Configuration
///
/// ```
/// use error_rail::{ComposableError, ErrorFormatConfig};
///
/// let config = ErrorFormatConfig {
///     separator: " | ".into(),
///     context_prefix: Some("[CTX] ".into()),
///     root_prefix: Some("[ERR] ".into()),
///     ..Default::default()
/// };
///
/// let err = ComposableError::new("error")
///     .with_context("context");
///
/// let chain = err.error_chain_with(config);
/// assert_eq!(chain, "[CTX] context | [ERR] error");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ErrorFormatConfig {
    /// Separator between items (default: `" -> "`)
    pub separator: alloc_type::String,

    /// Prefix for context items (default: `None`)
    pub context_prefix: Option<alloc_type::String>,

    /// Suffix for context items (default: `None`)
    pub context_suffix: Option<alloc_type::String>,

    /// Prefix for root error (default: `None`)
    pub root_prefix: Option<alloc_type::String>,

    /// Suffix for root error (default: `None`)
    pub root_suffix: Option<alloc_type::String>,

    /// Use multiline format (default: `false`)
    pub multiline: bool,

    /// Indentation for multiline format (default: `"  "`)
    pub indent: alloc_type::String,

    /// Show error code inline (default: `true`)
    pub show_code: bool,
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
        }
    }
}

impl ErrorFormatConfig {
    /// Creates a pretty formatter with tree-like multiline output.
    ///
    /// # Examples
    ///
    /// ```
    /// use error_rail::{ComposableError, ErrorFormatConfig};
    ///
    /// let err = ComposableError::new("connection failed")
    ///     .with_context("loading config")
    ///     .with_context("starting server");
    ///
    /// println!("{}", err.error_chain_with(ErrorFormatConfig::pretty()));
    /// // Output:
    /// // ┌ starting server
    /// // ├─ loading config
    /// // └─ connection failed
    /// ```
    pub fn pretty() -> Self {
        Self {
            separator: "\n".into(),
            context_prefix: Some("├─ ".into()),
            root_prefix: Some("└─ ".into()),
            multiline: true,
            ..Default::default()
        }
    }

    /// Creates a compact formatter with pipe separator.
    ///
    /// # Examples
    ///
    /// ```
    /// use error_rail::{ComposableError, ErrorFormatConfig};
    ///
    /// let err = ComposableError::new("error")
    ///     .with_context("context");
    ///
    /// let chain = err.error_chain_with(ErrorFormatConfig::compact());
    /// assert_eq!(chain, "context | error");
    /// ```
    pub fn compact() -> Self {
        Self {
            separator: " | ".into(),
            ..Default::default()
        }
    }

    /// Creates a formatter without error codes.
    ///
    /// # Examples
    ///
    /// ```
    /// use error_rail::{ComposableError, ErrorFormatConfig};
    ///
    /// let err = ComposableError::new("error")
    ///     .set_code(500);
    ///
    /// let chain = err.error_chain_with(ErrorFormatConfig::no_code());
    /// assert_eq!(chain, "error");
    /// ```
    pub fn no_code() -> Self {
        Self {
            show_code: false,
            ..Default::default()
        }
    }
}

impl ErrorFormatter for ErrorFormatConfig {
    fn format_item(&self, item: &dyn Display) -> String {
        let mut result = String::new();

        // This is a bit tricky - we need to know if this is a context or root error
        // For now, we'll apply context formatting by default and handle root separately
        // in the format_chain method
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

        // Handle multiline pretty format with special first line
        if self.multiline && self.context_prefix.is_some() {
            // First item gets special prefix
            if item_count > 0 {
                result.push_str("┌ ");
                result.push_str(&items[0].to_string());
            }

            // Middle items (contexts)
            for i in 1..item_count.saturating_sub(1) {
                result.push_str(&self.separator);
                if let Some(prefix) = &self.context_prefix {
                    result.push_str(prefix);
                }
                result.push_str(&items[i].to_string());
            }

            // Last item (root error)
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
            // Standard format
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    result.push_str(&self.separator);
                }

                // Apply root prefix to last item, context prefix to others
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

                // Apply suffixes
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
