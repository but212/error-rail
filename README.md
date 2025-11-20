# error-rail

Composable, metadata-friendly error utilities for Rust applications that need richer
context than `anyhow`-style strings but lighter ceremony than full-blown tracing.

## Notice

This library is an improved version that separates and refines the error module and Validated module from my Rustica library, which I created to study category theory and functional programming.

## Features

1. **Structured context** – attach messages, file/line locations, tags, and key/value
   metadata to any error.
2. **Composable collectors** – aggregate successes and failures through `Validation`
   and convert seamlessly between `Result`, `Validation`, and `ComposableError`.
3. **Ergonomic helpers** – macros (`context!`, `location!`, `tag!`, `metadata!`) and
   traits (`IntoErrorContext`, `WithError`) keep boilerplate low.

## Installation

```sh
cargo add error-rail
```

_No default features are required; the crate depends only on `serde` and `smallvec`._

## Quick start

```rust
use error_rail::{context, location, tag, with_context_result, ComposableError};

fn read_user_config() -> Result<String, std::io::Error> {
    std::fs::read_to_string("/etc/myapp/config.toml")
}

fn load_config() -> Result<String, Box<ComposableError<std::io::Error>>> {
    with_context_result(read_user_config(), context!("loading config"))
        .map_err(|err| err.with_context(location!()).with_context(tag!("config")))
}
```

## Structured context

`ComposableError<E>` wraps the original error `E`, a stack of `ErrorContext` values, and
an optional error code `C` (default `u32`). Context helpers:

| Helper                                           | Purpose                           |
| ------------------------------------------------ | --------------------------------- |
| `context!("format {}", arg)`                     | Lazily format human-readable text |
| `location!()`                                    | Capture `file!()` + `line!()`     |
| `tag!("auth")`                                   | Mark errors for later filtering   |
| `metadata!("user_id", user_id.to_string())`      | Store arbitrary key/value pairs   |

Use [`ErrorPipeline`](src/context/mod.rs) to chain operations and accumulate contexts
before finalizing into a `ComposableError`.

## Validation & conversion

`Validation<E, A>` collects every error instead of failing fast. Key APIs include:

- `Validation::invalid_many`, `iter_errors`, and iterator adapters (`Iter`, `ErrorsIter`).
- `FromIterator` impls for collecting iterators of `Result` or `Validation`.

Conversions live in [`convert`](src/convert/mod.rs):

- `validation_to_result`, `result_to_validation`
- `wrap_in_composable_result` / `_boxed`
- `flatten_composable_result`

## Macros & traits

- `context!`, `location!`, `tag!`, `metadata!`
- `IntoErrorContext` for plugging custom types into the context stack.
- `WithError` for transforming error types while preserving success values.

## Examples & doctests

Run the bundled examples directly from the crate root:

```sh
cargo run --example pipeline
cargo run --example validation_collect
```

This exercises the error pipeline and validation collectors end-to-end. To ensure
documentation stays accurate, execute all doctests whenever you touch public
APIs:

```sh
cargo test --doc
```

Consider wiring these commands into CI so regressions in docs or examples are
caught automatically.

## License

Apache-2.0
