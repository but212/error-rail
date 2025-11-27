use error_rail::{group, ComposableError};

#[test]
fn test_group_macro_basic() {
    let attempts = 3;
    let err = ComposableError::<&str>::new("connection failed").with_context(group!(
        message("retry attempt {}", attempts),
        tag("database"),
        location(file!(), line!()),
        metadata("host", "localhost:5432")
    ));

    // The message should combine all fields into one cohesive unit
    let expected_msg = format!(
        "[database] at {}:{}: retry attempt {} (host=localhost:5432)",
        file!(),
        9, // Line 9 is where location(file!(), line!()) is called
        attempts
    );
    assert_eq!(err.context().len(), 1);
    assert_eq!(err.context()[0].message(), expected_msg);
}

#[test]
fn test_group_macro_no_message() {
    let err = ComposableError::<&str>::new("error")
        .with_context(group!(tag("network"), metadata("timeout", "30s")));

    // Should display without colon since no message field
    assert_eq!(err.context().len(), 1);
    assert_eq!(err.context()[0].message(), "[network] (timeout=30s)");
}

#[test]
fn test_group_macro_empty() {
    let err = ComposableError::<&str>::new("error").with_context(group! {});

    // Empty group should render as empty string
    assert_eq!(err.context().len(), 1);
    assert_eq!(err.context()[0].message(), "");
}
