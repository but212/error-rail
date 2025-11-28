//! Library Development Pattern
//!
//! Designing error types for a public library while using error-rail internally.

use error_rail::{ComposableError, ErrorPipeline};

// Public error type (opaque to users)
#[derive(Debug, Clone)]
pub enum MyLibError {
    InvalidInput(String),
    ProcessingFailed,
}

impl std::fmt::Display for MyLibError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            MyLibError::InvalidInput(msg) => write!(f, "invalid input: {}", msg),
            MyLibError::ProcessingFailed => write!(f, "processing failed"),
        }
    }
}

impl std::error::Error for MyLibError {}

// Internal error handling with error-rail
// Note: InternalResult type alias kept for documentation purposes
#[allow(dead_code)]
type InternalResult<T> = Result<T, Box<ComposableError<MyLibError>>>;

fn validate_input(input: &str) -> Result<(), MyLibError> {
    if input.is_empty() {
        return Err(MyLibError::InvalidInput("input cannot be empty".into()));
    }
    Ok(())
}

fn process_data(input: &str) -> Result<String, Box<ComposableError<MyLibError>>> {
    // Simulate processing
    if input.len() > 100 {
        return ErrorPipeline::new(Err(MyLibError::ProcessingFailed))
            .with_context("processing data - input too large")
            .finish_boxed();
    }

    ErrorPipeline::new(validate_input(input))
        .with_context("validating input")
        .with_context("preparing data for processing")
        .map(|_| format!("Processed: {}", input))
        .finish_boxed()
}

// Public API - hides error-rail implementation
pub fn process_user_data(input: &str) -> Result<String, MyLibError> {
    match process_data(input) {
        Ok(result) => Ok(result),
        Err(boxed_error) => Err(boxed_error.core_error().clone()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_input() {
        let result = process_user_data("hello");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Processed: hello");
    }

    #[test]
    fn test_empty_input() {
        let result = process_user_data("");
        assert!(result.is_err());
        match result.unwrap_err() {
            MyLibError::InvalidInput(msg) => assert!(msg.contains("empty")),
            _ => panic!("Expected InvalidInput error"),
        }
    }

    #[test]
    fn test_large_input() {
        let large_input = "x".repeat(200);
        let result = process_user_data(&large_input);
        assert!(result.is_err());
        match result.unwrap_err() {
            MyLibError::ProcessingFailed => {} // Expected
            _ => panic!("Expected ProcessingFailed error"),
        }
    }
}

fn main() {
    // Example usage
    match process_user_data("test input") {
        Ok(result) => println!("Success: {}", result),
        Err(e) => println!("Error: {}", e),
    }
}
