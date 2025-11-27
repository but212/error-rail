# Quick Start Guide

Welcome to `error-rail`! This guide walks you through composable error handling step by step.

> **Run the Examples**: All code in this guide is available as a runnable example:
>
> ```sh
> cargo run --example quick_start
> ```

## Step 1: Your First ComposableError

The core type is `ComposableError<E>`. It wraps any error and lets you attach context.

```rust
use error_rail::{ComposableError, context};

// Wrap an error and add a message
let err = ComposableError::<&str>::new("file not found")
    .with_context(context!("loading user profile"));

println!("{}", err.error_chain());
// Output: loading user profile -> file not found
```

## Step 2: Using ErrorPipeline

`ErrorPipeline` provides a fluent API for wrapping `Result` values with context.

```rust
use error_rail::{ErrorPipeline, context, location, tag};

fn read_config() -> Result<String, Box<error_rail::ComposableError<std::io::Error>>> {
    ErrorPipeline::new(std::fs::read_to_string("config.toml"))
        .with_context(location!())              // Captures file:line automatically
        .with_context(tag!("config"))           // Categorical tag for filtering
        .with_context(context!("reading app configuration"))
        .finish_boxed()                         // Converts to boxed result
}

fn main() {
    match read_config() {
        Ok(content) => println!("Config loaded: {} bytes", content.len()),
        Err(e) => {
            // Print the full error chain
            eprintln!("Error: {}", e.error_chain());
        }
    }
}
```

## Step 3: Context Macros

`error-rail` provides five macros for adding context:

| Macro | What it does | Example |
|-------|--------------|---------|
| `context!` | Lazy formatted message | `context!("user_id: {}", 42)` |
| `location!` | Source file and line | `location!()` |
| `tag!` | Categorical label | `tag!("database")` |
| `metadata!` | Key-value pair | `metadata!("host", "localhost")` |
| `rail!` | Quick pipeline wrapper | `rail!(fs::read("file"))` |

```rust
use error_rail::{ComposableError, context, location, tag, metadata};

let err = ComposableError::<&str>::new("timeout")
    .with_context(tag!("http"))
    .with_context(location!())
    .with_context(metadata!("endpoint", "/api/users"))
    .with_context(context!("request failed after {} retries", 3));

println!("{}", err.error_chain());
// Output: request failed after 3 retries -> endpoint=/api/users -> src/main.rs:5 -> [http] -> timeout
```

### The `rail!` Shortcut

```rust
use error_rail::rail;

// This:
let result = rail!(std::fs::read_to_string("config.toml"));

// Is equivalent to:
// let result = ErrorPipeline::new(std::fs::read_to_string("config.toml")).finish_boxed();
```

## Step 4: Validation (Collecting Multiple Errors)

Use `Validation` when you want to collect all errors instead of stopping at the first one.

```rust
use error_rail::Validation;

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

fn main() {
    // Test with invalid inputs
    let username = validate_username("ab");      // Invalid: too short
    let email = validate_email("invalid");       // Invalid: no @

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
    // Output:
    // Found 2 validation errors:
    //   - username must be at least 3 characters
    //   - email must contain @
}
```

## Step 5: Error Codes

Attach numeric error codes for programmatic error handling:

```rust
use error_rail::{ComposableError, context, tag};

let err = ComposableError::<&str>::new("unauthorized")
    .with_context(tag!("auth"))
    .with_context(context!("invalid credentials"))
    .set_code(401);

// Access the code
if let Some(code) = err.error_code() {
    match code {
        401 => println!("Please log in again"),
        403 => println!("Access denied"),
        _ => println!("Error code: {}", code),
    }
}
```

## Step 6: Type Aliases

Use built-in type aliases to reduce boilerplate:

```rust
use error_rail::{ComposableResult, BoxedComposableResult, ComposableError};

// ComposableResult<T, E> = Result<T, ComposableError<E>>
fn parse_number(s: &str) -> ComposableResult<i32, &'static str> {
    s.parse().map_err(|_| ComposableError::new("invalid number"))
}

// BoxedComposableResult<T, E> = Result<T, Box<ComposableError<E>>>
fn load_config() -> BoxedComposableResult<String, std::io::Error> {
    error_rail::ErrorPipeline::new(std::fs::read_to_string("config.toml"))
        .finish_boxed()
}
```

## Next Steps

You've learned the basics! Here's what to explore next:

- **[README](../README.md)** - Full feature overview and module reference
- **[API Docs](https://docs.rs/error-rail)** - Complete API documentation
- **Examples** - Run more examples:

  ```sh
  cargo run --example pipeline           # Error pipeline chaining
  cargo run --example validation_collect # Advanced validation patterns
  cargo run --example readme_features    # All README examples
  ```

### Advanced Topics

- **`IntoErrorContext` trait** - Convert custom types to error context
- **`ErrorOps` trait** - Recovery and mapping operations
- **`WithError` trait** - Transform error types while preserving success values
- **`backtrace!` macro** - Capture stack traces (requires `std` feature)
