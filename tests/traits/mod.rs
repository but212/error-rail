use error_rail::traits::{ErrorCategory, ErrorOps, IntoErrorContext, WithError};
use error_rail::{context, ComposableError, ErrorContext};

#[test]
fn error_category_for_result_lifts_and_handles_errors() {
    let ok: Result<i32, &str> = <Result<(), &str>>::lift(10);
    assert_eq!(ok, Ok(10));

    let err: Result<i32, &str> = <Result<(), &str>>::handle_error("boom");
    assert_eq!(err, Err("boom"));
}

#[test]
fn with_error_maps_result_error_type() {
    let result: Result<i32, &str> = Err("oops");
    let mapped: Result<i32, String> = result.fmap_error(|e| format!("ERR:{e}"));

    assert_eq!(mapped.unwrap_err(), "ERR:oops");
}

#[test]
fn error_ops_recover_and_bimap() {
    let recovered = Err::<i32, &str>("missing").recover(|_| Ok(42));
    assert_eq!(recovered, Ok(42));

    let bimap = Ok::<i32, &str>(21).bimap_result(|x| x * 2, |e| e.to_uppercase());
    assert_eq!(bimap, Ok(42));

    let bimap_err = Err::<i32, &str>("bad").bimap_result(|x| x * 2, |e| e.to_uppercase());
    assert_eq!(bimap_err, Err("BAD".to_string()));
}

#[test]
fn into_error_context_supports_str_string_and_existing_context() {
    let ctx1 = "inline context".into_error_context();
    assert_eq!(ctx1.message(), "inline context");

    let ctx2 = String::from("owned").into_error_context();
    assert_eq!(ctx2.message(), "owned");

    let ctx3 = ErrorContext::tag("api").into_error_context();
    assert_eq!(ctx3, ErrorContext::tag("api"));
}

#[test]
fn context_macro_integrates_with_composable_error() {
    let err = ComposableError::<&str>::new("failed").with_context(context!("step: {}", 2));
    let contexts = err.context();

    assert_eq!(contexts.len(), 1);
    assert!(contexts[0].message().contains("step: 2"));
}

pub mod error_ops;
pub mod with_error;
