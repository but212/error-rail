# error-rail

[![Crates.io](https://img.shields.io/crates/v/error-rail)](https://crates.io/crates/error-rail)
[![Docs](https://docs.rs/error-rail/badge.svg)](https://docs.rs/error-rail)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![Ask DeepWiki](https://deepwiki.com/badge.svg)](https://deepwiki.com/but212/error-rail)

**Composable, lazy-evaluated error handling for Rust.**

```rust
use error_rail::prelude::*;

fn load_config() -> BoxedResult<String, std::io::Error> {
    std::fs::read_to_string("config.toml")
        .ctx("loading configuration")
}
```

## Features

- **Lazy context** — Format strings only when errors occur
- **Chainable context** — Build rich error traces with `.ctx()`
- **Validation accumulation** — Collect all errors, not just the first
- **Transient error classification** — Built-in retry support
- **Error fingerprinting** — Deduplicate errors in monitoring systems
- **Async-first** — Full async/await support with Tower & Tracing integration
- **`no_std` compatible** — Works in embedded and web environments

## Quick Start

```sh
cargo add error-rail
```

```rust
use error_rail::prelude::*;

fn read_config() -> BoxedResult<String, std::io::Error> {
    std::fs::read_to_string("config.toml")
        .ctx("loading configuration")
}

fn process() -> BoxedResult<String, std::io::Error> {
    read_config().ctx("processing configuration")
}

fn main() {
    if let Err(e) = process() {
        eprintln!("{}", e.error_chain());
        // processing configuration -> loading configuration -> No such file or directory
    }
}
```

## Core Concepts

### Context Methods

```rust
// Static context (zero allocation)
result.ctx("database connection failed")

// Lazy formatted context (evaluated only on error)
result.ctx(context!("user {} not found", user_id))

// Structured context with tags & metadata
result.ctx(group!(
    tag("database"),
    metadata("query_time_ms", "150")
))
```

### Validation (Collect All Errors)

```rust
use error_rail::Validation;

let results: Validation<&str, Vec<_>> = vec![
    validate_age(-5),
    validate_name(""),
].into_iter().collect();

// Both errors collected, not just the first
```

### Async Support

```rust
use error_rail::prelude_async::*;

async fn fetch_user(id: u64) -> BoxedResult<User, DbError> {
    database.get_user(id)
        .ctx("fetching user")
        .await
        .map_err(Box::new)
}
```

## Feature Flags

```toml
[dependencies]
error-rail = "0.8"                                    # Core (no_std)
error-rail = { version = "0.8", features = ["std"] }  # + backtraces
error-rail = { version = "0.8", features = ["serde"] } # + serde support
error-rail = { version = "0.8", features = ["async"] } # + async support
error-rail = { version = "0.8", features = ["tokio"] } # + retry, timeout
error-rail = { version = "0.8", features = ["tower"] } # + Tower middleware
error-rail = { version = "0.8", features = ["full"] }  # Everything
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
cargo run --example quick_start
cargo run --example async_api_patterns --features tokio
cargo run --example async_tower_integration --features tower
```

## License

Apache-2.0. See [LICENSE](LICENSE) and [NOTICE](NOTICE).
