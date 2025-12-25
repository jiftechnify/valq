# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.1] - 2025-12-25

### Fixed

- **Added `$crate` prefix to all internal macro invocations** ([#62](https://github.com/jiftechnify/valq/pull/62))

### Documentation

- **Refined crate-level documentation** ([#63](https://github.com/jiftechnify/valq/pull/63))
  - Refined example code to showcase every feature in `valq`

## [0.3.0] - 2025-12-17

### Added

- **`query_value_result!` macro** for Result-returning queries ([#56](https://github.com/jiftechnify/valq/pull/56))
  - A new flavor of `query_value!` that returns a `valq::Result` instead of an `Option`
  - Enables better error handling for queries where keys or indices might not exist
  - Supports all the same query syntax as `query_value!`

- **`transpose_tuple!` helper macro** for transposing tuples of Results/Options ([#58](https://github.com/jiftechnify/valq/pull/58))
  - Convert `(Option<A>, Option<B>, ...)` into `Option<(A, B, ...)>`
  - Convert `(Result<A>, Result<B>, ...)` into `Result<(A, B, ...)>`
  - Particularly useful when combining multiple `query_value!`/`query_value_result!` calls

## [0.2.0] - 2025-12-12

### Added

- **Deserialization operator (`>>`)** for deserializing queried values into arbitrary types ([#46](https://github.com/jiftechnify/valq/pull/46))
  - Deserialize queried values into any type implementing `serde::Deserialize`
  - Syntax: `query_value!(obj.field >> (Type))`
    - Don't forget to wrap the destination type with parentheses!

- **Null coalescing operator (`??`)** for unwrapping query results with default values ([#50](https://github.com/jiftechnify/valq/pull/50))
  - `... ?? <expr>` to provide a custom default value
  - `... ?? default` to use `Default::default()` as the fallback

### Changed

- **Arbitrary type casting with `->` operator** ([#46](https://github.com/jiftechnify/valq/pull/46))
  - No longer limited to hard-coded conversions!
  - Any `as_xxx()` method available on the value type can be used

- **Made syntax more JS-like** ([#47](https://github.com/jiftechnify/valq/pull/47))
  - More flexible bracket notation `[...]`
    - Put arbitrary Rust expressions and enjoy unlimited dynamic queries!
    - Removed `."key"` syntax in favor of revamped bracket notation
  - `??` operator for unwrapping `Option`s
  
### Documentation

- Removed tedious enumeration of macro matching patterns from documentation ([#49](https://github.com/jiftechnify/valq/pull/49))
- Added comprehensive examples for all major features
- Clarified query syntax specification

## [0.1.0] - 2021-12-19

Initial release with basic query functionality.

### Added

- `query_value!` macro for querying semi-structured data
  - Dot notation for accessing object properties (`.field`)
  - Bracket notation for array/object indexing (`[index]`)
  - Mutable reference extraction with `mut` prefix
  - Basic type casting using `as_***()` methods with `->` operator

[0.3.0]: https://github.com/jiftechnify/valq/compare/0.2.0...0.3.0
[0.2.0]: https://github.com/jiftechnify/valq/compare/0.1.0...0.2.0
[0.1.0]: https://github.com/jiftechnify/valq/releases/tag/0.1.0
