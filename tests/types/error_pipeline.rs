use error_rail::ErrorPipeline;

#[test]
fn test_error_pipeline_map_error() {
    let pipeline = ErrorPipeline::<i32, &str>::new(Err("error")).map_error(|e| e.len());

    let err = pipeline.finish().unwrap_err();
    assert_eq!(err.core_error(), &5);
}

#[test]
fn test_error_pipeline_recover() {
    let pipeline = ErrorPipeline::<i32, &str>::new(Err("error")).recover(|_| Ok(42));

    assert_eq!(pipeline.finish().unwrap(), 42);
}

#[test]
fn test_error_pipeline_map() {
    let pipeline = ErrorPipeline::<i32, &str>::new(Ok(21)).map(|x| x * 2);

    assert_eq!(pipeline.finish().unwrap(), 42);
}

#[test]
fn test_pipeline_recovery_clears_context() {
    // 1. Start with error and context
    let pipeline =
        ErrorPipeline::<i32, &str>::new(Err("initial")).with_context(error_rail::context!("ctx1"));

    // 2. Recover (should clear context and result in Ok)
    let pipeline = pipeline.fallback(42);

    // 3. Introduce new error after recovery
    let pipeline = pipeline.and_then(|_| -> Result<i32, &str> { Err("new error") });

    // 4. Assert that the final error has no context from before recovery
    let err = pipeline.finish().unwrap_err();
    assert_eq!(err.core_error(), &"new error");
    assert!(
        err.context().is_empty(),
        "Context from before recovery should be cleared."
    );
}

#[test]
fn test_pipeline_recover_safe_clears_context() {
    // 1. Start with error and context
    let pipeline =
        ErrorPipeline::<i32, &str>::new(Err("initial")).with_context(error_rail::context!("ctx1"));

    // 2. Recover_safe (should clear context and result in Ok)
    let pipeline = pipeline.recover_safe(|_| 42);

    // 3. Introduce new error after recovery
    let pipeline = pipeline.and_then(|_| -> Result<i32, &str> { Err("new error") });

    // 4. Assert that the final error has no context from before recovery
    let err = pipeline.finish().unwrap_err();
    assert_eq!(err.core_error(), &"new error");
    assert!(
        err.context().is_empty(),
        "Context from before recovery should be cleared."
    );
}

#[test]
fn test_fallback_on_ok_is_noop() {
    let pipeline_ok = ErrorPipeline::<i32, &str>::new(Ok(10)).fallback(42);
    let result_ok = pipeline_ok.finish().unwrap();
    assert_eq!(result_ok, 10);
}

#[test]
fn test_recover_safe_on_ok_is_noop() {
    let pipeline_ok = ErrorPipeline::<i32, &str>::new(Ok(10)).recover_safe(|_| 42);
    let result_ok = pipeline_ok.finish().unwrap();
    assert_eq!(result_ok, 10);
}
