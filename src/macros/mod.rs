//! Ergonomic macros for creating lazy or structured [`ErrorContext`](crate::types::ErrorContext).
//!
//! These macros provide convenient shortcuts for attaching rich metadata to errors:
//!
//! - [`macro@crate::rail`] - Wraps a `Result`-producing block and converts it into a
//!   [`BoxedComposableResult`](crate::types::BoxedComposableResult) via `ErrorPipeline::finish_boxed`.
//!   **Always returns boxed errors.**
//! - [`macro@crate::rail_unboxed`] - Wraps a `Result`-producing block and converts it into an
//!   unboxed [`ComposableResult`](crate::types::ComposableResult) via `ErrorPipeline::finish`.
//! - [`macro@crate::context`] - Defers formatting until the context is consumed, avoiding
//!   unnecessary allocations on the success path.
//! - [`macro@crate::group`] - Creates a lazily-evaluated grouped context that combines
//!   multiple fields (message, tags, location, metadata) into one cohesive unit while deferring
//!   all formatting until the error occurs.
//!
//! # Examples
//!
//! ```
//! use error_rail::{context, rail, rail_unboxed, group, ErrorPipeline};
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
//! // rail! - ALWAYS returns boxed error (8 bytes stack size)
//! let boxed_result = rail!({
//!     Err::<(), &str>("failed")
//! });
//!
//! // rail_unboxed! - returns unboxed error (larger stack size)
//! let unboxed_result = rail_unboxed!({
//!     Err::<(), &str>("failed")
//! });
//! ```
//!
//! ## Choosing Between rail! and rail_unboxed!
//!
//! - **Use `rail!`** for public APIs and most cases - smaller stack footprint (8 bytes)
//! - **Use `rail_unboxed!`** for internal code or performance-critical paths where you want to avoid heap allocation

/// Wraps a `Result`-producing expression or block and converts it into a
/// [`BoxedComposableResult`](crate::types::BoxedComposableResult).
///
/// **⚠️ IMPORTANT: This macro ALWAYS returns a boxed composable result.**
/// The error type is wrapped in a `Box<ComposableError<E>>` to reduce stack size.
/// If you need an unboxed result, use [`rail_unboxed!`](crate::rail_unboxed) instead.
///
/// This macro provides a convenient shorthand for creating an [`ErrorPipeline`](crate::ErrorPipeline)
/// and immediately calling `finish_boxed()` to box the result. It accepts either a single expression
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
/// is wrapped in a [`ComposableError`](crate::types::ComposableError) and boxed.
///
/// # Examples
///
/// ```rust
/// use error_rail::{rail, group};
///
/// // Simple expression - ALWAYS returns boxed error
/// let result = rail!(Err::<(), &str>("failed"));
/// assert!(result.is_err());
/// // Error type is Box<ComposableError<&str>>
/// let _: Box<error_rail::ComposableError<&str>> = result.unwrap_err();
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

/// Wraps a `Result`-producing expression or block and converts it into an
/// unboxed [`ComposableResult`](crate::types::ComposableResult).
///
/// This macro is similar to [`rail!`](crate::rail) but returns an unboxed error.
/// Use this when you need to avoid heap allocation or when working with APIs
/// that expect the unboxed `ComposableError<E>` type.
///
/// # Syntax
///
/// - `rail_unboxed!(expr)` - Wraps a single `Result`-producing expression
/// - `rail_unboxed!({ ... })` - Wraps a block that produces a `Result`
///
/// # Returns
///
/// A [`ComposableResult<T, E>`](crate::types::ComposableResult) where the error type
/// is wrapped in a [`ComposableError`](crate::types::ComposableError) but not boxed.
///
/// # Examples
///
/// ```rust
/// use error_rail::{rail_unboxed, group};
///
/// // Simple expression - returns unboxed error
/// let result = rail_unboxed!(Err::<(), &str>("failed"));
/// assert!(result.is_err());
/// // Error type is ComposableError<&str> (not boxed)
/// let _: error_rail::ComposableError<&str> = result.unwrap_err();
///
/// // Block syntax with multiple statements
/// let result = rail_unboxed!({
///     let value = std::fs::read_to_string("config.txt");
///     value
/// });
/// ```
#[macro_export]
macro_rules! rail_unboxed {
    ($expr:expr $(,)?) => {
        $crate::ErrorPipeline::new($expr).finish()
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

/// Combines multiple `Validation` results into a single one.
///
/// This macro allows you to validate multiple fields in parallel and accumulate
/// all errors if any occur. If all validations succeed, it returns a tuple
/// containing the successful values.
///
/// # Syntax
///
/// ```rust,ignore
/// validate!(
///     field1 = validation_expr1,
///     field2 = validation_expr2,
///     ...
/// )
/// ```
///
/// # Returns
///
/// `Validation<E, (T1, T2, ...)>` where `T1`, `T2` are the success types of the expressions.
///
/// # Examples
///
/// ```
/// use error_rail::{validate, validation::Validation};
///
/// let v1 = Validation::<&str, i32>::valid(1);
/// let v2 = Validation::<&str, i32>::valid(2);
///
/// let result = validate!(
///     a = v1,
///     b = v2
/// );
///
/// assert!(result.is_valid());
/// assert_eq!(result.into_value(), Some((1, 2)));
///
/// let e1 = Validation::<&str, i32>::invalid("error1");
/// let e2 = Validation::<&str, i32>::invalid("error2");
///
/// let result = validate!(
///     a = e1,
///     b = e2
/// );
///
/// assert!(result.is_invalid());
/// assert_eq!(result.into_errors().unwrap().len(), 2);
/// ```
#[macro_export]
macro_rules! validate {
    ($($key:ident = $val:expr),+ $(,)?) => {{
        match ($($val),+) {
            ( $( $crate::validation::Validation::Valid($key) ),+ ) => {
                $crate::validation::Validation::Valid( ($($key),+) )
            }
            ( $( ref $key ),+ ) => {
                let mut errors = $crate::ErrorVec::new();
                $(
                    if let $crate::validation::Validation::Invalid(e) = $key {
                        errors.extend(e.iter().cloned());
                    }
                )+
                $crate::validation::Validation::Invalid(errors.into())
            }
        }
    }};
}

// ============================================================================
// Async Macros (feature = "async")
// ============================================================================

/// Wraps a future in an [`AsyncErrorPipeline`](crate::async_ext::AsyncErrorPipeline).
///
/// This macro provides a convenient way to create an async error pipeline
/// from a future that returns a `Result`.
///
/// # Examples
///
/// ```rust,no_run
/// use error_rail::prelude_async::*;
///
/// #[derive(Debug)]
/// struct Data;
///
/// #[derive(Debug)]
/// struct ApiError;
///
/// async fn fetch_data(_id: u64) -> Result<Data, ApiError> {
///     Err(ApiError)
/// }
///
/// async fn example(id: u64) -> BoxedResult<Data, ApiError> {
///     rail_async!(fetch_data(id))
///         .with_context("fetching data")
///         .finish_boxed()
///         .await
/// }
/// ```
#[macro_export]
#[cfg(feature = "async")]
macro_rules! rail_async {
    ($fut:expr $(,)?) => {
        $crate::async_ext::AsyncErrorPipeline::new($fut)
    };
}

/// Attaches context to a future's error with format string support.
///
/// This macro provides a convenient shorthand for attaching context to
/// async operations, similar to how `context!` works for sync code.
///
/// # Syntax
///
/// - `ctx_async!(future, "literal message")` - Static message
/// - `ctx_async!(future, "format {}", arg)` - Formatted message (lazy)
///
/// # Examples
///
/// ```rust,no_run
/// use error_rail::prelude_async::*;
///
/// #[derive(Debug)]
/// struct User;
///
/// #[derive(Debug)]
/// struct Profile;
///
/// #[derive(Debug)]
/// struct ApiError;
///
/// async fn fetch_user(_id: u64) -> Result<User, ApiError> {
///     Err(ApiError)
/// }
///
/// async fn fetch_profile(_id: u64) -> Result<Profile, ApiError> {
///     Err(ApiError)
/// }
///
/// async fn example(id: u64) -> BoxedResult<User, ApiError> {
///     // Static message
///     let user = ctx_async!(fetch_user(id), "fetching user")
///         .await
///         .map_err(Box::new)?;
///
///     // With formatting (lazy evaluation)
///     let _profile = ctx_async!(fetch_profile(id), "fetching profile for user {}", id)
///         .await
///         .map_err(Box::new)?;
///
///     Ok(user)
/// }
/// ```
#[macro_export]
#[cfg(feature = "async")]
macro_rules! ctx_async {
    ($fut:expr, $msg:literal $(,)?) => {
        $crate::async_ext::FutureResultExt::ctx($fut, $msg)
    };
    ($fut:expr, $fmt:literal, $($arg:tt)* $(,)?) => {
        $crate::async_ext::FutureResultExt::with_ctx($fut, || format!($fmt, $($arg)*))
    };
}
