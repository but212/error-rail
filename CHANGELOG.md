# CHANGELOG

## [Unreleased]

### Added - Unreleased

- **Enhanced documentation**: Added comprehensive decision trees, quick reference tables, and common pitfalls guide to README
- **Improved Quick Start guide**: Added concept map showing three levels of complexity (Basic Usage, Validation, Advanced)
- **Migration guide**: Added "Migrating from anyhow" section with side-by-side comparisons
- **Performance tips**: Added quick decision guide and performance optimization recommendations

### Changed - Unreleased

- **Deprecated redundant type aliases**: `SimpleComposableError<E>` and `TaggedComposableError<E>` now marked as deprecated with clear migration guidance
- **Enhanced error messages**: Added `#[diagnostic::on_unimplemented]` to `IntoErrorContext` trait for better compiler guidance when trait bounds are not satisfied
- **Improved DX**: Added helpful implementation examples and links to documentation in trait documentation

### Deprecated - Unreleased

- `SimpleComposableError<E>` - Use `ComposableError<E>` directly (scheduled for removal in 0.5.0)
- `TaggedComposableError<E>` - Use `ComposableError<E>` with `ErrorContext::tag()` instead (scheduled for removal in 0.5.0)

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
