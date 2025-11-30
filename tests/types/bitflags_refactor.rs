//! Tests for the new bitflags refactoring functionality.

use error_rail::{ComposableError, ContextKind, ErrorContext, ErrorFormat, FingerprintOptions};

#[test]
fn test_error_format_bitflags() {
    // Test default format
    let default_format = ErrorFormat::default();
    assert!(default_format.contains(ErrorFormat::SHOW_CODE));
    assert!(!default_format.contains(ErrorFormat::MULTILINE));
    assert!(!default_format.contains(ErrorFormat::COMPACT));
    assert!(!default_format.contains(ErrorFormat::PRETTY));

    // Test pretty format
    let pretty_format = ErrorFormat::PRETTY | ErrorFormat::MULTILINE;
    assert!(pretty_format.contains(ErrorFormat::PRETTY));
    assert!(pretty_format.contains(ErrorFormat::MULTILINE));
    assert!(!pretty_format.contains(ErrorFormat::COMPACT));

    // Test compact format
    let compact_format = ErrorFormat::COMPACT;
    assert!(compact_format.contains(ErrorFormat::COMPACT));
    assert!(!compact_format.contains(ErrorFormat::SHOW_CODE));

    // Test bit operations
    let combined = ErrorFormat::SHOW_CODE | ErrorFormat::MULTILINE;
    assert!(combined.contains(ErrorFormat::SHOW_CODE));
    assert!(combined.contains(ErrorFormat::MULTILINE));
    assert!(!combined.contains(ErrorFormat::COMPACT));

    let without_code = combined.difference(ErrorFormat::SHOW_CODE);
    assert!(!without_code.contains(ErrorFormat::SHOW_CODE));
    assert!(without_code.contains(ErrorFormat::MULTILINE));
}

#[test]
fn test_context_kind_bitflags() {
    // Test simple context
    let simple_ctx = ErrorContext::new("test message");
    let kinds = simple_ctx.context_kinds();
    assert!(kinds.contains(ContextKind::MESSAGE));
    assert!(!kinds.contains(ContextKind::TAGS));
    assert!(!kinds.contains(ContextKind::LOCATION));
    assert!(!kinds.contains(ContextKind::METADATA));

    // Test tag context
    let tag_ctx = ErrorContext::tag("network");
    let kinds = tag_ctx.context_kinds();
    assert!(!kinds.contains(ContextKind::MESSAGE));
    assert!(kinds.contains(ContextKind::TAGS));
    assert!(!kinds.contains(ContextKind::LOCATION));
    assert!(!kinds.contains(ContextKind::METADATA));

    // Test location context
    let loc_ctx = ErrorContext::location("main.rs", 42);
    let kinds = loc_ctx.context_kinds();
    assert!(!kinds.contains(ContextKind::MESSAGE));
    assert!(!kinds.contains(ContextKind::TAGS));
    assert!(kinds.contains(ContextKind::LOCATION));
    assert!(!kinds.contains(ContextKind::METADATA));

    // Test metadata context
    let meta_ctx = ErrorContext::metadata("user_id", "42");
    let kinds = meta_ctx.context_kinds();
    assert!(!kinds.contains(ContextKind::MESSAGE));
    assert!(!kinds.contains(ContextKind::TAGS));
    assert!(!kinds.contains(ContextKind::LOCATION));
    assert!(kinds.contains(ContextKind::METADATA));

    // Test complex context with multiple kinds
    let complex_ctx = ErrorContext::builder()
        .message("connection failed")
        .tag("network")
        .location("main.rs", 42)
        .metadata("retry_count", "3")
        .build();
    let kinds = complex_ctx.context_kinds();
    assert!(kinds.contains(ContextKind::MESSAGE));
    assert!(kinds.contains(ContextKind::TAGS));
    assert!(kinds.contains(ContextKind::LOCATION));
    assert!(kinds.contains(ContextKind::METADATA));
    assert!(kinds.contains(ContextKind::ALL));
}

#[test]
fn test_fingerprint_options_bitflags() {
    // Test default options
    let default_opts = FingerprintOptions::default();
    assert!(default_opts.contains(FingerprintOptions::TAGS));
    assert!(default_opts.contains(FingerprintOptions::CODE));
    assert!(default_opts.contains(FingerprintOptions::MESSAGE));
    assert!(!default_opts.contains(FingerprintOptions::METADATA));

    // Test custom options
    let custom_opts = FingerprintOptions::TAGS | FingerprintOptions::CODE;
    assert!(custom_opts.contains(FingerprintOptions::TAGS));
    assert!(custom_opts.contains(FingerprintOptions::CODE));
    assert!(!custom_opts.contains(FingerprintOptions::MESSAGE));
    assert!(!custom_opts.contains(FingerprintOptions::METADATA));

    // Test ALL options
    let all_opts = FingerprintOptions::ALL;
    assert!(all_opts.contains(FingerprintOptions::TAGS));
    assert!(all_opts.contains(FingerprintOptions::CODE));
    assert!(all_opts.contains(FingerprintOptions::MESSAGE));
    assert!(all_opts.contains(FingerprintOptions::METADATA));
}

#[test]
fn test_bitflags_in_composable_error() {
    let err = ComposableError::new("database error")
        .with_context(ErrorContext::tag("db"))
        .with_context(ErrorContext::metadata("table", "users"))
        .set_code(500);

    // Test fingerprint with custom bitflags options
    let fp_tags_only = err
        .fingerprint_config()
        .with_options(FingerprintOptions::TAGS)
        .compute();

    let fp_all = err
        .fingerprint_config()
        .with_options(FingerprintOptions::ALL)
        .compute();

    assert_ne!(fp_tags_only, fp_all);

    // Test error formatting with bitflags
    let formatted = err
        .fmt()
        .with_format(ErrorFormat::COMPACT | ErrorFormat::SHOW_CODE)
        .to_string();

    assert!(formatted.contains("|"));
    assert!(formatted.contains("(code: 500)"));
}

#[test]
fn test_context_kind_filtering() {
    let contexts = vec![
        ErrorContext::new("simple message"),
        ErrorContext::tag("network"),
        ErrorContext::location("main.rs", 42),
        ErrorContext::metadata("user_id", "42"),
        ErrorContext::builder().message("complex").tag("db").build(),
    ];

    // Filter contexts that have tags using bitflags
    let tag_contexts: Vec<_> = contexts
        .iter()
        .filter(|ctx| ctx.context_kinds().contains(ContextKind::TAGS))
        .collect();

    assert_eq!(tag_contexts.len(), 2); // "network" and "db" contexts

    // Filter contexts that have messages using bitflags
    let message_contexts: Vec<_> = contexts
        .iter()
        .filter(|ctx| ctx.context_kinds().contains(ContextKind::MESSAGE))
        .collect();

    assert_eq!(message_contexts.len(), 2); // "simple message" and "complex" contexts
}
