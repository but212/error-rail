//! # Simple API - Beginner-Friendly Error Handling
//!
//! This module provides the **minimal surface area** for getting started with error-rail.
//! If you're new to the library, start here.
//!
//! # Golden Path (3 Rules)
//!
//! 1. **Return `BoxedResult` at function boundaries**
//! 2. **Add `.ctx()` only after I/O or external calls**
//! 3. **Use `Validation` when multiple errors can occur** (see [`crate::validation`])
//!
//! # Quick Start
//!
//! ```rust
//! use error_rail::simple::*;
//!
//! fn read_config() -> BoxedResult<String, std::io::Error> {
//!     std::fs::read_to_string("config.toml")
//!         .ctx("loading configuration")
//! }
//!
//! fn main() {
//!     if let Err(e) = read_config() {
//!         eprintln!("{}", e.error_chain());
//!         // loading configuration -> No such file or directory (os error 2)
//!     }
//! }
//! ```
//!
//! # What's Included
//!
//! | Item | Purpose |
//! |------|---------|
//! | [`BoxedResult`] | Return type for functions (8-byte stack footprint) |
//! | [`rail!`] | Wrap any `Result` and box the error |
//! | [`.ctx()`](ResultExt::ctx) | Add context to errors |
//! | [`.error_chain()`](ComposableError::error_chain) | Format error with full context chain |
//!
//! # What's NOT Included (Intentionally)
//!
//! These are available in [`crate::prelude`] or specialized modules:
//!
//! - `Validation` - For accumulating multiple errors (see [`crate::validation`])
//! - `Retry` / `TransientError` - For retry logic (see [`crate::intermediate`])
//! - `Fingerprint` - For error deduplication (see [`crate::intermediate`])
//! - `AsyncErrorPipeline` - For async chains (see [`crate::prelude_async`])
//!
//! # Anti-Patterns
//!
//! ```rust,compile_fail
//! # use error_rail::simple::*;
//! // ❌ DON'T: Chain .ctx() multiple times in one expression
//! fn bad() -> BoxedResult<String, std::io::Error> {
//!     std::fs::read_to_string("config.toml")
//!         .ctx("step 1")
//!         .ctx("step 2")  // Redundant - adds noise, not value
//! }
//! ```
//!
//! ```rust
//! # use error_rail::simple::*;
//! // ✅ DO: One .ctx() per I/O boundary
//! fn good() -> BoxedResult<String, std::io::Error> {
//!     std::fs::read_to_string("config.toml")
//!         .ctx("loading configuration")
//! }
//! ```
//!
//! # When NOT to Use error-rail
//!
//! - Simple scripts where you just print errors and exit
//! - Projects where the team has little Rust experience
//! - When `anyhow` or `eyre` already meets your needs
//!
//! # Relationship to std::error
//!
//! > **std::error defines error types. error-rail defines how errors flow.**
//!
//! error-rail wraps your existing error types and adds context propagation,
//! without requiring you to change your error definitions.

// Minimal macro - just rail! for beginners
pub use crate::rail;

// Core type for method access (error_chain, etc.)
pub use crate::types::ComposableError;

// Essential trait for .ctx() method
pub use crate::traits::ResultExt;

// The recommended return type
pub use crate::prelude::BoxedResult;
