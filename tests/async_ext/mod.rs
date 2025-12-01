//! Integration tests for async extensions.

#[cfg(feature = "async")]
mod future_ext_tests;

#[cfg(feature = "async")]
mod macro_tests;

#[cfg(feature = "async")]
mod pipeline_tests;

#[cfg(feature = "async")]
mod context_future;

#[cfg(feature = "async")]
mod pipeline;
