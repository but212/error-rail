//! Quick Start Example
//!
//! This example demonstrates the core features of error-rail as shown in docs/QUICK_START.md.
//! Run with: cargo run --example quick_start

use error_rail::{
    context, group, rail, ComposableError, ComposableResult, ErrorPipeline, Validation,
};

// =============================================================================
// Step 1: Your First ComposableError
// =============================================================================

fn step1_basic_composable_error() {
    println!("=== Step 1: Your First ComposableError ===\n");

    let err = ComposableError::<&str>::new("file not found")
        .with_context(context!("loading user profile"));

    println!("Error chain: {}", err.error_chain());
    // Output: loading user profile -> file not found
}

// =============================================================================
// Step 2: Using ErrorPipeline
// =============================================================================

fn read_config() -> Result<String, Box<ComposableError<std::io::Error>>> {
    ErrorPipeline::new(std::fs::read_to_string("config.toml"))
        .with_context(group!(
            location(file!(), line!()),
            tag("config"),
            message("reading app configuration")
        ))
        .finish_boxed()
}

fn step2_error_pipeline() {
    println!("\n=== Step 2: Using ErrorPipeline ===\n");

    match read_config() {
        Ok(content) => println!("Config loaded: {} bytes", content.len()),
        Err(e) => {
            println!("Error chain: {}", e.error_chain());
        }
    }
}

// =============================================================================
// Step 3: Context Macros
// =============================================================================

fn step3_context_macros() {
    println!("\n=== Step 3: Context Macros ===\n");

    let err = ComposableError::<&str>::new("timeout")
        .with_context(group!(
            tag("http"),
            location(file!(), line!()),
            metadata("endpoint", "/api/users")
        ))
        .with_context(context!("request failed after {} retries", 3));

    println!("Error chain: {}", err.error_chain());

    // Demonstrate rail! shortcut
    println!("\nUsing rail! macro:");
    let result = rail!(std::fs::read_to_string("nonexistent.txt"));
    if let Err(e) = result {
        println!("rail! error: {}", e.error_chain());
    }
}

// =============================================================================
// Step 4: Validation (Collecting Multiple Errors)
// =============================================================================

fn validate_username(name: &str) -> Validation<&'static str, &str> {
    if name.len() >= 3 {
        Validation::Valid(name)
    } else {
        Validation::invalid("username must be at least 3 characters")
    }
}

fn validate_email(email: &str) -> Validation<&'static str, &str> {
    if email.contains('@') {
        Validation::Valid(email)
    } else {
        Validation::invalid("email must contain @")
    }
}

fn step4_validation() {
    println!("\n=== Step 4: Validation (Collecting Multiple Errors) ===\n");

    // Test with invalid inputs
    let username = validate_username("ab"); // Invalid: too short
    let email = validate_email("invalid"); // Invalid: no @

    // Collect results - errors accumulate instead of short-circuiting
    let results: Validation<&str, Vec<&str>> = vec![username, email].into_iter().collect();

    match results {
        Validation::Valid(values) => println!("Valid: {:?}", values),
        Validation::Invalid(errors) => {
            println!("Found {} validation errors:", errors.len());
            for err in errors {
                println!("  - {}", err);
            }
        }
    }
}

// =============================================================================
// Step 5: Error Codes
// =============================================================================

fn step5_error_codes() {
    println!("\n=== Step 5: Error Codes ===\n");

    let err = ComposableError::<&str>::new("unauthorized")
        .with_context(group!(tag("auth"), message("invalid credentials")))
        .set_code(401);

    println!("Error chain: {}", err.error_chain());

    // Access the code
    if let Some(code) = err.error_code() {
        match code {
            401 => println!("Action: Please log in again"),
            403 => println!("Action: Access denied"),
            _ => println!("Error code: {}", code),
        }
    }
}

// =============================================================================
// Step 6: Type Aliases
// =============================================================================

fn parse_number(s: &str) -> ComposableResult<i32, &'static str> {
    s.parse()
        .map_err(|_| ComposableError::new("invalid number"))
}

fn step6_type_aliases() {
    println!("\n=== Step 6: Type Aliases ===\n");

    // ComposableResult<T, E> = Result<T, ComposableError<E>>
    match parse_number("42") {
        Ok(n) => println!("Parsed number: {}", n),
        Err(e) => println!("Parse error: {}", e.error_chain()),
    }

    match parse_number("not-a-number") {
        Ok(n) => println!("Parsed number: {}", n),
        Err(e) => println!("Parse error: {}", e.error_chain()),
    }
}

// =============================================================================
// Main
// =============================================================================

fn main() {
    println!("╔════════════════════════════════════════╗");
    println!("║     error-rail Quick Start Examples    ║");
    println!("╚════════════════════════════════════════╝\n");

    step1_basic_composable_error();
    step2_error_pipeline();
    step3_context_macros();
    step4_validation();
    step5_error_codes();
    step6_type_aliases();

    println!("\n✓ All examples completed!");
}
