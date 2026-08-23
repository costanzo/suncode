# Changes

## Source

- Removed `apply_patch` and `process` from the model-facing tool registry.
- Removed their now-unused schema modules.
- Removed the host prompt instruction that told the model to prefer `process`.
- Retained historical name translation and audited operation dispatch.

## Contracts and generated artifacts

- New provider requests advertise six built-in tools.
- No C ABI or persisted DTO changed.

## Configuration and persistence

- No changes.

## Tests

- Changed the registry test to assert the exact advertised tool-name set.

## Documentation

- Updated the runtime feature and specification descriptions.
