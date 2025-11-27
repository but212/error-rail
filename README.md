# error-rail

[![Crates.io Version](https://img.shields.io/crates/v/error-rail)](https://crates.io/crates/error-rail)
[![Documentation](https://docs.rs/error-rail/badge.svg)](https://docs.rs/error-rail)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![Ask DeepWiki](https://deepwiki.com/badge.svg)](https://deepwiki.com/but212/error-rail)

```rust
use error_rail::prelude::*;

fn load_config() -> BoxedResult<String, std::io::Error> {
    std::fs::read_to_string("config.toml")
        .ctx("loading configuration")
}
```

---

## Why error-rail?

Most error handling libraries format context eagerly—even on success paths where the context is never used. **error-rail** uses lazy evaluation, deferring string formatting until an error actually occurs.

**Benchmark Results** ([full details](docs/BENCHMARKS.md)):

| Scenario | error-rail | Eager formatting | Speedup |
|----------|------------|------------------|---------|
| Success path | 637 ns | 1,351 ns | **2.1x faster** |
| Error path | 941 ns | 835 ns | ~same |

Since most operations succeed (95%+), lazy evaluation provides significant real-world performance gains.

## Installation

```sh
cargo add error-rail
```

## 30-Second Quick Start

```rust
use error_rail::prelude::*;

// Add context to any Result with .ctx()
fn read_config() -> BoxedResult<String, std::io::Error> {
    std::fs::read_to_string("config.toml")
        .ctx("loading configuration")
}

// Chain multiple contexts
fn process() -> BoxedResult<(), std::io::Error> {
    let config = read_config()?;
    parse_config(&config)
        .ctx("parsing configuration")
}

fn main() {
    if let Err(e) = process() {
        eprintln!("{}", e.error_chain());
        // Output: parsing configuration -> loading configuration -> No such file...
    }
}
```

> **New to error-rail?** See the [Quick Start Guide](docs/QUICK_START.md) for step-by-step examples.

## Key Features

### 1. Structured Error Context

Wrap any error in `ComposableError` and attach layered metadata.

```rust
use error_rail::{ComposableError, context, group};

let err = ComposableError::<&str>::new("connection failed")
    .with_context(context!("retry attempt {}", 3))
    .with_context(group!(
        tag("database"),
        location(file!(), line!()),
        metadata("host", "localhost:5432")
    ))
    .set_code(500);

println!("{}", err.error_chain());
// Output: retry attempt 3 -> [database] at src/main.rs:7 (host=localhost:5432) -> connection failed (code: 500)
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

## When to Use What?

### Quick Reference

| Scenario | Recommended Type | Example |
|----------|------------------|---------|
| **Simple error wrapping** | `ComposableError<E>` | Internal error handling |
| **Function return type** | `BoxedComposableResult<T, E>` | Public API boundaries |
| **Adding context to Result** | `ErrorPipeline` | Wrapping I/O operations |
| **Form/input validation** | `Validation<E, T>` | Collecting all field errors |
| **Error chaining** | `ErrorPipeline` + `finish_boxed()` | Multi-step operations |

### Validation vs Result

| Feature | `Result<T, E>` | `Validation<E, T>` |
|---------|---------------|-------------------|
| **Short-circuit** | ✅ Yes (stops at first error) | ❌ No (collects all) |
| **Use case** | Sequential operations | Parallel validation |
| **Error count** | Single | Multiple |
| **Iterator support** | `?` operator | `.collect()` |

## Common Pitfalls

### 1. Forgetting to Box for Return Types

```rust
// ❌ Large stack size (48+ bytes per Result)
fn process() -> Result<Data, ComposableError<MyError>> { ... }

// ✅ Reduced stack size (8 bytes pointer)
fn process() -> BoxedComposableResult<Data, MyError> { ... }
```

### 2. Excessive Context Depth

```rust
// ❌ Adding context at every layer (O(n) performance)
db_call()
    .with_context(ctx1)
    .and_then(|x| validate(x).with_context(ctx2))
    .and_then(|x| transform(x).with_context(ctx3))
    // ... 20 more layers

// ✅ Add context at boundaries only
let result = db_call()
    .and_then(validate)
    .and_then(transform);

ErrorPipeline::new(result)
    .with_context(context!("user_id: {}", id))
    .finish_boxed()
```

### 3. Eager vs Lazy Context

```rust
// ❌ Eager: format! runs even on success
.with_context(ErrorContext::new(format!("data: {:?}", large_struct)))

// ✅ Lazy: format! only runs on error
.with_context(context!("data: {:?}", large_struct))
```

## Module Reference

| Module | Description |
|--------|-------------|
| `prelude` | **Start here!** Common imports: `ResultExt`, `BoxedResult`, macros |
| `context` | Context attachment: `with_context`, `accumulate_context`, `format_error_chain` |
| `convert` | Conversions between `Result`, `Validation`, and `ComposableError` |
| `macros` | `context!`, `location!`, `tag!`, `metadata!`, `rail!`, `impl_error_context!` |
| `traits` | `ResultExt`, `BoxedResultExt`, `IntoErrorContext`, `ErrorOps`, `WithError` |
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

## Integration Guides

- **[Quick Start Guide](docs/QUICK_START.md)** - Step-by-step tutorial

## License

Apache-2.0
