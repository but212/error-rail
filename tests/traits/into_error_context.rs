use error_rail::traits::IntoErrorContext;
use std::borrow::Cow;

#[test]
fn test_cow_into_error_context() {
    let cow_borrowed: Cow<'static, str> = Cow::Borrowed("borrowed");
    let ctx_borrowed = cow_borrowed.into_error_context();
    assert_eq!(ctx_borrowed.message(), "borrowed");

    let cow_owned: Cow<'static, str> = Cow::Owned("owned".to_string());
    let ctx_owned = cow_owned.into_error_context();
    assert_eq!(ctx_owned.message(), "owned");
}
