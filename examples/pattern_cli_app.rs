//! CLI Applications Pattern
//!
//! Providing helpful, user-friendly error messages in command-line tools.

use error_rail::{context, ComposableError, ErrorPipeline};

fn read_config_file(path: &str) -> Result<String, Box<ComposableError<std::io::Error>>> {
    ErrorPipeline::new(std::fs::read_to_string(path))
        .with_context(context!("reading configuration from '{}'", path))
        .finish_boxed()
}

fn parse_config(content: &str) -> Result<(), Box<ComposableError<String>>> {
    if content.is_empty() {
        return ErrorPipeline::new(Err("configuration is empty".to_string()))
            .with_context("parsing configuration")
            .finish_boxed();
    }
    Ok(())
}

fn load_and_parse_config(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let content = read_config_file(path).map_err(|e| e as Box<dyn std::error::Error>)?;
    // Use a simple error type that implements Error trait
    parse_config(&content).map_err(|e| {
        Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            e.error_chain(),
        )) as Box<dyn std::error::Error>
    })?;
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
        }
    }
}
