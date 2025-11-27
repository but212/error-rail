use error_rail::convert::*;
use error_rail::validation::Validation;

#[test]
fn validation_to_result_handles_both_variants() {
    let valid = Validation::<&str, i32>::valid(7);
    assert_eq!(validation_to_result(valid), Ok(7));

    let invalid = Validation::<&str, i32>::invalid("boom");
    assert_eq!(validation_to_result(invalid), Err("boom"));
}

#[test]
fn result_to_validation_preserves_state() {
    let ok: Result<i32, &str> = Ok(3);
    assert!(result_to_validation(ok).is_valid());

    let err: Result<i32, &str> = Err("fail");
    let validation = result_to_validation(err);
    assert!(validation.is_invalid());
    assert_eq!(validation.into_errors().unwrap()[0], "fail");
}

#[test]
fn wrap_and_flatten_round_trip_composable_error() {
    let plain: Result<i32, &str> = Err("missing");
    let wrapped = wrap_in_composable_result(plain);
    assert!(wrapped.is_err());

    let flattened = flatten_composable_result(wrapped);
    assert_eq!(flattened, Err("missing"));
}

#[test]
fn boxed_wrapper_behaves_like_unboxed() {
    let plain: Result<i32, &str> = Err("oops");
    let boxed = wrap_in_composable_result_boxed(plain);

    let err = boxed.unwrap_err();
    assert_eq!(err.core_error(), &"oops");
}

#[test]
fn collect_errors_accumulates_all_items() {
    let validation = collect_errors(["err1", "err2"]);
    assert!(validation.is_invalid());
    assert_eq!(validation.into_errors().unwrap().len(), 2);

    let empty: [&str; 0] = [];
    assert!(collect_errors(empty).is_valid());
}

#[test]
fn split_validation_errors_expands_invalid_case() {
    let validation: Validation<&str, i32> = Validation::invalid_many(["a", "b"]);
    let results: Vec<_> = split_validation_errors(validation).collect();
    assert_eq!(results, vec![Err("a"), Err("b")]);

    let validation: Validation<&str, i32> = Validation::valid(1);
    let results: Vec<_> = split_validation_errors(validation).collect();
    assert_eq!(results, vec![Ok(1)]);
}

#[test]
fn composable_conversions_preserve_core() {
    let composable = core_to_composable("core err");
    assert_eq!(composable.core_error(), &"core err");

    let core = composable_to_core(composable);
    assert_eq!(core, "core err");
}
