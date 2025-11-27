use error_rail::validation::Validation;

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
fn test_validation_iter_invalid() {
    let v: Validation<&str, i32> = Validation::invalid("error");
    assert_eq!(v.iter().next(), None);

    let mut v_mut = v.clone();
    assert_eq!(v_mut.iter_mut().next(), None);
}

#[test]
fn test_validation_iter_errors_valid() {
    let v: Validation<&str, i32> = Validation::valid(42);
    assert_eq!(v.iter_errors().next(), None);

    let mut v_mut = v.clone();
    assert_eq!(v_mut.iter_errors_mut().next(), None);
}

#[test]
fn iter_valid_yields_single_value_and_len_updates() {
    let v: Validation<&str, i32> = Validation::valid(7);
    let mut iter = v.iter();

    assert_eq!(iter.len(), 1);
    assert_eq!(iter.next(), Some(&7));
    assert_eq!(iter.len(), 0);
    assert_eq!(iter.next(), None);
}

#[test]
fn iter_invalid_has_zero_len_and_is_empty() {
    let v: Validation<&str, i32> = Validation::invalid("error");
    let mut iter = v.iter();

    assert_eq!(iter.len(), 0);
    assert_eq!(iter.next(), None);
}

#[test]
fn iter_errors_invalid_exposes_all_errors_and_len() {
    let v: Validation<&str, i32> = Validation::invalid_many(["a", "b", "c"]);
    let mut iter = v.iter_errors();

    assert_eq!(iter.len(), 3);
    assert_eq!(iter.next(), Some(&"a"));
    assert_eq!(iter.len(), 2);
    let rest: Vec<_> = iter.collect();
    assert_eq!(rest, vec![&"b", &"c"]);
}

#[test]
fn iter_errors_mut_allows_mutating_errors() {
    let mut v: Validation<String, i32> =
        Validation::invalid_many(["e1".to_string(), "e2".to_string()]);

    for error in v.iter_errors_mut() {
        error.push('!');
    }

    let collected: Vec<_> = v.iter_errors().cloned().collect();
    assert_eq!(collected, vec!["e1!".to_string(), "e2!".to_string()]);
}

#[test]
fn iter_mut_len_reflects_remaining_item() {
    let mut v: Validation<&str, i32> = Validation::valid(1);
    let mut iter = v.iter_mut();

    assert_eq!(iter.len(), 1);
    let _ = iter.next();
    assert_eq!(iter.len(), 0);
}

#[test]
fn iter_errors_empty_size_hint_and_len() {
    let v: Validation<&str, i32> = Validation::valid(0);
    let iter = v.iter_errors();

    assert_eq!(iter.len(), 0);
    assert_eq!(iter.size_hint(), (0, Some(0)));
}

#[test]
fn iter_errors_mut_empty_size_hint_and_len() {
    let mut v: Validation<&str, i32> = Validation::valid(0);
    let iter = v.iter_errors_mut();

    assert_eq!(iter.len(), 0);
    assert_eq!(iter.size_hint(), (0, Some(0)));
}

#[test]
fn iter_errors_mut_multi_size_hint_and_len() {
    let mut v: Validation<&str, i32> = Validation::invalid_many(["a", "b"]);
    let mut iter = v.iter_errors_mut();

    assert_eq!(iter.len(), 2);
    let (low, high) = iter.size_hint();
    assert_eq!(low, 2);
    assert_eq!(high, Some(2));

    let _ = iter.next();
    assert_eq!(iter.len(), 1);
}

#[test]
fn into_iterator_for_validation_yields_value_only_for_valid() {
    let valid: Validation<&str, i32> = Validation::valid(5);
    let values: Vec<_> = valid.into_iter().collect();
    assert_eq!(values, vec![5]);

    let invalid: Validation<&str, i32> = Validation::invalid("err");
    let values: Vec<_> = invalid.into_iter().collect();
    assert!(values.is_empty());
}

#[test]
fn into_iterator_len_reflects_remaining_items() {
    let valid: Validation<&str, i32> = Validation::valid(5);
    let mut iter = valid.into_iter();

    assert_eq!(iter.len(), 1);
    assert_eq!(iter.next(), Some(5));
    assert_eq!(iter.len(), 0);
}

#[test]
fn into_iterator_for_ref_and_mut_validation() {
    let v: Validation<&str, i32> = Validation::valid(10);
    let collected: Vec<_> = (&v).into_iter().collect();
    assert_eq!(collected, vec![&10]);

    let mut v2 = Validation::<&str, i32>::valid(3);
    for value in &mut v2 {
        *value *= 2;
    }
    assert_eq!(v2.into_value(), Some(6));
}

#[test]
fn collecting_results_into_validation_accumulates_errors() {
    let inputs = vec![Ok(1), Err("err1"), Err("err2")];
    let collected: Validation<&str, Vec<i32>> = inputs.into_iter().collect();

    assert!(collected.is_invalid());
    assert_eq!(collected.into_errors().unwrap().len(), 2);
}

#[test]
fn collecting_results_all_ok_produces_valid_collection() {
    let inputs = vec![Ok(1), Ok(2), Ok(3)];
    let collected: Validation<&str, Vec<i32>> = inputs.into_iter().collect();

    assert!(collected.is_valid());
    assert_eq!(collected.into_value().unwrap(), vec![1, 2, 3]);
}

#[test]
fn collecting_results_empty_iterator_is_valid_with_empty_collection() {
    let inputs: Vec<Result<i32, &str>> = Vec::new();
    let collected: Validation<&str, Vec<i32>> = inputs.into_iter().collect();

    assert!(collected.is_valid());
    assert!(collected.into_value().unwrap().is_empty());
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
fn collecting_validations_all_valid_produces_valid_collection() {
    let items = vec![Validation::valid(1), Validation::valid(2)];
    let collected: Validation<&str, Vec<i32>> = items.into_iter().collect();

    assert!(collected.is_valid());
    assert_eq!(collected.into_value().unwrap(), vec![1, 2]);
}

#[test]
fn collecting_into_custom_collection_type() {
    use smallvec::SmallVec;

    let inputs = vec![Ok(1), Err("err1"), Ok(2)];
    let collected: Validation<&str, SmallVec<[i32; 2]>> = inputs.into_iter().collect();

    assert!(collected.is_invalid());
    assert_eq!(collected.into_errors().unwrap().len(), 1);
}

#[test]
fn collecting_into_custom_collection_type_success() {
    use smallvec::{smallvec, SmallVec};

    let inputs = vec![Ok(1), Ok(2)];
    let collected: Validation<&str, SmallVec<[i32; 2]>> = inputs.into_iter().collect();

    assert!(collected.is_valid());
    let expected: SmallVec<[i32; 2]> = smallvec![1, 2];
    assert_eq!(collected.into_value().unwrap(), expected);
}
