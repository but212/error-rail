//! Example: Async API patterns with error-rail
//!
//! This example demonstrates common async error handling patterns
//! that work well with web frameworks like Axum, Actix-web, etc.
//!
//! # Requirements
//!
//! Run with: `cargo run --example async_api_patterns --features async-full,async-tokio`

use error_rail::context;
use error_rail::prelude_async::*;
use std::time::Duration;

// =============================================================================
// Domain types
// =============================================================================

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct User {
    id: u64,
    name: String,
    email: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct Order {
    id: u64,
    user_id: u64,
    total: f64,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
enum ApiError {
    NotFound(String),
    Database(String),
    Validation(String),
    Timeout,
    ServiceUnavailable,
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound(msg) => write!(f, "not found: {}", msg),
            Self::Database(msg) => write!(f, "database: {}", msg),
            Self::Validation(msg) => write!(f, "validation: {}", msg),
            Self::Timeout => write!(f, "request timed out"),
            Self::ServiceUnavailable => write!(f, "service unavailable"),
        }
    }
}

impl error_rail::traits::TransientError for ApiError {
    fn is_transient(&self) -> bool {
        matches!(self, ApiError::Timeout | ApiError::ServiceUnavailable)
    }

    fn retry_after_hint(&self) -> Option<Duration> {
        match self {
            ApiError::Timeout => Some(Duration::from_millis(100)),
            ApiError::ServiceUnavailable => Some(Duration::from_secs(1)),
            _ => None,
        }
    }
}

// =============================================================================
// Mock async operations
// =============================================================================

async fn fetch_user_from_db(id: u64) -> Result<User, ApiError> {
    // Simulate async database call
    tokio::time::sleep(Duration::from_millis(10)).await;

    match id {
        1 => Ok(User {
            id: 1,
            name: "Alice".to_string(),
            email: "alice@example.com".to_string(),
        }),
        2 => Ok(User {
            id: 2,
            name: "Bob".to_string(),
            email: "bob@example.com".to_string(),
        }),
        _ => Err(ApiError::NotFound(format!("user {}", id))),
    }
}

async fn fetch_orders_for_user(user_id: u64) -> Result<Vec<Order>, ApiError> {
    tokio::time::sleep(Duration::from_millis(10)).await;

    match user_id {
        1 => Ok(vec![
            Order {
                id: 101,
                user_id: 1,
                total: 99.99,
            },
            Order {
                id: 102,
                user_id: 1,
                total: 149.50,
            },
        ]),
        _ => Ok(vec![]),
    }
}

static mut CALL_COUNT: u32 = 0;

async fn flaky_external_service() -> Result<String, ApiError> {
    // Simulates a flaky service that fails first 2 times
    unsafe {
        CALL_COUNT += 1;
        if CALL_COUNT <= 2 {
            Err(ApiError::ServiceUnavailable)
        } else {
            Ok("external data".to_string())
        }
    }
}

// =============================================================================
// Pattern 1: Simple context attachment
// =============================================================================

async fn get_user(id: u64) -> BoxedResult<User, ApiError> {
    // Use .ctx() for simple static context
    fetch_user_from_db(id)
        .ctx("fetching user from database")
        .await
        .map_err(Box::new)
}

// =============================================================================
// Pattern 2: Lazy context with captured variables
// =============================================================================

async fn get_user_with_details(id: u64) -> BoxedResult<User, ApiError> {
    // Use .with_ctx() for lazy evaluation with captured variables
    fetch_user_from_db(id)
        .with_ctx(|| context!("fetching user {}", id))
        .await
        .map_err(Box::new)
}

// =============================================================================
// Pattern 3: AsyncErrorPipeline for complex flows
// =============================================================================

async fn get_user_with_orders(
    user_id: u64,
) -> Result<(User, Vec<Order>), error_rail::ComposableError<ApiError>> {
    // Step 1: Fetch user
    let user = rail_async!(fetch_user_from_db(user_id))
        .with_context(context!("loading user {}", user_id))
        .finish()
        .await?;

    // Step 2: Fetch orders (clone user_id for context)
    let user_name = user.name.clone();
    let orders = rail_async!(fetch_orders_for_user(user.id))
        .with_context(context!("loading orders for user {}", user_name))
        .finish()
        .await?;

    Ok((user, orders))
}

// =============================================================================
// Pattern 4: Multiple context layers
// =============================================================================

async fn api_handler_pattern(user_id: u64) -> Result<User, error_rail::ComposableError<ApiError>> {
    // For multiple context layers, use chained .ctx() on the future
    // Each .ctx() adds context to the same ComposableError
    fetch_user_from_db(user_id)
        .ctx("database query") // Most specific
        .await
        .map(|user| user)
        .map_err(|e| {
            e.with_context("user service")
                .with_context("GET /users/:id")
        })
}

// =============================================================================
// Pattern 5: Retry with TransientError
// =============================================================================

async fn call_with_retry() -> Result<String, error_rail::ComposableError<ApiError>> {
    // Reset call counter
    unsafe {
        CALL_COUNT = 0;
    }

    // Use retry_transient with automatic backoff
    retry_transient(
        flaky_external_service,
        ExponentialBackoff::new().with_max_attempts(5),
    )
    .await
}

// =============================================================================
// Pattern 6: Timeout handling
// =============================================================================

async fn fetch_with_timeout(user_id: u64) -> TimeoutResult<User, ApiError> {
    try_with_timeout(Duration::from_secs(5), fetch_user_from_db(user_id)).await
}

// =============================================================================
// Pattern 7: ctx_async! macro
// =============================================================================

async fn using_ctx_async_macro(id: u64) -> Result<User, error_rail::ComposableError<ApiError>> {
    // Static message - ctx_async! returns ContextFuture
    ctx_async!(fetch_user_from_db(id), "fetching user").await
}

// =============================================================================
// Main
// =============================================================================

#[tokio::main]
async fn main() {
    println!("=== Async API Patterns Example ===\n");

    // Pattern 1: Simple context
    println!("1. Simple context attachment:");
    match get_user(999).await {
        Ok(user) => println!("   Found: {:?}", user),
        Err(e) => println!("   Error: {}", e.error_chain()),
    }

    // Pattern 2: Lazy context
    println!("\n2. Lazy context with variables:");
    match get_user_with_details(999).await {
        Ok(user) => println!("   Found: {:?}", user),
        Err(e) => println!("   Error: {}", e.error_chain()),
    }

    // Pattern 3: Pipeline
    println!("\n3. AsyncErrorPipeline:");
    match get_user_with_orders(1).await {
        Ok((user, orders)) => {
            println!("   User: {:?}", user);
            println!("   Orders: {} total", orders.len());
        }
        Err(e) => println!("   Error: {}", e.error_chain()),
    }

    // Pattern 4: Multiple layers
    println!("\n4. Multiple context layers:");
    match api_handler_pattern(999).await {
        Ok(user) => println!("   Found: {:?}", user),
        Err(e) => println!("   Error chain: {}", e.error_chain()),
    }

    // Pattern 5: Retry
    println!("\n5. Retry with TransientError:");
    match call_with_retry().await {
        Ok(data) => println!("   Success after retries: {}", data),
        Err(e) => println!("   Error: {}", e.error_chain()),
    }

    // Pattern 6: Timeout
    println!("\n6. Timeout handling:");
    match fetch_with_timeout(1).await {
        TimeoutResult::Ok(user) => println!("   Found: {:?}", user),
        TimeoutResult::Err(e) => println!("   Error: {}", e.error_chain()),
        TimeoutResult::Timeout(d) => println!("   Timed out after {:?}", d),
    }

    // Pattern 7: ctx_async! macro
    println!("\n7. ctx_async! macro:");
    match using_ctx_async_macro(999).await {
        Ok(user) => println!("   Found: {:?}", user),
        Err(e) => println!("   Error: {}", e.error_chain()),
    }

    println!("\n=== Done ===");
}
