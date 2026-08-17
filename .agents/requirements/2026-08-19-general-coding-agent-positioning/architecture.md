# Architecture

## Current state

Phase 1 is a .NET 10 Avalonia desktop application embedding a Rust runtime SDK. Existing copy incorrectly elevates that deployment shape into the product definition.

## Proposed design

Describe SunCode as a general-purpose coding agent. Describe the embedded desktop runtime only in architecture and current-scope sections where it is technically relevant.

## Boundaries and dependencies

No runtime, client, SDK, persistence, provider, or operation boundary changes.

## Data and control flow

No changes.

## Security and failure handling

Reviewable authority, audited operations, credential handling, and recovery guarantees remain unchanged.

## Compatibility and migration

Documentation and interface copy only. Existing data and APIs require no migration.

## Risks and rollback

The main risk is overstating capabilities while broadening the product description. Copy must describe general coding purpose without claiming unimplemented hosted or client features.

## Open questions

- None.
