# Changes

## Source

- Advertised `bash` instead of `shell`.
- Changed the model-facing required field from `script` to `command`.
- Added OpenCode-compatible millisecond timeout and `workdir` semantics.
- Retained legacy shell/script/shell-field compatibility in core translation.
- Returned recoverable invalid and malformed tool arguments to the model as correlated tool results, allowing same-turn regeneration.
- Included failed tool results in persisted model context so repaired exchanges survive reloads.

## Contracts and generated artifacts

- No C ABI or SQLite schema change.
- The provider tool schema changed for new model requests.

## Configuration and persistence

- No configuration or persistence migration.

## Tests

- Added schema assertions and bash millisecond timeout translation coverage.
- Updated Avalonia projection fixtures to use the new bash command shape.
- Added an agent integration test covering invalid arguments, model feedback, and second-iteration completion.

## Documentation

- Updated runtime feature/specification and operations contract documentation.
