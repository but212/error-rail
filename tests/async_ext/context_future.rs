use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use error_rail::async_ext::ContextFuture;

#[test]
fn context_future_is_send_sync() {
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}

    // ContextFuture should be Send + Sync if inner types are
    assert_send::<ContextFuture<std::future::Ready<Result<(), ()>>, fn() -> &'static str>>();
    assert_sync::<ContextFuture<std::future::Ready<Result<(), ()>>, fn() -> &'static str>>();
}

struct PendingFuture;
impl Future for PendingFuture {
    type Output = Result<i32, &'static str>;
    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        Poll::Pending
    }
}

#[tokio::test]
async fn test_context_future_pending() {
    let fut = PendingFuture;
    let mut wrapped = ContextFuture::new(fut, || "context");

    // Manual polling to verify Pending
    use core::task::{RawWaker, RawWakerVTable, Waker};

    fn noop(_: *const ()) {}
    fn clone(p: *const ()) -> RawWaker {
        RawWaker::new(p, &VTABLE)
    }
    static VTABLE: RawWakerVTable = RawWakerVTable::new(clone, noop, noop, noop);
    let raw_waker = RawWaker::new(core::ptr::null(), &VTABLE);
    let waker = unsafe { Waker::from_raw(raw_waker) };

    let mut cx = Context::from_waker(&waker);
    let mut wrapped = Pin::new(&mut wrapped);

    assert_eq!(wrapped.as_mut().poll(&mut cx), Poll::Pending);
}
