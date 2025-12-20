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
    assert_eq!(ctx_meta.message(), "(key=val)");
}

#[test]
fn test_error_context_group_message_and_empty_group() {
    // GroupContext with explicit message should prefer the group message
    let ctx_group_msg = ErrorContext::Group(Box::new(GroupContext {
        message: Some("group-msg".into()),
        ..Default::default()
    }));
    assert_eq!(ctx_group_msg.message(), "group-msg");

    // Completely empty GroupContext should render as an empty string
    let ctx_empty = ErrorContext::Group(Box::new(GroupContext::default()));
    assert_eq!(ctx_empty.message(), "");
}

#[test]
fn test_error_context_tag_and_metadata() {
    let ctx_tag = ErrorContext::tag("tag1");
    assert_eq!(ctx_tag.message(), "[tag1]");

    let ctx_meta = ErrorContext::metadata("key", "val");
    assert_eq!(ctx_meta.message(), "(key=val)");
}

#[test]
fn test_error_context_builder() {
    let ctx = ErrorContext::builder()
        .message("msg")
        .location("file.rs", 10)
        .tag("tag1")
        .metadata("key", "val")
        .build();
    assert_eq!(ctx.message(), "[tag1] at file.rs:10: msg (key=val)");
}

#[test]
fn test_location_append_message() {
    let ctx = ErrorContext::builder()
        .location("file.rs", 10)
        .message("appended")
        .build();
    assert_eq!(ctx.message(), "at file.rs:10: appended");
}

#[test]
fn test_error_context_constructors() {
    let tag = ErrorContext::tag("network");
    assert_eq!(tag.message(), "[network]");

    let meta = ErrorContext::metadata("k", "v");
    assert_eq!(meta.message(), "(k=v)");
}

#[test]
fn test_error_context_message_composition() {
    let ctx = ErrorContext::builder()
        .location("file.rs", 10)
        .message("failed")
        .build();
    assert_eq!(ctx.message(), "at file.rs:10: failed");

    let ctx_no_loc = ErrorContext::builder().message("msg").build();
    assert_eq!(ctx_no_loc.message(), "msg");
}

#[test]
fn test_error_context_empty_group() {
    let ctx = ErrorContext::builder().build();
    assert_eq!(ctx.message(), "");
}

#[test]
fn test_metadata_constructor() {
    let ctx = ErrorContext::metadata("user_id", "42");
    assert_eq!(ctx.message(), "(user_id=42)");
}

#[test]
fn test_metadata_with_owned_strings() {
    let key = String::from("request_id");
    let value = String::from("abc-123");
    let ctx = ErrorContext::metadata(key, value);
    assert_eq!(ctx.message(), "(request_id=abc-123)");
}

#[test]
fn test_add_message_without_location_in_parts() {
    let ctx = ErrorContext::builder()
        .tag("db")
        .message("connection failed")
        .build();

    let msg = ctx.message();
    assert!(msg.contains("[db]"));
    assert!(msg.contains("connection failed"));
}

#[test]
fn test_add_message_with_tags_but_no_location() {
    let ctx = ErrorContext::builder()
        .tag("api")
        .tag("v2")
        .message("rate limited")
        .build();

    let msg = ctx.message();
    assert_eq!(msg, "[api, v2] rate limited");
}

#[test]
fn test_message_added_to_empty_parts_with_location_flag() {
    let ctx = ErrorContext::builder()
        .message("standalone message")
        .build();

    assert_eq!(ctx.message(), "standalone message");
}

#[test]
fn test_metadata_multiple_pairs() {
    let ctx = ErrorContext::builder()
        .metadata("key1", "value1")
        .metadata("key2", "value2")
        .build();

    let msg = ctx.message();
    assert!(msg.contains("key1=value1"));
    assert!(msg.contains("key2=value2"));
}

#[test]
fn test_all_fields_combined() {
    let ctx = ErrorContext::builder()
        .tag("network")
        .tag("timeout")
        .location("client.rs", 100)
        .message("connection timed out")
        .metadata("timeout_ms", "5000")
        .metadata("retry_count", "3")
        .build();

    let msg = ctx.message();
    assert!(msg.contains("[network, timeout]"));
    assert!(msg.contains("at client.rs:100"));
    assert!(msg.contains("connection timed out"));
    assert!(msg.contains("timeout_ms=5000"));
    assert!(msg.contains("retry_count=3"));
}
