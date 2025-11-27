use error_rail::types::LazyContext;
use error_rail::{backtrace, backtrace_force, ComposableError, ErrorPipeline};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

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

    // Should either be disabled backtrace (when env vars not set) or contain actual stack frames
    assert!(
        backtrace_str.contains("disabled backtrace") || backtrace_str.lines().count() > 1,
        "Expected either 'disabled backtrace' or multiple stack frame lines, got: {}",
        backtrace_str
    );
}

#[test]
fn backtrace_lazy_evaluation_works() {
    // Verify that the lazy context is only evaluated when the context is accessed.
    let was_called = Arc::new(AtomicBool::new(false));
    let was_called_clone = was_called.clone();

    // We can't easily observe the side-effect of `backtrace_force!` itself,
    // so we create a custom lazy context to test the lazy evaluation mechanism it relies on.
    let lazy_context = LazyContext::new(move || {
        was_called_clone.store(true, Ordering::SeqCst);
        "lazy message".to_string()
    });

    let err = ComposableError::<&str>::new("test error").with_context(lazy_context);

    // The closure should not have been called yet.
    assert!(
        !was_called.load(Ordering::SeqCst),
        "Closure was called before context access"
    );

    // Access the context, which should trigger the lazy evaluation.
    let _message = err.context()[0].message();

    // Now the closure should have been called.
    assert!(
        was_called.load(Ordering::SeqCst),
        "Closure was not called after context access"
    );
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
