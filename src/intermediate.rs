//! Intermediate API level for service developers.
//!
//! This module provides advanced error handling features suitable for service layer development,
//! including transient error handling, retry policies, and error formatting configuration.

// Transient Error Handling
pub use crate::traits::{TransientError, TransientErrorExt};

// Error Formatting
pub use crate::types::error_formatter::ErrorFormatter;

// Fingerprinting
pub use crate::types::composable_error::FingerprintConfig;
