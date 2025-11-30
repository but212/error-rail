use core::fmt::Display;
use error_rail::types::alloc_type::Vec;

use error_rail::types::error_formatter::ErrorFormatConfig;
use error_rail::types::error_formatter::ErrorFormatter;

struct TestDisplay(String);

impl Display for TestDisplay {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[test]
fn test_default_formatter() {
    let config = ErrorFormatConfig::default();
    let d1 = TestDisplay("context1".into());
    let d2 = TestDisplay("context2".into());
    let d3 = TestDisplay("error".into());

    let mut items = Vec::new();
    items.push(&d1 as &dyn Display);
    items.push(&d2 as &dyn Display);
    items.push(&d3 as &dyn Display);

    let result = config.format_chain(items.into_iter());
    assert_eq!(result, "context1 -> context2 -> error");
}

#[test]
fn test_pretty_formatter() {
    let config = ErrorFormatConfig::pretty();
    let d1 = TestDisplay("context1".into());
    let d2 = TestDisplay("context2".into());
    let d3 = TestDisplay("error".into());

    let mut items = Vec::new();
    items.push(&d1 as &dyn Display);
    items.push(&d2 as &dyn Display);
    items.push(&d3 as &dyn Display);

    let result = config.format_chain(items.into_iter());
    assert_eq!(result, "┌ context1\n├─ context2\n└─ error");
}

#[test]
fn test_compact_formatter() {
    let config = ErrorFormatConfig::compact();
    let d1 = TestDisplay("context".into());
    let d2 = TestDisplay("error".into());

    let mut items = Vec::new();
    items.push(&d1 as &dyn Display);
    items.push(&d2 as &dyn Display);

    let result = config.format_chain(items.into_iter());
    assert_eq!(result, "context | error");
}

#[test]
fn test_custom_prefix_suffix() {
    let config = ErrorFormatConfig {
        context_prefix: Some("[CTX] ".into()),
        root_prefix: Some("[ERR] ".into()),
        separator: " | ".into(),
        ..Default::default()
    };

    let d1 = TestDisplay("context".into());
    let d2 = TestDisplay("error".into());

    let mut items = Vec::new();
    items.push(&d1 as &dyn Display);
    items.push(&d2 as &dyn Display);

    let result = config.format_chain(items.into_iter());
    assert_eq!(result, "[CTX] context | [ERR] error");
}

#[test]
fn test_no_code_config() {
    let config = ErrorFormatConfig::no_code();
    assert!(!config.show_code);
}
