# Changes

## Source

- Replaced inline tool JSON previews with concise operation summaries and status labels.
- Added a selectable operation-detail overlay for request, result, and error content.
- Unified read-only JSON display formatting so shell operators such as `&&` remain readable instead of appearing as Unicode escapes.
- Preserved separate tool request/result/error values across snapshot and live projections.
- Marked argument-preparation failures as terminal failed tool states.
- Added non-empty legacy shell `command` compatibility and explicit empty-script validation.

## Contracts and generated artifacts

- No SDK payload or model-facing schema changed.

## Configuration and persistence

- No database schema or configuration changes.

## Tests

- Added desktop projection coverage for operation summaries and friendly invalid-argument errors.
- Added desktop regression coverage for readable shell operators in approval and tool details.
- Added Rust translation coverage for legacy shell command fallback and empty input rejection.

## Documentation

- Updated the Avalonia feature record and this requirement package.
