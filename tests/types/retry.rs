use core::fmt::Display;
use core::time::Duration;
use error_rail::{traits::TransientError, ErrorPipeline};

#[derive(Debug, Clone, PartialEq)]
struct RetryTestError {
    message: String,
    is_transient: bool,
    retry_after: Option<u64>,
}

impl RetryTestError {
    fn new(message: &str, is_transient: bool) -> Self {
        Self { message: message.to_string(), is_transient, retry_after: None }
    }

    fn with_retry_after(message: &str, is_transient: bool, retry_after: u64) -> Self {
        Self { message: message.to_string(), is_transient, retry_after: Some(retry_after) }
    }
}

impl Display for RetryTestError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl TransientError for RetryTestError {
    fn is_transient(&self) -> bool {
        self.is_transient
    }

    fn retry_after_hint(&self) -> Option<Duration> {
        self.retry_after.map(Duration::from_secs)
    }
}

#[test]
fn test_retry_ops_is_transient_true() {
    let pipeline: ErrorPipeline<u32, RetryTestError> =
        ErrorPipeline::new(Err(RetryTestError::new("timeout", true)));

    // Test RetryOps API correctly
    let retry_ops = pipeline.retry();
    assert!(retry_ops.is_transient());
}

#[test]
fn test_retry_ops_is_transient_false() {
    let pipeline: ErrorPipeline<u32, RetryTestError> =
        ErrorPipeline::new(Err(RetryTestError::new("auth error", false)));

    // Test RetryOps API correctly
    let retry_ops = pipeline.retry();
    assert!(!retry_ops.is_transient());
}

#[test]
fn test_retry_ops_is_transient_ok() {
    let pipeline: ErrorPipeline<u32, RetryTestError> = ErrorPipeline::new(Ok(42));

    // Test RetryOps API correctly
    let retry_ops = pipeline.retry();
    assert!(!retry_ops.is_transient());
}

#[test]
fn test_retry_ops_max_retries_metadata() {
    let result = ErrorPipeline::<u32, RetryTestError>::new(Err(RetryTestError::new("error", true)))
        .retry()
        .max_retries(5)
        .to_error_pipeline()
        .finish();

    if let Err(err) = result {
        let chain = err.error_chain();
        assert!(chain.contains("max_retries_hint=5"));
    }
}

#[test]
fn test_retry_ops_after_hint_metadata() {
    let result = ErrorPipeline::<u32, RetryTestError>::new(Err(RetryTestError::new("error", true)))
        .retry()
        .after_hint(Duration::from_secs(30))
        .to_error_pipeline()
        .finish();

    if let Err(err) = result {
        let chain = err.error_chain();
        // Duration debug format is "30s"
        assert!(chain.contains("retry_after_hint=30s"));
    }
}

#[test]
fn test_retry_ops_complex_retry_scenario() {
    let attempt_count = std::cell::Cell::new(0);

    // Test that recover_transient only calls the recovery function once for transient errors
    let result = ErrorPipeline::new(Err(RetryTestError::new("transient error", true)))
        .with_retry_context(1)
        .recover_transient(|_err| {
            attempt_count.set(attempt_count.get() + 1);
            Ok(42) // Single recovery attempt
        })
        .with_retry_context(2)
        .finish();

    assert!(result.is_ok());
    assert_eq!(attempt_count.get(), 1); // Only called once
}

#[test]
fn test_retry_ops_permanent_error_no_retry() {
    let attempt_count = std::cell::Cell::new(0);

    let result = ErrorPipeline::new(Err(RetryTestError::new("permanent error", false)))
        .with_retry_context(1)
        .recover_transient(|_err| {
            attempt_count.set(attempt_count.get() + 1);
            Ok(42) // This should never be called
        })
        .finish();

    assert!(result.is_err());
    assert_eq!(attempt_count.get(), 0);
}

#[test]
fn test_retry_ops_should_retry_transient() {
    let pipeline: ErrorPipeline<u32, RetryTestError> =
        ErrorPipeline::new(Err(RetryTestError::new("transient", true)));
    assert!(pipeline.should_retry().is_some());
}

#[test]
fn test_retry_ops_should_retry_permanent() {
    let pipeline: ErrorPipeline<u32, RetryTestError> =
        ErrorPipeline::new(Err(RetryTestError::new("permanent", false)));
    assert!(pipeline.should_retry().is_none());
}

#[test]
fn test_retry_ops_should_retry_success() {
    let pipeline: ErrorPipeline<u32, RetryTestError> = ErrorPipeline::new(Ok(42));
    assert!(pipeline.should_retry().is_none());
}

#[test]
fn test_retry_ops_retry_after_hint() {
    let pipeline: ErrorPipeline<u32, RetryTestError> =
        ErrorPipeline::new(Err(RetryTestError::with_retry_after("rate limited", true, 60)));
    assert_eq!(pipeline.retry_after_hint(), Some(Duration::from_secs(60)));
}

#[test]
fn test_retry_ops_retry_after_hint_none() {
    let pipeline: ErrorPipeline<u32, RetryTestError> =
        ErrorPipeline::new(Err(RetryTestError::new("transient", true)));
    assert_eq!(pipeline.retry_after_hint(), None);
}

#[test]
fn test_retry_ops_retry_after_hint_success() {
    let pipeline: ErrorPipeline<u32, RetryTestError> = ErrorPipeline::new(Ok(42));
    assert_eq!(pipeline.retry_after_hint(), None);
}

#[test]
fn test_retry_ops_chained_with_context() {
    let result =
        ErrorPipeline::<u32, RetryTestError>::new(Err(RetryTestError::new("network error", true)))
            .with_context(error_rail::context!("operation: api_call"))
            .with_retry_context(1)
            .recover_transient(|_err| Err(RetryTestError::new("timeout on retry", true)))
            .with_context(error_rail::context!("stage: retry_attempt"))
            .finish();

    assert!(result.is_err());

    if let Err(err) = result {
        let chain = err.error_chain();
        assert!(chain.contains("operation: api_call"));
        assert!(chain.contains("stage: retry_attempt"));
        assert!(chain.contains("retry_attempt=1"));
    }
}

#[test]
fn test_retry_ops_multiple_retry_attempts() {
    let attempts = std::cell::Cell::new(0);

    // Test that recover_transient only attempts recovery once for transient errors
    let result = ErrorPipeline::new(Err(RetryTestError::new("flaky service", true)))
        .with_retry_context(3)
        .recover_transient(|_err| {
            attempts.set(attempts.get() + 1);
            Ok(100) // Single successful recovery attempt
        })
        .finish();

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 100);
    assert_eq!(attempts.get(), 1); // Only called once, not in a loop
}

#[test]
fn test_retry_ops_backoff_simulation() {
    let attempt_count = std::cell::Cell::new(0);

    // Test that recover_transient only attempts recovery once, even with retry hints
    let result =
        ErrorPipeline::new(Err(RetryTestError::with_retry_after("service unavailable", true, 60)))
            .recover_transient(|_err| {
                attempt_count.set(attempt_count.get() + 1);
                Ok(200) // Single recovery attempt with backoff hint
            })
            .finish();

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 200);
    assert_eq!(attempt_count.get(), 1); // Only called once
}

#[test]
fn test_retry_ops_with_different_error_types() {
    #[derive(Debug)]
    struct NetworkError;

    impl TransientError for NetworkError {
        fn is_transient(&self) -> bool {
            true
        }
    }

    #[derive(Debug)]
    struct AuthError;

    impl TransientError for AuthError {
        fn is_transient(&self) -> bool {
            false
        }
    }

    // Test transient error
    let transient_pipeline: ErrorPipeline<(), NetworkError> = ErrorPipeline::new(Err(NetworkError));
    assert!(transient_pipeline.is_transient());

    // Test permanent error
    let permanent_pipeline: ErrorPipeline<(), AuthError> = ErrorPipeline::new(Err(AuthError));
    assert!(!permanent_pipeline.is_transient());
}
