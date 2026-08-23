# Changes

## Source

- Added session Full Control policy evaluation while preserving unknown-tool denial and all operation validation.
- Added `Allow for session` and a conditional Full Control warning with a direct `Turn off` action.
- Added selected-session configuration loading and stale-load protection in Avalonia.

## Contracts and generated artifacts

- Expanded `resolve_approval` with the `allow_session` decision.
- Updated the shared runtime SDK vector and written contract.

## Configuration and persistence

- Persisted `full_control` as a session-scoped boolean in the existing `configuration` table.
- Made approval resolution, configuration enablement, and authority audit insertion atomic.
- Audited explicit Full Control changes made through the SDK setting boundary.

## Tests

- Added policy coverage, database configuration/atomicity tests, SDK validation coverage, and an agent integration test for a later approval-free write.

## Documentation

- Updated runtime and Avalonia feature records plus the current runtime specification.
