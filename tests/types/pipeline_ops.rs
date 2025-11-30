use core::fmt::Display;
use core::time::Duration;
use error_rail::{traits::TransientError, ErrorPipeline};

#[derive(Debug, Clone, PartialEq)]
struct TestError {
    message: String,
    is_transient: bool,
}

impl TestError {
    fn new(message: &str, is_transient: bool) -> Self {
        Self {
            message: message.to_string(),
            is_transient,
        }
    }
}

impl Display for TestError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl TransientError for TestError {
    fn is_transient(&self) -> bool {
        self.is_transient
    }

    fn retry_after_hint(&self) -> Option<Duration> {
        if self.message.contains("rate_limit") {
            Some(Duration::from_secs(60))
        } else {
            None
        }
    }
}

#[test]
fn test_error_pipeline_recover_success() {
    let pipeline =
        ErrorPipeline::new(Err(TestError::new("network error", true))).recover(|_err| Ok(42));

    assert!(pipeline.finish().is_ok());
}

#[test]
fn test_error_pipeline_recover_failure() {
    let pipeline = ErrorPipeline::<i32, TestError>::new(Err(TestError::new("network error", true)))
        .recover(|_err| Err(TestError::new("recovery failed", false)));

    assert!(pipeline.finish().is_err());
}

#[test]
fn test_error_pipeline_recover_on_ok() {
    let pipeline = ErrorPipeline::<i32, TestError>::new(Ok(10)).recover(|_err| Ok(42));

    assert_eq!(pipeline.finish().unwrap(), 10);
}

#[test]
fn test_error_pipeline_fallback() {
    let pipeline =
        ErrorPipeline::<i32, TestError>::new(Err(TestError::new("error", false))).fallback(99);

    assert_eq!(pipeline.finish().unwrap(), 99);
}

#[test]
fn test_error_pipeline_fallback_on_ok() {
    let pipeline: ErrorPipeline<i32, TestError> = ErrorPipeline::new(Ok(10)).fallback(99);

    assert_eq!(pipeline.finish().unwrap(), 10);
}

#[test]
fn test_error_pipeline_recover_safe() {
    let pipeline = ErrorPipeline::<i32, TestError>::new(Err(TestError::new("error", false)))
        .recover_safe(|_err| 123);

    assert_eq!(pipeline.finish().unwrap(), 123);
}

#[test]
fn test_error_pipeline_recover_transient_success() {
    let pipeline = ErrorPipeline::new(Err(TestError::new("transient error", true)))
        .recover_transient(|_err| Ok(200));

    assert!(pipeline.finish().is_ok());
}

#[test]
fn test_error_pipeline_recover_transient_permanent_error() {
    let pipeline = ErrorPipeline::new(Err(TestError::new("permanent error", false)))
        .recover_transient(|_err| Ok(200));

    assert!(pipeline.finish().is_err());
}

#[test]
fn test_error_pipeline_recover_transient_recovery_fails() {
    let pipeline: ErrorPipeline<i32, TestError> =
        ErrorPipeline::new(Err(TestError::new("transient error", true)))
            .recover_transient(|_err| Err(TestError::new("recovery failed", false)));

    assert!(pipeline.finish().is_err());
}

#[test]
fn test_error_pipeline_is_transient_true() {
    let pipeline: ErrorPipeline<i32, TestError> =
        ErrorPipeline::new(Err(TestError::new("transient", true)));
    assert!(pipeline.is_transient());
}

#[test]
fn test_error_pipeline_is_transient_false() {
    let pipeline: ErrorPipeline<i32, TestError> =
        ErrorPipeline::new(Err(TestError::new("permanent", false)));
    assert!(!pipeline.is_transient());
}

#[test]
fn test_error_pipeline_is_transient_ok() {
    let pipeline: ErrorPipeline<i32, TestError> = ErrorPipeline::new(Ok(42));
    assert!(!pipeline.is_transient());
}

#[test]
fn test_error_pipeline_should_retry_transient() {
    let pipeline: ErrorPipeline<i32, TestError> =
        ErrorPipeline::new(Err(TestError::new("transient", true)));
    assert!(pipeline.should_retry().is_some());
}

#[test]
fn test_error_pipeline_should_retry_permanent() {
    let pipeline: ErrorPipeline<i32, TestError> =
        ErrorPipeline::new(Err(TestError::new("permanent", false)));
    assert!(pipeline.should_retry().is_none());
}

#[test]
fn test_error_pipeline_should_retry_ok() {
    let pipeline: ErrorPipeline<i32, TestError> = ErrorPipeline::new(Ok(42));
    assert!(pipeline.should_retry().is_none());
}

#[test]
fn test_error_pipeline_retry_after_hint() {
    let pipeline: ErrorPipeline<i32, TestError> =
        ErrorPipeline::new(Err(TestError::new("rate_limit error", true)));
    assert_eq!(pipeline.retry_after_hint(), Some(Duration::from_secs(60)));
}

#[test]
fn test_error_pipeline_retry_after_hint_none() {
    let pipeline: ErrorPipeline<i32, TestError> =
        ErrorPipeline::new(Err(TestError::new("normal error", true)));
    assert_eq!(pipeline.retry_after_hint(), None);
}

#[test]
fn test_error_pipeline_retry_after_hint_ok() {
    let pipeline: ErrorPipeline<i32, TestError> = ErrorPipeline::new(Ok(42));
    assert_eq!(pipeline.retry_after_hint(), None);
}

#[test]
fn test_error_pipeline_with_retry_context() {
    let result = ErrorPipeline::<u32, TestError>::new(Err(TestError::new("error", false)))
        .with_retry_context(3)
        .finish();

    if let Err(err) = result {
        let chain = err.error_chain();
        assert!(chain.contains("retry_attempt=3"));
    }
}

#[test]
fn test_error_pipeline_with_retry_context_on_ok() {
    let result = ErrorPipeline::<i32, TestError>::new(Ok(42))
        .with_retry_context(3)
        .finish();

    assert!(result.is_ok());
}

#[test]
fn test_error_pipeline_recovery_discards_contexts() {
    let result = ErrorPipeline::<u32, TestError>::new(Err(TestError::new("error", false)))
        .with_context(error_rail::context!("original context"))
        .recover(|_| Ok(42))
        .finish();

    assert!(result.is_ok());
}

#[test]
fn test_error_pipeline_transient_recovery_discards_contexts() {
    let result = ErrorPipeline::<u32, TestError>::new(Err(TestError::new("transient", true)))
        .with_context(error_rail::context!("original context"))
        .recover_transient(|_| Ok(42))
        .finish();

    assert!(result.is_ok());
}

#[test]
fn test_error_pipeline_chained_operations() {
    let result = ErrorPipeline::new(Err(TestError::new("initial error", true)))
        .with_context(error_rail::context!("step 1"))
        .recover(|_| Err(TestError::new("second error", false)))
        .with_context(error_rail::context!("step 2"))
        .fallback(100)
        .finish();

    assert_eq!(result.unwrap(), 100);
}

#[test]
fn test_error_pipeline_complex_transient_flow() {
    let attempt_count = std::cell::Cell::new(0);

    // Test that recover_transient only attempts recovery once for transient errors
    let result = ErrorPipeline::new(Err(TestError::new("transient error", true)))
        .with_context(error_rail::context!("initial attempt"))
        .recover_transient(|_err| {
            attempt_count.set(attempt_count.get() + 1);
            Ok(42) // Single recovery attempt
        })
        .finish();

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 42);
    assert_eq!(attempt_count.get(), 1); // Only called once
}
