# CHANGELOG

## [0.8.0]

### Added - 0.8.0

- **Runtime-agnostic async support** (`async` feature)
  - New `async_ext` module with core async types
  - `FutureResultExt` trait providing `.ctx()` and `.with_ctx()` methods on `Future<Output = Result<T, E>>`
  - `ContextFuture` wrapper for lazy error context evaluation (cancel-safe)
  - `AsyncErrorPipeline` for chainable async error handling operations
  - `prelude_async` module re-exporting all prelude items plus async-specific functionality
  - `rail_async!` macro for wrapping async operations into `AsyncErrorPipeline`
  - `ctx_async!` macro for attaching formatted context to futures ergonomically
  - Added `docs/QUICK_START_ASYNC.md` with async usage patterns and best practices

- **Async validation support** (`async` feature)
  - `validate_all_async<I>` function for running multiple async validations sequentially and accumulating all errors
  - `validate_seq_async` function for sequential validation where each step depends on previous result
  - Runtime-neutral design mirroring synchronous `Validation` semantics

- **Async retry support** (`async` feature)
  - `RetryPolicy` trait for pluggable retry strategies
  - `ExponentialBackoff` policy with builder pattern (`with_initial_delay`, `with_max_delay`, `with_max_attempts`, `with_multiplier`)
  - `FixedDelay` policy for constant-delay retries
  - `retry_with_policy` function: runtime-agnostic retry loop accepting custom `sleep_fn`
  - `retry_with_metadata` function returning `RetryResult<T, E>` with `result`, `attempts`, and `total_wait_time`
  - Structured context on failure (e.g., "exhausted N retry attempts", "permanent error, no retry")

- **Tokio integration** (`ecosystem` feature)
  - `retry_transient` convenience function using `tokio::time::sleep`
  - `retry_transient_n` for simple retry with attempt count
  - `try_with_timeout` wrapper returning `TimeoutResult<T, E>`
  - `TimeoutResult` enum with `Ok`, `Err`, and `Timeout` variants
  - `TimeoutError` type for timeout representation

- **Tower integration** (`tower` feature)
  - `ErrorRailLayer`: Tower `Layer` wrapping service errors in `ComposableError` with context
  - `ErrorRailService`: Tower `Service` wrapper for error context attachment
  - `ErrorRailFuture`: Future wrapper for lazy context evaluation
  - `ServiceErrorExt` trait providing `.with_error_context()` extension for any `Service`

- **Tracing integration** (`tracing` feature)
  - `FutureSpanExt` trait providing `.with_span_context()` and `.with_span()` for futures
  - `ResultSpanExt` trait providing `.with_current_span()` and `.with_span()` for Results
  - `SpanContextFuture`: Future wrapper capturing span context on error
  - `instrument_error` function to add current span context to any error

- **Comprehensive test coverage** for async functionality
  - `tests/async_ext/future_ext_tests.rs`: lazy evaluation and context chaining
  - `tests/async_ext/pipeline_tests.rs`: AsyncErrorPipeline operations
  - `tests/async_ext/validation_tests.rs`: all-valid, some-invalid, and sequential validation scenarios
  - `tests/async_ext/retry_tests.rs`: transient/permanent errors and exhausted attempts
  - `tests/async_ext/tokio_tests.rs`: Tokio-specific retry and timeout functionality
  - `tests/async_ext/tracing_tests.rs`: span context attachment

- **New examples**
  - `examples/async_api_patterns.rs`: demonstrates 7 common async error handling patterns
  - `examples/async_tower_integration.rs`: demonstrates Tower layer and service usage

## [0.7.1]

### Changed - 0.7.1

- Removed duplicate documentation in `src/prelude.rs`
- Cleaned up `clippy.toml` by removing unused settings and reorganizing by category
- Refactored `ComposableError::compute_fingerprint()` to eliminate code duplication by delegating to `FingerprintConfig::compute()`
- Enhanced `ComposableError::context()` documentation to warn about allocations and recommend `context_iter()` for zero-allocation use cases
- Identified and documented unused `src/types/pipeline_ops.rs` file (contains only deprecation notice, not referenced in module system)

### Fixed - 0.7.1

- Improved performance by inlining frequently called factory methods (`ErrorFormatConfig::pretty()`, `compact()`, `no_code()`, and `ErrorContextBuilder::new()`)

## [0.7.0]

### Added - 0.7.0

- **API Layering**: Introduced 3-level API hierarchy for different user expertise levels
  - **Beginner API** (`prelude`): Minimal exports for quick starts - `ComposableError`, `ErrorContext`, `ErrorPipeline`, `rail!`, `context!`, `group!` macros
  - **Intermediate API** (`intermediate`): Advanced patterns - `TransientError`, `ErrorFormatter`, fingerprinting
  - **Advanced API** (`advanced`): Low-level internals for library authors - `ErrorVec`, `ErrorContextBuilder`, `LazyContext`, internal types

- **ErrorPipeline Enhancements**: New builder methods for improved ergonomics
  - `.context()` - Alias for `with_context()` for more fluent API
  - `.step()` - Generic chaining function for operations
  - `.error_ops()` - Groups error handling strategies (recovery, retry, fingerprinting)
  - `.retry()` - Integrates `RetryOps` for configuring retry policies
  - **RetryOps Helper**: Added `to_error_pipeline()` method for converting RetryOps back to ErrorPipeline

- **Validation Normalization**: Simplified validation workflow
  - `validation::prelude` - Common validation exports in one place
  - `validate!` macro - DSL for combining multiple validations, automatically accumulates errors

- **RetryOps Integration**: Structured retry support
  - `RetryOps` struct encapsulates retry-related operations
  - Methods: `is_transient()`, `max_retries()`, `after_hint()`
  - Integrated into `ErrorPipeline` via `.retry()` method

- **ErrorFormatter Integration**: Unified formatting configuration
  - `ErrorFormatBuilder` - Builder API for customizing error display
  - Methods: `with_separator()`, `reverse_context()`, `show_code()`, `pretty()`, `compact()`
  - Integrated into `ComposableError` via `.fmt()` and `.format_with()`
  - Example: `err.fmt().pretty().show_code(false).to_string()`

- **Internal Abstraction**: Common abstractions for consistency
  - `Accumulator<T>` - Unified accumulation logic for errors and contexts
  - Replaces direct `ErrorVec` usage in both `ErrorPipeline` and `Validation`
  - Implements `PartialOrd`, `Ord`, `Hash`, `FromIterator` for full compatibility

- **Benchmark Suite Enhancements**: Comprehensive performance testing improvements
  - **Expanded Coverage**: 30+ benchmarks (67% increase from 18 benchmarks)
  - **Optimized Configuration**: Custom Criterion settings (100 samples, 3s warm-up, 5s measurement, 5% noise threshold)
  - **9 Organized Groups**: Core operations, retry operations, error conversions, context operations, scaling tests, pipeline operations, validation operations, real-world scenarios, memory & allocation
  - **Parameterized Scaling Tests**: Context depth (1-50 layers), validation batch size (10-5000 items), pipeline chain length (2-20 operations)
  - **Retry Operations Coverage**: Transient error handling, recovery patterns, classification benchmarks
  - **Error Conversion Patterns**: Type transformations (map_core, std::io → domain, serde → domain, conversion chains)
  - **Real-World Scenarios**: HTTP request simulation, database transaction rollback, microservice error propagation with mixed success/error ratios
  - **Memory Analysis**: Large metadata contexts, String vs static str allocation patterns
  - **Throughput Measurements**: Elements per second metrics for batch operations

### Changed - 0.7.0

- **Module Consolidation**: Simplified trait organization
  - Consolidated 6 separate trait files into single `traits/mod.rs`
  - Removed: `error_category.rs`, `error_ops.rs`, `into_error_context.rs`, `result_ext.rs`, `transient.rs`, `with_error.rs`
  - Easier navigation and clearer import paths

- **Internal Improvements**: Enhanced type safety and consistency
  - `ErrorPipeline` now uses `Accumulator<ErrorContext>` for context storage
  - `Validation` now uses `Accumulator<E>` for error storage
  - All internal accumulation logic unified through common abstraction

### Breaking Changes - 0.7.0

- **Removed deprecated macros**: `location!`, `tag!`, `metadata!`
  - **Migration**: Use the `group!` macro instead
  - **Before**: `err.with_context(location!()).with_context(tag!("db"))`
  - **After**: `err.with_context(group!(location(file!(), line!()), tag("db")))`
  - These macros were deprecated in 0.5.0 and have now been removed

- **Removed deprecated methods**:
  - `WithError::to_result()` - Use `to_result_first()` or `to_result_all()` instead
  - `Validation::to_result()` - Use the `Validation::to_result()` method which returns `Result<A, ErrorVec<E>>`
  - `result::to_result()` - Use trait methods directly
  - These methods were deprecated in 0.6.0 and have now been removed

### Removed - 0.7.0

- Deprecated macros: `location!`, `tag!`, `metadata!` (use `group!` macro)
- Deprecated methods: `WithError::to_result()`, old `Validation::to_result()`, `result::to_result()`

## [0.6.0]

### Deprecated - 0.6.0

- **`WithError::to_result()` method deprecated**: Will be removed in future versions
  - **Migration**: Use `to_result_first()` for first-error-only or `to_result_all()` for all errors
  - **Rationale**: Provides explicit error handling choices to prevent accidental error information loss

### Added - 0.6.0

- **Transient Error Classification** (`TransientError` trait): New trait for classifying errors as transient or permanent
  - `is_transient()` / `is_permanent()` - Classify error recoverability
  - `retry_after_hint()` - Optional duration hint for retry backoff
  - `max_retries_hint()` - Optional maximum retry count
  - `TransientErrorExt` extension trait with `retry_if_transient()` for Result types
  - Blanket implementation for `std::io::Error` (with `std` feature)
  - Enables integration with external retry libraries (backoff, retry, again, tokio-retry)

- **ErrorPipeline Retry Utilities**: New methods for retry pattern integration
  - `is_transient()` - Check if current error is transient (borrows)
  - `recover_transient()` - Attempt recovery only for transient errors
  - `should_retry()` - Returns `Option<Self>` for retry loop control (consumes)
  - `retry_after_hint()` - Get retry delay hint from error
  - `with_retry_context(attempt)` - Add retry attempt metadata to error context

- **Error Fingerprinting**: Automatic fingerprint generation for error deduplication
  - `ComposableError::fingerprint()` - Generate u64 fingerprint from tags + code + message
  - `ComposableError::fingerprint_hex()` - Get fingerprint as 16-character hex string
  - `ComposableError::fingerprint_config()` - Customizable fingerprint builder
  - `FingerprintConfig` - Configure which components to include (tags, code, message, metadata)
  - Useful for Sentry issue grouping, log deduplication, and alert throttling

- **Integration Example**: New `examples/retry_integration.rs` demonstrating:
  - Implementing `TransientError` for domain errors
  - Manual retry loops with `ErrorPipeline`
  - Using `recover_transient()` for single retry attempts
  - Integration patterns with external retry libraries
  - Error fingerprinting for deduplication

- **`rail_unboxed!` macro**: New macro for unboxed composable results
  - Returns `ComposableResult<T, E>` instead of `BoxedComposableResult<T, E>`
  - Use when you need to avoid heap allocation or for internal/performance-critical code
  - Complements existing `rail!` macro which always returns boxed results

- **`WithError::to_result_first()` method**: Explicit method for first-error-only conversion
  - Returns `Result<T, E>` with only the first error from multi-error scenarios
  - Clearer intent than the deprecated `to_result()` method

- **ErrorFormatter System**: New flexible error chain formatting
  - Added `ErrorFormatter` trait for custom error chain formatting implementations
  - Added `ErrorFormatConfig` with built-in formatting configurations (pretty, compact, etc.)
  - Added `ComposableError::error_chain_with()` method for custom formatting
  - Supports both simple configuration and advanced custom formatters
  - Enables JSON, uppercase, prefixed, and other custom output formats

- **Error Handling Pattern Examples**: New runnable demonstration examples
  - Added `examples/pattern_cli_app.rs` - CLI error handling with user-friendly messages
  - Added `examples/pattern_http_api.rs` - HTTP API error responses with status codes
  - Added `examples/pattern_library_dev.rs` - Library development with public API boundaries
  - Added `examples/pattern_service_layer.rs` - Service layer error context addition
  - All examples use proper ErrorPipeline pattern and compile without warnings
  - Include comprehensive test coverage for library development pattern

- **Error Handling Patterns Documentation**: New comprehensive patterns guide
  - Added `docs/PATTERNS.md` with real-world usage patterns and best practices
  - Includes 4 practical patterns: Service Layer, HTTP API, CLI Applications, Library Development
  - Features working examples using ErrorPipeline for proper error composition
  - Demonstrates correct ErrorPipeline approach instead of problematic .ctx() chaining
  - Updated README.md to reference the new patterns guide
  - Maintains backward compatibility with existing behavior

- **`WithError::to_result_all()` method**: New method for preserving all errors
  - Returns `Result<T, ErrorVec<E>>` with all accumulated errors
  - Prevents error information loss in multi-error scenarios
  - Uses `ErrorVec<E>` (SmallVec) for no_std compatibility and performance

## [0.5.1]

### Fixed - 0.5.1

- Fixed inconsistencies in README documentation

## [0.5.0]

### Breaking Changes - 0.5.0

- **`GroupContext::message()` now combines all available fields**:
  - **Before**: Displayed only the first available field in priority order (message → location → tags → metadata)
  - **After**: Combines all fields into one cohesive unit with format `[tag1, tag2] at file:line: message (key1=value1, key2=value2)`
  - **Rationale**: Improves readability by presenting each context as a unified information unit rather than fragmented parts separated by "->" arrows
  - **Migration**: Code parsing `ErrorContext::message()` output needs to be updated to handle the new combined format. Single-field contexts remain unchanged.
  - **Recommended**: Use `ErrorContext::builder()` to create grouped contexts that display as cohesive units in error chains:

    ```rust
    // Before: Multiple separate contexts
    err.with_context(tag!("database"))
       .with_context(location!())
       .with_context(metadata!("host", "localhost"))
    // Output: -> [database] -> at main.rs:42 -> (host=localhost)
    
    // After: Single grouped context  
    err.with_context(ErrorContext::builder()
        .tag("database")
        .location(file!(), line!())
        .metadata("host", "localhost")
        .build())
    // Output: -> [database] at main.rs:42 (host=localhost)
    ```

### Added - 0.5.0

- **`ResultExt` trait**: New ergonomic extension trait for adding context to `Result` types
  - `.ctx(msg)` - Add static context message
  - `.ctx_with(|| format!(...))` - Add lazily-evaluated context (2.1x faster on success)
- **`BoxedResultExt` trait**: Chain contexts on already-boxed `ComposableError` results
  - `.ctx_boxed(msg)` - Add context to boxed error
  - `.ctx_boxed_with(|| ...)` - Add lazy context to boxed error
- **`prelude` module**: Convenient re-exports for quick starts
  - Import everything with `use error_rail::prelude::*;`
  - Includes `BoxedResult<T, E>` type alias for ergonomic return types

### Changed - 0.5.0

- **Removed redundant type aliases**: `SimpleComposableError<E>` and `TaggedComposableError<E>` have been completely removed
  - **Rationale**: These aliases provided no additional functionality over `ComposableError<E>` and created unnecessary complexity
  - **Migration**: Use `ComposableError<E>` directly - these were simple type aliases with no behavior change
  - **Impact**: Code using these aliases will need to be updated to use `ComposableError<E>` instead
- **Enhanced error messages**: Added `#[diagnostic::on_unimplemented]` to `IntoErrorContext` trait for better compiler guidance when trait bounds are not satisfied
- **Improved DX**: Added helpful implementation examples and links to documentation in trait documentation
- **Deprecated individual context macros**: `location!()`, `tag!()`, and `metadata!()` macros are now deprecated in favor of the new `group!` macro
  - **Rationale**: The new `group!` macro provides lazy evaluation for grouped contexts, combining multiple fields into a single cohesive unit while maintaining performance benefits
  - **Migration**: Replace individual macro calls with function-call style `group!()`:

    ```rust
    // Before: Multiple separate contexts (eager allocation)
    err.with_context(location!())
       .with_context(tag!("database"))
       .with_context(metadata!("host", "localhost"));
    // Output: -> at main.rs:42 -> [database] -> (host=localhost)
    
    // After: Single grouped context (lazy evaluation)
    err.with_context(group!(
        location(file!(), line!()),
        tag("database"),
        metadata("host", "localhost")
    ));
    // Output: -> [database] at main.rs:42 (host=localhost)
    ```

  - **Benefits**:
    - Lazy evaluation: No string formatting until error occurs
    - Unified display: All fields appear as one cohesive context unit
    - Better performance: Reduced allocations on success paths
  - **Removal timeline**: Deprecated macros will be removed in a future version
  - **New exports**: `group!` macro and `LazyGroupContext` type added to prelude

### Deprecated - 0.5.0

- `location!()`, `tag!()`, and `metadata!()` macros - Use `group!` macro instead (scheduled for removal in a future version)

### Removed - 0.5.0

- `SimpleComposableError<E>` - Use `ComposableError<E>` directly (completely removed)
- `TaggedComposableError<E>` - Use `ComposableError<E>` with `ErrorContext::tag()` instead (completely removed)

## [0.4.0]

### Breaking Changes - 0.4.0

- **Library is `no_std`-compatible by default**: The crate builds without `std` when the `std` feature is disabled, and uses standard library types when the `std` feature is explicitly enabled. The default configuration (`default = []`) provides `no_std` compatibility.
  - **Migration**: Code that relies on `std`-specific functionality should continue to work when the `std` feature is enabled. In `no_std` mode, the library uses `alloc` types for heap allocations.

- **`core::error::Error` implementation for `ComposableError<E>` now requires `Send + Sync` bounds**:
  - **Before**: `impl<E: std::error::Error + 'static> std::error::Error for ComposableError<E>`
  - **After**: `impl<E: core::error::Error + Send + Sync + 'static> core::error::Error for ComposableError<E>`
  - **Rationale**: This change enables better interoperability with error handling libraries like `anyhow` and `eyre` in concurrent contexts, ensuring `ComposableError` can be safely shared across thread boundaries.
  - **Migration**: Error types wrapped in `ComposableError` must now implement `Send + Sync`. Most standard error types already satisfy these bounds. For custom error types that are not thread-safe, either:
    - Add `Send + Sync` implementations if the type can be made thread-safe
    - Use `ComposableError` without relying on the `Error` trait implementation
    - Wrap non-thread-safe errors in a thread-safe container (e.g., `Arc<Mutex<_>>`)
  
- **`ComposableError::context()` method signature changed**:
  - **Before**: `pub fn context(&self) -> Vec<ErrorContext>`
  - **After**: `pub fn context(&self) -> ErrorVec<ErrorContext>`

- **`extract_context` now returns owned `ErrorVec`**: Changed from `Vec<ErrorContext>` to `ErrorVec<ErrorContext>` (already noted above, now consistent with `context()` method).

- **`split_validation_errors` returns lazy iterator**: Now returns `SplitValidationIter` instead of `Vec<Result<T, E>>`. This allows for zero-allocation iteration over validation results.
  - **Migration**: Code that previously collected results into a `Vec` should use `.collect()` on the returned iterator.

### Changed - 0.4.0

- Replaced `std` usage with `core` and `alloc` across `context`, `convert`, `types`, and `validation` modules for `no_std` compatibility.
- **Introduced `types::alloc_type` module with conditional type aliases**: Added unified `Box`, `String`, `Cow`, and `Vec` type aliases that automatically use `std` types when the `std` feature is enabled or `alloc` types in `no_std` mode, eliminating direct `alloc::` prefixes throughout the codebase.
- **Enhanced backtrace macros**: Added `backtrace_force!()` macro that always captures stack traces regardless of `RUST_BACKTRACE`/`RUST_LIB_BACKTRACE` environment variables, while the existing `backtrace!()` macro continues to respect environment settings for production use. Improved test coverage to validate actual stack frame capture rather than just non-empty strings.
- **Improved serde no_std support**: Configured serde with `default-features = false` and `features = ["derive", "alloc"]` for proper `no_std` compatibility while maintaining full `std` support when the `std` feature is enabled.
- `collect_errors` and `Validation::from_iter` now use `ErrorVec` / `SmallVec` internally to reduce heap allocations.
- `split_validation_errors` is now lazy, avoiding immediate vector allocation.
- `std::error::Error` -> `core::error::Error`.
- Restructured Cargo features:
  - `default = []` (no features enabled by default).
  - `serde` feature now enables optional serde support and forwards to `smallvec/serde`.
  - `full` feature acts as a convenience bundle that enables `serde` and `std`.
- `alloc` crate is now unconditionally linked to ensure consistent type usage (`alloc::string::String`, etc.) across `std` and `no_std` builds.

## [0.3.1]

### Added - 0.3.1

- **Validation**: Added `zip` method to combine two `Validation` instances into a tuple

### Fixed - 0.3.1

- `ErrorPipeline::recover` now correctly discards all pending contexts on successful recovery (fixes a longstanding inconsistency between docs and behavior)

### Changed - 0.3.1

- **ErrorVec**: Reduced inline capacity from 4 to **2** elements to lower maximum stack usage per error from ~1KB to ~500B.

## [0.3.0]

### Breaking Changes - 0.3.0

- **Removed generic `C` parameter from `ComposableError`**: The error code type is now fixed to `u32` instead of being generic.

  - Changed: `ComposableError<E, C>` → `ComposableError<E>`
  - Type aliases updated: `ComposableResult`, `SimpleComposableError`, `TaggedComposableError`, etc.
  - **Migration**: Users relying on custom error code types (e.g., `&str`, enums) should migrate to using `ErrorContext::tag` or `ErrorContext::metadata`.

- **ErrorPipeline method renaming**:

  - `finish()` → `finish_boxed()` (returns `BoxedComposableResult`)
  - `finish_without_box()` → `finish()` (returns `ComposableResult`)
  - The `rail!` macro now uses `finish_boxed()` internally
  - **Migration**: Replace `.finish()` with `.finish_boxed()` for boxed results, or `.finish_without_box()` with `.finish()` for unboxed results.

- **`ErrorContext` now uses `Cow<'static, str>`**:
  - Changed `String` fields to `Cow<'static, str>` in `ErrorContext`, `GroupContext`, and `Location` to reduce allocations.
  - `ErrorContext::new`, `tag`, `metadata`, `location` now accept `Into<Cow<'static, str>>`.
  - **Migration**: Most code using string literals (`&'static str`) or `String` will continue to work. Code that previously passed non-static string slices (`&str`) will need to be updated to explicitly create an owned `String` (e.g., using `.to_owned()`) before passing it to context-creating functions. Custom construction of `ErrorContext` variants will need to wrap strings in `Cow::Borrowed` or `Cow::Owned`.

### Added - 0.3.0

- **ErrorContextBuilder**: New fluent builder API for creating complex error contexts

  - `ErrorContext::builder()` - Creates a new builder
  - `ErrorContext::group(message)` - Starts a builder with a message
  - Builder methods: `.message()`, `.tag()`, `.metadata()`, `.location()`, `.build()`
  - Example:

    ```rust
    let ctx = ErrorContext::builder()
        .message("connection failed")
        .tag("network")
        .metadata("host", "localhost")
        .build();
    ```

- **Enhanced type safety and ergonomics**:
  - Added `#[must_use]` annotations to core types and combinators (`Validation`, `ComposableError`, `ErrorPipeline`, etc.) to surface ignored results as compile-time warnings.
  - Relaxed trait bounds for better reuse: removed unnecessary `E: Clone` / `E: Default` constraints from `ErrorCategory`, `WithError`, `ErrorOps`, `validation_to_result`, and `Validation` conversion helpers.
  - Improved iterator ergonomics for `Validation`: implemented `FusedIterator` for `Iter`, `IterMut`, `ErrorsIter`, `ErrorsIterMut`, and `IntoIter` to better integrate with the standard iterator ecosystem.

### Changed - 0.3.0

- **Performance optimization**: `GroupContext` now uses `SmallVec` instead of `Vec` for `tags` and `metadata` fields

  - Reduces heap allocations for common cases with 1-2 tags or metadata entries
  - Inline storage for up to 2 elements per collection
  - **Benchmark results**: Up to 50% performance improvement in context operations

- **Zero-allocation for static strings**:

  - `ErrorContext` now avoids heap allocation when created from static string slices (e.g., string literals, `file!()` macro).
  - `IntoErrorContext` implemented for `&'static str` and `Cow<'static, str>`.

- **Safer error/validation combinators**:

  - Marked core types and combinators with `#[must_use]` (e.g. `Validation`,
    `ComposableError`, `ErrorPipeline` and their builder-style methods) so ignored
    results surface as compile-time warnings.

- **Relaxed trait bounds for better reuse**:
  - Removed unnecessary `E: Clone` / `E: Default` constraints from
    `ErrorCategory`, `WithError`, `ErrorOps`, `validation_to_result`, and
    `Validation` conversion helpers, making trait impls more broadly applicable.

### Fixed - 0.3.0

- Updated all examples, tests, and documentation to reflect API changes
- Fixed doctests in `src/convert/mod.rs` and `src/macros/mod.rs`
- All 121 tests passing

## [0.2.1]

### Added - 0.2.1

- Implemented `std::error::Error` for `ComposableError` when the core error implements `Error`, improving interoperability with `anyhow`/`eyre`.
- Enhanced `Display` implementation to support alternate formatting (`{:#}`) for multi-line, structured error output.
- Generalized `FromIterator` for `Validation` to support collecting into any collection type (e.g., `HashSet`, `SmallVec`) instead of just `Vec`.
- Added `impl_error_context!` macro for easy `IntoErrorContext` implementations on custom types.
- Added `fallback` and `recover_safe` recovery methods to `ErrorPipeline` for default value recovery.
- Documented Serde support for `Validation` and added tests confirming serialization/deserialization.
- Organized tests into `tests/unit` submodule.
- Added `backtrace!` macro for lazy backtrace capture.

## [0.2.0]

### Added - 0.2.0

- Implemented customizable error formatting via `ErrorFormatter` with builder pattern.

### Changed - 0.2.0

- Refactored `ErrorContext` to use `Simple` and `Group` variants for better structure and flexibility.
- **Breaking**: `ErrorContext` enum variants have changed. `Message`, `Location`, `Tag`, `Metadata` are now consolidated into `Simple(String)` and `Group(GroupContext)`.

## [0.1.0]

### Added - 0.1.0

- Initial release
