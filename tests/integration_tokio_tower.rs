//! Integration tests for Tokio + Tower functionality
//! This file ensures both dev-dependencies are detected by cargo-udeps

#![cfg(feature = "tower")]

use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::Duration;

use error_rail::prelude_async::*;
use error_rail::tower::{ErrorRailLayer, ServiceErrorExt};
use error_rail::traits::TransientError;
use error_rail::ComposableError;
use tower::{Layer, Service};

#[derive(Debug, Clone)]
enum IntegrationError {
    Transient(String),
    Permanent(String),
}

impl std::fmt::Display for IntegrationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IntegrationError::Transient(msg) => write!(f, "transient: {}", msg),
            IntegrationError::Permanent(msg) => write!(f, "permanent: {}", msg),
        }
    }
}

impl TransientError for IntegrationError {
    fn is_transient(&self) -> bool {
        matches!(self, IntegrationError::Transient(_))
    }
}

/// A mock service that simulates network operations with tokio
#[derive(Clone)]
struct AsyncMockService {
    failure_rate: u32,
    call_count: Arc<AtomicU32>,
}

impl AsyncMockService {
    fn new(failure_rate: u32) -> Self {
        Self {
            failure_rate,
            call_count: Arc::new(AtomicU32::new(0)),
        }
    }
}

impl Service<String> for AsyncMockService {
    type Response = String;
    type Error = IntegrationError;
    type Future = Pin<Box<dyn Future<Output = Result<String, IntegrationError>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: String) -> Self::Future {
        let call_count = self.call_count.clone();
        let failure_rate = self.failure_rate;

        Box::pin(async move {
            let count = call_count.fetch_add(1, Ordering::SeqCst);

            // Simulate async work
            tokio::time::sleep(Duration::from_millis(10)).await;

            if count % failure_rate == 0 {
                Err(IntegrationError::Transient(
                    "Service temporarily unavailable".to_string(),
                ))
            } else {
                Ok(format!("processed: {}", req))
            }
        })
    }
}

#[tokio::test]
async fn tower_service_with_tokio_runtime() {
    let layer = ErrorRailLayer::new("integration-test");
    let mut service = layer.layer(AsyncMockService::new(10)); // Fail every 10 calls

    let result = service.call("test-request".to_string()).await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "processed: test-request");
}

#[tokio::test]
async fn tower_service_error_with_context() {
    let layer = ErrorRailLayer::new("api-gateway");
    let mut service = layer.layer(AsyncMockService::new(1)); // Always fail for testing

    let result = service.call("failing-request".to_string()).await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.error_chain().contains("api-gateway"));
    assert!(err
        .error_chain()
        .contains("Service temporarily unavailable"));
}

#[tokio::test]
async fn tower_service_with_retry_using_tokio() {
    let counter = Arc::new(AtomicU32::new(0));
    let counter_clone = counter.clone();

    // Use retry_transient with a service-like function
    let result = retry_transient(
        move || {
            let c = counter_clone.clone();
            async move {
                let count = c.fetch_add(1, Ordering::SeqCst);
                if count < 2 {
                    Err(IntegrationError::Transient("temporary failure".into()))
                } else {
                    Ok("success-after-retry".to_string())
                }
            }
        },
        ExponentialBackoff::new().with_max_attempts(5),
    )
    .await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "success-after-retry");
    assert_eq!(counter.load(Ordering::SeqCst), 3);
}

#[tokio::test]
async fn tower_service_timeout_with_tokio() {
    let layer = ErrorRailLayer::new("timeout-service");
    let mut service = layer.layer(AsyncMockService::new(100)); // Don't fail

    // Test with timeout
    let result: TimeoutResult<String, ComposableError<IntegrationError>> = try_with_timeout(
        Duration::from_millis(100),
        service.call("timeout-test".to_string()),
    )
    .await;

    assert!(result.is_ok());
    match result {
        TimeoutResult::Ok(value) => assert_eq!(value, "processed: timeout-test"),
        _ => panic!("Expected Ok result"),
    }
}

#[tokio::test]
async fn tower_service_with_multiple_layers() {
    let layer1 = ErrorRailLayer::new("auth-layer");
    let layer2 = ErrorRailLayer::new("logging-layer");

    let inner = AsyncMockService::new(50);
    let service1 = layer1.layer(inner);
    let mut service2 = layer2.layer(service1);

    let result = service2.call("multi-layer-test".to_string()).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn tower_service_error_ext_with_tokio() {
    let mut service = AsyncMockService::new(1) // Always fail
        .with_error_context("user-service");

    let result = service.call("error-ext-test".to_string()).await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.error_chain().contains("user-service"));
}

#[tokio::test]
async fn complex_integration_scenario() {
    // Simulate a real complex scenario with Tower + Tokio + ErrorRail
    let counter = Arc::new(AtomicU32::new(0));
    let counter_clone = counter.clone();

    let result = retry_transient(
        move || {
            let c = counter_clone.clone();
            async move {
                let count = c.fetch_add(1, Ordering::SeqCst);

                // Simulate authentication with Tower
                let auth_service = AsyncMockService::new(100); // Don't fail auth
                let auth_layer = ErrorRailLayer::new("auth");
                let mut auth = auth_layer.layer(auth_service);

                let auth_result = auth.call("user-token".to_string()).await;

                if count < 2 {
                    Err(IntegrationError::Transient("Rate limited".to_string()))
                } else if auth_result.is_err() {
                    Err(IntegrationError::Permanent("Auth failed".to_string()))
                } else {
                    Ok("integration-success".to_string())
                }
            }
        },
        ExponentialBackoff::new().with_max_attempts(3),
    )
    .await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "integration-success");
}
