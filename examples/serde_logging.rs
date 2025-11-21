//! Example: Structured Logging with Serde
//!
//! This example demonstrates how to use `ComposableError` with `serde` to generate
//! structured JSON logs. This is useful for sending error reports to logging
//! infrastructure (e.g., ELK stack, Splunk, CloudWatch).

use error_rail::{ComposableError, ErrorContext};
use serde::Serialize;

#[derive(Debug, Serialize)]
struct MyError {
    details: String,
}

impl std::fmt::Display for MyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl std::error::Error for MyError {}

fn main() {
    // 1. Create a base error
    let base_error = MyError {
        details: "Database connection timeout".to_string(),
    };

    // 2. Wrap it in ComposableError and add rich context
    let error = ComposableError::<MyError, u32>::new(base_error)
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
