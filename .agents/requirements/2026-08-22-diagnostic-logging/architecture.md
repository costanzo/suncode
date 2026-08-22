# Architecture

## Current state

The desktop used an ad hoc stderr writer and the runtime used direct `eprintln!` calls.

## Proposed design

Add one small logger module per host language. Each module owns level parsing, timestamped formatting, a process-local synchronized file writer, and stderr mirroring. The desktop and runtime intentionally use different files.

## Boundaries and dependencies

The desktop logger is an Avalonia infrastructure utility. The Rust logger is a runtime-core utility. Neither is part of the SDK protocol or persistence layer.

## Data and control flow

The runtime initializes its logger after loading `Config`; the desktop initializes before Avalonia starts. Calls below the configured minimum level are discarded. Accepted records are appended to the component-specific file and flushed to stderr.

## Security and failure handling

Call sites must log identifiers, states, counts, and timings only. Credentials and content remain excluded. File failures do not stop the application; stderr remains available.

## Compatibility and migration

No protocol, database, or ABI change is introduced. Existing diagnostic call sites use the new level-aware utility.

## Risks and rollback

Synchronous flushes add small diagnostic overhead and can be disabled with `SUNCODE_LOG_LEVEL=OFF`. Removing the logger modules restores the prior stderr-only behavior without data migration.

## Open questions

- Rotation and retention require a later operational decision.
