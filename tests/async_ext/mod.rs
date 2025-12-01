//! Integration tests for async extensions.

#[cfg(feature = "async")]
mod future_ext_tests;

#[cfg(feature = "async")]
mod macro_tests;

#[cfg(feature = "async")]
mod pipeline_tests;

#[cfg(feature = "async-retry")]
mod retry_tests;

#[cfg(feature = "async-validation")]
mod validation_tests;

#[cfg(feature = "async-retry")]
mod retry;

#[cfg(feature = "async-tokio")]
mod tokio_tests;

#[cfg(feature = "tracing")]
mod tracing_tests;
