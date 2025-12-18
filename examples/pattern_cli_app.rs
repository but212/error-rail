//! CLI Applications Pattern
//!
//! Providing helpful, user-friendly error messages in command-line tools.

use error_rail::{context, ComposableError, ErrorPipeline};

/// Custom error type for parsing operations that implements std::error::Error
#[derive(Debug)]
struct ParseError(String);

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for ParseError {}

fn read_config_file(path: &str) -> Result<String, Box<ComposableError<std::io::Error>>> {
    ErrorPipeline::new(std::fs::read_to_string(path))
        .with_context(context!("reading configuration from '{}'", path))
        .finish_boxed()
}

fn parse_config(content: &str) -> Result<(), Box<ComposableError<ParseError>>> {
    if content.is_empty() {
        return ErrorPipeline::new(Err(ParseError("configuration is empty".to_string())))
            .with_context("parsing configuration")
            .finish_boxed();
    }
    Ok(())
}

fn load_and_parse_config(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let content = read_config_file(path).map_err(|e| e as Box<dyn std::error::Error>)?;
    parse_config(&content)?;
    Ok(())
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let config_path = args.get(1).map(|s| s.as_str()).unwrap_or("config.toml");

    match load_and_parse_config(config_path) {
        Ok(_) => println!("✓ Configuration loaded successfully"),
        Err(e) => {
            eprintln!("✗ Error: {}", e);

            // In debug mode, show full error chain
            if std::env::var("DEBUG").is_ok() {
                eprintln!("\nDebug trace:");
                eprintln!("{:#}", e);
            } else {
                eprintln!("\n💡 Hint: Run with DEBUG=1 for detailed error trace");
            }

            std::process::exit(1);
        },
    }
}
