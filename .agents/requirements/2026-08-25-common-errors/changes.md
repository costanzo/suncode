# Changes

## Source

- Added `agent/crates/common` with `BusinessError`.
- Replaced core, LLM, and data public error types with the shared type.
- Removed `thiserror` from the affected crates.

## Tests

- Added shared error shape coverage.
- Ran workspace library tests, formatting, and clippy.
