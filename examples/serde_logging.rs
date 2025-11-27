//! Example: Structured Logging with Serde
//!
//! This example demonstrates how to use `ComposableError` with `serde` to generate
//! structured JSON logs when the serde feature is enabled. When serde is not available,
//! it shows basic error handling functionality.
//! This is useful for sending error reports to logging
//! infrastructure (e.g., ELK stack, Splunk, CloudWatch).

use error_rail::{ComposableError, ErrorContext};
#[cfg(feature = "serde")]
use serde::Serialize;

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug)]
struct MyError {
    details: String,
}

impl std::fmt::Display for MyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl core::error::Error for MyError {}

#[cfg(feature = "serde")]
fn demonstrate_serde_logging() {
    println!("=== Serde Feature Enabled ===");

    // 1. Create a base error
    let base_error = MyError {
        details: "Database connection timeout".to_string(),
    };

    // 2. Wrap it in ComposableError and add rich context
    let error = ComposableError::<MyError>::new(base_error)
        .with_context(ErrorContext::tag("database"))
        .with_context(ErrorContext::metadata("host", "db-primary-01"))
        .with_context(ErrorContext::metadata("retry_count", "3"))
        .with_context(ErrorContext::location(file!(), line!()))
        .set_code(503);

    // 3. Serialize to JSON
    // Note: In a real application, you might use `serde_json::to_string` or similar.
    // Here we print it to stdout.
    match serde_json::to_string_pretty(&error) {
        Ok(json) => {
            println!("Structured Error Log:\n{}", json);
        }
        Err(e) => {
            eprintln!("Failed to serialize error: {}", e);
        }
    }
}

#[cfg(not(feature = "serde"))]
fn demonstrate_basic_logging() {
    println!("=== Serde Feature Not Available ===");
    println!("This example requires the 'serde' feature to demonstrate structured JSON logging.");
    println!("To enable serde support, run: cargo run --example serde_logging --features serde");
    println!("\nHowever, you can still use ComposableError for basic error handling:");

    // Demonstrate basic error handling without serde
    let base_error = MyError {
        details: "Database connection timeout".to_string(),
    };

    let error = ComposableError::<MyError>::new(base_error)
        .with_context(ErrorContext::tag("database"))
        .with_context(ErrorContext::metadata("host", "db-primary-01"))
        .with_context(ErrorContext::metadata("retry_count", "3"))
        .with_context(ErrorContext::location(file!(), line!()))
        .set_code(503);

    println!("Error chain: {}", error.error_chain());
    println!("Error display: {}", error);
}

fn main() {
    #[cfg(feature = "serde")]
    {
        demonstrate_serde_logging();
    }

    #[cfg(not(feature = "serde"))]
    {
        demonstrate_basic_logging();
    }
}
