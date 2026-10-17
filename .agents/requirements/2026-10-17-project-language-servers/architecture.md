# Architecture

## Current state

One global `language_server` row stores an arbitrary structured command and desired enabled state. Rust creates a distinct in-memory runtime per project, but every project receives the same persisted arguments and environment.

## Proposed design

Rust owns an immutable catalog keyed by stable server kind. A project-scoped persisted configuration selects and configures one catalog entry. The initial `jdtls` adapter validates typed fields and constructs the executable, `-data` argument, environment, root matching, and timeouts.

## Boundaries and dependencies

Rust remains authoritative for catalog definitions, validation, path derivation, process launch, LSP semantics, and persistence. SDKs expose typed catalog and project-configuration DTOs. Avalonia presents server-specific pages and never constructs launch arguments.

## Data and control flow

Settings requests the catalog for the selected project, edits the typed jdtls configuration, and submits one optimistic project mutation. Rust persists it, reconciles that project's runtime, and returns the effective configuration and runtime state.

## Security and failure handling

Processes continue to launch without a shell using the filtered environment. Managed data directories stay under SunCode local data and are unique by stable project identity. Failures are recoverable, project-scoped results and do not make the agent unavailable.

## Compatibility and migration

The existing global free-form table and SDK methods require a migration decision before implementation. Existing definitions must not be silently applied to every project as typed built-ins.

## Risks and rollback

Server-specific configuration increases SDK and UI surface area. Keeping the existing LSP client and runtime semantics behind a typed launch adapter limits the change. The design-system work is independently revertible before production contracts change.

## Open questions

- Persistence shape and migration strategy.
- Optional global installation defaults.
- Legacy custom-server support.
