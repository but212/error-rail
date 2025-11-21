# CHANGELOG

## [0.2.0]

### Changed

- Refactored `ErrorContext` to use `Simple` and `Group` variants for better structure and flexibility.
- **Breaking**: `ErrorContext` enum variants have changed. `Message`, `Location`, `Tag`, `Metadata` are now consolidated into `Simple(String)` and `Group(GroupContext)`.

## [0.1.0]

### Added

- Initial release
