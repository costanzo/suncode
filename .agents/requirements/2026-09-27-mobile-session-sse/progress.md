# Progress

- Status: Complete
- Last updated: 2026-09-27

## Completed

- Migrated Mobile event delivery to per-Session SSE and foreground `/v1/sync` polling.
- Added event cursor persistence, ordered duplicate suppression, `410` recovery, and app lifecycle transport switching.
- Updated HTTP and AsyncAPI contracts and added focused protocol DTO coverage.
- Android build/tests and iOS Simulator Kotlin compile checks passed.

## In progress

- None.

## Blocked

- Remote Server deployment is external to this repository and must implement the contract before runtime integration.

## Log

### 2026-09-27

- Completed client and contract migration; verified Android and iOS Simulator Kotlin targets.
