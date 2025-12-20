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
//! let ctx = ErrorContext::group("context")
//!     .tag("db")
//!     .location("main.rs", 42)
//!     .metadata("retry_count", "3")
//!     .build();
//!
//! assert_eq!(msg.message(), "database connection failed");
//! assert!(ctx.message().contains("[db]"));
//! ```
use crate::types::alloc_type::String;
use crate::types::alloc_type::{Cow, Vec};
use crate::ErrorVec;
use core::fmt::Display;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;

#[cfg(not(feature = "std"))]
use alloc::format;
#[cfg(not(feature = "std"))]
use alloc::string::ToString;
#[cfg(feature = "std")]
use std::format;
#[cfg(feature = "std")]
use std::string::ToString;

/// Structured metadata attached to a [`ComposableError`](crate::types::ComposableError).
///
/// Context entries can store free-form messages, precise source locations,
/// feature tags, or arbitrary key/value metadata. Helper constructors are
/// provided for the most common variants.
///
/// # Variants
///
/// - `Simple(Cow<'static, str>)`: A plain text message describing the error context.
/// - `Group(GroupContext)`: A rich context containing multiple pieces of information.
///
/// # Examples
///
/// ```
/// use error_rail::ErrorContext;
///
///
/// let msg = ErrorContext::new("database connection failed");
/// let ctx = ErrorContext::group("context")
///     .tag("db")
///     .location("main.rs", 42)
///     .metadata("retry_count", "3")
///     .build();
/// ```
use crate::types::alloc_type::Box;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ErrorContext {
    Simple(Cow<'static, str>),
    Group(Box<GroupContext>),
}

/// A rich context containing multiple pieces of information.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct GroupContext {
    /// Optional message describing this context
    pub message: Option<Cow<'static, str>>,
    /// Optional source location where the error occurred
    pub location: Option<Location>,
    /// Tags for categorizing and filtering errors
    pub tags: SmallVec<[Cow<'static, str>; 2]>,
    /// Arbitrary key-value metadata pairs
    pub metadata: SmallVec<[(Cow<'static, str>, Cow<'static, str>); 2]>,
}

/// Source file and line number where the error occurred.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Location {
    /// Source file path
    pub file: Cow<'static, str>,
    /// Line number in the source file
    pub line: u32,
}

impl ErrorContext {
    /// Creates a plain message context.
    ///
    /// This is the most common way to add descriptive text to an error.
    ///
    /// # Arguments
    ///
    /// * `message` - Any type that can be converted into a `Cow<'static, str>`.
    ///
    /// # Examples
    /// ```
    /// use error_rail::ErrorContext;
    ///
    /// let ctx = ErrorContext::new("fetching profile");
    /// assert_eq!(ctx.message(), "fetching profile");
    /// ```
    #[inline]
    pub fn new<S: Into<Cow<'static, str>>>(message: S) -> Self {
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
    pub fn location<S: Into<Cow<'static, str>>>(file: S, line: u32) -> Self {
        Self::Group(Box::new(GroupContext {
            location: Some(Location { file: file.into(), line }),
            ..Default::default()
        }))
    }

    /// Annotates the error with a short tag (e.g. `auth`, `cache`).
    ///
    /// Tags are useful for filtering or categorizing errors in logs and monitoring systems.
    ///
    /// # Arguments
    ///
    /// * `tag` - Any type that can be converted into a `Cow<'static, str>`.
    ///
    /// # Examples
    /// ```
    /// use error_rail::ErrorContext;
    ///
    /// let ctx = ErrorContext::tag("network");
    /// assert_eq!(ctx.message(), "[network]");
    /// ```
    #[inline]
    pub fn tag<S: Into<Cow<'static, str>>>(tag: S) -> Self {
        Self::Group(Box::new(GroupContext {
            tags: smallvec::smallvec![tag.into()],
            ..Default::default()
        }))
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
    /// let ctx = ErrorContext::metadata("user_id", "42");
    /// assert_eq!(ctx.message(), "(user_id=42)");
    /// ```
    #[inline]
    pub fn metadata<K: Into<Cow<'static, str>>, V: Into<Cow<'static, str>>>(
        key: K,
        value: V,
    ) -> Self {
        Self::Group(Box::new(GroupContext {
            metadata: smallvec::smallvec![(key.into(), value.into())],
            ..Default::default()
        }))
    }

    /// Renders the context as a human-friendly string.
    ///
    /// Each variant is formatted differently:
    /// - `Simple`: Returns the message as-is.
    /// - `Group`: Combines all available fields into one cohesive unit.
    ///
    /// # Examples
    ///
    /// Single field contexts remain unchanged:
    /// ```rust
    /// use error_rail::ErrorContext;
    ///
    /// let ctx = ErrorContext::location("main.rs", 10);
    /// assert_eq!(ctx.message(), "at main.rs:10");
    /// ```
    ///
    /// Multiple fields are combined into one unit:
    /// ```rust
    /// use error_rail::ErrorContext;
    ///
    /// let ctx = ErrorContext::builder()
    ///     .tag("db")
    ///     .tag("network")
    ///     .location("main.rs", 42)
    ///     .message("connection failed")
    ///     .metadata("retry_count", "3")
    ///     .build();
    ///     
    /// assert_eq!(ctx.message(), "[db, network] at main.rs:42: connection failed (retry_count=3)");
    /// ```
    #[inline]
    pub fn message(&self) -> Cow<'_, str> {
        match self {
            Self::Simple(s) => Cow::Borrowed(s.as_ref()),
            Self::Group(g) => {
                let mut parts = ErrorVec::new();

                // Add tags if present
                if !g.tags.is_empty() {
                    parts.push(format!("[{}]", g.tags.join(", ")));
                }

                // Add location if present
                if let Some(loc) = &g.location {
                    parts.push(format!("at {}:{}", loc.file, loc.line));
                }

                // Add message if present
                if let Some(msg) = &g.message {
                    Self::add_message_to_parts(&mut parts, msg.as_ref(), g.location.is_some());
                }

                // Add metadata if present
                if !g.metadata.is_empty() {
                    let meta_str = g
                        .metadata
                        .iter()
                        .map(|(k, v)| format!("{}={}", k, v))
                        .collect::<Vec<_>>()
                        .join(", ");
                    parts.push(format!("({})", meta_str));
                }

                if parts.is_empty() {
                    Cow::Borrowed("")
                } else {
                    Cow::Owned(parts.join(" "))
                }
            },
        }
    }

    /// Helper function to add message to parts with proper location formatting.
    /// If has_location is true and the last part is a location string (starts with "at "),
    /// appends the message to that location. Otherwise, adds as a new part.
    fn add_message_to_parts(parts: &mut ErrorVec<String>, msg: &str, has_location: bool) {
        // Only attempt to merge with location if has_location is true AND
        // there's actually a location part to merge with
        let should_merge = has_location && parts.last().is_some_and(|p| p.starts_with("at "));

        if should_merge {
            // Safe: we just verified last() exists and starts with "at "
            if let Some(last_part) = parts.last_mut() {
                *last_part = format!("{}: {}", last_part, msg);
            }
        } else {
            parts.push(msg.to_string());
        }
    }
}

impl Display for ErrorContext {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.message())
    }
}

impl core::error::Error for ErrorContext {}

/// Builder for creating complex [`ErrorContext::Group`] entries.
///
/// # Examples
///
/// ```
/// use error_rail::ErrorContext;
///
/// let ctx = ErrorContext::builder()
///     .message("connection failed")
///     .tag("network")
///     .metadata("host", "localhost")
///     .build();
/// ```
#[derive(Debug, Default)]
pub struct ErrorContextBuilder {
    context: GroupContext,
}

impl ErrorContextBuilder {
    /// Creates a new builder for constructing complex [`ErrorContext::Group`] entries.
    ///
    /// # Examples
    ///
    /// ```
    /// use error_rail::ErrorContext;
    ///
    /// let ctx = ErrorContext::builder()
    ///     .message("operation failed")
    ///     .build();
    /// ```
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the message for this context.
    ///
    /// # Arguments
    ///
    /// * `msg` - Any type that can be converted into a `Cow<'static, str>`.
    ///
    /// # Examples
    ///
    /// ```
    /// use error_rail::ErrorContext;
    ///
    /// let ctx = ErrorContext::builder()
    ///     .message("database query failed")
    ///     .build();
    /// ```
    pub fn message<S: Into<Cow<'static, str>>>(mut self, msg: S) -> Self {
        self.context.message = Some(msg.into());
        self
    }

    /// Sets the source location for this context.
    ///
    /// Typically used with the `file!()` and `line!()` macros to automatically
    /// record where the error occurred.
    ///
    /// # Arguments
    ///
    /// * `file` - The source file path.
    /// * `line` - The line number in the source file.
    ///
    /// # Examples
    ///
    /// ```
    /// use error_rail::ErrorContext;
    ///
    /// let ctx = ErrorContext::builder()
    ///     .location(file!(), line!())
    ///     .build();
    /// ```
    pub fn location<S: Into<Cow<'static, str>>>(mut self, file: S, line: u32) -> Self {
        self.context.location = Some(Location { file: file.into(), line });
        self
    }

    /// Adds a tag to this context.
    ///
    /// Tags are useful for categorizing and filtering errors. Multiple tags
    /// can be added by calling this method multiple times.
    ///
    /// # Arguments
    ///
    /// * `tag` - Any type that can be converted into a `Cow<'static, str>`.
    ///
    /// # Examples
    ///
    /// ```
    /// use error_rail::ErrorContext;
    ///
    /// let ctx = ErrorContext::builder()
    ///     .tag("network")
    ///     .tag("timeout")
    ///     .build();
    /// ```
    pub fn tag<S: Into<Cow<'static, str>>>(mut self, tag: S) -> Self {
        self.context.tags.push(tag.into());
        self
    }

    /// Adds a metadata key-value pair to this context.
    ///
    /// Metadata provides structured information that can be parsed by log
    /// aggregators or monitoring tools. Multiple metadata pairs can be added
    /// by calling this method multiple times.
    ///
    /// # Arguments
    ///
    /// * `key` - The metadata key.
    /// * `value` - The metadata value.
    ///
    /// # Examples
    ///
    /// ```
    /// use error_rail::ErrorContext;
    ///
    /// let ctx = ErrorContext::builder()
    ///     .metadata("user_id", "12345")
    ///     .metadata("request_id", "abc-def")
    ///     .build();
    /// ```
    pub fn metadata<K: Into<Cow<'static, str>>, V: Into<Cow<'static, str>>>(
        mut self,
        key: K,
        value: V,
    ) -> Self {
        self.context.metadata.push((key.into(), value.into()));
        self
    }

    /// Builds and returns the final [`ErrorContext`].
    ///
    /// Consumes the builder and produces an [`ErrorContext::Group`] variant
    /// containing all the accumulated information.
    ///
    /// # Examples
    ///
    /// ```
    /// use error_rail::ErrorContext;
    ///
    /// let ctx = ErrorContext::builder()
    ///     .message("connection failed")
    ///     .tag("network")
    ///     .metadata("host", "localhost")
    ///     .build();
    /// ```
    pub fn build(self) -> ErrorContext {
        ErrorContext::Group(Box::new(self.context))
    }
}

impl ErrorContext {
    /// Creates a new [`ErrorContextBuilder`] for constructing complex contexts.
    ///
    /// This is the starting point for building [`ErrorContext::Group`] entries
    /// with multiple pieces of information such as location, tags, and metadata.
    ///
    /// # Examples
    ///
    /// ```
    /// use error_rail::ErrorContext;
    ///
    /// let ctx = ErrorContext::builder()
    ///     .message("operation failed")
    ///     .tag("critical")
    ///     .build();
    /// ```
    pub fn builder() -> ErrorContextBuilder {
        ErrorContextBuilder::new()
    }

    /// Creates a group context initialized with a message.
    ///
    /// This is a convenience method that starts a builder with a message already set.
    /// Unlike [`ErrorContext::new`] which creates a `Simple` variant, this creates
    /// a `Group` variant that can be further enhanced with additional context.
    ///
    /// # Arguments
    ///
    /// * `message` - Any type that can be converted into a `Cow<'static, str>`.
    ///
    /// # Examples
    ///
    /// ```
    /// use error_rail::ErrorContext;
    ///
    /// let ctx = ErrorContext::group("database error")
    ///     .tag("db")
    ///     .metadata("table", "users")
    ///     .build();
    /// ```
    pub fn group<S: Into<Cow<'static, str>>>(message: S) -> ErrorContextBuilder {
        ErrorContextBuilder::new().message(message)
    }
}
