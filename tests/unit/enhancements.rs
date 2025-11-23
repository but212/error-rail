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
