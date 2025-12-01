use error_rail::async_ext::AsyncErrorPipeline;

#[test]
fn pipeline_is_send_sync() {
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}

    assert_send::<AsyncErrorPipeline<std::future::Ready<Result<(), ()>>>>();
    assert_sync::<AsyncErrorPipeline<std::future::Ready<Result<(), ()>>>>();
}
