//! Tests for Tower integration.
#![cfg(feature = "ecosystem")]

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use error_rail::tower::{ErrorRailLayer, ErrorRailService, ServiceErrorExt};
use error_rail::ComposableError;
use tower::{Layer, Service};

/// A simple mock service for testing.
#[derive(Clone)]
struct MockService {
    should_fail: bool,
}

impl MockService {
    fn success() -> Self {
        Self { should_fail: false }
    }

    fn failing() -> Self {
        Self { should_fail: true }
    }
}

impl Service<String> for MockService {
    type Response = String;
    type Error = &'static str;
    type Future = Pin<Box<dyn Future<Output = Result<String, &'static str>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: String) -> Self::Future {
        let should_fail = self.should_fail;
        Box::pin(async move {
            if should_fail {
                Err("service error")
            } else {
                Ok(format!("processed: {}", req))
            }
        })
    }
}

#[tokio::test]
async fn layer_creates_service() {
    let layer = ErrorRailLayer::new("test-context");
    let inner = MockService::success();
    let _service: ErrorRailService<MockService, &str> = layer.layer(inner);
}

#[tokio::test]
async fn service_passes_through_success() {
    let layer = ErrorRailLayer::new("test-context");
    let mut service = layer.layer(MockService::success());

    let result = service.call("hello".to_string()).await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "processed: hello");
}

#[tokio::test]
async fn service_wraps_error_with_context() {
    let layer = ErrorRailLayer::new("api-gateway");
    let mut service = layer.layer(MockService::failing());

    let result = service.call("test".to_string()).await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.error_chain().contains("api-gateway"));
    assert!(err.error_chain().contains("service error"));
}

#[tokio::test]
async fn service_error_ext_trait() {
    let mut service = MockService::failing().with_error_context("user-service");

    let result = service.call("request".to_string()).await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.error_chain().contains("user-service"));
}

#[tokio::test]
async fn service_inner_access() {
    let layer = ErrorRailLayer::new("context");
    let service = layer.layer(MockService::success());

    // Test inner access methods
    let _inner: &MockService = service.inner();
}

#[tokio::test]
async fn service_into_inner() {
    let layer = ErrorRailLayer::new("context");
    let service = layer.layer(MockService::success());

    let _inner: MockService = service.into_inner();
}

#[test]
fn layer_is_clone() {
    fn assert_clone<T: Clone>() {}
    assert_clone::<ErrorRailLayer<&str>>();
}

#[test]
fn service_is_clone() {
    let layer = ErrorRailLayer::new("context");
    let service = layer.layer(MockService::success());
    let _service2 = service.clone();
}

#[tokio::test]
async fn multiple_layers_stack() {
    let layer1 = ErrorRailLayer::new("layer-1");
    let layer2 = ErrorRailLayer::new("layer-2");

    let inner = MockService::failing();
    let service1 = layer1.layer(inner);
    let mut service2 = layer2.layer(service1);

    let result: Result<String, ComposableError<ComposableError<&str>>> =
        service2.call("test".to_string()).await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    // Both layers should have added context
    let chain = err.error_chain();
    assert!(chain.contains("layer-2"));
    // Inner layer context is in the nested error
}
