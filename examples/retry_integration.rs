//! Example: Integrating error-rail with external retry libraries
//!
//! This example demonstrates how to use error-rail's TransientError trait
//! and ErrorPipeline methods to integrate with external retry/circuit-breaker
//! libraries without error-rail implementing retry logic itself.
//!
//! # Design Philosophy
//!
//! error-rail focuses on error classification and context, not retry logic.
//! This separation of concerns allows you to:
//! - Use your preferred retry library (backoff, retry, again, tokio-retry)
//! - Implement custom retry strategies
//! - Easily swap retry implementations without changing error handling code

use error_rail::{
    context, group, traits::TransientError, ComposableError, ErrorContext, ErrorPipeline,
};
use std::time::Duration;

// =============================================================================
// Step 1: Define domain errors with TransientError trait
// =============================================================================

/// Example API error that classifies transient vs permanent failures
#[derive(Debug, Clone)]
pub enum ApiError {
    /// Rate limited - transient, should respect Retry-After header
    RateLimited { retry_after_secs: u64 },
    /// Network timeout - transient, can retry immediately
    Timeout,
    /// Server overloaded - transient, use exponential backoff
    ServiceUnavailable,
    /// Invalid request - permanent, don't retry
    BadRequest(String),
    /// Authentication failed - permanent, don't retry
    Unauthorized,
    /// Resource not found - permanent, don't retry
    NotFound,
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RateLimited { retry_after_secs } => {
                write!(f, "rate limited, retry after {} seconds", retry_after_secs)
            },
            Self::Timeout => write!(f, "request timed out"),
            Self::ServiceUnavailable => write!(f, "service temporarily unavailable"),
            Self::BadRequest(msg) => write!(f, "bad request: {}", msg),
            Self::Unauthorized => write!(f, "unauthorized"),
            Self::NotFound => write!(f, "resource not found"),
        }
    }
}

impl TransientError for ApiError {
    fn is_transient(&self) -> bool {
        matches!(
            self,
            ApiError::RateLimited { .. } | ApiError::Timeout | ApiError::ServiceUnavailable
        )
    }

    fn retry_after_hint(&self) -> Option<Duration> {
        match self {
            ApiError::RateLimited { retry_after_secs } => {
                Some(Duration::from_secs(*retry_after_secs))
            },
            ApiError::Timeout => Some(Duration::from_millis(100)),
            ApiError::ServiceUnavailable => Some(Duration::from_secs(1)),
            _ => None,
        }
    }

    fn max_retries_hint(&self) -> Option<u32> {
        match self {
            ApiError::RateLimited { .. } => Some(3),
            ApiError::Timeout => Some(5),
            ApiError::ServiceUnavailable => Some(3),
            _ => None,
        }
    }
}

// =============================================================================
// Step 2: Using ErrorPipeline with retry helpers
// =============================================================================

/// Simulates an API call that might fail
fn call_api(attempt: u32) -> Result<String, ApiError> {
    match attempt {
        1 => Err(ApiError::Timeout),
        2 => Err(ApiError::RateLimited { retry_after_secs: 1 }),
        3 => Ok("Success!".to_string()),
        _ => Err(ApiError::ServiceUnavailable),
    }
}

/// Example: Manual retry loop using ErrorPipeline
fn manual_retry_example() -> Result<String, ComposableError<ApiError>> {
    let max_attempts = 5;

    for attempt in 1..=max_attempts {
        let result = call_api(attempt);

        let pipeline = ErrorPipeline::new(result)
            .with_context(group!(tag("api"), metadata("attempt", format!("{}", attempt))))
            .with_retry_context(attempt);

        // Use is_transient() to check without consuming pipeline
        if pipeline.is_transient() {
            if let Some(wait_time) = pipeline.retry_after_hint() {
                println!(
                    "Attempt {} failed (transient), waiting {:?} before retry",
                    attempt, wait_time
                );
                // In real code: std::thread::sleep(wait_time);
            }
            continue;
        }

        // Either success or permanent error - stop retrying
        return pipeline.finish();
    }

    Err(ComposableError::new(ApiError::ServiceUnavailable)
        .with_context(context!("max retries exceeded"))
        .set_code(503))
}

/// Example: Using recover_transient for single retry attempt
fn recover_transient_example() -> Result<String, ComposableError<ApiError>> {
    let result = call_api(1); // Will fail with Timeout

    ErrorPipeline::new(result)
        .with_context(group!(tag("api"), tag("fetch_user")))
        .recover_transient(|_err| {
            // Try again
            call_api(3) // This will succeed
        })
        .finish()
}

// =============================================================================
// Step 3: Integration pattern with external retry library (pseudo-code)
// =============================================================================

/// Example integration pattern with `backoff` crate (pseudo-code)
///
/// ```ignore
/// use backoff::{ExponentialBackoff, Error as BackoffError};
///
/// fn with_backoff<T, E>(operation: impl Fn() -> Result<T, E>) -> Result<T, ComposableError<E>>
/// where
///     E: TransientError + Clone + std::fmt::Display,
/// {
///     let backoff = ExponentialBackoff::default();
///
///     backoff::retry(backoff, || {
///         let result = operation();
///         let pipeline = ErrorPipeline::new(result);
///
///         // Use should_retry to determine if backoff should continue
///         match pipeline.should_retry() {
///             Some(p) => {
///                 // Transient error - tell backoff to retry
///                 Err(BackoffError::transient(p.finish().unwrap_err()))
///             }
///             None => {
///                 // Success or permanent error - stop
///                 pipeline.finish().map_err(BackoffError::permanent)
///             }
///         }
///     })
/// }
/// ```
fn _integration_pattern_placeholder() {}

// =============================================================================
// Step 4: Fingerprint for error deduplication
// =============================================================================

fn fingerprint_example() {
    let err1 = ComposableError::new(ApiError::Timeout)
        .with_context(ErrorContext::tag("api"))
        .with_context(ErrorContext::tag("user_service"))
        .set_code(504);

    let err2 = ComposableError::new(ApiError::Timeout)
        .with_context(ErrorContext::tag("api"))
        .with_context(ErrorContext::tag("user_service"))
        .set_code(504);

    // Same error configuration = same fingerprint
    assert_eq!(err1.fingerprint(), err2.fingerprint());
    println!("Error fingerprint: {}", err1.fingerprint_hex());

    // Fingerprint can be used for:
    // - Sentry issue grouping
    // - Log deduplication
    // - Alert throttling
}

fn main() {
    println!("=== Manual Retry Example ===");
    match manual_retry_example() {
        Ok(result) => println!("Success: {}", result),
        Err(err) => println!("Failed: {:#}", err),
    }

    println!("\n=== Recover Transient Example ===");
    match recover_transient_example() {
        Ok(result) => println!("Success: {}", result),
        Err(err) => println!("Failed: {:#}", err),
    }

    println!("\n=== Fingerprint Example ===");
    fingerprint_example();
}
