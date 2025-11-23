use error_rail::traits::{ErrorCategory, WithError};
use error_rail::validation::Validation;

#[test]
fn error_category_lift_creates_valid_value() {
    let v: Validation<String, i32> = <Validation<String, ()> as ErrorCategory<String>>::lift(42);

    assert!(v.is_valid());
    assert_eq!(v.into_value(), Some(42));
}

#[test]
fn error_category_handle_error_creates_invalid_with_single_error() {
    let v: Validation<String, i32> =
        <Validation<String, ()> as ErrorCategory<String>>::handle_error("err".to_string());

    assert!(v.is_invalid());
    let errors = v.into_errors().unwrap();
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0], "err".to_string());
}

#[test]
fn with_error_fmap_error_transforms_all_errors() {
    let v: Validation<&str, i32> = Validation::invalid_many(["e1", "e2"]);
    let mapped: Validation<String, i32> = v.fmap_error(|e| format!("E:{e}"));

    let errors: Vec<_> = mapped.into_errors().unwrap().into_iter().collect();
    assert_eq!(errors, vec!["E:e1".to_string(), "E:e2".to_string()]);
}

#[test]
fn with_error_fmap_error_keeps_valid_unchanged() {
    let v: Validation<&str, i32> = Validation::valid(10);
    let mapped: Validation<String, i32> = v.fmap_error(|e| format!("E:{e}"));

    assert!(mapped.is_valid());
    assert_eq!(mapped.into_value(), Some(10));
}

#[test]
fn with_error_to_result_valid_uses_trait_signature() {
    let v: Validation<&str, i32> = Validation::valid(42);

    let result: Result<i32, &str> = <Validation<&str, i32> as WithError<&str>>::to_result(v);

    assert_eq!(result, Ok(42));
}

#[test]
fn with_error_to_result_invalid_returns_first_error() {
    let v: Validation<&str, i32> = Validation::invalid_many(["first", "second"]);

    let result: Result<i32, &str> = <Validation<&str, i32> as WithError<&str>>::to_result(v);

    assert_eq!(result, Err("first"));
}
