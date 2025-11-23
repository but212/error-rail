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

#[test]
fn iterators_over_errors_and_values_work() {
    let mut valid = Validation::<&str, i32>::valid(3);
    if let Some(value) = valid.iter_mut().next() {
        *value = 4;
    }
    assert_eq!(valid.into_value(), Some(4));

    let validation: Validation<&str, i32> = Validation::invalid_many(["x", "y"]);
    let collected: Vec<_> = validation.iter_errors().cloned().collect();
    assert_eq!(collected, vec!["x", "y"]);
}

#[test]
fn collecting_results_into_validation_accumulates_errors() {
    let inputs = vec![Ok(1), Err("err1"), Err("err2")];
    let collected: Validation<&str, Vec<i32>> = inputs.into_iter().collect();

    assert!(collected.is_invalid());
    assert_eq!(collected.into_errors().unwrap().len(), 2);
}

#[test]
fn collecting_validations_preserves_all_errors() {
    let items = vec![
        Validation::valid(10),
        Validation::invalid("bad"),
        Validation::invalid("worse"),
    ];

    let collected: Validation<&str, Vec<i32>> = items.into_iter().collect();
    assert!(collected.is_invalid());
    assert_eq!(collected.into_errors().unwrap().len(), 2);
}

#[test]
fn collecting_into_custom_collection_type() {
    use smallvec::SmallVec;

    let inputs = vec![Ok(1), Err("err1"), Ok(2)];
    let collected: Validation<&str, SmallVec<[i32; 4]>> = inputs.into_iter().collect();

    assert!(collected.is_invalid());
    assert_eq!(collected.into_errors().unwrap().len(), 1);
}
