//! Example: Tower integration with error-rail
//!
//! This example demonstrates how to use error-rail's Tower integration
//! to add consistent error context across service layers.
//!
//! # Requirements
//!
//! Run with: `cargo run --example async_tower_integration --features tower`

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use error_rail::tower::{ErrorRailLayer, ServiceErrorExt};
use tower::{Layer, Service};

// =============================================================================
// Domain types and errors
// =============================================================================

#[derive(Debug, Clone)]
struct User {
    #[allow(dead_code)]
    id: u64,
    #[allow(dead_code)]
    name: String,
}

#[derive(Debug, Clone)]
struct ApiError(String);

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// =============================================================================
// Mock services
// =============================================================================

/// Database service that fetches users
#[derive(Clone)]
struct DatabaseService;

impl Service<u64> for DatabaseService {
    type Response = User;
    type Error = ApiError;
    type Future = Pin<Box<dyn Future<Output = Result<User, ApiError>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, id: u64) -> Self::Future {
        Box::pin(async move {
            match id {
                1 => Ok(User {
                    id: 1,
                    name: "Alice".to_string(),
                }),
                2 => Ok(User {
                    id: 2,
                    name: "Bob".to_string(),
                }),
                _ => Err(ApiError(format!("user {} not found", id))),
            }
        })
    }
}

// =============================================================================
// Using ErrorRailLayer
// =============================================================================

#[tokio::main]
async fn main() {
    println!("=== Tower Integration Example ===\n");

    // Example 1: Single layer with ErrorRailLayer
    println!("1. Using ErrorRailLayer:");
    let layer = ErrorRailLayer::new("database-layer");
    let mut service = layer.layer(DatabaseService);

    match service.call(1).await {
        Ok(user) => println!("   Success: {:?}", user),
        Err(err) => println!("   Error: {}", err.error_chain()),
    }

    match service.call(999).await {
        Ok(user) => println!("   Success: {:?}", user),
        Err(err) => println!("   Error: {}", err.error_chain()),
    }

    // Example 2: Using ServiceErrorExt trait
    println!("\n2. Using ServiceErrorExt trait:");
    let mut service = DatabaseService.with_error_context("api-gateway");

    match service.call(1).await {
        Ok(user) => println!("   Success: {:?}", user),
        Err(err) => println!("   Error: {}", err.error_chain()),
    }

    match service.call(999).await {
        Ok(user) => println!("   Success: {:?}", user),
        Err(err) => println!("   Error: {}", err.error_chain()),
    }

    // Example 3: Inner service access
    println!("\n3. Accessing inner service:");
    let layer = ErrorRailLayer::new("wrapper");
    let wrapped = layer.layer(DatabaseService);
    let _inner: &DatabaseService = wrapped.inner();
    println!("   Inner service accessible via .inner()");

    println!("\n=== Done ===");
}
