//! Rich, structured metadata for error contexts.
//!
//! This module defines [`ErrorContext`], an enum representing different kinds
//! of contextual information that can be attached to errors throughout the
//! error-handling pipeline.
//!
//! # Variants
//!
//! - **`Simple`**: Free-form text describing what was happening when the error occurred.
//! - **`Group`**: A rich context containing location, tags, metadata, and an optional message.
//!
//! # Usage
//!
//! Use the constructor methods to create context entries, then attach them to
//! errors via [`ComposableError::with_context`](crate::types::ComposableError::with_context)
//! or the [`context!`](crate::context!) macro.
//!
//! # Examples
//!
//! ```
//! use error_rail::ErrorContext;
//!
//! let msg = ErrorContext::new("database connection failed");
//! let loc = ErrorContext::location("main.rs", 42);
//! let tag = ErrorContext::tag("db");
//! let meta = ErrorContext::metadata("retry_count", "3");
//! ```
use serde::{Deserialize, Serialize};
use std::fmt::Display;

/// Structured metadata attached to a [`ComposableError`](crate::types::ComposableError).
///
/// Context entries can store free-form messages, precise source locations,
/// feature tags, or arbitrary key/value metadata. Helper constructors are
/// provided for the most common variants.
///
/// # Variants
///
/// - `Simple(String)`: A plain text message describing the error context.
/// - `Group(GroupContext)`: A rich context containing multiple pieces of information.
///
/// # Examples
///
/// ```
/// use error_rail::ErrorContext;
///
/// let msg = ErrorContext::new("database connection failed");
/// let loc = ErrorContext::location("main.rs", 42);
/// let tag = ErrorContext::tag("db");
/// let meta = ErrorContext::metadata("retry_count", "3");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ErrorContext {
    Simple(String),
    Group(GroupContext),
}

/// A rich context containing multiple pieces of information.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct GroupContext {
    pub message: Option<String>,
    pub location: Option<Location>,
    pub tags: Vec<String>,
    pub metadata: Vec<(String, String)>,
}

/// Source file and line number where the error occurred.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Location {
    pub file: String,
    pub line: u32,
}

impl ErrorContext {
    /// Creates a plain message context.
    ///
    /// This is the most common way to add descriptive text to an error.
    ///
    /// # Arguments
    ///
    /// * `message` - Any type that can be converted into a `String`.
    ///
    /// # Examples
    /// ```
    /// use error_rail::ErrorContext;
    ///
    /// let ctx = ErrorContext::new("fetching profile");
    /// assert_eq!(ctx.message(), "fetching profile");
    /// ```
    #[inline]
    pub fn new<S: Into<String>>(message: S) -> Self {
        Self::Simple(message.into())
    }

    /// Captures the file/line pair where an error occurred.
    ///
    /// Typically used with the `file!()` and `line!()` macros to automatically
    /// record the source location.
    ///
    /// # Arguments
    ///
    /// * `file` - The source file path.
    /// * `line` - The line number in the source file.
    ///
    /// # Examples
    /// ```
    /// use error_rail::ErrorContext;
    ///
    /// let ctx = ErrorContext::location(file!(), line!());
    /// ```
    #[inline]
    pub fn location(file: &str, line: u32) -> Self {
        Self::Group(GroupContext {
            location: Some(Location {
                file: file.to_string(),
                line,
            }),
            ..Default::default()
        })
    }

    /// Annotates the error with a short tag (e.g. `auth`, `cache`).
    ///
    /// Tags are useful for filtering or categorizing errors in logs and monitoring systems.
    ///
    /// # Arguments
    ///
    /// * `tag` - Any type that can be converted into a `String`.
    ///
    /// # Examples
    /// ```
    /// use error_rail::ErrorContext;
    ///
    /// let ctx = ErrorContext::tag("network");
    /// assert_eq!(ctx.message(), "[network]");
    /// ```
    #[inline]
    pub fn tag<S: Into<String>>(tag: S) -> Self {
        Self::Group(GroupContext {
            tags: vec![tag.into()],
            ..Default::default()
        })
    }

    /// Adds arbitrary key/value metadata for downstream logging/filters.
    ///
    /// Use this to attach structured data that can be parsed by log aggregators
    /// or monitoring tools.
    ///
    /// # Arguments
    ///
    /// * `key` - The metadata key.
    /// * `value` - The metadata value.
    ///
    /// # Examples
    /// ```
    /// use error_rail::ErrorContext;
    ///
    /// let ctx = ErrorContext::metadata("user_id", 42.to_string());
    /// assert_eq!(ctx.message(), "user_id=42");
    /// ```
    #[inline]
    pub fn metadata<K: Into<String>, V: Into<String>>(key: K, value: V) -> Self {
        Self::Group(GroupContext {
            metadata: vec![(key.into(), value.into())],
            ..Default::default()
        })
    }

    /// Renders the context as a human-friendly string.
    ///
    /// Each variant is formatted differently:
    /// - `Simple`: Returns the message as-is.
    /// - `Group`: Formats the content based on what's available.
    ///
    /// # Examples
    /// ```
    /// use error_rail::ErrorContext;
    ///
    /// let ctx = ErrorContext::location("main.rs", 10);
    /// assert_eq!(ctx.message(), "at main.rs:10");
    /// ```
    #[inline]
    pub fn message(&self) -> std::borrow::Cow<'_, str> {
        match self {
            Self::Simple(s) => std::borrow::Cow::Borrowed(s.as_str()),
            Self::Group(g) => {
                if let Some(msg) = &g.message {
                    return std::borrow::Cow::Borrowed(msg.as_str());
                }
                if let Some(loc) = &g.location {
                    return std::borrow::Cow::Owned(format!("at {}:{}", loc.file, loc.line));
                }
                if !g.tags.is_empty() {
                    // Join tags with comma if multiple? Or just show first?
                    // Previous behavior was single tag -> "[tag]".
                    // Let's format as "[tag1, tag2]"
                    return std::borrow::Cow::Owned(format!("[{}]", g.tags.join(", ")));
                }
                if !g.metadata.is_empty() {
                    // Previous behavior was single key-value -> "key=value".
                    // Let's format as "key1=value1, key2=value2"
                    let meta_str = g
                        .metadata
                        .iter()
                        .map(|(k, v)| format!("{}={}", k, v))
                        .collect::<Vec<_>>()
                        .join(", ");
                    return std::borrow::Cow::Owned(meta_str);
                }
                std::borrow::Cow::Borrowed("")
            }
        }
    }
}

impl Display for ErrorContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message())
    }
}

impl std::error::Error for ErrorContext {}
