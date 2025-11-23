# CHANGELOG

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
