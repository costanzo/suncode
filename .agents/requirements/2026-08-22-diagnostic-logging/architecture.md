# Architecture

## Current state

The desktop used an ad hoc stderr writer and the runtime used direct `eprintln!` calls.

## Proposed design

Add one small logger module per host language. Each module owns level parsing, timestamped formatting, a process-local synchronized file writer, bounded size-based rotation, and stderr mirroring. The desktop and runtime intentionally use different files.

## Boundaries and dependencies

The desktop logger is an Avalonia infrastructure utility. The Rust logger is a runtime-core utility. Rust owns durable logging settings in `configuration`; Avalonia obtains them only through the SDK.

## Data and control flow

The runtime opens SQLite, loads the global logging rows, and configures its logger before normal runtime diagnostics. The desktop begins with stderr available, then reads the same effective settings through the SDK and configures its file logger. Calls below the configured minimum level are discarded. Before an append that would exceed the configured size, the active file is renamed through the numbered retention set and a new active file is opened. Accepted records are appended to the component-specific file and flushed to stderr.

## Security and failure handling

Call sites must log identifiers, states, counts, and timings only. Credentials and content remain excluded. File failures do not stop the application; stderr remains available.

## Compatibility and migration

The existing settings SDK shape carries logging values, so no ABI method is added. Fresh databases seed the four global rows. Data/database location remains external bootstrap configuration because the runtime must locate SQLite before it can read the rows.

## Risks and rollback

Synchronous flushes add small diagnostic overhead and can be disabled with `log_level=OFF`. Removing the logger modules would leave the persisted rows unused.

## Open questions

- Time-based rotation and compression require a later operational decision.
