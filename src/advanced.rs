//! Advanced API level for library authors and power users.
//!
//! This module exposes internal structures and low-level building blocks.
//! Use these types when you need to extend the library or build custom error abstractions.

// Core Internals
pub use crate::types::composable_error::ComposableError;
pub use crate::types::ErrorVec;

// Context Builders
pub use crate::types::error_context::{ErrorContextBuilder, GroupContext, Location};

// Lazy Context
pub use crate::types::lazy_context::{LazyContext, LazyGroupContext};

// Low-level Pipeline Operations
// (ErrorPipeline is exported in prelude, but advanced usage might need specific traits or internals)
pub use crate::types::error_pipeline::ErrorPipeline;
