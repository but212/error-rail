//! Simple Quick Start Example
//!
//! This example demonstrates the minimal API surface of error-rail.
//! No feature flags required - works with the default configuration.
//!
//! Run with: `cargo run --example simple_quick_start`
//!
//! ## What You'll Learn
//!
//! 1. Use `BoxedResult` as the return type
//! 2. Add context with `.ctx()`
//! 3. Display error chains with `.error_chain()`

use error_rail::simple::*;

/// Simulates reading a configuration file.
///
/// In a real application, this would be `std::fs::read_to_string`.
fn read_config() -> BoxedResult<String, &'static str> {
    // Simulate a file not found error
    Err("file not found").ctx("loading configuration")
}

/// Processes the configuration.
///
/// Notice how we use `.ctx_boxed()` for already-boxed results.
fn process_config() -> BoxedResult<String, &'static str> {
    let config = read_config().ctx_boxed("processing application config")?;
    Ok(config.to_uppercase())
}

/// Initializes the application.
fn init_app() -> BoxedResult<(), &'static str> {
    let _config = process_config().ctx_boxed("initializing application")?;
    Ok(())
}

fn main() {
    println!("=== error-rail simple::* Quick Start ===\n");

    // Golden Path Rule #1: Return BoxedResult at function boundaries
    // Golden Path Rule #2: Add .ctx() only after I/O or external calls

    match init_app() {
        Ok(()) => println!("Application initialized successfully!"),
        Err(e) => {
            // Display the full error chain
            println!("Error chain:\n{}", e.error_chain());

            // Or display with default formatting
            println!("\nDefault display:\n{}", e);

            // Or display with alternate formatting (cascaded)
            println!("\nCascaded display:\n{:#}", e);
        },
    }
}
