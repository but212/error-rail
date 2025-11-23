use error_rail::validation::Validation;

#[test]
fn valid_and_invalid_helpers_behave_as_expected() {
    let valid = Validation::<&str, i32>::valid(5);
    assert!(valid.is_valid());
    assert_eq!(valid.into_value(), Some(5));

    let invalid = Validation::<&str, i32>::invalid("missing");
    assert!(invalid.is_invalid());
    let errors = invalid.into_errors().unwrap();
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0], "missing");
}

#[test]
fn map_and_and_then_chain_success_values() {
    let result = Validation::<&str, i32>::valid(4)
        .map(|x| x * 2)
        .and_then(|x| {
            if x == 8 {
                Validation::valid(x + 1)
            } else {
                Validation::invalid("unexpected")
            }
        });

    assert_eq!(result.into_value(), Some(9));
}

#[test]
fn map_err_transforms_all_errors() {
    let validation: Validation<&str, i32> = Validation::invalid_many(["a", "b"]);
    let mapped = validation.map_err(|e| format!("ERR:{e}"));

    let errors: Vec<_> = mapped.into_errors().unwrap().into_iter().collect();
    assert_eq!(errors, vec!["ERR:a".to_string(), "ERR:b".to_string()]);
}

#[test]
fn to_result_preserves_all_errors_in_vec() {
    let validation: Validation<&str, i32> = Validation::invalid_many(["first", "second"]);
    let result = validation.to_result();

    assert_eq!(result.unwrap_err().len(), 2);
}

#[test]
fn from_result_converts_single_error() {
    let ok = Validation::from_result(Ok::<_, &str>(42));
    assert!(ok.is_valid());

    let err = Validation::from_result(Err::<i32, &str>("boom"));
    assert!(err.is_invalid());
    assert_eq!(err.into_errors().unwrap()[0], "boom");
}

pub mod core;
pub mod iter;
pub mod traits;
