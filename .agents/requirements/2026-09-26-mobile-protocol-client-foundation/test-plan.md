# Test Plan

## Scope

Shared serialization and contract field names.

## Unit tests

Decode representative pairing, session detail, sync, and event JSON payloads.

## Integration and conformance tests

The shared client boundary should be exercised against a Remote Server fixture for pairing, 401 refresh, Host Project hydration, sync reset, and WebSocket reconnect before release.

## Regression checks

Existing UI source remains unchanged in behavior.

## Manual checks

Review DTO names against both remote-control contract documents.

## Commands and results

- `git diff --check` — expected to pass.
- Android Gradle tests — blocked when Android SDK is unavailable.

## Residual risks

The current repository has no committed Remote Server fixture, and real iOS camera/device verification requires a full Xcode installation.
