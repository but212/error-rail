//! Tests for async validation functions.

use error_rail::prelude_async::*;
use error_rail::Validation;

fn validate_positive(n: i32) -> std::future::Ready<Validation<&'static str, i32>> {
    std::future::ready(if n > 0 {
        Validation::Valid(n)
    } else {
        Validation::invalid("must be positive")
    })
}

fn validate_even(n: i32) -> std::future::Ready<Validation<&'static str, i32>> {
    std::future::ready(if n % 2 == 0 {
        Validation::Valid(n)
    } else {
        Validation::invalid("must be even")
    })
}

fn validate_less_than_100(n: i32) -> std::future::Ready<Validation<&'static str, i32>> {
    std::future::ready(if n < 100 {
        Validation::Valid(n)
    } else {
        Validation::invalid("must be less than 100")
    })
}

#[tokio::test]
async fn validate_all_async_all_valid() {
    let result =
        validate_all_async([validate_positive(10), validate_even(10), validate_less_than_100(10)])
            .await;

    assert!(result.is_valid());
    let values = result.into_value().unwrap();
    assert_eq!(values, vec![10, 10, 10]);
}

#[tokio::test]
async fn validate_all_async_some_invalid() {
    let result =
        validate_all_async([validate_positive(-5), validate_even(3), validate_less_than_100(50)])
            .await;

    assert!(result.is_invalid());
    let errors: Vec<_> = result.iter_errors().collect();
    assert_eq!(errors.len(), 2);
    assert!(errors.contains(&&"must be positive"));
    assert!(errors.contains(&&"must be even"));
}

#[tokio::test]
async fn validate_all_async_all_invalid() {
    let result =
        validate_all_async([validate_positive(-1), validate_even(3), validate_less_than_100(200)])
            .await;

    assert!(result.is_invalid());
    let errors: Vec<_> = result.iter_errors().collect();
    assert_eq!(errors.len(), 3);
}

#[tokio::test]
async fn validate_all_async_mixed_validators() {
    // Test with same future type (validate_even)
    let result = validate_all_async([
        validate_even(2),
        validate_even(4),
        validate_even(5), // odd, invalid
    ])
    .await;

    assert!(result.is_invalid());
    let errors: Vec<_> = result.iter_errors().collect();
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0], &"must be even");
}

#[tokio::test]
async fn validate_seq_async_all_pass() {
    let validators: Vec<fn(i32) -> std::future::Ready<Validation<&'static str, i32>>> =
        vec![validate_positive, validate_even, validate_less_than_100];

    let result = validate_seq_async(10i32, validators).await;

    assert!(result.is_valid());
    assert_eq!(result.into_value(), Some(10));
}

#[tokio::test]
async fn validate_seq_async_stops_on_first_invalid() {
    let validators: Vec<fn(i32) -> std::future::Ready<Validation<&'static str, i32>>> =
        vec![validate_positive, validate_even];

    let result = validate_seq_async(-5i32, validators).await;

    assert!(result.is_invalid());
    let errors: Vec<_> = result.iter_errors().collect();
    // Only one error because it stops at first invalid
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0], &"must be positive");
}

#[tokio::test]
async fn validate_all_async_empty() {
    let result: Validation<&str, Vec<i32>> =
        validate_all_async(std::iter::empty::<std::future::Ready<Validation<&str, i32>>>()).await;

    assert!(result.is_valid());
    assert_eq!(result.into_value(), Some(vec![]));
}
