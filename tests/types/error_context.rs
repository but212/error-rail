use error_rail::ErrorContext;

#[test]
fn test_error_context_message_variants() {
    // Group with message
    let ctx_msg = ErrorContext::new("msg");
    assert_eq!(ctx_msg.message(), "msg");

    // Group with location
    let ctx_loc = ErrorContext::location("file.rs", 10);
    assert_eq!(ctx_loc.message(), "at file.rs:10");

    // Group with tags
    let ctx_tags = ErrorContext::tag("tag1");
    assert_eq!(ctx_tags.message(), "[tag1]");

    // Group with metadata
    let ctx_meta = ErrorContext::metadata("key", "val");
    assert_eq!(ctx_meta.message(), "key=val");
}
