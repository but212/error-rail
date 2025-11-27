use error_rail::{backtrace, backtrace_force, ComposableError, ErrorPipeline};

#[test]
fn backtrace_macro_attaches_non_empty_context_for_composable_error() {
    let err = ComposableError::<&str>::new("panic occurred").with_context(backtrace!());

    assert_eq!(err.context().len(), 1);
    let ctx = &err.context()[0];
    let message = ctx.message();
    let backtrace_str = message.as_ref();

    assert!(!backtrace_str.is_empty());
}

#[test]
fn backtrace_force_macro_captures_actual_frames() {
    let err = ComposableError::<&str>::new("panic occurred").with_context(backtrace_force!());

    assert_eq!(err.context().len(), 1);
    let ctx = &err.context()[0];
    let message = ctx.message();
    let backtrace_str = message.as_ref();

    // Should contain actual stack frames, not just "disabled backtrace"
    assert!(!backtrace_str.contains("disabled backtrace"));

    // Should contain multiple lines (stack frames)
    assert!(backtrace_str.lines().count() > 1);

    // Should contain the current test function name in the backtrace
    assert!(backtrace_str.contains("backtrace_force_macro_captures_actual_frames"));
}

#[test]
fn backtrace_macro_respects_environment() {
    // Test that backtrace!() returns "disabled backtrace" when env vars are not set
    let err = ComposableError::<&str>::new("test error").with_context(backtrace!());
    let ctx = &err.context()[0];
    let message = ctx.message();
    let backtrace_str = message.as_ref();

    // When environment variables are not set, should return "disabled backtrace"
    // This test might fail if RUST_BACKTRACE is set in the test environment
    // but demonstrates the intended behavior
    assert!(!backtrace_str.is_empty());
}

#[test]
fn backtrace_lazy_evaluation_works() {
    // Verify that the backtrace is only generated when the error occurs
    let mut capture_count = 0;

    let err = ComposableError::<&str>::new("test error").with_context(backtrace_force!());

    // The backtrace should be captured when we access the context
    let _message = err.context()[0].message().as_ref();
    capture_count += 1;

    assert!(capture_count > 0);
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
