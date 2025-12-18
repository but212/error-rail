use std::io::{Error, ErrorKind};
use std::time::Duration;

use error_rail::{TransientError, TransientErrorExt};

#[derive(Debug)]
struct TestError {
    transient: bool,
    retry_after: Option<Duration>,
}

impl TransientError for TestError {
    fn is_transient(&self) -> bool {
        self.transient
    }

    fn retry_after_hint(&self) -> Option<Duration> {
        self.retry_after
    }
}

#[test]
fn test_transient_classification() {
    let transient = TestError { transient: true, retry_after: None };
    assert!(transient.is_transient());
    assert!(!transient.is_permanent());

    let permanent = TestError { transient: false, retry_after: None };
    assert!(!permanent.is_transient());
    assert!(permanent.is_permanent());
}

#[test]
fn test_retry_after_hint() {
    let with_hint = TestError { transient: true, retry_after: Some(Duration::from_secs(5)) };
    assert_eq!(with_hint.retry_after_hint(), Some(Duration::from_secs(5)));

    let without_hint = TestError { transient: true, retry_after: None };
    assert_eq!(without_hint.retry_after_hint(), None);
}

#[test]
fn test_retry_if_transient() {
    let ok: Result<i32, TestError> = Ok(42);
    assert!(ok.retry_if_transient().is_none());

    let transient_err: Result<i32, TestError> =
        Err(TestError { transient: true, retry_after: None });
    assert!(transient_err.retry_if_transient().is_some());

    let permanent_err: Result<i32, TestError> =
        Err(TestError { transient: false, retry_after: None });
    assert!(permanent_err.retry_if_transient().is_none());
}

#[derive(Debug)]
struct DefaultHintError;

impl TransientError for DefaultHintError {
    fn is_transient(&self) -> bool {
        true
    }
}

#[test]
fn test_default_hints() {
    let err = DefaultHintError;
    assert_eq!(err.retry_after_hint(), None);
    assert_eq!(err.max_retries_hint(), None);
}

#[test]
fn test_io_error_connection_refused_is_transient() {
    let err = Error::new(ErrorKind::ConnectionRefused, "connection refused");
    assert!(err.is_transient());
    assert!(!err.is_permanent());
}

#[test]
fn test_io_error_connection_reset_is_transient() {
    let err = Error::new(ErrorKind::ConnectionReset, "connection reset");
    assert!(err.is_transient());
}

#[test]
fn test_io_error_connection_aborted_is_transient() {
    let err = Error::new(ErrorKind::ConnectionAborted, "connection aborted");
    assert!(err.is_transient());
}

#[test]
fn test_io_error_timed_out_is_transient() {
    let err = Error::new(ErrorKind::TimedOut, "operation timed out");
    assert!(err.is_transient());
}

#[test]
fn test_io_error_interrupted_is_transient() {
    let err = Error::new(ErrorKind::Interrupted, "operation interrupted");
    assert!(err.is_transient());
}

#[test]
fn test_io_error_would_block_is_transient() {
    let err = Error::new(ErrorKind::WouldBlock, "would block");
    assert!(err.is_transient());
}

#[test]
fn test_io_error_not_found_is_permanent() {
    let err = Error::new(ErrorKind::NotFound, "file not found");
    assert!(!err.is_transient());
    assert!(err.is_permanent());
}

#[test]
fn test_io_error_permission_denied_is_permanent() {
    let err = Error::new(ErrorKind::PermissionDenied, "permission denied");
    assert!(!err.is_transient());
    assert!(err.is_permanent());
}

#[test]
fn test_io_error_already_exists_is_permanent() {
    let err = Error::new(ErrorKind::AlreadyExists, "file already exists");
    assert!(!err.is_transient());
}

#[test]
fn test_io_error_invalid_input_is_permanent() {
    let err = Error::new(ErrorKind::InvalidInput, "invalid input");
    assert!(!err.is_transient());
}

#[test]
fn test_io_error_invalid_data_is_permanent() {
    let err = Error::new(ErrorKind::InvalidData, "invalid data");
    assert!(!err.is_transient());
}

#[test]
fn test_io_error_other_is_permanent() {
    let err = Error::other("some other error");
    assert!(!err.is_transient());
}
