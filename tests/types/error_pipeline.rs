use error_rail::ErrorPipeline;

#[test]
fn test_error_pipeline_map_error() {
    let pipeline = ErrorPipeline::<i32, &str>::new(Err("error")).map_error(|e| e.len());

    let err = pipeline.finish_boxed().unwrap_err();
    assert_eq!(err.core_error(), &5);
}

#[test]
fn test_error_pipeline_recover() {
    let pipeline = ErrorPipeline::<i32, &str>::new(Err("error")).recover(|_| Ok(42));

    assert_eq!(pipeline.finish_boxed().unwrap(), 42);
}

#[test]
fn test_error_pipeline_recover_clears_context() {
    // 1. Start with error and multiple contexts
    let pipeline = ErrorPipeline::<i32, &str>::new(Err("initial"))
        .with_context(error_rail::context!("ctx1"))
        .with_context(error_rail::context!("ctx2"))
        .with_context(error_rail::tag!("database"));

    // 2. Recover successfully (should clear all contexts and result in Ok)
    let pipeline = pipeline.recover(|_| Ok(42));

    // 3. Introduce new error after recovery
    let pipeline = pipeline.and_then(|_| -> Result<i32, &str> { Err("new error") });

    // 4. Assert that the final error has no context from before recovery
    let err = pipeline.finish_boxed().unwrap_err();
    assert_eq!(err.core_error(), &"new error");
    assert!(
        err.context().is_empty(),
        "All contexts from before recovery should be cleared."
    );
}

#[test]
fn test_error_pipeline_recover_failed_preserves_context() {
    // 1. Start with error and multiple contexts
    let pipeline = ErrorPipeline::<i32, &str>::new(Err("initial"))
        .with_context(error_rail::context!("ctx1"))
        .with_context(error_rail::context!("ctx2"))
        .with_context(error_rail::tag!("database"));

    // 2. Attempt recovery but fail (should preserve all contexts)
    let pipeline = pipeline.recover(|_| Err("recovery failed"));

    // 3. Assert that the error still has all original contexts
    let err = pipeline.finish_boxed().unwrap_err();
    assert_eq!(err.core_error(), &"recovery failed");
    assert!(
        !err.context().is_empty(),
        "All contexts should be preserved when recovery fails."
    );
    assert!(err
        .context()
        .iter()
        .any(|ctx| ctx.to_string().contains("ctx1")));
    assert!(err
        .context()
        .iter()
        .any(|ctx| ctx.to_string().contains("ctx2")));
}

#[test]
fn test_error_pipeline_chained_recovery_clears_all_contexts() {
    // 1. Start with error and accumulate multiple contexts
    let pipeline = ErrorPipeline::<i32, &str>::new(Err("initial"))
        .with_context(error_rail::context!("ctx1"))
        .with_context(error_rail::context!("ctx2"))
        .with_context(error_rail::tag!("database"));

    // 2. First recovery attempt fails (should preserve contexts)
    let pipeline = pipeline.recover(|_| Err("first recovery failed"));

    // 3. Add more context after failed recovery
    let pipeline = pipeline.with_context(error_rail::context!("ctx3"));

    // 4. Second recovery attempt succeeds (should clear ALL contexts)
    let pipeline = pipeline.recover(|_| Ok(42));

    // 5. Introduce new error after successful recovery
    let pipeline = pipeline.and_then(|_| -> Result<i32, &str> { Err("final error") });

    // 6. Assert that the final error has NO contexts from before recovery
    let err = pipeline.finish_boxed().unwrap_err();
    assert_eq!(err.core_error(), &"final error");
    assert!(
        err.context().is_empty(),
        "All accumulated contexts should be cleared after successful recovery."
    );
}

#[test]
fn test_error_pipeline_context_after_successful_recovery_starts_fresh() {
    // 1. Start with error and contexts
    let pipeline = ErrorPipeline::<i32, &str>::new(Err("initial"))
        .with_context(error_rail::context!("old_ctx1"))
        .with_context(error_rail::context!("old_ctx2"));

    // 2. Recover successfully (clears all contexts)
    let pipeline = pipeline.recover(|_| Ok(42));

    // 3. Add new context after recovery - this should NOT accumulate since result is Ok
    let pipeline = pipeline.with_context(error_rail::context!("new_ctx"));

    // 4. Introduce error after adding new context
    let pipeline = pipeline.and_then(|_| -> Result<i32, &str> { Err("new error") });

    // 5. Assert no contexts are present (new_ctx was never accumulated on success path)
    let err = pipeline.finish_boxed().unwrap_err();
    assert_eq!(err.core_error(), &"new error");
    assert_eq!(err.context().len(), 0); // with_context on Ok result is no-op
}

#[test]
fn test_error_pipeline_map() {
    let pipeline = ErrorPipeline::<i32, &str>::new(Ok(21)).map(|x| x * 2);

    assert_eq!(pipeline.finish_boxed().unwrap(), 42);
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
    let err = pipeline.finish_boxed().unwrap_err();
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
    let err = pipeline.finish_boxed().unwrap_err();
    assert_eq!(err.core_error(), &"new error");
    assert!(
        err.context().is_empty(),
        "Context from before recovery should be cleared."
    );
}

#[test]
fn test_fallback_on_ok_is_noop() {
    let pipeline_ok = ErrorPipeline::<i32, &str>::new(Ok(10)).fallback(42);
    let result_ok = pipeline_ok.finish_boxed().unwrap();
    assert_eq!(result_ok, 10);
}

#[test]
fn test_recover_safe_on_ok_is_noop() {
    let pipeline_ok = ErrorPipeline::<i32, &str>::new(Ok(10)).recover_safe(|_| 42);
    let result_ok = pipeline_ok.finish_boxed().unwrap();
    assert_eq!(result_ok, 10);
}
