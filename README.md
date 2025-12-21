# error-rail

[![Crates.io](https://img.shields.io/crates/v/error-rail)](https://crates.io/crates/error-rail)
[![Docs](https://docs.rs/error-rail/badge.svg)](https://docs.rs/error-rail)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![Ask DeepWiki](https://deepwiki.com/badge.svg)](https://deepwiki.com/but212/error-rail)

**Composable, lazy-evaluated error handling for Rust.**

> **std::error defines error types. error-rail defines how errors flow.**

```rust
use error_rail::prelude::*;

fn load_config() -> BoxedResult<String, std::io::Error> {
    std::fs::read_to_string("config.toml")
        .ctx("loading configuration")
}
```

## Features

- **Lazy formatting** — Use `context!` / `.ctx_with(...)` to format strings only when errors occur
- **Chainable context** — Stack multiple contexts with `ErrorPipeline::with_context()`
- **Validation accumulation** — Collect all errors, not just the first
- **Transient error classification** — Built-in retry support
- **Error fingerprinting** — Deduplicate errors in monitoring systems
- **Async-first** — Full async/await support with Tower & Tracing integration
- **`no_std` compatible** — Works in embedded and web environments

## Quick Start

```sh
cargo add error-rail
```

### For Beginners — `simple`

```rust
use error_rail::simple::*;

fn read_config() -> BoxedResult<String, std::io::Error> {
    std::fs::read_to_string("config.toml")
        .ctx("loading configuration")
}

fn main() {
    if let Err(e) = read_config() {
        eprintln!("{}", e.error_chain());
        // loading configuration -> No such file or directory (os error 2)
    }
}
```

### For General Use — `prelude`

```rust
use error_rail::prelude::*;

fn process() -> BoxedResult<String, std::io::Error> {
    let config = std::fs::read_to_string("config.toml")
        .ctx("loading configuration")?;
    Ok(config)
}
```

## API Levels

Most projects only need `simple` or `prelude`.

| Module | When to Use | What's Included |
|--------|------------|-----------------|
| `simple` | Getting started | `BoxedResult`, `rail!`, `.ctx()`, `.error_chain()` |
| `prelude` | When structured context is needed | + `context!`, `group!`, `ErrorPipeline` |
| `intermediate` | Service development | + `TransientError`, `Fingerprint` |
| `advanced` | Library development | + internal builders, `ErrorVec` |
| `prelude_async` | Async code | + `AsyncErrorPipeline`, retry, timeout |

---

## Context Chaining (Core Concepts)

### 1. ErrorPipeline — Recommended for multiple contexts

```rust
use error_rail::prelude::*;

fn fetch_user(id: u64) -> BoxedResult<String, &'static str> {
    ErrorPipeline::new(Err("db error"))
        .with_context("querying users table")
        .with_context(group!(tag("db"), metadata("user_id", "42")))
        .with_context(context!("fetching user {}", id))
        .finish_boxed()
}
// Error chain: fetching user 42 -> [db] (user_id=42) -> querying users table -> db error
```

### 2. ResultExt — Single context

```rust
use error_rail::prelude::*;

// .ctx() returns BoxedResult<T, E>
fn inner() -> BoxedResult<i32, &'static str> {
    Err("db error").ctx("querying database")
}

// Chain with .ctx_boxed() on BoxedResult
fn outer() -> BoxedResult<i32, &'static str> {
    inner().ctx_boxed("in user service")
}
// Error chain: in user service -> querying database -> db error
```

### 3. Async — Direct chaining

```rust
use error_rail::prelude_async::*;

async fn fetch_user(id: u64) -> BoxedResult<String, ApiError> {
    database.get_user(id)
        .ctx("fetching user")           // FutureResultExt::ctx()
        .ctx("in user service")         // Direct chaining possible!
        .await
        .map_err(Box::new)
}
```

---

## Lazy Context (Performance Optimization)

```rust
use error_rail::prelude::*;

let user_id = 42;

// ✅ context! — lazy evaluation (only on error)
result.ctx(context!("user {} not found", user_id))

// ✅ .ctx_with() — closure for complex logic
result.ctx_with(|| format!("user {} not found", user_id))

// ❌ format!() — always evaluated (even on success)
result.ctx(format!("user {} not found", user_id))
```

## Structured Context

```rust
use error_rail::prelude::*;

// Tags & metadata
result.ctx(group!(
    tag("database"),
    metadata("query_time_ms", "150"),
    location(file!(), line!())
))
```

## Validation (Error Collection)

> Use the `error_rail::validation` module

```rust
use error_rail::validation::Validation;

fn validate_age(age: i32) -> Validation<&'static str, i32> {
    if age >= 0 && age <= 150 {
        Validation::Valid(age)
    } else {
        Validation::invalid("age must be between 0 and 150")
    }
}

let results: Validation<&str, Vec<_>> = vec![
    validate_age(-5),
    validate_name(""),
].into_iter().collect();

// All errors are collected (not just the first one)
```

## Transient Errors & Retry

```rust
use error_rail::traits::TransientError;
use std::time::Duration;

#[derive(Debug)]
enum ApiError {
    Timeout,
    RateLimited(u64),
    NotFound,
}

impl TransientError for ApiError {
    fn is_transient(&self) -> bool {
        matches!(self, ApiError::Timeout | ApiError::RateLimited(_))
    }

    fn retry_after_hint(&self) -> Option<Duration> {
        match self {
            ApiError::RateLimited(secs) => Some(Duration::from_secs(*secs)),
            _ => None,
        }
    }
}
```

## Error Fingerprinting

```rust
use error_rail::prelude::*;

let err = ComposableError::new("database timeout")
    .with_context(ErrorContext::tag("db"))
    .set_code(504);

// For Sentry grouping, log deduplication
println!("Fingerprint: {}", err.fingerprint_hex());
```

---

## Anti-Patterns

```rust
use error_rail::simple::*;

// ❌ DON'T: Excessive context at every step
fn bad() -> BoxedResult<(), &'static str> {
    let a = step_a().ctx("step a")?;
    let b = step_b(a).ctx("step b")?;  // Noise, not value
    step_c(b).ctx("step c")
}

// ✅ DO: One .ctx() per I/O boundary
fn good() -> BoxedResult<String, std::io::Error> {
    std::fs::read_to_string("file.txt").ctx("reading input")
}
```

## When NOT to Use error-rail

- Simple scripts that just print errors and exit
- When `anyhow` or `eyre` already meets your needs
- Teams with little Rust experience

---

## Feature Flags

```toml
[dependencies]
error-rail = "0.10"                                    # Core (no_std)
error-rail = { version = "0.10", features = ["std"] }  # + backtraces
error-rail = { version = "0.10", features = ["serde"] } # + serde support
error-rail = { version = "0.10", features = ["async"] } # + async support
error-rail = { version = "0.10", features = ["tokio"] } # + retry, timeout
error-rail = { version = "0.10", features = ["tower"] } # + Tower middleware
error-rail = { version = "0.10", features = ["full"] }  # Everything
```

## Documentation

| Resource | Description |
|----------|-------------|
| [Quick Start](docs/QUICK_START.md) | Step-by-step tutorial |
| [Async Guide](docs/QUICK_START_ASYNC.md) | Async patterns |
| [Patterns](docs/PATTERNS.md) | Real-world examples |
| [Benchmarks](docs/BENCHMARKS.md) | Performance analysis |
| [API Docs](https://docs.rs/error-rail) | Full API reference |

## Examples

```sh
cargo run --example readme_features           # Validate README examples
cargo run --example quick_start
cargo run --example async_api_patterns --features tokio
cargo run --example async_tower_integration --features tower
```

---

## Contributing

Issues and PRs are welcome!

### Development

```bash
# Run tests
cargo test

# Validate README examples
cargo run --example readme_features

# Lint check
cargo clippy --all-features

# Doc tests
cargo test --doc
```

### Guidelines

- **Bug reports**: Submit to GitHub Issues with a reproducible example
- **Feature requests**: Discuss via issues first, then PR
- **Pull Requests**: Tests required, must pass `cargo clippy`

---

## License

Apache-2.0. See [LICENSE](LICENSE) and [NOTICE](NOTICE).
