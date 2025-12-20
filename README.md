# error-rail

[![Crates.io](https://img.shields.io/crates/v/error-rail)](https://crates.io/crates/error-rail)
[![Docs](https://docs.rs/error-rail/badge.svg)](https://docs.rs/error-rail)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![Ask DeepWiki](https://deepwiki.com/badge.svg)](https://deepwiki.com/but212/error-rail)

**Composable, lazy-evaluated error handling for Rust.**

> **std::error defines error types. error-rail defines how errors flow.**

```rust
use error_rail::simple::*;

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

**For beginners** — Start with `simple`:

```rust
use error_rail::simple::*;

fn read_config() -> BoxedResult<String, std::io::Error> {
    std::fs::read_to_string("config.toml")
        .ctx("loading configuration")
}

fn main() {
    if let Err(e) = read_config() {
        eprintln!("{}", e.error_chain());
        // loading configuration -> No such file or directory
    }
}
```

**For general use** — Use `prelude`:

```rust
use error_rail::prelude::*;

fn process() -> BoxedResult<String, std::io::Error> {
    let config = std::fs::read_to_string("config.toml")
        .ctx("loading configuration")?;
    Ok(config)
}
```

## API Levels (You do NOT need to learn all of these)

Start here → `simple`  
When you need more → `prelude`  
Only if you are building services → `intermediate`  
Almost never → `advanced`

| Module | When to Use | What's Included |
|--------|------------|----------------|
| `simple` | First time using error-rail | `BoxedResult`, `rail!`, `.ctx()`, `.error_chain()` |
| `prelude` | When you need structured context | + `context!`, `group!`, `ErrorPipeline` |
| `intermediate` | Building services | + `TransientError`, `Fingerprint`, formatting |
| `advanced` | Writing libraries | + internal builders, `ErrorVec` |
| `prelude_async` | Async code | + `AsyncErrorPipeline`, retry, timeout |

## Core Concepts (Advanced — skip on first read)

> **If you are using `simple`, you can skip this entire section.**

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

> **Note: This is available in `error_rail::validation`, not in `simple`.**

```rust
use error_rail::validation::Validation;

let results: Validation<&str, Vec<_>> = vec![
    validate_age(-5),
    validate_name(""),
].into_iter().collect();

// Both errors collected, not just the first
```

### Macros

```rust
use error_rail::prelude::*;

// rail! - Wrap any Result in ErrorPipeline and box it
let result = rail!(std::fs::read_to_string("config.toml"));

// context! - Lazy formatted context (only evaluated on error)
result.ctx(context!("loading config for user {}", user_id))

// group! - Structured context with tags & metadata
result.ctx(group!(tag("config"), metadata("path", "config.toml")))
```

```rust
use error_rail::prelude_async::*;

// rail_async! - Async version of rail!
async fn load() -> BoxedResult<String, IoError> {
    rail_async!(tokio::fs::read_to_string("config.toml"))
        .with_context("loading config")
        .finish_boxed()
        .await
}
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

## ❌ Anti-Patterns

```rust
// ❌ DON'T: Chain .ctx() multiple times
fn bad() -> BoxedResult<T, E> {
    foo().ctx("a").ctx("b").ctx("c")  // Noise, not value
}

// ✅ DO: One .ctx() per I/O boundary
fn good() -> BoxedResult<T, E> {
    let data = read_file().ctx("reading input")?;
    let parsed = parse(data).ctx("parsing input")?;
    Ok(parsed)
}
```

### Avoid Glob Imports in Large Projects

For better IDE support and compile times in large codebases:

```rust
// ❌ Glob import (okay for small projects)
use error_rail::prelude::*;

// ✅ Explicit imports (recommended for large projects)
use error_rail::prelude::{BoxedResult, ResultExt, rail};
```

## When should I move from `simple` to `prelude`?

Move to `prelude` when you need:

- **Structured context** - tags and metadata for better error categorization
- **Lazy formatted messages** - format strings only when errors occur
- **ErrorPipeline** - for building libraries or complex error chains
- **Writing a library** - not just an application

> You can stay with `simple` for a long time! It's designed to be sufficient for most applications.

## When NOT to Use error-rail

- Simple scripts that just print errors and exit
- Teams with little Rust experience
- When `anyhow` or `eyre` already meets your needs

## Feature Flags

```toml
[dependencies]
error-rail = "0.9"                                    # Core (no_std)
error-rail = { version = "0.9", features = ["std"] }  # + backtraces
error-rail = { version = "0.9", features = ["serde"] } # + serde support
error-rail = { version = "0.9", features = ["async"] } # + async support
error-rail = { version = "0.9", features = ["tokio"] } # + retry, timeout
error-rail = { version = "0.9", features = ["tower"] } # + Tower middleware
error-rail = { version = "0.9", features = ["full"] }  # Everything
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
