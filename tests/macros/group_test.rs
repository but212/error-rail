use error_rail::{assert_err_eq, group, ComposableError};

#[test]
fn test_group_macro_basic() {
    let attempts = 3;
    let err = ComposableError::<&str>::new("connection failed").with_context(group!(
        message("retry attempt {}", attempts),
        tag("database"),
        location(file!(), line!()),
        metadata("host", "localhost:5432")
    ));

    // Verify exact formatting of the context message
    let contexts = err.context();
    assert_eq!(contexts.len(), 1);
    let msg = contexts[0].message();

    // The format should be: [database] at file:line: retry attempt 3 (host=localhost:5432)
    assert!(msg.contains("[database]"), "message should contain tag");
    assert!(msg.contains("retry attempt 3"), "message should contain formatted retry attempt");
    assert!(msg.contains("(host=localhost:5432)"), "message should contain metadata");

    // Verify the structure: [tag] at file:line: message (metadata)
    // Should have exactly 2 colons: one after line number, one in metadata
    assert_eq!(msg.matches(':').count(), 2, "should have exactly 2 colons");

    // Also verify using assert_err_eq for substring matching
    let res: Result<(), ComposableError<&str>> = Err(err);
    assert_err_eq!(res, "database");
    assert_err_eq!(res, "localhost:5432");
}

#[test]
fn test_group_macro_no_message() {
    let err = ComposableError::<&str>::new("error")
        .with_context(group!(tag("network"), metadata("timeout", "30s")));

    // Verify exact formatting: no message field means no colon prefix
    assert_eq!(err.context().len(), 1);
    assert_eq!(err.context()[0].message(), "[network] (timeout=30s)");

    let res: Result<(), ComposableError<&str>> = Err(err);
    assert_err_eq!(res, "network");
    assert_err_eq!(res, "30s");
}

#[test]
fn test_group_macro_empty() {
    let err = ComposableError::<&str>::new("error").with_context(group! {});

    // Empty group should render as empty string
    assert_eq!(err.context().len(), 1);
    assert_eq!(err.context()[0].message(), "");

    let res: Result<(), ComposableError<&str>> = Err(err);
    assert_err_eq!(res, "error");
}
