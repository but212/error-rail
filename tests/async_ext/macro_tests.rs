//! Tests for async macros (rail_async!, ctx_async!).

use error_rail::prelude_async::*;

#[tokio::test]
async fn rail_async_basic() {
    let result = rail_async!(async { Err::<i32, _>("error") })
        .with_context("context")
        .finish()
        .await;

    assert!(result.is_err());
    let chain = result.unwrap_err().error_chain();
    assert!(chain.contains("context"));
    assert!(chain.contains("error"));
}

#[tokio::test]
async fn rail_async_success() {
    let result = rail_async!(async { Ok::<_, &str>(42) })
        .with_context("unused context")
        .finish()
        .await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 42);
}

#[tokio::test]
async fn ctx_async_static_message() {
    let result = ctx_async!(async { Err::<i32, _>("base") }, "static msg").await;

    assert!(result.is_err());
    let chain = result.unwrap_err().error_chain();
    assert!(chain.contains("static msg"));
    assert!(chain.contains("base"));
}

#[tokio::test]
async fn ctx_async_formatted_message() {
    let id = 42u64;
    let result = ctx_async!(async { Err::<i32, _>("not found") }, "user {}", id).await;

    assert!(result.is_err());
    let chain = result.unwrap_err().error_chain();
    assert!(chain.contains("user 42"));
    assert!(chain.contains("not found"));
}

#[tokio::test]
async fn ctx_async_success_path() {
    let result = ctx_async!(async { Ok::<_, &str>(100) }, "unused").await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 100);
}
