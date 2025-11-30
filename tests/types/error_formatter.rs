use core::fmt::Display;
use error_rail::types::alloc_type::Vec;

use error_rail::types::error_formatter::ErrorFormat;
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
    assert!(!config.format.contains(ErrorFormat::SHOW_CODE));
}

#[test]
fn test_empty_iterator() {
    let config = ErrorFormatConfig::default();
    let items: Vec<&dyn core::fmt::Display> = Vec::new();

    let result = config.format_chain(items.into_iter());
    assert_eq!(result, "");
}

#[test]
fn test_single_item() {
    let config = ErrorFormatConfig::default();
    let d1 = TestDisplay("single error".into());

    let mut items = Vec::new();
    items.push(&d1 as &dyn Display);

    let result = config.format_chain(items.into_iter());
    assert_eq!(result, "single error");
}

#[test]
fn test_special_characters() {
    let config = ErrorFormatConfig::default();
    let d1 = TestDisplay("error with \n newlines".into());
    let d2 = TestDisplay("error with \t tabs".into());
    let d3 = TestDisplay("error with \"quotes\"".into());

    let mut items = Vec::new();
    items.push(&d1 as &dyn Display);
    items.push(&d2 as &dyn Display);
    items.push(&d3 as &dyn Display);

    let result = config.format_chain(items.into_iter());
    assert!(result.contains("error with \n newlines"));
    assert!(result.contains("error with \t tabs"));
    assert!(result.contains("error with \"quotes\""));
}

#[test]
fn test_unicode_characters() {
    let config = ErrorFormatConfig::default();
    let d1 = TestDisplay("에러 한글".into());
    let d2 = TestDisplay("エラー日本語".into());
    let d3 = TestDisplay("error 🚀 emoji".into());

    let mut items = Vec::new();
    items.push(&d1 as &dyn Display);
    items.push(&d2 as &dyn Display);
    items.push(&d3 as &dyn Display);

    let result = config.format_chain(items.into_iter());
    assert!(result.contains("에러 한글"));
    assert!(result.contains("エラー日本語"));
    assert!(result.contains("error 🚀 emoji"));
}

#[test]
fn test_very_long_messages() {
    let config = ErrorFormatConfig::default();
    let long_msg = "A".repeat(1000);
    let d1 = TestDisplay(long_msg.clone());
    let d2 = TestDisplay("short".into());

    let mut items = Vec::new();
    items.push(&d1 as &dyn Display);
    items.push(&d2 as &dyn Display);

    let result = config.format_chain(items.into_iter());
    assert!(result.len() > 1000);
    assert!(result.contains("short"));
}

#[test]
fn test_pretty_formatter_empty() {
    let config = ErrorFormatConfig::pretty();
    let items: Vec<&dyn core::fmt::Display> = Vec::new();

    let result = config.format_chain(items.into_iter());
    assert_eq!(result, "");
}

#[test]
fn test_pretty_formatter_single_item() {
    let config = ErrorFormatConfig::pretty();
    let d1 = TestDisplay("single error".into());

    let mut items = Vec::new();
    items.push(&d1 as &dyn Display);

    let result = config.format_chain(items.into_iter());
    assert_eq!(result, "┌ single error");
}

#[test]
fn test_compact_formatter_empty() {
    let config = ErrorFormatConfig::compact();
    let items: Vec<&dyn core::fmt::Display> = Vec::new();

    let result = config.format_chain(items.into_iter());
    assert_eq!(result, "");
}

#[test]
fn test_compact_formatter_single_item() {
    let config = ErrorFormatConfig::compact();
    let d1 = TestDisplay("single error".into());

    let mut items = Vec::new();
    items.push(&d1 as &dyn Display);

    let result = config.format_chain(items.into_iter());
    assert_eq!(result, "single error");
}

#[test]
fn test_custom_separator_empty() {
    let config = ErrorFormatConfig {
        separator: " CUSTOM ".into(),
        ..Default::default()
    };
    let items: Vec<&dyn core::fmt::Display> = Vec::new();

    let result = config.format_chain(items.into_iter());
    assert_eq!(result, "");
}

#[test]
fn test_custom_prefix_suffix_empty() {
    let config = ErrorFormatConfig {
        context_prefix: Some("[CTX] ".into()),
        root_prefix: Some("[ERR] ".into()),
        separator: " | ".into(),
        ..Default::default()
    };
    let items: Vec<&dyn core::fmt::Display> = Vec::new();

    let result = config.format_chain(items.into_iter());
    assert_eq!(result, "");
}

#[test]
fn test_many_items_chain() {
    let config = ErrorFormatConfig::default();
    let mut items = Vec::new();

    for i in 0..20 {
        let display = TestDisplay(format!("context_{}", i));
        items.push(display);
    }

    let display_refs: Vec<&dyn Display> = items.iter().map(|d| d as &dyn Display).collect();
    let result = config.format_chain(display_refs.into_iter());

    // Should contain all contexts in reverse order
    for i in (0..20).rev() {
        assert!(result.contains(&format!("context_{}", i)));
    }
}

#[test]
fn test_pretty_formatter_many_items() {
    let config = ErrorFormatConfig::pretty();
    let mut items = Vec::new();

    for i in 0..5 {
        let display = TestDisplay(format!("context_{}", i));
        items.push(display);
    }

    let display_refs: Vec<&dyn Display> = items.iter().map(|d| d as &dyn Display).collect();
    let result = config.format_chain(display_refs.into_iter());

    // Should start with ┌ for first item, have ├─ for middle, └─ for last
    assert!(result.starts_with("┌ context_0"));
    assert!(result.contains("├─ context_"));
    assert!(result.contains("└─ context_4"));
}

#[test]
fn test_zero_width_characters() {
    let config = ErrorFormatConfig::default();
    let d1 = TestDisplay("error\u{200B}zero\u{200C}width".into()); // Zero-width spaces
    let d2 = TestDisplay("normal".into());

    let mut items = Vec::new();
    items.push(&d1 as &dyn Display);
    items.push(&d2 as &dyn Display);

    let result = config.format_chain(items.into_iter());
    assert!(result.contains("error\u{200B}zero\u{200C}width"));
    assert!(result.contains("normal"));
}
