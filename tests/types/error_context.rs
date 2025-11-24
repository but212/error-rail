use error_rail::{ErrorContext, GroupContext};

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

#[test]
fn test_error_context_group_message_and_empty_group() {
    // GroupContext with explicit message should prefer the group message
    let ctx_group_msg = ErrorContext::Group(GroupContext {
        message: Some("group-msg".into()),
        ..Default::default()
    });
    assert_eq!(ctx_group_msg.message(), "group-msg");

    // Completely empty GroupContext should render as an empty string
    let ctx_empty = ErrorContext::Group(GroupContext::default());
    assert_eq!(ctx_empty.message(), "");
}

#[test]
fn test_error_context_tag_and_metadata() {
    let ctx_tag = ErrorContext::tag("tag1");
    assert_eq!(ctx_tag.message(), "[tag1]");

    let ctx_meta = ErrorContext::metadata("key", "val");
    assert_eq!(ctx_meta.message(), "key=val");
}

#[test]
fn test_error_context_builder() {
    let ctx = ErrorContext::builder()
        .message("msg")
        .location("file.rs", 10)
        .tag("tag1")
        .metadata("key", "val")
        .build();
    assert_eq!(ctx.message(), "msg");
}
