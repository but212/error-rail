use error_rail::context;
use error_rail::traits::IntoErrorContext;

#[test]
fn context_macro_formats_message_lazily() {
    let ctx = context!("operation: {}", 7).into_error_context();

    assert_eq!(ctx.message(), "operation: 7");
}

#[cfg(feature = "std")]
pub mod backtrace;
pub mod group_test;
pub mod impl_error_context;
