//! HTTP API Error Responses Pattern
//!
//! Converting internal errors to structured HTTP responses with appropriate
//! status codes.

use error_rail::{context, ComposableError, ErrorPipeline};

#[derive(Debug)]
enum ApiError {
    NotFound,
    BadRequest(String),
    #[allow(dead_code)]
    Unauthorized,
    #[allow(dead_code)]
    Internal(String),
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ApiError::NotFound => write!(f, "resource not found"),
            ApiError::Unauthorized => write!(f, "unauthorized"),
            ApiError::BadRequest(msg) => write!(f, "bad request: {}", msg),
            ApiError::Internal(msg) => write!(f, "internal error: {}", msg),
        }
    }
}

impl std::error::Error for ApiError {}

// Map error to HTTP status code
fn error_to_status_code(err: &ComposableError<ApiError>) -> u16 {
    match err.core_error() {
        ApiError::NotFound => 404,
        ApiError::Unauthorized => 401,
        ApiError::BadRequest(_) => 400,
        ApiError::Internal(_) => 500,
    }
}

// API handler
fn get_resource(resource_id: &str) -> Result<String, Box<ComposableError<ApiError>>> {
    if resource_id.is_empty() {
        return ErrorPipeline::new(Err(ApiError::BadRequest("resource_id cannot be empty".into())))
            .with_context("validating resource_id")
            .finish_boxed();
    }

    // Simulate resource fetch
    ErrorPipeline::new(Err(ApiError::NotFound))
        .with_context(context!("fetching resource: {}", resource_id))
        .finish_boxed()
}

// Convert to HTTP response
fn handle_request(resource_id: &str) -> (u16, String) {
    match get_resource(resource_id) {
        Ok(data) => (200, data),
        Err(e) => {
            let status = error_to_status_code(&*e);
            let body = if status >= 500 {
                // Include full error chain for 5xx errors (for debugging)
                e.error_chain()
            } else {
                // Only show core error for 4xx errors (security)
                e.core_error().to_string()
            };
            (status, body)
        },
    }
}

fn main() {
    let (status, body) = handle_request("");
    println!("Status: {}, Body: {}", status, body);
    // Output: Status: 400, Body: bad request: resource_id cannot be empty
}
