# Architecture

## Current state

Phase 1 has no executable third-party extensions. The trusted runtime and Rust core remain the only implemented execution path.

## Proposed design

Future extensions run as independent child processes. The runtime owns extension lifecycle, schema validation, policy preflight, identity, and audit. Rust owns process launch, sandbox materialization, resource limits, credential injection, and core capability enforcement. A worker thread may be used for convenience inside an already isolated host, but never as the security boundary.

## Boundaries and dependencies

```text
trusted TypeScript runtime
        |
  extension host protocol
        |
independent extension process
        |
Rust sandbox and capability broker
```

## Data and control flow

1. Discover and validate metadata without starting code.
2. Require explicit activation and resolve a pinned manifest.
3. Start the child through Rust with a requested profile.
4. Complete a handshake carrying extension identity, protocol version, capabilities, limits, and provenance.
5. Re-authorize every request and return bounded results or artifact references.
6. Quarantine on crash, timeout, protocol violation, or capability mismatch.

## Security and failure handling

Isolation is only claimed where the platform reports the requested enforcement. Weak or unavailable sandbox setup fails closed. Extension identity is attached to every operation and cannot be supplied by model text or extension payload metadata.

## Compatibility and migration

No Phase 1 data migration is required. Future manifests and protocol versions must declare compatibility ranges and integrity metadata.

## Risks and rollback

The main risks are platform capability skew, IPC complexity, startup cost, and secret leakage. Rollback is to keep the extension disabled and use built-in trusted adapters only.

## Open questions

- Exact sandbox primitives and minimum OS versions.
- Extension protocol and artifact transport.
- Secret-handle resolution channel.
