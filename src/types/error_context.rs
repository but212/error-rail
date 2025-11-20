//! Rich, structured metadata for error contexts.
//!
//! This module defines [`ErrorContext`], an enum representing different kinds
//! of contextual information that can be attached to errors throughout the
//! error-handling pipeline.
//!
//! # Variants
//!
//! - **`Message`**: Free-form text describing what was happening when the error occurred.
//! - **`Location`**: File path and line number, typically captured via `file!()` and `line!()`.
//! - **`Tag`**: A short label for categorizing errors (e.g., `"auth"`, `"db"`).
//! - **`Metadata`**: Arbitrary key-value pairs for structured logging or filtering.
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
/// - `Message(String)`: A plain text message describing the error context.
/// - `Location { file: String, line: u32 }`: Source file and line number where the error occurred.
/// - `Tag(String)`: A short categorization tag (e.g., `"auth"`, `"cache"`).
/// - `Metadata { key: String, value: String }`: Arbitrary key-value pair for structured logging.
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
    Message(String),
    Location { file: String, line: u32 },
    Tag(String),
    Metadata { key: String, value: String },
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
        Self::Message(message.into())
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
        Self::Location {
            file: file.to_string(),
            line,
        }
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
        Self::Tag(tag.into())
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
        Self::Metadata {
            key: key.into(),
            value: value.into(),
        }
    }

    /// Renders the context as a human-friendly string.
    ///
    /// Each variant is formatted differently:
    /// - `Message`: Returns the message as-is.
    /// - `Location`: Formats as `"at <file>:<line>"`.
    /// - `Tag`: Formats as `"[<tag>]"`.
    /// - `Metadata`: Formats as `"<key>=<value>"`.
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
            Self::Message(s) => std::borrow::Cow::Borrowed(s.as_str()),
            Self::Location { file, line } => {
                std::borrow::Cow::Owned(format!("at {}:{}", file, line))
            }
            Self::Tag(t) => std::borrow::Cow::Owned(format!("[{}]", t)),
            Self::Metadata { key, value } => std::borrow::Cow::Owned(format!("{}={}", key, value)),
        }
    }
}

impl Display for ErrorContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message())
    }
}

impl std::error::Error for ErrorContext {}
