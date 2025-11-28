//! Ergonomic macros for creating lazy or structured [`ErrorContext`](crate::types::ErrorContext).
//!
//! These macros provide convenient shortcuts for attaching rich metadata to errors:
//!
//! - [`macro@crate::rail`] - Wraps a `Result`-producing block and converts it into a
//!   [`BoxedComposableResult`](crate::types::BoxedComposableResult) via `ErrorPipeline::finish`.
//! - [`macro@crate::context`] - Defers formatting until the context is consumed, avoiding
//!   unnecessary allocations on the success path.
//! - [`macro@crate::location`] - Automatically captures the current file path and line number
//!   using `file!()` and `line!()`.
//! - [`macro@crate::tag`] - Attaches a short categorical label for filtering and searching.
//! - [`macro@crate::group`] - Creates a lazily-evaluated grouped context that combines
//!   multiple fields (message, tags, location, metadata) into one cohesive unit while deferring
//!   all formatting until the error occurs.
//!
//! # Examples
//!
//! ```
//! use error_rail::{context, rail, group, ErrorPipeline};
//!
//! let result: Result<(), &str> = Err("failed");
//! let pipeline = ErrorPipeline::new(result)
//!     .with_context(context!("user_id: {}", 123))
//!     .with_context(group!(
//!         tag("auth"),
//!         location(file!(), line!()),
//!         metadata("retry_count", "3")
//!     ))
//!     .finish_boxed();
//!
//! assert!(pipeline.is_err());
//!
//! // Equivalent rail! shorthand that also returns a boxed composable result
//! let _ = rail!({
//!     Err::<(), &str>("failed")
//!         .map_err(|err| err)
//! });
//! ```

/// Wraps a `Result`-producing expression or block and converts it into a
/// [`BoxedComposableResult`](crate::types::BoxedComposableResult).
///
/// This macro provides a convenient shorthand for creating an [`ErrorPipeline`](crate::ErrorPipeline)
/// and immediately calling `finish()` to box the result. It accepts either a single expression
/// or a block of code that produces a `Result`.
///
/// # Syntax
///
/// - `rail!(expr)` - Wraps a single `Result`-producing expression
/// - `rail!({ ... })` - Wraps a block that produces a `Result`
///
/// # Returns
///
/// A [`BoxedComposableResult<T, E>`](crate::types::BoxedComposableResult) where the error type
/// is wrapped in a [`ComposableError`](crate::types::ComposableError).
///
/// # Examples
///
/// ```rust
/// use error_rail::{rail, group};
///
/// // Simple expression
/// let result = rail!(Err::<(), &str>("failed"));
/// assert!(result.is_err());
///
/// // Block syntax with multiple statements
/// let result = rail!({
///     let value = std::fs::read_to_string("config.txt");
///     value
/// });
///
/// // Using with group! macro for structured context
/// let result = rail!({
///     std::fs::read_to_string("config.txt")
/// })
/// .map_err(|e| e.with_context(group!(
///     tag("config"),
///     location(file!(), line!()),
///     metadata("file", "config.txt")
/// )));
/// ```
#[macro_export]
macro_rules! rail {
    ($expr:expr $(,)?) => {
        $crate::ErrorPipeline::new($expr).finish_boxed()
    };
}

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
/// let err = ComposableError::<&str>::new("auth failed")
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
/// This macro creates an [`ErrorContext::location`](crate::types::ErrorContext::location)
/// using the `file!()` and `line!()` built-in macros.
///
/// # Deprecated
///
/// Use [`group!`](crate::group) instead since version 0.5.0.
#[macro_export]
#[deprecated(since = "0.5.0", note = "Use `group!` instead")]
macro_rules! location {
    () => {
        $crate::types::ErrorContext::location(file!(), line!())
    };
}

/// Creates a categorical tag for error classification.
///
/// This macro creates an [`ErrorContext::tag`](crate::types::ErrorContext::tag) that can be
/// used to categorize and filter errors.
///
/// # Deprecated
///
/// Use [`group!`](crate::group) instead since version 0.5.0.
///
/// # Arguments
///
/// * `$tag` - A string or expression that can be converted into a tag
#[macro_export]
#[deprecated(since = "0.5.0", note = "Use `group!` instead")]
macro_rules! tag {
    ($tag:expr) => {
        $crate::types::ErrorContext::tag($tag)
    };
}

/// Creates a key-value metadata pair for structured error context.
///
/// This macro creates an [`ErrorContext::metadata`](crate::types::ErrorContext::metadata)
/// entry.
///
/// # Deprecated
///
/// Use [`group!`](crate::group) instead since version 0.5.0.
///
/// # Arguments
///
/// * `$key` - The metadata key
/// * `$value` - The metadata value
#[macro_export]
#[deprecated(since = "0.5.0", note = "Use `group!` instead")]
macro_rules! metadata {
    ($key:expr, $value:expr) => {
        $crate::types::ErrorContext::metadata($key, $value)
    };
}

/// Implements `IntoErrorContext` for a custom type.
///
/// This macro simplifies the implementation of the [`IntoErrorContext`](crate::traits::IntoErrorContext)
/// trait for user-defined types. It converts the type into an [`ErrorContext`](crate::types::ErrorContext)
/// using its `Display` implementation.
///
/// # Arguments
///
/// * `$type` - The type to implement `IntoErrorContext` for.
///
/// # Examples
///
/// ```
/// use error_rail::{impl_error_context, ErrorContext, traits::IntoErrorContext};
/// use std::fmt;
///
/// struct MyError {
///     code: u32,
/// }
///
/// impl fmt::Display for MyError {
///     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
///         write!(f, "Error code: {}", self.code)
///     }
/// }
///
/// impl_error_context!(MyError);
///
/// let err = MyError { code: 404 };
/// let ctx = err.into_error_context();
/// assert_eq!(ctx.to_string(), "Error code: 404");
/// ```
#[macro_export]
macro_rules! impl_error_context {
    ($type:ty) => {
        impl $crate::traits::IntoErrorContext for $type {
            fn into_error_context(self) -> $crate::types::ErrorContext {
                $crate::types::ErrorContext::new(self.to_string())
            }
        }
    };
}

/// Creates a grouped error context that combines multiple context types.
///
/// This macro creates a lazily-evaluated grouped context that combines message,
/// tags, location, and metadata into a single cohesive unit. All formatting is
/// deferred until the error actually occurs, avoiding unnecessary allocations
/// on the success path.
///
/// # Arguments
///
/// The macro accepts function-call style arguments:
/// * `message("format string", args...)` - Optional formatted message
/// * `tag("label")` - Categorical tags (can be repeated)
/// * `location(file, line)` - Source file and line number
/// * `metadata("key", "value")` - Key-value pairs (can be repeated)
///
/// # Examples
///
/// ```
/// use error_rail::{group, ComposableError};
///
/// let attempts = 3;
/// let err = ComposableError::<&str>::new("auth failed")
///     .with_context(group!(
///         message("user_id: {}", attempts),
///         tag("auth"),
///         location(file!(), line!()),
///         metadata("retry_count", "3"),
///         metadata("timeout", "30s")
///     ));
/// ```
#[macro_export]
macro_rules! group {
    // Empty group
    () => {
        $crate::types::LazyGroupContext::new(move || {
            $crate::types::ErrorContext::Group($crate::types::GroupContext::default())
        })
    };

    // With fields - use function-call style
    (
        $($field:ident($($arg:tt)*)),* $(,)?
    ) => {
        $crate::types::LazyGroupContext::new(move || {
            let mut builder = $crate::types::ErrorContext::builder();
            $(
                $crate::__group_field!(builder, $field, $($arg)*);
            )*
            builder.build()
        })
    };
}

/// Internal macro for processing individual group fields
#[macro_export]
#[doc(hidden)]
macro_rules! __group_field {
    // Message field
    ($builder:expr, message, $($arg:tt)*) => {
        $builder = $builder.message(format!($($arg)*));
    };

    // Tag field
    ($builder:expr, tag, $tag:expr) => {
        $builder = $builder.tag($tag);
    };

    // Location field
    ($builder:expr, location, $file:expr, $line:expr) => {
        $builder = $builder.location($file, $line);
    };

    // Metadata field
    ($builder:expr, metadata, $key:expr, $value:expr) => {
        $builder = $builder.metadata($key, $value);
    };
}

/// Captures the current backtrace as lazy error context.
///
/// This macro creates a [`LazyContext`](crate::types::LazyContext) that captures the stack
/// backtrace only when the error actually occurs, avoiding the performance overhead of
/// backtrace generation on the success path.
///
/// The backtrace is captured using [`std::backtrace::Backtrace::capture()`] and converted
/// to a string representation when the context is evaluated.
///
/// # Examples
///
/// ```
/// use error_rail::{ComposableError, backtrace};
///
/// let err = ComposableError::<&str>::new("panic occurred")
///     .with_context(backtrace!());
/// ```
#[macro_export]
#[cfg(feature = "std")]
macro_rules! backtrace {
    () => {{
        $crate::types::LazyContext::new(|| std::backtrace::Backtrace::capture().to_string())
    }};
}

/// Creates a backtrace context that always captures regardless of environment.
///
/// This macro uses `force_capture()` to always generate a backtrace, ignoring
/// `RUST_BACKTRACE`/`RUST_LIB_BACKTRACE` settings. Use this for debugging
/// scenarios where you need guaranteed backtrace information.
///
/// # Performance Note
///
/// This has higher overhead than `backtrace!()` since it always captures
/// stack frames, regardless of environment settings.
///
/// # Examples
///
/// ```
/// use error_rail::{ComposableError, backtrace_force};
///
/// let err = ComposableError::<&str>::new("panic occurred")
///     .with_context(backtrace_force!());
/// ```
#[macro_export]
#[cfg(feature = "std")]
macro_rules! backtrace_force {
    () => {{
        $crate::types::LazyContext::new(|| std::backtrace::Backtrace::force_capture().to_string())
    }};
}
