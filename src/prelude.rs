//! Convenience re-exports for common usage patterns.
//!
//! This prelude module provides the most commonly used items for quick starts.
//! Import everything with:
//!
//! ```
//! use error_rail::prelude::*;
//! ```
//!
//! # What's Included
//!
//! - **Macros**: [`context!`], [`group!`], [`rail!`]
//! - **Types**: [`ComposableError`], [`ErrorContext`], [`ErrorPipeline`], [`LazyGroupContext`]
//! - **Traits**: [`ResultExt`], [`BoxedResultExt`], [`IntoErrorContext`]
//!
//! # Examples
//!
//! ## 30-Second Quick Start
//!
//! ```
//! use error_rail::prelude::*;
//!
//! fn load_config() -> BoxedResult<String, std::io::Error> {
//!     std::fs::read_to_string("config.toml")
//!         .ctx("loading configuration")
//! }
//! ```
//!
//! ## With Lazy Context (2.1x Faster)
//!
//! ```
//! use error_rail::prelude::*;
//!
//! fn process_user(id: u64) -> BoxedResult<(), &'static str> {
//!     let result: Result<(), &str> = Err("not found");
//!     result.ctx_with(|| format!("processing user {}", id))
//! }
//! ```

// Macros
pub use crate::{context, group, rail};

// Core types
pub use crate::types::lazy_context::LazyGroupContext;
pub use crate::types::{ComposableError, ErrorContext, ErrorPipeline};

// Traits
pub use crate::traits::{BoxedResultExt, IntoErrorContext, ResultExt};

// Convenient type alias
use crate::types::alloc_type::Box;

/// Convenient result type alias for functions returning boxed composable errors.
///
/// This is the recommended return type for public API functions as it has
/// minimal stack footprint (8 bytes) while providing full error context.
///
/// # Examples
///
/// ```
/// use error_rail::prelude::*;
///
/// fn read_file(path: &str) -> BoxedResult<String, std::io::Error> {
///     std::fs::read_to_string(path)
///         .ctx("reading file")
/// }
/// ```
pub type BoxedResult<T, E> = Result<T, Box<ComposableError<E>>>;
