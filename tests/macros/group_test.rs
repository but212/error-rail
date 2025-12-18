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

    // The message should combine all fields into one cohesive unit
    let res: Result<(), ComposableError<&str>> = Err(err);
    assert_err_eq!(res, "database");
    assert_err_eq!(res, "localhost:5432");
    assert_err_eq!(res, format!("retry attempt {}", attempts));
}

#[test]
fn test_group_macro_no_message() {
    let res: Result<(), ComposableError<&str>> = Err(ComposableError::<&str>::new("error")
        .with_context(group!(tag("network"), metadata("timeout", "30s"))));

    assert_err_eq!(res, "network");
    assert_err_eq!(res, "30s");
}

#[test]
fn test_group_macro_empty() {
    let res: Result<(), ComposableError<&str>> =
        Err(ComposableError::<&str>::new("error").with_context(group! {}));

    assert_err_eq!(res, "error");
}
