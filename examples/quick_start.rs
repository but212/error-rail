use error_rail::validation::Validation;
use error_rail::{context, location, tag, ComposableError};

fn perform_task_basic() -> Result<(), Box<ComposableError<std::io::Error>>> {
    // Wrap a standard error
    let result = std::fs::read_to_string("config.toml").map_err(|e| ComposableError::new(e));

    // Add context
    let res = result
        .map_err(|e| e.with_context(context!("loading configuration")))
        .map(|_| ());

    // Box the error if your return type requires it
    res.map_err(Box::new)
}

fn perform_task_macros() -> Result<(), Box<ComposableError<std::io::Error>>> {
    let res = std::fs::read_to_string("config.toml")
        .map_err(|e| {
            ComposableError::new(e)
                .with_context(location!())
                .with_context(tag!("config"))
                .with_context(context!("failed to read config file"))
        })
        .map(|_| ());

    res.map_err(Box::new)
}

fn validate_input(input: &str) -> Validation<String, ()> {
    if input.is_empty() {
        Validation::invalid("input cannot be empty".to_string())
    } else {
        Validation::Valid(())
    }
}

fn main() {
    println!("Running Quick Start examples...");

    // 1. Basic Error Context
    println!("\n1. Basic Error Context:");
    match perform_task_basic() {
        Ok(_) => println!("Success!"),
        Err(e) => println!("Error: {:?}", e), // Debug print to show structure
    }

    // 2. Using Macros
    println!("\n2. Using Macros:");
    match perform_task_macros() {
        Ok(_) => println!("Success!"),
        Err(e) => println!("Error: {:?}", e),
    }

    // 3. Collecting Errors
    println!("\n3. Collecting Errors:");
    let inputs = vec!["", "valid", ""];
    let results: Validation<String, Vec<()>> = inputs.into_iter().map(validate_input).collect();

    if let Validation::Invalid(errors) = results {
        println!("Found {} errors:", errors.len());
        for err in errors {
            println!("- {}", err);
        }
    }
}
