use error_rail::{
    context, context_accumulator, context_fn, error_pipeline, with_context, with_context_result,
    ErrorContext, ErrorPipeline,
};

#[test]
fn with_context_attaches_single_context() {
    let err = with_context("io failed", ErrorContext::tag("disk"));

    let contexts = err.context();
    assert_eq!(contexts.len(), 1);
    assert_eq!(contexts[0], ErrorContext::tag("disk"));
    assert_eq!(err.core_error(), &"io failed");
}

#[test]
fn with_context_result_boxes_composable_error() {
    let result: Result<(), &str> = Err("boom");
    let enriched = with_context_result(result, ErrorContext::new("during sync"));

    let err = enriched.unwrap_err();
    let contexts = err.context();
    assert_eq!(contexts.len(), 1);
    assert_eq!(contexts[0].message(), "during sync");
}

#[test]
fn with_context_result_leaves_success_untouched() {
    let ok: Result<&str, &str> = Ok("value");
    let enriched = with_context_result(ok, ErrorContext::tag("unused"));

    assert_eq!(enriched, Ok("value"));
}

#[test]
fn error_pipeline_applies_contexts_on_error() {
    let error = ErrorPipeline::<(), &str>::new(Err("fail"))
        .with_context(context!("user_id: {}", 42))
        .with_context(ErrorContext::tag("auth"))
        .finish()
        .unwrap_err();

    let contexts = error.context();
    assert_eq!(contexts.len(), 2);
    assert!(contexts[0].message().contains("auth"));
    assert!(contexts[1].message().contains("user_id: 42"));
}

#[test]
fn error_pipeline_skips_contexts_when_successful() {
    let value = ErrorPipeline::new(Ok::<_, &str>(10))
        .with_context(ErrorContext::tag("should not appear"))
        .finish();

    assert_eq!(value, Ok(10));
}

#[test]
fn context_fn_accumulates_reusable_context() {
    let wrapper = context_fn(ErrorContext::tag("db"));
    let err = wrapper("timeout");

    let contexts = err.context();
    assert_eq!(contexts.len(), 1);
    assert_eq!(err.core_error(), &"timeout");
    assert_eq!(contexts[0], ErrorContext::tag("db"));
}

#[test]
fn context_accumulator_attaches_multiple_entries() {
    let add_contexts = context_accumulator([
        ErrorContext::tag("api"),
        ErrorContext::metadata("request_id", "req-1"),
    ]);

    let err = add_contexts("rate limit");
    let contexts = err.context();

    assert_eq!(contexts.len(), 2);
    assert_eq!(contexts[0], ErrorContext::metadata("request_id", "req-1"));
    assert_eq!(contexts[1], ErrorContext::tag("api"));
}

#[test]
fn error_pipeline_helper_function_matches_constructor() {
    let pipeline_from_fn = error_pipeline::<(), &str>(Err("fail"))
        .with_context(ErrorContext::tag("fn"))
        .finish();

    let pipeline_from_new = ErrorPipeline::new(Err::<(), &str>("fail"))
        .with_context(ErrorContext::tag("fn"))
        .finish();

    assert_eq!(
        pipeline_from_fn.unwrap_err().context(),
        pipeline_from_new.unwrap_err().context()
    );
}
