//! Service Layer Error Handling Pattern
//!
//! Converting domain-specific errors into service-layer errors with contextual
//! information about the operation being performed.

use error_rail::{context, ComposableError, ErrorPipeline};

// Domain layer error
#[derive(Debug)]
enum DbError {
    #[allow(dead_code)]
    ConnectionFailed,
    #[allow(dead_code)]
    QueryFailed(String),
    NotFound,
}

impl std::fmt::Display for DbError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            DbError::ConnectionFailed => write!(f, "database connection failed"),
            DbError::QueryFailed(q) => write!(f, "query failed: {}", q),
            DbError::NotFound => write!(f, "record not found"),
        }
    }
}

impl std::error::Error for DbError {}

// Service layer functions
fn fetch_user_from_db(_user_id: u64) -> Result<String, DbError> {
    // Simulate database operation
    Err(DbError::NotFound)
}

fn process_user_request(user_id: u64) -> Result<String, Box<ComposableError<DbError>>> {
    let user_id_str = user_id.to_string(); // Use the parameter to avoid warning
    ErrorPipeline::new(fetch_user_from_db(user_id))
        .with_context(context!("processing user request for user_id: {}", user_id_str))
        .with_context(context!("fetching user profile for user_id: {}", user_id))
        .with_context("formatting profile data")
        .map(|data| format!("Profile: {}", data))
        .finish_boxed()
}

fn main() {
    match process_user_request(42) {
        Ok(profile) => println!("{}", profile),
        Err(e) => {
            eprintln!("Error: {}", e.error_chain());
            // Output: processing user request -> fetching user profile for user_id: 42 -> record not found
        },
    }
}
