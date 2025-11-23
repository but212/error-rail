use error_rail::ErrorContext;

#[test]
fn builder_creates_group_context() {
    let ctx = ErrorContext::builder()
        .message("connection failed")
        .tag("network")
        .metadata("host", "localhost")
        .build();

    assert_eq!(ctx.message(), "connection failed");

    if let ErrorContext::Group(g) = ctx {
        assert_eq!(g.message, Some("connection failed".into()));
        assert!(g.tags.contains(&"network".to_string()));
        assert!(g
            .metadata
            .contains(&("host".to_string(), "localhost".to_string())));
    } else {
        panic!("Expected Group context");
    }
}

#[test]
fn builder_accumulates_multiple_tags_and_metadata() {
    let ctx = ErrorContext::builder()
        .tag("t1")
        .tag("t2")
        .metadata("k1", "v1")
        .metadata("k2", "v2")
        .build();

    if let ErrorContext::Group(g) = ctx {
        assert_eq!(g.tags.len(), 2);
        assert!(g.tags.contains(&"t1".to_string()));
        assert!(g.tags.contains(&"t2".to_string()));

        assert_eq!(g.metadata.len(), 2);
        assert!(g.metadata.contains(&("k1".to_string(), "v1".to_string())));
        assert!(g.metadata.contains(&("k2".to_string(), "v2".to_string())));
    } else {
        panic!("Expected Group context");
    }
}

#[test]
fn group_factory_method_starts_builder() {
    let ctx = ErrorContext::group("error occurred")
        .tag("important")
        .build();

    assert_eq!(ctx.message(), "error occurred");
    if let ErrorContext::Group(g) = ctx {
        assert!(g.tags.contains(&"important".to_string()));
    } else {
        panic!("Expected Group context");
    }
}
