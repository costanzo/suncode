# Changes

## Source

- Added `question` tool schema, validation, events, and continuation handling.
- Added question answer/reject C ABI and C# wrappers.
- Added Avalonia single-select, multi-select, custom-answer, submit, and skip controls.

## Persistence

- Reused the current turn recovery snapshot; no new table or migration path was added.

## Tests

- Added question argument validation and recovery snapshot tests.

## Documentation

- Updated the SDK contract and current Rust/Avalonia feature facts.
