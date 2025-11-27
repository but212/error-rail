use error_rail::traits::{BoxedResultExt, ResultExt};
use error_rail::ComposableError;

#[test]
fn test_ctx_on_err() {
    let result: Result<(), &str> = Err("original");
    let with_ctx = result.ctx("context added");
    assert!(with_ctx.is_err());

    let err = with_ctx.unwrap_err();
    assert!(err.error_chain().contains("context added"));
    assert!(err.error_chain().contains("original"));
}

#[test]
fn test_ctx_on_ok() {
    let result: Result<i32, &str> = Ok(42);
    let with_ctx = result.ctx("should not appear");
    assert_eq!(with_ctx.unwrap(), 42);
}

#[test]
fn test_ctx_with_lazy() {
    let mut called = false;
    let result: Result<(), &str> = Ok(());

    // Closure should NOT be called for Ok
    let _ = result.map_err(|e| {
        called = true;
        ComposableError::new(e)
    });
    assert!(!called);
}

#[test]
fn test_chaining() {
    let result: Result<(), &str> = Err("base error");
    let chained = result
        .ctx("first context")
        .map(|_| ())
        .ctx_boxed("second context");

    let err = chained.unwrap_err();
    let chain = err.error_chain();
    assert!(chain.contains("first context"));
    assert!(chain.contains("second context"));
}
