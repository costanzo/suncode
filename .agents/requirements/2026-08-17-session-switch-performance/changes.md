# Changes

## Source

- Added a bulk observable collection for bounded supporting-state replacement.
- Atomically replace the conversation message source so the virtualized list cannot retain realized rows from the previous session.
- Moved session snapshot projection to a worker thread and retained latest-selection guards.
- Replaced the non-virtualized conversation items control with an explicit virtualizing stack panel.
- Coalesced scroll-to-end requests.
- Added a version-aware loading animation that appears only when loading exceeds 120 ms.

## Contracts and generated artifacts

- No contract or generated artifact changes planned.

## Configuration and persistence

- None.

## Tests

- Added desktop tests for bulk replacement notification behavior, snapshot projection fidelity, and atomic message-source rebinding.

## Documentation

- Added this requirement package.
- Updated the implemented Avalonia feature description.
