//! Ergonomic macros for creating lazy or structured [`ErrorContext`](crate::types::ErrorContext).
//!
//! These macros provide convenient shortcuts for attaching rich metadata to errors:
//!
//! - [`macro@crate::context`] - Defers formatting until the context is consumed, avoiding
//!   unnecessary allocations on the success path.
//! - [`macro@crate::location`] - Automatically captures the current file path and line number
//!   using `file!()` and `line!()`.
//! - [`macro@crate::tag`] - Attaches a short categorical label for filtering and searching.
//! - [`macro@crate::metadata`] - Adds arbitrary key-value pairs for structured logging.
//!
//! # Examples
//!
//! ```
//! use error_rail::{context, location, tag, metadata, ErrorPipeline};
//!
//! let result: Result<(), &str> = Err("failed");
//! let pipeline = ErrorPipeline::new(result)
//!     .with_context(context!("user_id: {}", 123))
//!     .with_context(location!())
//!     .with_context(tag!("auth"))
//!     .with_context(metadata!("retry_count", "3"))
//!     .finish();
//! ```

/// Creates a lazily-evaluated error context that defers string formatting.
///
/// This macro wraps the provided format string and arguments in a [`LazyContext`](crate::types::LazyContext),
/// which only evaluates the closure when the error actually occurs. This avoids the performance
/// overhead of string formatting on the success path.
///
/// # Arguments
///
/// Accepts the same arguments as the standard `format!` macro.
///
/// # Examples
///
/// ```
/// use error_rail::{context, ComposableError};
///
/// let user_id = 42;
/// let err = ComposableError::<&str, u32>::new("auth failed")
///     .with_context(context!("user_id: {}", user_id));
/// ```
#[macro_export]
macro_rules! context {
    ($($arg:tt)*) => {
        $crate::types::LazyContext::new(move || format!($($arg)*))
    };
}

/// Captures the current source file and line number as error context.
///
/// This macro creates an [`ErrorContext::Location`](crate::types::ErrorContext::Location)
/// using the `file!()` and `line!()` built-in macros, providing precise source location
/// information for debugging.
///
/// # Examples
///
/// ```
/// use error_rail::{location, ComposableError};
///
/// let err = ComposableError::<&str, u32>::new("io error")
///     .with_context(location!());
/// ```
#[macro_export]
macro_rules! location {
    () => {
        $crate::types::ErrorContext::location(file!(), line!())
    };
}

/// Creates a categorical tag for error classification.
///
/// This macro creates an [`ErrorContext::Tag`](crate::types::ErrorContext::Tag) that can be
/// used to categorize and filter errors by domain (e.g., "db", "auth", "network").
///
/// # Arguments
///
/// * `$tag` - A string or expression that can be converted into a tag
///
/// # Examples
///
/// ```
/// use error_rail::{tag, ComposableError};
///
/// let err = ComposableError::<&str, u32>::new("connection failed")
///     .with_context(tag!("network"));
/// ```
#[macro_export]
macro_rules! tag {
    ($tag:expr) => {
        $crate::types::ErrorContext::tag($tag)
    };
}

/// Creates a key-value metadata pair for structured error context.
///
/// This macro creates an [`ErrorContext::Metadata`](crate::types::ErrorContext::Metadata)
/// entry that can be used for structured logging, filtering, or monitoring.
///
/// # Arguments
///
/// * `$key` - The metadata key
/// * `$value` - The metadata value
///
/// # Examples
///
/// ```
/// use error_rail::{metadata, ComposableError};
///
/// let err = ComposableError::<&str, u32>::new("rate limit exceeded")
///     .with_context(metadata!("retry_after", "60"));
/// ```
#[macro_export]
macro_rules! metadata {
    ($key:expr, $value:expr) => {
        $crate::types::ErrorContext::metadata($key, $value)
    };
}
