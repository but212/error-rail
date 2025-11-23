use error_rail::{
    impl_error_context, traits::IntoErrorContext, validation::Validation, ErrorPipeline,
};
use serde::{Deserialize, Serialize};
use std::fmt;

// --- Test impl_error_context! macro ---

struct MyError {
    code: u32,
}

impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Error code: {}", self.code)
    }
}

impl_error_context!(MyError);

#[test]
fn test_impl_error_context_macro() {
    let err = MyError { code: 404 };
    let ctx = err.into_error_context();
    assert_eq!(ctx.to_string(), "Error code: 404");
}

// --- Test ErrorPipeline recovery methods ---

#[test]
fn test_pipeline_fallback() {
    let pipeline = ErrorPipeline::<i32, &str>::new(Err("error")).fallback(42);
    let result = pipeline.finish();
    assert_eq!(result.unwrap(), 42);

    let pipeline_ok = ErrorPipeline::<i32, &str>::new(Ok(10)).fallback(42);
    let result_ok = pipeline_ok.finish();
    assert_eq!(result_ok.unwrap(), 10);
}

#[test]
fn test_pipeline_recover_safe() {
    let pipeline = ErrorPipeline::<i32, &str>::new(Err("error")).recover_safe(|_| 42);
    let result = pipeline.finish();
    assert_eq!(result.unwrap(), 42);

    let pipeline_ok = ErrorPipeline::<i32, &str>::new(Ok(10)).recover_safe(|_| 42);
    let result_ok = pipeline_ok.finish();
    assert_eq!(result_ok.unwrap(), 10);
}

// --- Test Validation Serde support ---

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct TestData {
    id: i32,
}

#[test]
fn test_validation_serde() {
    let valid = Validation::<String, TestData>::valid(TestData { id: 1 });
    let serialized = serde_json::to_string(&valid).unwrap();
    let deserialized: Validation<String, TestData> = serde_json::from_str(&serialized).unwrap();
    assert_eq!(valid, deserialized);

    let invalid = Validation::<String, TestData>::invalid("error".to_string());
    let serialized_err = serde_json::to_string(&invalid).unwrap();
    let deserialized_err: Validation<String, TestData> =
        serde_json::from_str(&serialized_err).unwrap();
    assert_eq!(invalid, deserialized_err);
}
