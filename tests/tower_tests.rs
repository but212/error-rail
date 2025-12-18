//! Tests for Tower integration.
#![cfg(feature = "tower")]

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use error_rail::tower::{ErrorRailLayer, ErrorRailService, ServiceErrorExt};
use error_rail::ComposableError;
use tower::{Layer, Service};

#[derive(Clone)]
struct SimpleService;

impl tower::Service<()> for SimpleService {
    type Response = ();
    type Error = &'static str;
    type Future =
        std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), &'static str>> + Send>>;

    fn poll_ready(
        &mut self,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }

    fn call(&mut self, _req: ()) -> Self::Future {
        Box::pin(async { Ok(()) })
    }
}

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
    let mut service: ErrorRailService<MockService, &str> =
        MockService::failing().with_error_context("user-service");

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

#[test]
fn layer_clones_context() {
    let layer = ErrorRailLayer::new("test-context");
    let layer2 = layer.clone();
    assert_eq!(*layer.context(), *layer2.context());
}

#[tokio::test]
async fn test_service_inner_access() {
    let mut service = ErrorRailService::new("inner", "ctx");
    assert_eq!(*service.inner(), "inner");
    assert_eq!(*service.inner_mut(), "inner");
    assert_eq!(service.into_inner(), "inner");
}

struct MockStrService {
    #[allow(dead_code)]
    should_fail: bool,
}

impl MockStrService {
    fn failing() -> Self {
        Self { should_fail: true }
    }
}

impl Service<&'static str> for MockStrService {
    type Response = &'static str;
    type Error = &'static str;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Err("not ready"))
    }

    fn call(&mut self, _req: &'static str) -> Self::Future {
        Box::pin(async { Err("failed") })
    }
}

#[test]
fn test_service_poll_ready_error() {
    let mut service = ErrorRailService::new(MockStrService::failing(), "ctx");
    let waker = unsafe {
        use core::task::{RawWaker, RawWakerVTable, Waker};
        fn noop(_: *const ()) {}
        fn clone(p: *const ()) -> RawWaker {
            RawWaker::new(p, &VTABLE)
        }
        static VTABLE: RawWakerVTable = RawWakerVTable::new(clone, noop, noop, noop);
        Waker::from_raw(RawWaker::new(core::ptr::null(), &VTABLE))
    };
    let mut cx = Context::from_waker(&waker);
    assert!(service.poll_ready(&mut cx).is_ready());
    let res = match service.poll_ready(&mut cx) {
        Poll::Ready(Err(e)) => e,
        _ => panic!("expected error"),
    };
    assert_eq!(*res.core_error(), "not ready");
}

#[tokio::test]
async fn test_future_pending() {
    struct PendingService;
    impl Service<()> for PendingService {
        type Response = ();
        type Error = ();
        type Future = PendingFuture;
        fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), ()>> {
            Poll::Ready(Ok(()))
        }
        fn call(&mut self, _: ()) -> Self::Future {
            PendingFuture
        }
    }

    struct PendingFuture;
    impl Future for PendingFuture {
        type Output = Result<(), ()>;
        fn poll(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Self::Output> {
            Poll::Pending
        }
    }

    let mut service = ErrorRailService::new(PendingService, "ctx");
    let mut fut = service.call(());

    let waker = unsafe {
        use core::task::{RawWaker, RawWakerVTable, Waker};
        fn noop(_: *const ()) {}
        fn clone(p: *const ()) -> RawWaker {
            RawWaker::new(p, &VTABLE)
        }
        static VTABLE: RawWakerVTable = RawWakerVTable::new(clone, noop, noop, noop);
        Waker::from_raw(RawWaker::new(core::ptr::null(), &VTABLE))
    };
    let mut cx = Context::from_waker(&waker);
    assert_eq!(Pin::new(&mut fut).poll(&mut cx), Poll::Pending);
}

#[test]
fn test_error_rail_service_context() {
    let layer = ErrorRailLayer::new("test-context");
    let service: ErrorRailService<SimpleService, &str> = layer.layer(SimpleService);

    let ctx = service.context();
    assert_eq!(*ctx, "test-context");
}

#[test]
fn test_error_rail_service_context_owned_string() {
    let context = String::from("owned-context");
    let layer = ErrorRailLayer::new(context);
    let service: ErrorRailService<SimpleService, String> = layer.layer(SimpleService);

    let ctx = service.context();
    assert_eq!(ctx, "owned-context");
}

#[test]
fn test_error_rail_service_context_with_error_context() {
    use error_rail::ErrorContext;

    let context = ErrorContext::tag("api");
    let layer = ErrorRailLayer::new(context.clone());
    let service: ErrorRailService<SimpleService, ErrorContext> = layer.layer(SimpleService);

    let ctx = service.context();
    assert_eq!(ctx.message(), "[api]");
}
