//! Tests for AsyncErrorPipeline.

use error_rail::prelude_async::*;

#[tokio::test]
async fn pipeline_basic_success() {
    let result = AsyncErrorPipeline::new(async { Ok::<_, &str>(42) })
        .with_context("should not appear")
        .finish()
        .await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 42);
}

#[tokio::test]
async fn pipeline_basic_error() {
    let result = AsyncErrorPipeline::new(async { Err::<i32, _>("base error") })
        .with_context("operation context")
        .finish()
        .await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.error_chain().contains("operation context"));
    assert!(err.error_chain().contains("base error"));
}

#[tokio::test]
async fn pipeline_finish_boxed() {
    let result: BoxedResult<i32, &str> = AsyncErrorPipeline::new(async { Err::<i32, _>("error") })
        .with_context("context")
        .finish_boxed()
        .await;

    assert!(result.is_err());
    let boxed_err = result.unwrap_err();
    assert!(boxed_err.error_chain().contains("context"));
}

#[tokio::test]
async fn pipeline_multiple_contexts() {
    let result = AsyncErrorPipeline::new(async { Err::<i32, _>("base") })
        .with_context("ctx1")
        .with_context("ctx2")
        .with_context("ctx3")
        .finish()
        .await;

    assert!(result.is_err());
    let chain = result.unwrap_err().error_chain();
    assert!(chain.contains("ctx1"));
    assert!(chain.contains("ctx2"));
    assert!(chain.contains("ctx3"));
    assert!(chain.contains("base"));
}

#[tokio::test]
async fn pipeline_with_context_fn() {
    let user_id = 123u64;
    let result = AsyncErrorPipeline::new(async { Err::<i32, _>("not found") })
        .with_context_fn(|| format!("user_id: {}", user_id))
        .finish()
        .await;

    assert!(result.is_err());
    let chain = result.unwrap_err().error_chain();
    assert!(chain.contains("user_id: 123"));
}

#[tokio::test]
async fn pipeline_map_err() {
    let result = AsyncErrorPipeline::new(async { Err::<i32, _>("original") })
        .with_context("context")
        .map_err(|e| e.map_core(|_| "transformed"))
        .finish()
        .await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.core_error(), &"transformed");
    assert!(err.error_chain().contains("context"));
}
