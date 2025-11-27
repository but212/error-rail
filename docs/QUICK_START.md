# Quick Start Guide

Welcome to `error-rail`! This guide walks you through composable error handling step by step.

> **Run the Examples**: All code in this guide is available as a runnable example:
>
> ```sh
> cargo run --example quick_start
> ```

## Concept Map

Before diving in, here's a mental model of `error-rail`:

```text
┌─────────────────────────────────────────────────────────────────┐
│                     error-rail at a Glance                      │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Level 1: Basic Usage (95% of cases)                            │
│  ─────────────────────────────────────                          │
│  ErrorPipeline::new(result)                                     │
│      .with_context(context!("what happened"))                   │
│      .finish_boxed()                                            │
│                                                                 │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Level 2: Validation (form/input validation)                    │
│  ───────────────────────────────────────────                    │
│  let results: Validation<E, Vec<T>> =                           │
│      inputs.into_iter().map(validate).collect();                │
│                                                                 │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Level 3: Advanced (library authors)                            │
│  ────────────────────────────────────                           │
│  impl IntoErrorContext for MyType { ... }                       │
│  Custom ErrorCategory implementations                           │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

**Start at Level 1** — you can build complete applications without ever touching Level 2 or 3.

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
| `group!` | Combined grouped context (lazy) | `group!(tag("db"), location(file!(), line!()), metadata("host", "localhost"))` |
| `rail!` | Quick pipeline wrapper | `rail!(fs::read("file"))` |

```rust
use error_rail::{ComposableError, context, group};

let err = ComposableError::<&str>::new("timeout")
    .with_context(context!("request failed after {} retries", 3))
    .with_context(group!(
        tag("http"),
        location(file!(), line!()),
        metadata("endpoint", "/api/users")
    ));

println!("{}", err.error_chain());
// Output: request failed after 3 retries -> [http] at src/main.rs:5 (endpoint=/api/users) -> timeout
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

### Quick Decision Guide

| Question | Answer |
|----------|--------|
| **Which type for function returns?** | `BoxedComposableResult<T, E>` (8 bytes stack) |
| **Which type for internal handling?** | `ComposableResult<T, E>` (zero-copy) |
| **Need to collect all errors?** | Use `Validation<E, T>` |
| **Adding context to existing Result?** | Use `ErrorPipeline` |

### Performance Tips

1. **Use `context!` macro** — It's lazy and only formats on error
2. **Box at boundaries** — Use `finish_boxed()` for function returns
3. **Limit context depth** — Add context at module boundaries, not every function

### Advanced Topics

- **`IntoErrorContext` trait** - Convert custom types to error context
- **`ErrorOps` trait** - Recovery and mapping operations
- **`WithError` trait** - Transform error types while preserving success values
- **`backtrace!` macro** - Capture stack traces (requires `std` feature)

### Migrating from anyhow

| anyhow | error-rail |
|--------|------------|
| `anyhow::Result<T>` | `BoxedComposableResult<T, E>` |
| `.context("msg")` | `.with_context(context!("msg"))` |
| `anyhow!("error")` | `ComposableError::new("error")` |
| `bail!("error")` | `return Err(Box::new(ComposableError::new("error")))` |
