//! README Features Example
//!
//! This example demonstrates all the key features shown in README.md.
//! Run with: cargo run --example readme_features

use error_rail::{context, group, rail, ComposableError, ErrorContext, ErrorPipeline, Validation};

// =============================================================================
// Feature 1: Structured Error Context
// =============================================================================

fn feature1_structured_context() {
    println!("=== Feature 1: Structured Error Context ===\n");

    let err = ComposableError::<&str>::new("connection failed")
        .with_context(context!("retry attempt {}", 3))
        .with_context(
            ErrorContext::builder()
                .tag("database")
                .location(file!(), line!())
                .metadata("host", "localhost:5432")
                .build(),
        )
        .set_code(500);

    println!("{}", err.error_chain());
    // Output: retry attempt 3 -> [database] at examples\readme_features.rs:19 (host=localhost:5432) -> connection failed (code: 500)
}

// =============================================================================
// Feature 2: Error Pipeline
// =============================================================================

fn fetch_user(_id: u64) -> Result<String, &'static str> {
    Err("user not found")
}

fn feature2_error_pipeline() {
    println!("\n=== Feature 2: Error Pipeline ===\n");

    let result = ErrorPipeline::new(fetch_user(42))
        .with_context(group!(
            tag("user-service"),
            message("user_id: {}", 42),
            location(file!(), line!())
        ))
        .map_error(|e| format!("Fetch failed: {}", e))
        .finish_boxed();

    if let Err(e) = result {
        println!("{}", e.error_chain());
    }
}

// =============================================================================
// Feature 3: Validation Accumulation
// =============================================================================

fn validate_age(age: i32) -> Validation<&'static str, i32> {
    if age >= 0 && age <= 150 {
        Validation::Valid(age)
    } else {
        Validation::invalid("age must be between 0 and 150")
    }
}

fn validate_name(name: &str) -> Validation<&'static str, String> {
    if !name.is_empty() {
        Validation::Valid(name.to_string())
    } else {
        Validation::invalid("name cannot be empty")
    }
}

fn feature3_validation() {
    println!("\n=== Feature 3: Validation Accumulation ===\n");

    // Collect all validation results - errors accumulate
    let results: Validation<&str, Vec<String>> = vec![
        validate_age(-5).map(|v| v.to_string()),
        validate_name("").map(|v| v),
    ]
    .into_iter()
    .collect();

    match results {
        Validation::Valid(values) => println!("All valid: {:?}", values),
        Validation::Invalid(errors) => {
            println!("Found {} errors:", errors.len());
            for err in errors {
                println!("  - {}", err);
            }
        }
    }
}

// =============================================================================
// Feature 4: Zero-Cost Lazy Context
// =============================================================================

#[derive(Debug)]
struct LargePayload {
    data: Vec<u8>,
}

fn process(_data: &LargePayload) -> Result<(), &'static str> {
    Ok(()) // Success path - format! is never called
}

fn feature4_lazy_context() {
    println!("\n=== Feature 4: Zero-Cost Lazy Context ===\n");

    let payload = LargePayload {
        data: vec![1, 2, 3, 4, 5],
    };
    let payload_len = payload.data.len();

    // format!() is NEVER called on success path
    let result = ErrorPipeline::new(process(&payload))
        .with_context(context!("processing payload with {} bytes", payload_len))
        .finish_boxed();

    match result {
        Ok(_) => println!("Success! (format! was never called)"),
        Err(e) => println!("Error: {}", e.error_chain()),
    }

    // Demonstrate with error case
    fn process_fail(_data: &LargePayload) -> Result<(), &'static str> {
        Err("processing failed")
    }

    let payload2 = LargePayload {
        data: vec![1, 2, 3],
    };
    let payload2_len = payload2.data.len();

    let result2 = ErrorPipeline::new(process_fail(&payload2))
        .with_context(context!("processing payload with {} bytes", payload2_len))
        .finish_boxed();

    if let Err(e) = result2 {
        println!("Error path (format! called): {}", e.error_chain());
    }
}

// =============================================================================
// Feature 5: Convenient Macros
// =============================================================================

fn feature5_macros() {
    println!("\n=== Feature 5: Convenient Macros ===\n");

    // context! - lazy formatted message
    let err1 = ComposableError::<&str>::new("error").with_context(context!("user_id: {}", 123));
    println!("context!: {}", err1.error_chain());

    // group! - combined grouped context
    let err2 =
        ComposableError::<&str>::new("error").with_context(group!(location(file!(), line!())));
    println!("group! (location): {}", err2.error_chain());

    let err3 = ComposableError::<&str>::new("error").with_context(group!(tag("database")));
    println!("group! (tag): {}", err3.error_chain());

    let err4 =
        ComposableError::<&str>::new("error").with_context(group!(metadata("host", "localhost")));
    println!("group! (metadata): {}", err4.error_chain());

    // rail! - quick pipeline wrapper
    println!("\nrail! macro:");
    let result = rail!(std::fs::read_to_string("nonexistent.toml"));
    if let Err(e) = result {
        println!("rail!: {}", e.error_chain());
    }
}

// =============================================================================
// Feature 6: Quick Start Example
// =============================================================================

fn load_config() -> Result<String, Box<ComposableError<std::io::Error>>> {
    ErrorPipeline::new(std::fs::read_to_string("config.toml"))
        .with_context(group!(
            tag("config"),
            message("loading application config"),
            location(file!(), line!())
        ))
        .finish_boxed()
}

fn feature6_quick_start() {
    println!("\n=== Feature 6: Quick Start Example ===\n");

    match load_config() {
        Ok(content) => println!("Loaded: {} bytes", content.len()),
        Err(e) => println!("Error chain: {}", e.error_chain()),
    }
}

// =============================================================================
// Main
// =============================================================================

fn main() {
    println!("╔════════════════════════════════════════╗");
    println!("║    error-rail README Features Demo     ║");
    println!("╚════════════════════════════════════════╝\n");

    feature1_structured_context();
    feature2_error_pipeline();
    feature3_validation();
    feature4_lazy_context();
    feature5_macros();
    feature6_quick_start();

    println!("\n✓ All features demonstrated!");
}
