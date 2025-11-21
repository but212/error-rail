# Quick Start Guide

Welcome to `error-rail`! This guide will help you get started with composable error handling in Rust without getting bogged down in complex traits.

## 1. Basic Error Context

The core of `error-rail` is `ComposableError`. It allows you to wrap any error and add structured context.

```rust
use error_rail::{ComposableError, ErrorContext, context};

fn perform_task() -> Result<(), Box<ComposableError<std::io::Error>>> {
    // Wrap a standard error
    let result = std::fs::read_to_string("config.toml")
        .map_err(|e| ComposableError::new(e));

    // Add context
    let res = result.map_err(|e| e.with_context(context!("loading configuration")))
        .map(|_| ());
    
    // Box the error if your return type requires it
    res.map_err(Box::new)
}
```

## 2. Using Macros

`error-rail` provides macros to make adding context easier:

- `context!`: Adds a human-readable message.
- `location!`: Adds the file and line number.
- `tag!`: Adds a tag for filtering or categorization.

```rust
use error_rail::{context, location, tag, ComposableError};

fn perform_task() -> Result<(), Box<ComposableError<std::io::Error>>> {
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
```

## 3. Collecting Errors (Validation)

Sometimes you want to collect multiple errors instead of stopping at the first one. Use `Validation`.

```rust
use error_rail::validation::Validation;

fn validate_input(input: &str) -> Validation<String, ()> {
    if input.is_empty() {
        Validation::invalid("input cannot be empty".to_string())
    } else {
        Validation::Valid(())
    }
}

fn main() {
    let inputs = vec!["", "valid", ""];
    // Collects into Validation<String, Vec<()>>
    let results: Validation<String, Vec<()>> = inputs
        .into_iter()
        .map(validate_input)
        .collect();

    if let Validation::Invalid(errors) = results {
        println!("Found {} errors:", errors.len());
        for err in errors {
            println!("- {}", err);
        }
    }
}
```

## 4. Next Steps

Once you are comfortable with these basics, you can explore:

- **Error Pipelines**: For more complex error transformation chains.
- **Custom Contexts**: Implement `IntoErrorContext` for your own types.
- **Traits**: `ErrorOps` and `WithError` for deeper integration.

See the [README](README.md) and [API Documentation](https://docs.rs/error-rail) for more details.
