# Changes

## Source

- Added seconds-to-milliseconds timeout validation at the core tool boundary.
- Added continuous output capture with bounded tail previews and full-output artifact streaming.
- Added Unix process groups and Windows process-tree termination.
- Propagated turn cancellation into blocking operations through an atomic flag.
- Added the libc dependency for Unix process-group signals.

## Contracts and generated artifacts

- Removed timeout_ms from advertised process and shell schemas.
- Kept internal timeout_ms operation parameters for the audited dispatcher.

## Configuration and persistence

- No database schema or persisted configuration changes.

## Tests

- Added timeout translation validation tests.
- Added large-output artifact and process cancellation tests.

## Documentation

- Added this requirement package.
