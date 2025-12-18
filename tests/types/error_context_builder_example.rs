use error_rail::{ComposableError, ErrorContext};

#[test]
fn test_error_context_builder_example() {
    // Test the example from CHANGELOG.md
    let err = ComposableError::<&str>::new("connection failed").with_context(
        ErrorContext::builder()
            .tag("database")
            .location(file!(), line!())
            .metadata("host", "localhost")
            .build(),
    );

    // Should display as a single grouped context
    let expected_msg = format!(
        "[database] at {}:{} (host=localhost)",
        file!(),
        line!() - 9 // Line where ErrorContext::builder() is called
    );

    assert_eq!(err.context().len(), 1);
    assert_eq!(err.context()[0].message(), expected_msg);
}

#[test]
fn test_error_context_builder_group_with_message() {
    // Test builder with message field
    let err = ComposableError::<&str>::new("base error").with_context(
        ErrorContext::builder()
            .message("processing failed")
            .tag("network")
            .location(file!(), line!())
            .metadata("timeout", "30s")
            .build(),
    );

    let expected_msg =
        format!("[network] at {}:{}: processing failed (timeout=30s)", file!(), line!() - 6);

    assert_eq!(err.context().len(), 1);
    assert_eq!(err.context()[0].message(), expected_msg);
}
