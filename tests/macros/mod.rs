use error_rail::traits::IntoErrorContext;
use error_rail::{context, location, metadata, tag, ErrorContext};

#[test]
fn context_macro_formats_message_lazily() {
    let ctx = context!("operation: {}", 7).into_error_context();

    assert_eq!(ctx.message(), "operation: 7");
}

#[test]
fn location_macro_captures_file_and_line() {
    let expected_line = line!() + 1;
    let ctx = location!();

    match ctx {
        ErrorContext::Group(group) => {
            if let Some(loc) = group.location {
                let normalized = loc.file.replace('\\', "/");
                assert!(normalized.ends_with("tests/macros/mod.rs"));
                assert_eq!(loc.line, expected_line);
            } else {
                panic!("location! should produce Group variant with location");
            }
        }
        _ => panic!("location! should produce Group variant"),
    }
}

#[test]
fn tag_macro_builds_tag_context() {
    let ctx = tag!("network");

    assert_eq!(ctx, ErrorContext::tag("network"));
    assert_eq!(ctx.message(), "[network]");
}

#[test]
fn metadata_macro_builds_kv_pair() {
    let ctx = metadata!("attempt", "3");

    assert_eq!(ctx, ErrorContext::metadata("attempt", "3"));
    assert_eq!(ctx.message(), "attempt=3");
}
