# error-rail

[![Crates.io Version](https://img.shields.io/crates/v/error-rail)](https://crates.io/crates/error-rail)
[![Documentation](https://docs.rs/error-rail/badge.svg)](https://docs.rs/error-rail)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![Ask DeepWiki](https://deepwiki.com/badge.svg)](https://deepwiki.com/but212/error-rail)

**Composable, metadata-friendly error utilities for Rust.**

`error-rail` provides a middle ground between simple string errors and full-blown tracing systems. It allows you to attach rich, structured context to errors and collect multiple validation failures without early returns.

## Quick Start

> **New to error-rail?** Check out the [Quick Start Guide](QUICK_START.md) for a beginner-friendly introduction.

## Key Features

### 1. Structured Context

Wrap any error in `ComposableError` and attach layered metadata like messages, file locations, and tags.

```rust
use error_rail::{ComposableError, context, location, tag};

let err = ComposableError::<&str, u32>::new("db error")
    .with_context(tag!("database"))
    .with_context(location!())
    .with_context(context!("failed to connect"));

// Print the full error chain
println!("{}", err.error_chain());
// Output: failed to connect -> at src/main.rs:10 -> [database] -> db error
```

### 2. Validation Accumulation

Collect multiple errors instead of failing fast. Ideal for form validation or batch processing.

```rust
use error_rail::validation::Validation;

let v1 = Validation::<&str, i32>::valid(10);
let v2 = Validation::<&str, i32>::invalid("too small");
let combined: Validation<&str, Vec<i32>> = vec![v1, v2].into_iter().collect();

assert!(combined.is_invalid());
```

### 3. Performance Optimization

Use lazy context evaluation to avoid expensive string formatting on the success path.

```rust
use error_rail::{ErrorPipeline, context};

let result = ErrorPipeline::new(risky_operation())
    .with_context(context!("computed: {:?}", large_struct)) // Only runs if error occurs
    .finish();
```

> **Note**: The `context!` macro uses `LazyContext` internally.
> This means the `format!` call and its arguments are evaluated only if an error actually occurs.
>
> In a synthetic benchmark with a 100-string payload, attaching a lazy context on the
> success path measured ~3.8 ns/iter, essentially identical to a baseline pipeline (~3.9 ns).
> Eagerly formatting the same payload took ~6.9 µs/iter (~1,800× slower). On the error path,
> lazy and eager contexts have similar cost because both must format the message.

### 4. Error Pipeline

Chain context and transformations in a fluent interface.

```rust
use error_rail::{ErrorPipeline, context, tag};

let result = ErrorPipeline::new(database_query())
    .with_context(tag!("database"))
    .with_context(context!("user_id: {}", user_id))
    .map_err(|e| format!("Query failed: {}", e))
    .with_context(context!("operation: fetch_profile"))
    .finish();
```

### 5. Ergonomic Traits

- **`IntoErrorContext`**: Convert your own types into error context.
- **`WithError`**: Transform error types while preserving success values.
- **`ErrorOps`**: Unified operations for recovery and mapping.

## Module Overview

- **`context`**: Functions for wrapping errors with context:
  - `with_context` / `with_context_result` - Add context to errors
  - `accumulate_context` - Attach multiple contexts at once
  - `error_pipeline` - Create error processing pipelines
  - `format_error_chain` - Format errors as human-readable chains

- **`convert`**: Convert between `Result`, `Validation`, and `ComposableError`.

- **`macros`**: Ergonomic shortcuts for context creation:
  - `context!` - Lazy string formatting (deferred until error occurs)
  - `location!` - Capture source file/line automatically  
  - `tag!` - Add categorical tags
  - `metadata!` - Attach key-value pairs
  - `rail!` - Shorthand for `ErrorPipeline::new(...).finish()`

- **`traits`**: Core traits for error handling:
  - `IntoErrorContext` - Convert types to error context
  - `ErrorOps` - Recovery and mapping operations
  - `WithError` - Remap error types
  - `ErrorCategory` - Categorical abstraction (internal)

- **`types`**: Error structures and utilities:
  - `ComposableError<E, C>` - Main error wrapper with context stack
  - `ErrorContext` - Structured metadata (messages, locations, tags, metadata)
  - `ErrorPipeline<T, E>` - Builder for chaining context and transformations
  - `LazyContext<F>` - Deferred context evaluation for performance

- **`validation`**: Accumulating validation results:
  - `Validation<E, A>` - Either valid value or accumulated errors
  - Iterators and collectors for aggregating multiple validations

## Installation

```sh
cargo add error-rail
```

## Examples

Run the bundled examples to see `error-rail` in action:

```sh
cargo run --example quick_start
cargo run --example readme_features
cargo run --example pipeline
cargo run --example validation_collect
```

## License

Apache-2.0
