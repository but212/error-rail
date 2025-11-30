use error_rail::traits::IntoErrorContext;
use error_rail::{
    extract_context, format_error_chain, ComposableError, ErrorContext, ErrorPipeline, LazyContext,
};

mod formatting;

#[test]
fn composable_error_accumulates_contexts_and_code() {
    let err = ComposableError::new("read failure")
        .with_context(ErrorContext::tag("fs"))
        .with_context(ErrorContext::location("lib.rs", 10))
        .set_code(404);

    assert_eq!(err.error_code(), Some(404));
    let contexts = err.context();
    assert_eq!(contexts.len(), 2);
    assert_eq!(contexts[0], ErrorContext::location("lib.rs", 10));
    assert_eq!(contexts[1], ErrorContext::tag("fs"));
}

#[test]
fn format_error_chain_orders_contexts() {
    let err = ComposableError::new("boom")
        .with_context(ErrorContext::tag("pipeline"))
        .with_context(ErrorContext::new("step 2"));

    let chain = format_error_chain(&err);
    assert!(chain.contains("step 2"));
    assert!(chain.contains("[pipeline]"));
    assert!(chain.ends_with("boom"));
}

#[test]
fn extract_context_returns_lifo_order() {
    let err = ComposableError::new("oops")
        .with_context(ErrorContext::new("first"))
        .with_context(ErrorContext::new("second"));

    let contexts = extract_context(&err);
    assert_eq!(contexts[0].message(), "second");
    assert_eq!(contexts[1].message(), "first");
}

#[test]
fn lazy_context_evaluates_on_use() {
    let lazy = LazyContext::new(|| "computed".to_string());
    let ctx = lazy.into_error_context();

    assert_eq!(ctx.message(), "computed");
}

#[test]
fn error_pipeline_finish_without_box_returns_composable_error() {
    let error = ErrorPipeline::<(), &str>::new(Err("fail"))
        .with_context(ErrorContext::tag("ops"))
        .finish()
        .unwrap_err();

    assert_eq!(error.context().len(), 1);
    assert_eq!(error.context()[0], ErrorContext::tag("ops"));
}

#[test]
fn composable_error_map_core_preserves_context() {
    let err = ComposableError::<&str>::new("fail")
        .with_context(ErrorContext::tag("map"))
        .map_core(|msg| format!("wrapped: {msg}"));

    assert_eq!(err.core_error(), "wrapped: fail");
    assert_eq!(err.context().len(), 1);
}

pub mod accumulator;
pub mod composable_error;
pub mod error_context;
pub mod error_context_builder;
pub mod error_context_builder_example;
pub mod error_formatter;
pub mod error_pipeline;
pub mod pipeline_ops;
pub mod retry;
