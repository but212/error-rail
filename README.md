# error-rail

[![Crates.io Version](https://img.shields.io/crates/v/error-rail)](https://crates.io/crates/error-rail)
[![Documentation](https://docs.rs/error-rail/badge.svg)](https://docs.rs/error-rail)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![Ask DeepWiki](https://deepwiki.com/badge.svg)](https://deepwiki.com/but212/error-rail)

**Composable, metadata-friendly error utilities for Rust.**

`error-rail` bridges the gap between simple string errors and full tracing systems. Attach rich, structured context to errors and collect multiple validation failures—all with zero-cost abstractions on the success path.

## Installation

```sh
cargo add error-rail
```

## Quick Start

> **New to error-rail?** Check out the [Quick Start Guide](docs/QUICK_START.md) for step-by-step examples.

```rust
use error_rail::{ComposableError, ErrorPipeline, context, location, tag};

// Wrap any error with rich context
fn load_config() -> Result<String, Box<ComposableError<std::io::Error>>> {
    ErrorPipeline::new(std::fs::read_to_string("config.toml"))
        .with_context(location!())           // Capture file:line
        .with_context(tag!("config"))        // Categorical tag
        .with_context(context!("loading application config"))
        .finish_boxed()
}

fn main() {
    match load_config() {
        Ok(content) => println!("Loaded: {} bytes", content.len()),
        Err(e) => eprintln!("Error chain: {}", e.error_chain()),
        // Output: loading application config -> [config] -> src/main.rs:6 -> No such file...
    }
}
```

## Key Features

### 1. Structured Error Context

Wrap any error in `ComposableError` and attach layered metadata.

```rust
use error_rail::{ComposableError, context, location, tag, metadata};

let err = ComposableError::<&str>::new("connection failed")
    .with_context(tag!("database"))
    .with_context(location!())
    .with_context(metadata!("host", "localhost:5432"))
    .with_context(context!("retry attempt {}", 3))
    .set_code(500);

println!("{}", err.error_chain());
// Output: retry attempt 3 -> host=localhost:5432 -> src/main.rs:7 -> [database] -> connection failed (code: 500)
```

### 2. Error Pipeline

Chain context and transformations with a fluent builder API.

```rust
use error_rail::{ErrorPipeline, context, location, tag};

fn fetch_user(id: u64) -> Result<String, &'static str> {
    Err("user not found")
}

let result = ErrorPipeline::new(fetch_user(42))
    .with_context(tag!("user-service"))
    .with_context(context!("user_id: {}", 42))
    .with_context(location!())
    .map_error(|e| format!("Fetch failed: {}", e))  // Transform error type
    .finish_boxed();

if let Err(e) = result {
    eprintln!("{}", e.error_chain());
}
```

### 3. Validation Accumulation

Collect multiple errors instead of failing fast—ideal for form validation.

```rust
use error_rail::Validation;

fn validate_age(age: i32) -> Validation<&'static str, i32> {
    if age >= 0 && age <= 150 {
        Validation::Valid(age)
    } else {
        Validation::invalid("age must be between 0 and 150")
    }
}

fn validate_name(name: &str) -> Validation<&'static str, &str> {
    if !name.is_empty() {
        Validation::Valid(name)
    } else {
        Validation::invalid("name cannot be empty")
    }
}

// Collect all validation results
let results: Validation<&str, Vec<_>> = vec![
    validate_age(-5).map(|v| v.to_string()),
    validate_name("").map(|v| v.to_string()),
].into_iter().collect();

match results {
    Validation::Valid(values) => println!("All valid: {:?}", values),
    Validation::Invalid(errors) => {
        for err in errors {
            println!("Error: {}", err);
        }
    }
}
// Output:
// Error: age must be between 0 and 150
// Error: name cannot be empty
```

### 4. Zero-Cost Lazy Context

The `context!` macro defers string formatting until an error actually occurs.

```rust
use error_rail::{ErrorPipeline, context};

#[derive(Debug)]
struct LargePayload { /* ... */ }

fn process(data: &LargePayload) -> Result<(), &'static str> {
    Ok(()) // Success path
}

let payload = LargePayload { /* ... */ };

// format!() is NEVER called on success path
let result = ErrorPipeline::new(process(&payload))
    .with_context(context!("processing payload: {:?}", payload))
    .finish_boxed();
```

> **Performance**: In benchmarks, lazy context adds near-zero overhead on success. Eager formatting can be orders of magnitude slower.

### 5. Convenient Macros

| Macro | Purpose | Example |
|-------|---------|---------|
| `context!` | Lazy formatted message | `context!("user_id: {}", id)` |
| `location!` | Capture file:line | `location!()` |
| `tag!` | Categorical label | `tag!("database")` |
| `metadata!` | Key-value pair | `metadata!("host", "localhost")` |
| `rail!` | Quick pipeline wrapper | `rail!(fallible_fn())` |

```rust
use error_rail::{rail, ComposableError};

// rail! is shorthand for ErrorPipeline::new(...).finish_boxed()
let result = rail!(std::fs::read_to_string("config.toml"));
```

### 6. Type Aliases for Ergonomics

```rust
use error_rail::{ComposableResult, BoxedComposableResult};

// Instead of Result<T, ComposableError<E>>
fn parse_config() -> ComposableResult<Config, ParseError> { /* ... */ }

// Instead of Result<T, Box<ComposableError<E>>>
fn load_file() -> BoxedComposableResult<String, std::io::Error> { /* ... */ }
```

## Module Reference

| Module | Description |
|--------|-------------|
| `context` | Context attachment functions: `with_context`, `accumulate_context`, `format_error_chain` |
| `convert` | Conversions between `Result`, `Validation`, and `ComposableError` |
| `macros` | `context!`, `location!`, `tag!`, `metadata!`, `rail!`, `impl_error_context!` |
| `traits` | `IntoErrorContext`, `ErrorOps`, `WithError` |
| `types` | `ComposableError`, `ErrorContext`, `ErrorPipeline`, `LazyContext` |
| `validation` | `Validation<E, A>` type with collectors and iterators |

## `no_std` Support

`error-rail` is `no_std` compatible by default. It requires only the `alloc` crate.

```toml
[dependencies]
error-rail = { version = "0.4", default-features = false }
```

Enable `std` features when needed:

```toml
[dependencies]
error-rail = { version = "0.4", features = ["std"] }  # Enables backtrace! macro
```

## Examples

```sh
cargo run --example quick_start       # Basic usage patterns
cargo run --example readme_features   # All features from this README
cargo run --example pipeline          # Error pipeline chaining
cargo run --example validation_collect # Validation accumulation
```

## License

Apache-2.0
