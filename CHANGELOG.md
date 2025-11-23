# CHANGELOG

## [0.2.1]

### Added - 0.2.1

- Implemented `std::error::Error` for `ComposableError` when the core error implements `Error`, improving interoperability with `anyhow`/`eyre`.
- Enhanced `Display` implementation to support alternate formatting (`{:#}`) for multi-line, structured error output.
- Generalized `FromIterator` for `Validation` to support collecting into any collection type (e.g., `HashSet`, `SmallVec`) instead of just `Vec`.
- Added `impl_error_context!` macro for easy `IntoErrorContext` implementations on custom types.
- Added `fallback` and `recover_safe` recovery methods to `ErrorPipeline` for default value recovery.
- Documented Serde support for `Validation` and added tests confirming serialization/deserialization.
- Organized tests into `tests/unit` submodule.

## [0.2.0]

### Added - 0.2.0

- Implemented customizable error formatting via `ErrorFormatter` with builder pattern.

### Changed - 0.2.0

- Refactored `ErrorContext` to use `Simple` and `Group` variants for better structure and flexibility.
- **Breaking**: `ErrorContext` enum variants have changed. `Message`, `Location`, `Tag`, `Metadata` are now consolidated into `Simple(String)` and `Group(GroupContext)`.

## [0.1.0]

### Added - 0.1.0

- Initial release
