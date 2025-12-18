//! Integration tests for async extensions.

#[cfg(feature = "async")]
mod context_future;

#[cfg(feature = "async")]
mod future_ext_tests;

#[cfg(feature = "async")]
mod macro_tests;

#[cfg(feature = "async")]
mod pipeline_tests;

#[cfg(feature = "async")]
mod pipeline;

#[cfg(feature = "async")]
mod retry_tests;

#[cfg(feature = "async")]
mod validation_tests;

#[cfg(feature = "async")]
mod retry;

#[cfg(feature = "tokio")]
mod tokio_tests;

#[cfg(feature = "tracing")]
mod tracing_tests;
