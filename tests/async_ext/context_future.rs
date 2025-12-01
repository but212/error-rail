use error_rail::async_ext::ContextFuture;

#[test]
fn context_future_is_send_sync() {
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}

    // ContextFuture should be Send + Sync if inner types are
    assert_send::<ContextFuture<std::future::Ready<Result<(), ()>>, fn() -> &'static str>>();
    assert_sync::<ContextFuture<std::future::Ready<Result<(), ()>>, fn() -> &'static str>>();
}
