use error_rail::ComposableError;

#[test]
fn default_formatting_matches_previous_behavior() {
    let err = ComposableError::<&str, u32>::new("core error")
        .with_context("ctx1")
        .with_context("ctx2")
        .set_code(500);

    let output = err.to_string();
    assert_eq!(output, "ctx2 -> ctx1 -> core error (code: 500)");
}

#[test]
fn custom_separator() {
    let err = ComposableError::<&str, u32>::new("core error")
        .with_context("ctx1")
        .with_context("ctx2");

    let output = format!("{}", err.fmt().with_separator(" | "));
    assert_eq!(output, "ctx2 | ctx1 | core error");
}

#[test]
fn reverse_context_order() {
    let err = ComposableError::<&str, u32>::new("core error")
        .with_context("ctx1")
        .with_context("ctx2");

    let output = format!("{}", err.fmt().reverse_context(true));
    assert_eq!(output, "ctx1 -> ctx2 -> core error");
}

#[test]
fn hide_error_code() {
    let err = ComposableError::<&str, u32>::new("core error").set_code(500);

    let output = format!("{}", err.fmt().show_code(false));
    assert_eq!(output, "core error");
}

#[test]
fn complex_formatting_combination() {
    let err = ComposableError::<&str, u32>::new("core error")
        .with_context("ctx1")
        .with_context("ctx2")
        .set_code(500);

    let output = format!(
        "{}",
        err.fmt()
            .with_separator(" > ")
            .reverse_context(true)
            .show_code(false)
    );
    assert_eq!(output, "ctx1 > ctx2 > core error");
}
