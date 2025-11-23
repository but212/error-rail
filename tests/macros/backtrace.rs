use error_rail::{backtrace, ComposableError, ErrorPipeline};

#[test]
fn backtrace_macro_attaches_non_empty_context_for_composable_error() {
    let err = ComposableError::<&str>::new("panic occurred").with_context(backtrace!());

    assert_eq!(err.context().len(), 1);
    let ctx = &err.context()[0];
    assert!(!ctx.message().as_ref().is_empty());
}

#[test]
fn backtrace_macro_attaches_non_empty_context_for_error_pipeline() {
    let error = ErrorPipeline::<(), &str>::new(Err("fail"))
        .with_context(backtrace!())
        .finish()
        .unwrap_err();

    assert_eq!(error.context().len(), 1);
    let ctx = &error.context()[0];
    assert!(!ctx.message().as_ref().is_empty());
}

#[test]
fn backtrace_macro_does_not_change_success_pipeline_result() {
    let result = ErrorPipeline::<(), &str>::new(Ok(()))
        .with_context(backtrace!())
        .finish();

    assert!(result.is_ok());
}
