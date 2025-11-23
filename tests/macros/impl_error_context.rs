use error_rail::{impl_error_context, traits::IntoErrorContext};
use std::fmt;

struct MyError {
    code: u32,
}

impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Error code: {}", self.code)
    }
}

impl_error_context!(MyError);

#[test]
fn test_impl_error_context_macro() {
    let err = MyError { code: 404 };
    let ctx = err.into_error_context();
    assert_eq!(ctx.to_string(), "Error code: 404");
}
