# Changes

## Source

- Added an internally synchronized model-provider registry with atomic route replacement.
- Added the typed Rust SDK endpoint update and hand-written C ABI/C# binding.
- Added Avalonia provider navigation folding, URL editing, validation feedback, and live model projection refresh.
- Updated the design-system Settings specimen with matching navigation and URL controls.

## Contracts and generated artifacts

- Updated the hand-written SDK and persistence contracts; no generated artifacts are used.

## Configuration and persistence

- No schema change; the operation updates the existing `llm_model_provider.endpoint` field while preserving credentials and catalog metadata.

## Tests

- Added registry replacement and SDK/FFI endpoint validation, persistence, normalization, and live-projection tests.

## Documentation

- Updated stable agent/desktop feature documentation and clarified the architecture's deferred provider-catalog scope.
