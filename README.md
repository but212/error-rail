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

### 3. Ergonomic Traits

- **`IntoErrorContext`**: Convert your own types into error context.
- **`WithError`**: Transform error types while preserving success values.
- **`ErrorOps`**: Unified operations for recovery and mapping.

## Module Overview

- **`context`**: Core context types and pipeline builders.
- **`convert`**: Helpers to switch between `Result`, `Validation`, and `ComposableError`.
- **`macros`**: `context!`, `location!`, `tag!`, `metadata!`, `rail` for easy context creation.
- **`traits`**: Foundational traits (`ErrorCategory`, `ErrorOps`, etc.).
- **`types`**: `ComposableError` and related type aliases.
- **`validation`**: The `Validation` enum and accumulators.

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
