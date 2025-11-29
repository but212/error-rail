use error_rail::traits::WithError;

#[test]
fn test_with_error_fmap_error_ok() {
    let result: Result<i32, &str> = Ok(42);
    let mapped = result.fmap_error(|e| format!("Error: {}", e));
    assert_eq!(mapped, Ok(42));
}

#[test]
fn test_with_error_to_result_first_ok() {
    let result: Result<i32, &str> = Ok(42);
    assert_eq!(result.to_result_first(), Ok(42));
}

#[test]
#[allow(deprecated)]
fn test_with_error_to_result() {
    let result: Result<i32, &str> = Ok(42);
    assert_eq!(result.to_result(), Ok(42));
}

#[test]
fn test_with_error_to_result_all() {
    let result: Result<i32, &str> = Ok(42);
    assert_eq!(result.to_result_all(), Ok(42));

    let err_result: Result<i32, &str> = Err("error");
    let all_errs = err_result.to_result_all();
    assert!(all_errs.is_err());
    assert_eq!(all_errs.unwrap_err().len(), 1);
}
