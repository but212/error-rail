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

### **Benchmark Results** ([full methodology](docs/BENCHMARKS.md))

| Metric                     | Performance                      |
|----------------------------|----------------------------------|
| Error creation             | ~296 ns                          |
| Error propagation overhead | ~9% vs raw Result                |
| Serialization              | ~684 ns                          |
| Validation overhead        | ~5% vs manual collection         |
| Validation throughput      | 5.37 M elements/sec (5000 items) |

### **Performance Advantages**

| Feature                                | Performance Gain                              | Real-world Impact                          |
|----------------------------------------|-----------------------------------------------|--------------------------------------------|
| **Lazy context!() vs eager format!()** | **7x faster** on success paths (within error-rail) | Primary benefit - most operations succeed |
| **Static str vs String allocation**    | **13x faster** (within error-rail)           | Use `&'static str` when possible           |

### **Key Performance Insights**

- **Environment**: Rust 1.90.0, Windows 11, Intel i5-9400F, Criterion 0.7.0
- **Lazy evaluation**: 613 ns vs 4,277 ns on success paths - **7x faster**
- **Real-world scenarios**: HTTP request ~933 ns, DB transaction ~881 ns, microservice error propagation ~608 ns
- **Scaling**: Context depth scales linearly (50 layers = 12µs)
- **Throughput**: Validation maintains 5.37 M elements/sec at 5000 items

> **Why lazy evaluation matters**: Since most operations succeed (95%+ in production), the 7x speedup of `context!()` vs `format!()` within error-rail provides meaningful real-world performance gains, keeping error handling efficient on the happy path.

## Requirements

- **Rust**: 1.81.0 or later (MSRV)
- **Edition**: 2021

## Installation

```sh
cargo add error-rail
```

Or add to `Cargo.toml`:

```toml
[dependencies]
error-rail = "0.7"
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
fn process() -> BoxedResult<String, std::io::Error> {
    read_config()
        .ctx("processing configuration")
}

fn main() {
    if let Err(e) = process() {
        eprintln!("{}", e.error_chain());
        // Output: processing configuration -> loading configuration -> No such file or directory (os error 2)
    }
}
```

> **New to error-rail?** See the [Quick Start Guide](docs/QUICK_START.md) for step-by-step examples.

## API Levels (New in 0.7.0)

error-rail provides a 3-level API hierarchy to match your expertise level:

### Beginner API (`prelude`)

Start here! Everything you need for common error handling:

```rust
use error_rail::prelude::*;

fn load_config() -> BoxedResult<String, std::io::Error> {
    std::fs::read_to_string("config.toml")
        .ctx("loading configuration")
}
```

**Exports**: `ComposableError`, `ErrorContext`, `ErrorPipeline`, `rail!`, `context!`, `group!`, `ResultExt`, `BoxedResultExt`

### Intermediate API (`intermediate`)

Advanced patterns for service developers:

```rust
use error_rail::intermediate::*;

// Classify errors as transient/permanent
impl TransientError for MyError {
    fn is_transient(&self) -> bool { /* ... */ }
}

// Custom error formatting
let formatted = err.fmt().pretty().show_code(false).to_string();
```

**Exports**: `TransientError`, `ErrorFormatter`, `FingerprintConfig`

### Advanced API (`advanced`)

Low-level internals for library authors:

```rust
use error_rail::advanced::*;

// Direct access to internal types
let vec: ErrorVec<_> = /* ... */;
let builder = ErrorContextBuilder::new();
```

**Exports**: `ErrorVec`, `ErrorContextBuilder`, `LazyContext`, `LazyGroupContext`, `GroupContext`

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
use error_rail::{ErrorPipeline, context, group};

fn fetch_user(id: u64) -> Result<String, &'static str> {
    Err("user not found")
}

let result = ErrorPipeline::new(fetch_user(42))
    .with_context(group!(
        tag("user-service"),
        location(file!(), line!())
    ))
    .with_context(context!("user_id: {}", 42))
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

// New in 0.7.0: validate! macro for cleaner syntax
use error_rail::validate;

let age_result = validate_age(-5);
let name_result = validate_name("");

let combined = validate!(
    age = age_result,
    name = name_result
);
// Returns Validation<E, (i32, &str)> with all errors accumulated
```

### 4. Efficient Lazy Context

The `context!` macro defers string formatting until an error actually occurs.

```rust
use error_rail::{ErrorPipeline, context};

#[derive(Debug)]
struct LargePayload { /* ... */ }

fn process(data: &LargePayload) -> Result<(), &'static str> {
    Ok(()) // Success path
}

let payload = LargePayload { /* ... */ };

// format!() is not called on success path
let result = ErrorPipeline::new(process(&payload))
    .with_context(context!("processing payload: {:?}", payload))
    .finish_boxed();
```

> **Performance**: In benchmarks, lazy context adds minimal overhead on success. Eager formatting can be significantly slower.

### When to Use `.ctx()` vs `context!()`

| Method           | Use Case                          | Example                                   |
|------------------|-----------------------------------|-------------------------------------------|
| `.ctx("static")` | Simple static messages            | `.ctx("loading file")`                    |
| `context!()`     | Formatted messages with variables | `context!("user_id: {}", id)`             |
| `.ctx_with()`    | Complex closure logic             | `.ctx_with(\|\| expensive_calculation())` |

**Decision Guide:**

- Use `.ctx("static")` for simple strings - no allocation overhead
- Use `.ctx(context!())` when formatting variables - **7x faster** on success paths
- Both methods add the same context type, only evaluation timing differs

```rust
// ✅ Simple static context
result.ctx("database connection failed")

// ✅ Lazy formatted context (recommended for variables)
result.ctx(context!("user {} not found", user_id))

// ❌ Eager formatting (slow on success paths)
result.ctx(format!("user {} not found", user_id))
```

### 5. Convenient Macros

| Macro | Purpose | Example |
|-------|---------|---------|
| `context!` | Lazy formatted message | `context!("user_id: {}", id)` |
| `group!` | Structured context with multiple fields | `group!(tag("db"), location(file!(), line!()))` |
| `rail!` | Quick pipeline wrapper | `rail!(fallible_fn())` |

```rust
use error_rail::{rail, ComposableError};

// rail! is shorthand for ErrorPipeline::new(...).finish_boxed()
let result = rail!(std::fs::read_to_string("config.toml"));
```

### 6. Transient Error Classification

Classify errors as transient (retryable) or permanent for integration with retry libraries.

```rust
use error_rail::{ErrorPipeline, traits::TransientError};
use std::time::Duration;

#[derive(Debug)]
enum ApiError {
    Timeout,           // Transient - retry
    RateLimited(u64),  // Transient - retry after delay
    NotFound,          // Permanent - don't retry
}

impl TransientError for ApiError {
    fn is_transient(&self) -> bool {
        matches!(self, ApiError::Timeout | ApiError::RateLimited(_))
    }

    fn retry_after_hint(&self) -> Option<Duration> {
        match self {
            ApiError::RateLimited(secs) => Some(Duration::from_secs(*secs)),
            ApiError::Timeout => Some(Duration::from_millis(100)),
            _ => None,
        }
    }
}

// Use with ErrorPipeline
fn fetch_with_retry() {
    let result: Result<String, ApiError> = Err(ApiError::Timeout);
    let pipeline = ErrorPipeline::new(result);

    if pipeline.is_transient() {
        // Retry logic here
        println!("Retrying after {:?}", pipeline.retry_after_hint());
    }
}
```

> **Note**: error-rail does not implement retry logic itself. Use external libraries like `backoff`, `retry`, or `tokio-retry`.

### 7. Error Fingerprinting

Generate unique fingerprints for error deduplication in monitoring systems.

```rust
use error_rail::{ComposableError, ErrorContext};

let err = ComposableError::new("database timeout")
    .with_context(ErrorContext::tag("db"))
    .with_context(ErrorContext::tag("users"))
    .set_code(504);

// Generate fingerprint for Sentry/logging deduplication
println!("Fingerprint: {}", err.fingerprint_hex());
// Example output: "a1b2c3d4e5f67890"

// Customize what's included in fingerprint
let fp = err.fingerprint_config()
    .include_message(false)  // Ignore variable message content
    .include_metadata(true)  // Include metadata
    .compute_hex();
```

### 8. Type Aliases for Ergonomics

```rust
use error_rail::prelude::*;

// Recommended: BoxedResult for public API (8 bytes stack)
fn load_file() -> BoxedResult<String, std::io::Error> {
    std::fs::read_to_string("file.txt").ctx("loading file")
}

// Alternative: ComposableResult for internal use (larger stack)
use error_rail::ComposableResult;
fn internal_op() -> ComposableResult<i32, &'static str> {
    Ok(42)
}
```

#### Type Aliases Comparison

| Type Alias                    | Definition                           | Stack Size | Use When                        |
|-------------------------------|--------------------------------------|------------|---------------------------------|
| `BoxedResult<T, E>`           | `Result<T, Box<ComposableError<E>>>` | 8 bytes    | **Recommended** for public APIs |
| `BoxedComposableResult<T, E>` | Same as `BoxedResult`                | 8 bytes    | Legacy alias (identical)        |
| `ComposableResult<T, E>`      | `Result<T, ComposableError<E>>`      | 48+ bytes  | Internal functions only         |

> **Note**: `BoxedResult` and `BoxedComposableResult` are identical. Use `BoxedResult` for brevity.

## When to Use What?

### Quick Reference

| Scenario                     | Recommended Type                      | Example                         |
|------------------------------|---------------------------------------|---------------------------------|
| **Simple error wrapping**    | `ComposableError<E>`                  | Internal error handling         |
| **Function return type**     | `BoxedResult<T, E>`                   | Public API boundaries           |
| **Adding context to Result** | `ErrorPipeline`                       | Wrapping I/O operations         |
| **Form/input validation**    | `Validation<E, T>`                    | Collecting all field errors     |
| **Error chaining**           | `ErrorPipeline` + `finish_boxed()`    | Multi-step operations           |
| **Retry logic**              | `TransientError` trait                | Network timeouts, rate limiting |
| **Error deduplication**      | `fingerprint()` / `fingerprint_hex()` | Sentry grouping, log dedup      |

### Validation vs Result

| Feature              | `Result<T, E>`                 | `Validation<E, T>`  |
|----------------------|--------------------------------|---------------------|
| **Short-circuit**    | Yes (stops at first error)     | ❌ No (collects all) |
| **Use case**         | Sequential operations          | Parallel validation |
| **Error count**      | Single                         | Multiple            |
| **Iterator support** | `?` operator                   | `.collect()`        |

## Common Pitfalls

### 1. Forgetting to Box for Return Types

```rust
// ❌ Large stack size (48+ bytes per Result)
fn process() -> Result<Data, ComposableError<MyError>> { ... }

// ✅ Reduced stack size (8 bytes pointer)
fn process() -> BoxedResult<Data, MyError> { ... }
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

| Module       | Description                                                                                  |
|--------------|----------------------------------------------------------------------------------------------|
| `prelude`    | **Start here!** Common imports: `ResultExt`, `BoxedResult`, macros                           |
| `context`    | Context attachment: `with_context`, `accumulate_context`, `format_error_chain`               |
| `convert`    | Conversions between `Result`, `Validation`, and `ComposableError`                            |
| `macros`     | `context!`, `group!`, `rail!`, `impl_error_context!`                                         |
| `traits`     | `ResultExt`, `BoxedResultExt`, `IntoErrorContext`, `ErrorOps`, `WithError`, `TransientError` |
| `types`      | `ComposableError`, `ErrorContext`, `ErrorPipeline`, `LazyContext`, `FingerprintConfig`       |
| `validation` | `Validation<E, A>` type with collectors and iterators                                        |

## Feature Flags

| Feature | Description                                           | Default |
|---------|-------------------------------------------------------|---------|
| `std`   | Standard library support (enables `backtrace!` macro) | ❌ No    |
| `serde` | Serialization/deserialization support                 | ❌ No    |
| `full`  | Enable all features (`std` + `serde`)                 | ❌ No    |

### Usage Examples

```toml
# Default (no_std compatible, requires alloc)
[dependencies]
error-rail = "0.7"

# With std library support (e.g., for backtraces)
[dependencies]
error-rail = { version = "0.7", features = ["std"] }

# With serialization support
[dependencies]
error-rail = { version = "0.7", features = ["serde"] }

# All features enabled
[dependencies]
error-rail = { version = "0.7", features = ["full"] }
```

### `no_std` Support

`error-rail` is `no_std` compatible by default. It requires only the `alloc` crate.

```toml
# Minimal no_std usage
[dependencies]
error-rail = { version = "0.7", default-features = false }
```

> **Note**: Some features like `backtrace!` require the `std` feature.

## Examples

```sh
cargo run --example quick_start         # Basic usage patterns
cargo run --example readme_features     # All features from this README
cargo run --example pipeline            # Error pipeline chaining
cargo run --example validation_collect  # Validation accumulation
cargo run --example retry_integration   # Retry patterns & fingerprinting
```

## Integration Guides

- **[Quick Start Guide](docs/QUICK_START.md)** - Step-by-step tutorial
- **[Error Handling Patterns](docs/PATTERNS.md)** - Real-world usage patterns and best practices

## Glossary

| Term                | Definition                                                                |
|---------------------|---------------------------------------------------------------------------|
| **Context**         | Additional information attached to an error (location, tags, messages)    |
| **Metadata**        | Key-value pairs within context (subset of context)                        |
| **Pipeline**        | Builder pattern for chaining error operations (`ErrorPipeline`)           |
| **Boxed Error**     | Heap-allocated error via `Box<ComposableError<E>>` for reduced stack size |
| **Lazy Evaluation** | Deferred computation until actually needed (e.g., `context!` macro)       |
| **Validation**      | Accumulating type that collects all errors instead of short-circuiting    |
| **Transient Error** | Temporary failure that may succeed on retry (e.g., timeout, rate limit)   |
| **Fingerprint**     | Unique hash of error components for deduplication and grouping            |

## License

Licensed under the Apache License 2.0. See [LICENSE](LICENSE) for details.

For third-party attributions, see [NOTICE](NOTICE).
