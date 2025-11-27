use error_rail::validation::Validation;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[test]
fn test_validation_map_invalid() {
    let v: Validation<&str, i32> = Validation::invalid("error");
    let mapped = v.map(|x| x * 2);
    assert!(mapped.is_invalid());
}

#[test]
fn test_validation_and_then_invalid() {
    let v: Validation<&str, i32> = Validation::invalid("error");
    let chained = v.and_then(|x| Validation::valid(x * 2));
    assert!(chained.is_invalid());
}

#[test]
fn test_validation_or_else_valid() {
    let v: Validation<&str, i32> = Validation::valid(42);
    let recovered = v.or_else(|_| Validation::valid(0));
    assert_eq!(recovered.into_value(), Some(42));
}

#[test]
fn test_validation_or_else_invalid() {
    let v: Validation<&str, i32> = Validation::invalid("error");
    let recovered = v.or_else(|_| Validation::valid(0));
    assert_eq!(recovered.into_value(), Some(0));
}

#[test]
fn test_validation_zip_first_invalid() {
    let v1: Validation<&str, i32> = Validation::invalid("error1");
    let v2: Validation<&str, String> = Validation::valid("hello".to_string());
    let result = v1.zip(v2);
    assert!(result.is_invalid());
    let errors = result.into_errors().unwrap();
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0], "error1");
}

#[test]
fn test_validation_zip_second_invalid() {
    let v1: Validation<&str, i32> = Validation::valid(42);
    let v2: Validation<&str, String> = Validation::invalid("error2");
    let result = v1.zip(v2);
    assert!(result.is_invalid());
    let errors = result.into_errors().unwrap();
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0], "error2");
}

#[test]
fn test_validation_map_err_valid() {
    let v: Validation<&str, i32> = Validation::valid(42);
    let mapped = v.map_err(|e| format!("Error: {}", e));
    assert!(mapped.is_valid());
}

#[test]
fn test_validation_to_result_valid() {
    let v: Validation<&str, i32> = Validation::valid(42);
    assert_eq!(v.to_result(), Ok(42));
}

#[test]
fn test_validation_into_errors_valid() {
    let v: Validation<&str, i32> = Validation::valid(42);
    assert!(v.into_errors().is_none());
}

#[test]
fn test_validation_into_value_invalid() {
    let v: Validation<&str, i32> = Validation::invalid("error");
    assert!(v.into_value().is_none());
}

#[cfg(feature = "serde")]
#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct TestData {
    id: i32,
}

#[test]
#[cfg(feature = "serde")]
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
